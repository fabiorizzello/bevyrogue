---
spike: SP2
related: RESEARCH.md, DECISION.md, INTERFACE-OPTIONS.md
status: complete
created: 2026-05-12
---

# SP2 — Migration plan

Sequencing for migrating the 6 existing blueprints + adding the 2 new ones (kitsune_grace, holy_aegis) onto the Option B `Blueprint` trait + `BlueprintRegistry` architecture.

## Sequencing principle

**Migrate the simplest, most-isolated blueprint first** to shake out the trait surface, then escalate. Within each step, keep the legacy `transitions_for_action_checked` and the typed `ValidationSnapshot` fields working as compat shims so existing tests pass unchanged. Drop shims only after all migrations land.

## Step 0 — Infra (no blueprint migration yet)

**Deliverables:**
- `src/combat/blueprint_registry.rs` (new) — defines `Blueprint` trait, `BlueprintId` newtype, `BlueprintRegistry` resource, `BlueprintSnapshot` enum, `BlueprintCtx` struct.
- `src/combat/blueprints/mod.rs` adds a parallel `transitions_for_action_checked_via_registry` that takes `&BlueprintRegistry` and forwards to `registry.dispatch_commit`. Existing `transitions_for_action_checked` (static `BLUEPRINTS` array) is kept untouched.
- `src/combat/kernel.rs::register_default_blueprints(app)` exists but registers **zero** blueprints (empty registry).
- `src/main.rs` calls `register_default_blueprints(app)` after `register_combat_kernel_runtime(app)`.

**Test impact:** none. New code paths are dead code.

**LOC:** ~150 added, 0 removed, 0 modified outside the new file.

## Step 1 — Migrate `tentomon` (BatteryLoop) first

**Why first:** BatteryLoop is the smallest typed-state blueprint (3 signal kinds, simple state), uses the canonical generic envelope, and has well-isolated tests (`tests/battery_loop_kernel.rs`, `tests/tentomon_blueprint.rs`). It is also the original reference implementation cited in the memory notes for the generic envelope migration — so it is closest to "done".

