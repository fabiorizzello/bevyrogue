# S12 Research — RosterEntry blueprint-keyed + ValidationSnapshot from registry

## Summary
Depth: **targeted research**. The local code already has the generic registry/runtime pattern for timelines, signals, and passive hooks, but **roster schema and validation observability are still hardcoded around Twin Core / Holy Support-era fields**.

Three load-bearing findings:
1. **`UnitDef` is still structurally coupled to digimon-named metadata**, via `twin_core` and `holy_support` fields in `src/data/units_ron.rs`. Those metadata types appear to be **dead outside schema/default/test scaffolding**: research found no runtime consumers beyond constructors/default assertions.
2. **`ValidationSnapshot` is still a shared hardcoded struct**, not a registry product. `src/combat/observability.rs` still names `twin_core`, `holy_support`, `predator_loop`, `battery_loop`, and `precision_mind_game` directly, and `capture_validation_snapshot()` still treats `TwinCoreState` as a mandatory resource.
3. The roadmap’s after-this proof (**“add new digimon touches only `blueprints/<new>/` + `assets/data/digimon/<new>/`”**) is **broader than the current title suggests**. Even if S12 fixes `UnitDef` and `ValidationSnapshot`, the codebase still has central add-new-digimon chokepoints: `src/combat/kernel.rs` plugin registration, `src/combat/blueprints/mod.rs` dispatch list, `register_all_blueprint_exts()`, and the global `assets/data/units.ron` loader.

Because of the research-slice tools policy, I could not run `cargo test`; verification below is a planner-facing proposed ladder, not fresh execution evidence.

## Skills Discovered
- No new skill install needed.
- Directly relevant preinstalled skills already exist:
  - `bevy`
  - `rust-best-practices`
  - `design-an-interface`
- The `Skill(...)` tool itself was not exposed in this harness, so I applied the **design-an-interface** rule manually: compare multiple schema/registry shapes before recommending one.

## Recommendation
Treat S12 as **two required refactors plus one explicit scope decision**.

### R1. Replace digimon-named roster metadata with a generic owner-keyed payload
Recommended shape: **owner-scoped generic entries**, not a central enum and not raw untyped `Value` blobs.

Why:
- A central `enum BlueprintRosterPayload { TwinCore(...), HolySupport(...), ... }` still forces shared-file edits per digimon.
- A raw `HashMap<String, Value>` is flexible but weakly validated and awkward to reason about in tests.
- A small generic owner-keyed payload preserves extension-friendliness while keeping deterministic serialization and blueprint-local decoding.

Recommended direction:
```rust
pub struct UnitDef {
    // existing generic fields...
    #[serde(default)]
    pub blueprint_entries: Vec<BlueprintRosterEntry>,
}

pub struct BlueprintRosterEntry {
    pub owner: String,
    #[serde(default)]
    pub fields: Vec<BlueprintField>,
}

pub struct BlueprintField {
    pub key: String,
    pub value: BlueprintValue,
}

pub enum BlueprintValue {
    Int(i64),
    Bool(bool),
    Text(String),
}
```

Use `Vec` rather than `HashMap` in the serialized shape so RON order stays stable and formatting/tests remain deterministic. Blueprint modules can expose local helpers to decode their own entries.

### R2. Add a registry axis for validation snapshot contributors
Recommended direction: extend `ExtRegistries` with **`ValidationExt`** and make `ValidationSnapshot` collect blueprint-owned fields by iterating the registry.

Suggested contract:
```rust
pub struct ValidationExt;
impl ExtPoint for ValidationExt {
    type Fn = fn(&World) -> Option<ValidationField>;
}

pub struct ValidationField {
    pub owner: &'static str,
    pub entries: Vec<(String, String)>,
}
```

Then:
- `capture_validation_snapshot()` builds the shared kernel/core fields as today.
- Blueprint-owned diagnostic surfaces come from registry iteration, sorted deterministically by `owner` then by entry key.
- `format_validation_snapshot()` renders generic `owner=...` sections instead of shared digimon-named struct fields.

