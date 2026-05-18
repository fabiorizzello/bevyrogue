# S10 Research — Patamon + Renamon migrated + kernel digimon-free

## Summary
Depth: **targeted research**. The codebase already has the post-S08/S09 migration pattern for Dorumon, Tentomon, and Twin Core, but **Patamon and Renamon still terminate in typed kernel transitions/resources**, and the broader grep gate for a digimon-free kernel is still far from green. The biggest blockers are `src/combat/kernel.rs`, `src/combat/observability.rs`, `src/combat/events.rs`, `src/combat/battery_loop.rs`, and `src/combat/precision_mind_game.rs`.

Important scope finding for the planner: **S10 is not only “wire Patamon + Renamon like Dorumon.”** If S10 must satisfy its stated after-this proof (`rg ... src/combat/ --glob '!blueprints/**' -> 0`), then it also needs the final cleanup of earlier typed seams still living outside `blueprints/`. Deferring that cleanup to S12 would leave S10 unverifiable against its own exit criterion.

## Skills Discovered
- No new skill install needed.
- Relevant preinstalled skills already cover the core stack:
  - `bevy`
  - `rust-best-practices`
  - `verify-before-complete`

## Recommendation
Use the **existing Blueprint-envelope pattern** from Dorumon/Tentomon/Twin Core as the migration template:
1. Patamon custom signals should emit `CombatKernelTransition::Blueprint { owner: "patamon", ... }` instead of `CombatKernelTransition::HolySupport(...)`.
2. Renamon custom signals should emit `CombatKernelTransition::Blueprint { owner: "renamon", ... }` instead of `CombatKernelTransition::PrecisionMindGame(...)`.
3. The Patamon/Renamon runtime state machines should be **owned by blueprint modules**, decode only their own `owner`, and preserve any useful typed local transition structs/snapshots inside `src/combat/blueprints/**`.
4. To actually hit the grep gate, remove remaining digimon-named seams from `kernel.rs`, `events.rs`, `observability.rs`, `mod.rs`, and any standalone digimon runtime modules outside `blueprints/`.

The safest sequence is: **Patamon transport → Renamon transport → kernel/event cleanup → observability cleanup → grep proof**.

## Implementation Landscape

### Existing good pattern to copy
- `src/combat/blueprints/dorumon/signals.rs`
  - `dispatch()` returns `CombatKernelTransition::Blueprint` only.
- `src/combat/blueprints/dorumon/identity.rs`
  - Decodes `Blueprint { owner, name, payload }`, applies local typed state, then emits preserved typed resolved seam `CombatEventKind::PredatorLoopResolved`.
- `src/combat/blueprints/twin_core/mod.rs`
  - Blueprint-owned plugin, hook, state, and applier all live under `blueprints/`.
- `src/combat/battery_loop.rs`
  - Already decodes Tentomon `Blueprint` transitions, but the file itself still sits outside `blueprints/`, so it still fails the grep gate.

### Patamon current state
- `src/combat/blueprints/patamon/signals.rs`
  - Still dispatches `CombatKernelTransition::HolySupport(HolySupportTransition::build_grace(...))`.
- `src/combat/blueprints/patamon/identity.rs`
  - Owns `HolySupportState`, `HolySupportSnapshot`, hook logic, and applier system, but they still depend on typed `HolySupport*` transitions defined in `kernel.rs`.
- `src/combat/blueprints/patamon/mod.rs`
  - Already has `PatamonPlugin` and passive runtime registration; ownership is mostly in the right place.

### Renamon current state
- `src/combat/blueprints/renamon.rs`
  - Still dispatches `CombatKernelTransition::PrecisionMindGame(...)` for custom signals.
  - Passive runtime exists here already (`register_passive_runtime`).
- `src/combat/precision_mind_game.rs`
  - Entire state machine and applier are still outside `blueprints/` and named after Renamon’s mechanic.
- `src/combat/kernel.rs`
  - Still registers `PrecisionMindGameHook`, `PrecisionMindGameState`, and its applier system directly.
  - There is **no Renamon plugin** analogous to `TwinCorePlugin` / `PatamonPlugin` yet.

### Kernel/event surfaces still blocking digimon-free grep
- `src/combat/kernel.rs`
  - `CombatKernelTransition` still includes `BatteryLoop`, `HolySupport`, `PredatorLoop`, and `PrecisionMindGame` variants.
  - `HolySupport*` and `PrecisionMindGame*` types are still declared here.
  - `register_combat_kernel_runtime()` still wires digimon-specific resources/systems/hooks.
- `src/combat/events.rs`
  - Still re-exports `BatteryLoopTransition` / `PredatorLoopTransition` and carries `BatteryLoopResolved` / `PredatorLoopResolved` named event variants.
- `src/combat/observability.rs`
  - Largest remaining offender: named snapshot fields (`twin_core`, `holy_support`, `predator_loop`, `battery_loop`, `precision_mind_game`) and digimon-specific formatters.
- `src/combat/mod.rs`
  - Still exposes named digimon mechanics in public module docs/re-exports (`battery_loop`, `precision_mind_game`).

### Data / roster seams relevant but probably not S10’s first proof
- `src/data/units_ron.rs`
  - `UnitDef` still has hardcoded `twin_core` and `holy_support` metadata fields.
- `src/combat/bootstrap.rs`
  - Manual `UnitDef` constructors still initialize `twin_core` / `holy_support` defaults.
- These are explicitly aligned with later milestone scope (`C2`, `C3`) and are not the first thing to touch unless S10 expands beyond kernel grep.