**Deliverables:**
- `src/combat/blueprints/tentomon.rs` gains `pub struct TentomonBlueprint;` with `impl Blueprint for TentomonBlueprint`. The `dispatch` free function is renamed to `commit_signals` body inside the trait impl. The `OWNER` constant becomes `id()` returning `BlueprintId("tentomon")`.
- `snapshot()` extracts `BatteryLoopState` from `World` and wraps in `BlueprintSnapshot::BatteryLoop(BatteryLoopSnapshot::from(...))`.
- `on_event()` returns empty (BatteryLoop will gain `IncomingDamage` listener in M017 S03e for the Path B block-reaction; that's a separate slice).
- `register_default_blueprints` adds `registry.register(TentomonBlueprint)`.
- `blueprints/mod.rs::BLUEPRINTS` array: remove the `tentomon` entry. `transitions_for_action_checked` becomes a forwarder for `tentomon` and a static-lookup for the remaining 5.

**Compat shims:**
- `transitions_for_action_checked` checks both the registry (for migrated blueprints) and the static array (for un-migrated). Returns first match.
- `ValidationSnapshot.battery_loop: Option<BatteryLoopSnapshot>` field is computed from `registry.snapshot_all(world).get(&BlueprintId("tentomon"))` via an accessor function for transition; the typed field is kept.

**Tests affected:**

| Test file | Touches | Expected impact |
|---|---|---|
| `tests/battery_loop_kernel.rs` | Direct `apply_battery_loop_transitions_system`, BatteryLoopTransition assertions | No-op — apply system unchanged |
| `tests/tentomon_blueprint.rs` | `dispatch_custom_signal("tentomon", ...)` | No-op — forwarder preserved |
| `tests/validation_snapshot.rs` | `snapshot.battery_loop` | No-op — typed field preserved |
| `tests/combat_cli_shared_surface.rs` | snapshot serialization | No-op |
| `tests/digimon_signal_registry.rs` | static `BLUEPRINTS` array iteration | **Signature change** — must iterate via registry |
| `tests/roster_smoke.rs` | full combat smoke | No-op |

**Validation:** all 6 tests must pass with no semantic change. `digimon_signal_registry.rs` gets an updated reference to iterate `registry.blueprints` instead of the static array.

## Step 2 — Migrate `patamon` (HolySupport)

**Why second:** Single signal kind (`build_holy_support_grace`), tightest scope, and the upcoming `holy_aegis` listener (round-3 passive) will hang off the same blueprint module. Migrating patamon first means kitsune_grace and holy_aegis can be added as listener-only `on_event` overrides without first introducing a new blueprint module.

**Deliverables:**
- `src/combat/blueprints/patamon.rs` gains `pub struct PatamonBlueprint;` with `impl Blueprint`.
- `commit_signals` body = existing dispatch logic.
- `snapshot` reads `HolySupportState` and returns `BlueprintSnapshot::HolySupport(_)`.
- `on_event` still empty (added in Step 7).

**Compat shims:** same shape as Step 1.

**Tests affected:**

| Test file | Touches | Expected impact |
|---|---|---|
| `tests/holy_support_affordance.rs` | HolySupportState invariants | No-op |
| `tests/holy_support_mechanics.rs` | apply system | No-op |
| `tests/holy_support_resolution.rs` | Effect → transition | No-op |
| `tests/holy_support_roster_contract.rs` | roster bootstrap with patamon | No-op |
| `tests/patamon_blueprint_seam.rs` | dispatch_custom_signal("patamon", ...) | No-op forwarder |
| `tests/patamon_revive.rs` | Revive effect | No-op |
| `tests/validation_snapshot.rs` | `snapshot.holy_support` | No-op typed field |

## Step 3 — Migrate `dorumon` (PredatorLoop)

**Why third:** Larger state machine; tests several cap/decay invariants. Doing it after a known-good migration of two simpler blueprints means the trait surface is stable.

**Deliverables, shims, tests:** mirror Step 1 shape.

**Tests affected:** `tests/dorumon_blueprint.rs`, `tests/dorumon_predator_runtime.rs`, `tests/predator_loop_kernel.rs`. All no-op under the shim.

## Step 4 — Migrate `renamon` (PrecisionMindGame)

**Why fourth:** All 4 signals use the `Empty` payload — exercise the trait's payload-agnostic path. Kitsune Grace listener (Step 7) will attach to this blueprint module.

**Tests affected:** `tests/renamon_precision_runtime.rs`. No-op.

## Step 5 — Migrate `agumon` AND `gabumon` (TwinCore, paired)

**Why bundled:** Both blueprints share `TwinCoreState`. The trait impls are siblings. Migrating one without the other would force `TwinCoreState` to be read by two unmigrated halves and one migrated half — messy.

**Deliverables:**
- `pub struct TwinCoreFireBlueprint;` (Agumon) and `pub struct TwinCoreIceBlueprint;` (Gabumon). Both have `commit_signals` overriding; `snapshot()` is implemented on **only one** (say, Agumon) and returns `BlueprintSnapshot::TwinCore(_)`. Gabumon returns `BlueprintSnapshot::Empty` to avoid double-counting.
- Alternative: introduce a "snapshot owner" marker. SP2 sketch picks the simpler "one owns it" approach.

**Tests affected:** `tests/twin_core_integration.rs`, `tests/twin_core_mechanics.rs`, `tests/twin_core_resolution.rs`, `tests/twin_core_roster_contract.rs`. All no-op under shims.

## Step 6 — Drop shims and cleanup

After all 6 blueprints are on the trait:

**Deliverables:**
- Delete the static `BLUEPRINTS` array in `blueprints/mod.rs`.
- `transitions_for_action_checked` becomes a one-line forwarder to `registry.dispatch_commit`.
- `ValidationSnapshot.blueprint_snapshots: HashMap<BlueprintId, BlueprintSnapshot>` becomes the canonical field. The 5 typed fields (`twin_core`, `holy_support`, `predator_loop`, `precision_mind_game`, `battery_loop`) become `#[deprecated]` accessor methods that read from the map.
- Optional: convert typed accessors into proper deprecation warnings; full removal in a follow-up PR.

**Test impact:** ~5 tests that read snapshot fields directly may need a one-line accessor switch (`snapshot.twin_core` → `snapshot.twin_core()`). Search:

```bash
rg 'snapshot\.(twin_core|holy_support|predator_loop|battery_loop|precision_mind_game)' tests/
```

Expected ~10-15 lines across `tests/validation_snapshot.rs` and `tests/combat_cli_shared_surface.rs`.

## Step 7 — Add `kitsune_grace` (Renamon passive)

**New file:** `src/combat/blueprints/kitsune_grace.rs` (or extend `renamon.rs` with a second `Blueprint` impl).

```rust
pub struct KitsuneGraceBlueprint;
impl Blueprint for KitsuneGraceBlueprint {
    fn id(&self) -> BlueprintId { BlueprintId("kitsune_grace") }
    // commit_signals: default empty (listener-only)
    fn on_event(&self, event: &CombatEvent, ctx: &BlueprintCtx) -> Vec<Effect> {
        match &event.kind {
            CombatEventKind::UltimateUsed { actor } => {
                let self_id = ctx.find_unit_id_for_blueprint(self.id())?;
                if *actor == self_id { return vec![]; }
                if ctx.team_of(*actor) != ctx.team_of(self_id) { return vec![]; }
                vec![Effect::AdvanceTurn { actor: TargetRef::Self_, pct: 10 }]
            }
            _ => vec![],
        }
    }
    // snapshot: default empty (stateless)
}
```

**Prerequisite SP1 work:**
- `CombatEventKind::UltimateUsed { actor: UnitId }` must exist (SP1 §"Reactive signature bus" recommendation).
- `Effect::AdvanceTurn { actor: TargetRef, pct: i8 }` must exist (SP3 §8 — replaces `SelfAdvance`).
- `BlueprintCtx::find_unit_id_for_blueprint` + `team_of` accessors must exist.

**Test impact:** new integration test `tests/kitsune_grace_listener.rs` (~80 lines) asserting that ally ult ⇒ Renamon AV gain; self-cast ult ⇒ no gain; enemy ult ⇒ no gain.

## Step 8 — Add `holy_aegis` (Patamon passive)

**New impl** (extends `patamon.rs` or sibling file):

```rust
pub struct HolyAegisBlueprint;
impl Blueprint for HolyAegisBlueprint {
    fn id(&self) -> BlueprintId { BlueprintId("holy_aegis") }
    fn on_event(&self, event: &CombatEvent, ctx: &BlueprintCtx) -> Vec<Effect> {
        match &event.kind {
            CombatEventKind::CombatStarted => {
                vec![Effect::ApplyBuff {
                    id: "holy_aegis".into(),
                    target: TargetRef::AoE { side: AllyTeamInclSelf },
                    mul: Some(0.10),
                    kind: BuffKind::DR,
                    dur: BuffDuration::Permanent,
                }]
            }
            CombatEventKind::UnitDied { unit, .. } if *unit == ctx.find_unit_id_for_blueprint(self.id()).unwrap() => {
                vec![Effect::EmitCleanse {
                    target: TargetRef::AoE { side: AllyTeam },
                    count: u8::MAX,
                    filter: CleanseFilter::ById("holy_aegis".into()),
                    priority: CleansePriority::OldestFirst,
                }]
            }
            _ => vec![],
        }
    }
}
```

**Prerequisite SP1 work:**
- `CombatEventKind::CombatStarted` must exist (likely already present; verify).
- `CombatEventKind::UnitDied { unit, killer }` must exist (SP1 §"Reactive signature bus").
- SP3 add-now: `Effect::ApplyBuff`, `BuffKind::DR`, `BuffDuration::Permanent`, `Effect::EmitCleanse`, `CleanseFilter::ById`, `TargetRef::AoE { side: AllyTeamInclSelf }`.

**Test impact:** new integration test `tests/holy_aegis_listener.rs` (~120 lines).

## Compat-shim strategy summary

During Steps 1–5, every migration follows this pattern to keep existing tests green:

1. **Keep the static `BLUEPRINTS` array** with the un-migrated blueprints.
2. `transitions_for_action_checked(signal, action)` checks the registry first; if no match, falls through to the static array.
3. `ValidationSnapshot` keeps the 5 typed fields as primary; the new `blueprint_snapshots: HashMap` is **additive** during migration. Step 6 inverts the relationship.
4. Each `apply_*_transitions_system` is **unchanged** throughout. The trait does not touch the apply-side pipeline.
5. The legacy Digimon-specific kernel-transition variants (`CombatKernelTransition::TwinCore(_)` etc.) are **preserved** indefinitely. They are the canonical typed output for the 5 stateful blueprints. New blueprints emit `Effect` cascades, not new typed transitions.

## Test inventory (`tests/` files that touch blueprints)

Result of `rg "twin_core|predator_loop|battery_loop|holy_aegis|kitsune_grace|fur_cloak|holy_support|precision_mind_game" tests/`:

| Test | Blueprint(s) | Expected impact |
|---|---|---|
| `tests/twin_core_integration.rs` | twin_core (agumon+gabumon) | no-op |
| `tests/twin_core_mechanics.rs` | twin_core | no-op |
| `tests/twin_core_resolution.rs` | twin_core | no-op |
| `tests/twin_core_roster_contract.rs` | twin_core | no-op |
| `tests/predator_loop_kernel.rs` | dorumon | no-op |
| `tests/dorumon_blueprint.rs` | dorumon | no-op |
| `tests/dorumon_predator_runtime.rs` | dorumon | no-op |
| `tests/battery_loop_kernel.rs` | tentomon | no-op |
| `tests/tentomon_blueprint.rs` | tentomon | no-op |
| `tests/holy_support_affordance.rs` | patamon | no-op |
| `tests/holy_support_mechanics.rs` | patamon | no-op |
| `tests/holy_support_resolution.rs` | patamon | no-op |
| `tests/holy_support_roster_contract.rs` | patamon | no-op |
| `tests/patamon_blueprint_seam.rs` | patamon | no-op |
| `tests/patamon_revive.rs` | patamon | no-op |
| `tests/renamon_precision_runtime.rs` | renamon | no-op |
| `tests/digimon_signal_registry.rs` | static BLUEPRINTS array | **signature change** — must iterate registry |
| `tests/validation_snapshot.rs` | typed snapshot fields | **one-line accessor switch** at Step 6 |
| `tests/combat_cli_shared_surface.rs` | snapshot serialization | **stable serde** required across rewrite |
| `tests/follow_up_chains.rs` | indirect (via twin_core triggers) | no-op |
| `tests/bootstrap_spawn_composition.rs` | indirect (party setup) | no-op |
| `tests/tempo_resistance.rs` | indirect (turn manip) | no-op |
| `tests/roster_smoke.rs` | full smoke | no-op |
| `tests/presentation_metadata_boundary.rs` | indirect (snapshot keys) | possible serde-key adjustment |

**Tally:** 23 affected files. **20 no-op**, **3 require one-line surface changes** (`digimon_signal_registry`, `validation_snapshot`, `combat_cli_shared_surface`). No semantic test changes; only API-surface shape.

Two **new** test files added in Steps 7 + 8: `tests/kitsune_grace_listener.rs`, `tests/holy_aegis_listener.rs`.

## Estimated PR/slice breakdown for M017

- **Slice A (Step 0)**: introduce trait, registry, snapshot enum. ~150 LOC, zero behavioural change.
- **Slice B (Steps 1+2)**: migrate tentomon + patamon. ~200 LOC delta, all tests stay green via shims.
- **Slice C (Steps 3+4)**: migrate dorumon + renamon. ~250 LOC delta.
- **Slice D (Step 5)**: migrate agumon + gabumon (paired). ~250 LOC delta.
- **Slice E (Step 6)**: drop shims, deprecate typed snapshot fields. ~100 LOC delta, ~15 lines of test-surface fixup.
- **Slice F (Step 7)**: add kitsune_grace listener. Depends on SP1 `UltimateUsed` event + SP3 `AdvanceTurn` effect landed.
- **Slice G (Step 8)**: add holy_aegis listener. Depends on SP1 `UnitDied` + `CombatStarted` events + SP3 `ApplyBuff`/`EmitCleanse`/`BuffKind::DR`/`BuffDuration::Permanent` effects landed.

Slices F + G can run in parallel after E lands. A–E is a strict chain.

## Out of scope for migration

- Converting per-blueprint state from `Resource` to `Component<UnitId>` (multi-instance support). Deferred per RESEARCH.md §"Surprises" #2.
- Hot-reloadable blueprint RON (would require Option C; explicitly rejected).
- Removing the typed `CombatKernelTransition::TwinCore`/etc. variants. Kept indefinitely as canonical typed outputs.