Important behavioral change: **remove the hard dependency on `TwinCoreState`**. Twin Core should become “present if registered/resource exists,” same as the other blueprint surfaces.

### R3. Force a planning decision on the “add new digimon only 2 dirs” proof
Current code still requires edits outside those dirs for a truly new digimon:
- `src/combat/kernel.rs` — explicit plugin list
- `src/combat/blueprints/mod.rs` — `BLUEPRINTS` dispatch table
- `src/combat/blueprints/mod.rs::register_all_blueprint_exts()` — currently only Agumon exts
- `src/data/mod.rs`, `src/bin/combat_cli.rs`, `src/headless.rs`, `src/windowed.rs`, many tests — all still load `assets/data/units.ron`
- `assets/data/digimon/` does not exist yet in the working tree

So the planner should choose one of two honest outcomes:
1. **Narrow S12** to roster-payload + validation-registry only, and explicitly note that the roadmap’s “2 dirs only” proof remains blocked by later/additional work.
2. **Broaden S12** to also address central blueprint registration and/or roster-fragment loading, because otherwise the stated after-this proof is not actually reachable.

## Manual Design-Interface Pass
Following the `design-an-interface` principle (“contrast matters more than first draft”), these are the three viable shapes:

### Option A — Central typed payload enum
```rust
enum BlueprintRosterPayload {
    TwinCore(TwinCoreRosterMetadata),
    HolySupport(HolySupportRosterMetadata),
    // more digimon here...
}
```
**Reject.** It just relocates hardcoding into a new shared enum.

### Option B — Raw dynamic map
```rust
blueprint: BTreeMap<String, BTreeMap<String, String>>
```
**Acceptable but weak.** Easy to extend, but all validation becomes stringly typed and every blueprint must hand-parse scalars.

### Option C — Owner-keyed generic scalar payload + local decoders
```rust
Vec<BlueprintRosterEntry { owner, fields: Vec<BlueprintField { key, value }> }>
```
**Recommended.** Shared layer remains generic; blueprint modules retain typed interpretation; serialization remains deterministic.

## Implementation Landscape

### `src/data/units_ron.rs`
- `UnitDef` still contains:
  - `pub twin_core: TwinCoreRosterMetadata`
  - `pub holy_support: HolySupportRosterMetadata`
- The supporting enums/structs are defined only here and appear unused at runtime.
- Asset round-trip tests and parse fixtures still construct those fields explicitly.
- This is the primary schema file for S12.

### `assets/data/units.ron`
- The canonical roster currently **does not contain** `twin_core:` or `holy_support:` entries at all; it relies on serde defaults.
- That means the schema migration can likely be **additive first** with low asset churn.
- It also means the hardcoded metadata fields are not carrying current production data.

### `src/combat/bootstrap.rs`
- `taichi_def()` and other manual `UnitDef` constructors still initialize `twin_core` / `holy_support` defaults.
- Any `UnitDef` schema change must update these constructors.

### `src/combat/api/registry.rs`
- `ExtRegistries` still has **7 axes only**.
- This is the natural place to add `ValidationExt` if S12 follows milestone context literally (`Registry<ValidationExt>`).
- The default-empty registry tests here will need extension.

### `src/combat/observability.rs`
This is the core S12 observability refactor.
- `ValidationSnapshot` still has named fields:
  - `twin_core: ValidationTwinCoreSnapshot`
  - `holy_support: Option<HolySupportSnapshot>`
  - `predator_loop: Option<PredatorLoopSnapshot>`
  - `battery_loop: Option<BatteryLoopSnapshot>`
  - `precision_mind_game: Option<PrecisionMindGameSnapshot>`
- `capture_validation_snapshot()` still directly queries/resources each blueprint-owned state.
- `TwinCoreState` is still required via `ok_or(MissingResource("TwinCoreState"))`.
- `format_validation_snapshot()` still renders digimon-named output sections.