### Verification scaffolding already present
- `tests/patamon_blueprint_seam.rs`
  - Confirms Patamon custom signal parsing/dispatch and runtime state update.
- `tests/holy_support_resolution.rs`
  - Confirms Patamon ultimate → transition → state/snapshot path.
- `tests/compiled_timeline_tohakken.rs`
  - Confirms Renamon timeline ordering and signal registration path.
- `tests/digimon_signal_registry.rs`
  - Confirms Renamon dispatch currently routes to `PrecisionMindGame` transitions.
- `tests/validation_snapshot.rs`
  - Confirms current named snapshot surfaces; this will need updates if S10 removes named observability fields.
- `tests/combat_cli_shared_surface.rs`
  - Useful smoke proof for shared surfaces from the binary; currently checks `holy_support=grace=` in CLI output, so it will definitely need revision if observability becomes generic.

## Natural Seams
1. **Patamon transport seam**
   - Files: `src/combat/blueprints/patamon/signals.rs`, `src/combat/blueprints/patamon/identity.rs`, `src/combat/blueprints/patamon/mod.rs`, Patamon tests.
   - Goal: switch raw dispatch to `Blueprint` owner envelope while keeping local Patamon state deterministic.

2. **Renamon transport + ownership seam**
   - Files: `src/combat/blueprints/renamon.rs`, `src/combat/precision_mind_game.rs`, `src/combat/kernel.rs`, Renamon tests.
   - Goal: move Renamon runtime ownership out of kernel; likely introduce a Renamon plugin or split `renamon.rs` into a module directory similar to Patamon.

3. **Kernel/event cleanup seam**
   - Files: `src/combat/kernel.rs`, `src/combat/events.rs`, `src/combat/mod.rs`.
   - Goal: remove named digimon transition variants/types from kernel-facing shared surfaces.
   - Highest ambiguity: whether typed resolved events remain in `CombatEventKind` or move to a generic blueprint-resolved seam.

4. **Observability cleanup seam**
   - Files: `src/combat/observability.rs`, `tests/validation_snapshot.rs`, `tests/combat_cli_shared_surface.rs`, any snapshot-focused tests.
   - Goal: eliminate digimon names from shared observability code. This is the largest proof blocker after `kernel.rs`.

## First Proof
The first high-risk proof is **not a test run**; it is an architectural grep proof:
- After planning the edits, the planner should expect a checkpoint where
  `rg -E "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/ --glob '!blueprints/**'`
  drops sharply or reaches zero.

Reason: current counts show the kernel-free claim is still structurally false, and most of the work is naming/surface ownership cleanup, not raw behavior.

Current rough remaining counts from research scan:
- total grep hits in `src/combat/` excluding `blueprints/**`: **339**
- top files:
  - `src/combat/observability.rs`: **128**
  - `src/combat/battery_loop.rs`: **99**
  - `src/combat/precision_mind_game.rs`: **59**
  - `src/combat/kernel.rs`: **39**

## Risks / Watch-outs
- **Scope trap:** S10 title sounds like a two-digimon migration, but the after-this proof implies a broader shared-surface cleanup.
- **Observability blast radius:** `tests/validation_snapshot.rs` and `tests/combat_cli_shared_surface.rs` are coupled to named snapshot text. Expect large assertion rewrites.
- **Resolved-event seam decision:** S09 preserved typed resolved seams. If that rule still stands, planner must decide whether those seams move under blueprint-owned message types or become generic `BlueprintResolved` events. Keeping `BatteryLoopResolved` / `PredatorLoopResolved` inside `CombatEventKind` will fail the grep gate.
- **Renamon ownership gap:** unlike Patamon/Twin Core/Dorumon/Tentomon, Renamon has no plugin-owned runtime registration yet.
- **Compile-time ext registration footgun:** `src/combat/blueprints/mod.rs::register_all_blueprint_exts()` currently only registers Agumon ext points. If S10 introduces new blueprint compile-time timeline refs, that registration path may need expansion.
- **Cross-slice tension with S12:** if planner wants to defer generic snapshot/registry work, it must explicitly note that S10 then cannot honestly claim the grep-based “kernel digimon-free” exit criterion.

## Verification
Recommended verification ladder for the execution slice:
1. **Structural proofs**
   - `rg -E "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/ --glob '!blueprints/**'`
   - `rg "enum Effect" src/data/skills_ron.rs`
2. **Targeted behavior tests**
   - `cargo test --test patamon_blueprint_seam`
   - `cargo test --test holy_support_resolution`
   - `cargo test --test compiled_timeline_tohakken`
   - `cargo test --test digimon_signal_registry`
   - `cargo test --test validation_snapshot`
   - `cargo test --test combat_cli_shared_surface`
3. **Regression tests for earlier migrated owners**
   - `cargo test --test battery_loop_kernel`
   - `cargo test --test dorumon_predator_runtime`
   - `cargo test --test event_stream`
4. **Build proofs**
   - `cargo check`
   - `cargo check --features windowed`
5. **Smoke proof**
   - If a windowed/manual proof is required for S10 closure, use the existing binary path as a cheap pre-smoke (`cargo test --test combat_cli_shared_surface`) before any human UI encounter run.

## Planner Notes
- The cleanest planner decomposition is probably **4 tasks**, not 1–2:
  1. Patamon transport migration.
  2. Renamon transport + runtime ownership migration.
  3. Shared kernel/event surface cleanup.
  4. Observability/test snapshot cleanup + grep proof.
- If task 3 stalls on resolved-event design, resolve that decision early; otherwise task 4 will thrash.