### `src/headless.rs`, `src/windowed.rs`, `src/bin/combat_cli.rs`
- These only call `capture_validation_snapshot()` / `format_validation_snapshot()`.
- They are likely shallow follow-up edits once the snapshot type changes.
- `combat_cli.rs` still hardcodes `data/units.ron` existence/parse expectations.

### `src/combat/kernel.rs` and `src/combat/blueprints/mod.rs`
These are not named by the slice title, but they matter for the roadmap proof.
- `kernel.rs` still adds blueprint plugins explicitly.
- `blueprints/mod.rs` still maintains a central dispatch table (`BLUEPRINTS`) and `register_all_blueprint_exts()` currently only registers Agumon exts.
- This means “new digimon without shared edits” is not true yet, even after roster/validation cleanup.

### Test surface coupled to the current shape
Highest-value tests likely to move with S12:
- `tests/validation_snapshot.rs`
- `tests/predator_loop_kernel.rs` (constructs `ValidationSnapshot` directly)
- `tests/holy_support_affordance.rs`
- `tests/holy_support_mechanics.rs`
- `tests/holy_support_resolution.rs`
- `tests/patamon_blueprint_seam.rs`
- `tests/twin_core_integration.rs`
- `tests/dorumon_predator_runtime.rs`
- `tests/renamon_precision_runtime.rs`
- `tests/presentation_metadata_boundary.rs`
- `tests/combat_cli_shared_surface.rs`

### Manual `UnitDef` constructor footprint
Project scan found explicit `UnitDef { ... }` constructors in many places that will need schema updates if the fields are removed:
- `tests/bootstrap_spawn_composition.rs` — 6 constructors
- `tests/tempo_resistance.rs` — 4
- `tests/follow_up_chains.rs` — 3
- `tests/roster_smoke.rs` — 2
- `src/data/units_ron.rs` — 2
- `src/combat/bootstrap.rs` — 2
- plus single constructors in `tests/twin_core_integration.rs`, `tests/follow_up_triggers.rs`, `tests/combat_coherence.rs`, `tests/pipeline_dispatch.rs`, `tests/form_identity.rs`, `tests/resource_caps.rs`

## Natural Seams

### Seam 1 — UnitDef schema cleanup
Files:
- `src/data/units_ron.rs`
- `assets/data/units.ron`
- `src/combat/bootstrap.rs`
- tests with manual `UnitDef` construction

Goal:
- Remove digimon-named roster metadata fields.
- Introduce generic owner-keyed blueprint payload.
- Preserve deterministic RON round-trip and backward-compatible defaults where possible.

### Seam 2 — Validation registry axis
Files:
- `src/combat/api/registry.rs`
- likely `src/combat/api/mod.rs`
- blueprint modules that need to register validation contributors

Goal:
- Add `ValidationExt` without disturbing the existing 7-axis mental model more than necessary.
- Establish the contract all blueprint validation contributors must follow.

### Seam 3 — ValidationSnapshot refactor
Files:
- `src/combat/observability.rs`
- callers in `src/headless.rs`, `src/windowed.rs`, `src/bin/combat_cli.rs`
- snapshot-focused tests

Goal:
- Replace named blueprint fields with registry-driven fields.
- Remove mandatory Twin Core dependency.
- Preserve deterministic formatting and low-noise CLI/headless output.

### Seam 4 — “add new digimon” proof alignment
Files:
- `src/combat/kernel.rs`
- `src/combat/blueprints/mod.rs`
- `src/data/mod.rs`
- asset layout / loader paths
- proof tests

Goal:
- Either explicitly defer this broader extension-friendliness work,
- or include it now so the roadmap after-this statement becomes true.

## First Proof
The highest-value first proof is **structural**, not behavioral:

1. `src/data/units_ron.rs` no longer declares digimon-named roster metadata types/fields.
2. `src/combat/observability.rs` no longer hardcodes blueprint-owned snapshot fields.
3. `capture_validation_snapshot()` no longer hard-fails on missing `TwinCoreState`.

Planner-friendly grep checkpoints:
- `rg -n "TwinCoreRosterMetadata|HolySupportRosterMetadata|pub twin_core:|pub holy_support:" src/data src/combat tests`
- `rg -n "ValidationTwinCoreSnapshot|holy_support: Option<|predator_loop: Option<|battery_loop: Option<|precision_mind_game: Option<" src/combat tests`
- `rg -n "TwinCoreState\)|MissingResource\(\"TwinCoreState\"\)" src/combat/observability.rs`

Separate honesty check for the roadmap proof:
- `find assets/data -maxdepth 2 -type d | sort`
- `rg -n "assets/data/digimon|data/digimon|data/units\.ron" src tests`

If those last checks still show a monolithic `units.ron` world and no `assets/data/digimon/<x>/`, then “2 dirs only” is still not true regardless of schema cleanup.

## Risks / Watch-outs
- **Roadmap/title mismatch:** S12 title sounds local, but the after-this proof implies broader extension-registration and asset-layout work.
- **Observability assertion blast radius:** many tests assert exact formatted snapshot strings; expect wide but mechanical updates.
- **Determinism risk:** if registry-driven validation fields are stored in `HashMap` order, snapshot formatting will become flaky. Sort by owner and key before rendering.
- **Over-design risk:** the current hardcoded roster metadata is barely used. Avoid inventing a large generic schema if only a tiny scalar payload is needed.
- **Twin Core specialness leak:** today Twin Core is required while the others are optional. If that asymmetry survives, S12 has not actually achieved registry-owned validation.
- **Central registration remains:** even after S12, `kernel.rs` plugin lists and `blueprints/mod.rs` dispatch tables can still force shared edits per digimon.
- **Agumon-only ext registration footgun:** `register_all_blueprint_exts()` currently only wires Agumon. If the planner broadens extension-friendliness work, this function becomes part of the fix surface.

## Verification
Fresh execution was blocked by the research-slice read-only tools policy, so these are recommended executor checks:

1. **Schema / structure checks**
   - `rg -n "TwinCoreRosterMetadata|HolySupportRosterMetadata|pub twin_core:|pub holy_support:" src/data src/combat tests`
   - `rg -n "ValidationTwinCoreSnapshot|holy_support: Option<|predator_loop: Option<|battery_loop: Option<|precision_mind_game: Option<" src/combat tests`
   - `find assets/data -maxdepth 2 -type d | sort`
   - `rg -n "assets/data/digimon|data/digimon|data/units\.ron" src tests`

2. **Focused tests**
   - `cargo test --test validation_snapshot`
   - `cargo test --test combat_cli_shared_surface`
   - `cargo test --test holy_support_affordance`
   - `cargo test --test holy_support_mechanics`
   - `cargo test --test holy_support_resolution`
   - `cargo test --test patamon_blueprint_seam`
   - `cargo test --test twin_core_integration`
   - `cargo test --test dorumon_predator_runtime`
   - `cargo test --test renamon_precision_runtime`
   - `cargo test --test predator_loop_kernel`

3. **Roster / constructor regressions**
   - `cargo test --test roster_smoke`
   - `cargo test --test bootstrap_spawn_composition`
   - `cargo test --test tempo_resistance`
   - `cargo test --test follow_up_chains`

4. **Build checks**
   - `cargo check`
   - `cargo check --features windowed`

## Planner Notes
- Best decomposition is probably **3 tasks minimum**, possibly 4 if the roadmap proof is taken literally:
  1. `UnitDef` schema replacement + constructor/test fallout.
  2. `ValidationExt` axis + `ValidationSnapshot` registry refactor.
  3. Snapshot/test/CLI formatting realignment.
  4. Optional but likely necessary: central blueprint-registration / asset-layout work for the “2 dirs only” proof.
- If the planner wants to keep S12 small, it should explicitly **downgrade the after-this proof** to “no shared roster/validation edits for new blueprint-owned metadata,” because the stronger 2-dir claim is not reachable from the current architecture without more than roster/snapshot work.
- The most leverage comes from deciding the interface once. If the payload shape thrashes mid-execution, the constructor/test fallout will multiply quickly.
