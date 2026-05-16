# S09 Research: Dorumon + Tentomon migrated (Predator Loop + Battery Loop)

## Executive Summary

S09 is **targeted research**, not deep research. The core mechanics already exist and are covered by deterministic tests; the real work is to migrate the remaining **Dorumon/Tentomon kernel-coupled transition paths** onto the same generic `CombatKernelTransition::Blueprint { owner, name, payload }` pattern used by S08 Twin Core, while preserving the current state machines and event/observability surfaces.

The highest-risk finding is simple: **Dorumon and Tentomon still dispatch digimon-specific kernel transition variants**.

- `src/combat/blueprints/dorumon/signals.rs` maps custom signals to `CombatKernelTransition::PredatorLoop(...)`.
- `src/combat/blueprints/tentomon.rs` maps custom signals to `CombatKernelTransition::BatteryLoop(...)`.
- `src/combat/kernel.rs` still owns the `BatteryLoop*` and `PredatorLoop*` enums/structs plus the `CombatKernelTransition::{BatteryLoop, PredatorLoop}` variants.

That is the exact same class of coupling S08 removed for Twin Core. The clean migration template already exists in `src/combat/blueprints/twin_core/mod.rs`: emit generic `Blueprint` transitions, let the blueprint-owned runtime system decode `owner/name/payload`, and keep the state machine local to the blueprint module.

The slice-level success criteria line up with that migration:
- **“Predator Loop write in JSONL”** → Dorumon signals should round-trip through generic `Blueprint` transition events in the combat log, not through `CombatKernelTransition::PredatorLoop`.
- **“Battery Loop deterministico”** → Tentomon must keep its existing deterministic charge/block behavior after the event-path rewrite.

## Requirements / Constraints

Relevant slice support from preloaded context:
- M021 success criterion eventually requires `rg "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/ --glob '!blueprints/**'` → 0.
- S08 already validated the pattern for removing a digimon-specific transition variant in favor of `Blueprint { owner, ... }`.
- Headless-first still applies: no windowed-only wiring in the runtime path.

Useful durable context from memory:
- **MEM002:** passive listeners that enqueue state changes must flush between runner steps; do not regress PassiveRunner semantics while touching Dorumon/Tentomon.
- **MEM004:** `BlockReactionTriggered` is already the canonical event surface for Tentomon-style passive mitigation; preserve it.
- **MEM006:** canonical passives are bootstrapped from runtime/plugin wiring using fixed owners/UnitIds; do not reintroduce per-test setup.
- **MEM007:** Twin Core already proved the generic `Blueprint` owner path through `CombatKernelTransition`.

## Skills Discovered

Directly relevant installed skills already present in the environment:
- `bevy`
- `rust-best-practices`

No additional external skill discovery was needed; the work is entirely inside the local Bevy/Rust combat framework.

## Implementation Landscape

### 1. The migration target already exists: Twin Core is the reference pattern

`src/combat/blueprints/twin_core/mod.rs` is the best reference for S09.

What it does:
- emits `CombatKernelTransition::Blueprint { owner, name, payload }` from hooks
- consumes those generic transitions inside `apply_twin_core_transitions_system`
- keeps the state machine local to the blueprint module
- leaves `kernel.rs` free of `TwinCore*` transition variants

This is the pattern S09 should copy for both Predator Loop and Battery Loop.

### 2. Dorumon is still on the old digimon-specific kernel path

Relevant files:
- `src/combat/blueprints/dorumon/signals.rs`
- `src/combat/blueprints/dorumon/mod.rs`
- `src/combat/blueprints/dorumon/hooks.rs`
- `src/combat/blueprints/dorumon/identity.rs`
- `tests/dorumon_blueprint.rs`
- `tests/dorumon_predator_runtime.rs`
- `tests/predator_loop_kernel.rs`

Current behavior:
- `signals.rs` dispatches `build_exploit`, `apply_prey_lock`, `consume_prey_lock_payoff`, `enter_berserk` into `CombatKernelTransition::PredatorLoop(...)`.
- `identity.rs::apply_predator_loop_transitions_system` listens for `CombatEventKind::OnKernelTransition`, matches `CombatKernelTransition::PredatorLoop(...)`, applies state, then emits `CombatEventKind::PredatorLoopResolved { transition }`.
- `hooks.rs` adds the cycle-tick transition by pushing `CombatKernelTransition::PredatorLoop(PredatorLoopTransition::tick())` on wrapped tactical cycles.
- Runtime/plugin ownership is already correct: `DorumonPlugin` owns resource init + transition applier system + hook registration.

Important nuance:
- The **state machine itself is already blueprint-local** (`PredatorLoopState` lives under `blueprints/dorumon/identity.rs`).
- The coupling problem is mainly the **event envelope and kernel-owned transition types**.

### 3. Tentomon is still on the old digimon-specific kernel path

Relevant files:
- `src/combat/blueprints/tentomon.rs`
- `src/combat/battery_loop.rs`
- `tests/tentomon_blueprint.rs`
- `tests/battery_loop_kernel.rs`
- `tests/passive_reactive_canon.rs`

Current behavior:
- `tentomon.rs::dispatch()` maps `build_static_charge`, `build_circuit_charge`, `spend_circuit_charge` into `CombatKernelTransition::BatteryLoop(...)`.
- `src/combat/battery_loop.rs::apply_battery_loop_transitions_system` listens for `CombatEventKind::OnKernelTransition`, matches `CombatKernelTransition::BatteryLoop(...)`, and updates `BatteryLoopState`.
- Tentomon’s passive block reaction is separate and already deterministic through `IncomingDamage` + `DamageModifierLedger` + `CombatRng`.

Important nuance:
- The **deterministic behavior is already proven** in `tests/battery_loop_kernel.rs` and `tests/passive_reactive_canon.rs`.
- S09 should avoid redesigning the Battery Loop rules; it should primarily rewrite the transition/event path.

### 4. Kernel/event/observability still expose Dorumon/Tentomon-specific transition types

Kernel-coupled files:
- `src/combat/kernel.rs`
- `src/combat/events.rs`
- `src/combat/observability.rs`
- `src/combat/api/applier.rs`

Current coupling surfaces:
- `kernel.rs` defines `BatteryLoopSignal`, `BatteryLoopTransition`, `PredatorLoopSignal`, `PredatorLoopTransition` and includes `CombatKernelTransition::{BatteryLoop, PredatorLoop}`.
- `events.rs` re-exports those types and defines typed resolved events:
  - `BatteryLoopResolved { transition: BatteryLoopTransition }`
  - `PredatorLoopResolved { transition: PredatorLoopTransition }`
- `observability.rs` formats those typed transitions directly.
- `api/applier.rs` contains Tentomon-specific pre-damage mitigation wiring that directly reaches `BatteryLoopState`, but that part is not inherently a kernel-transition coupling problem and should probably stay as-is for this slice.

Implication:
- The slice likely needs to move the `BatteryLoop*` and `PredatorLoop*` transition types out of `kernel.rs` and re-home imports in `events.rs`/`observability.rs`, similar to what S08 did for Twin Core.
- The resolved event types can remain typed if desired; the roadmap only requires the **raw kernel transition write** to go generic.

### 5. Asset/timeline coverage is mixed; don’t over-scope active-skill migration

Quick scan of `assets/data/skills.ron`:
- `dorumon_ult` → **no timeline**, emits 2 Dorumon custom signals
- `dorumon_follow_up` → **timeline present**
- `tentomon_basic`, `petit_thunder`, `tentomon_follow_up` → **timelines present**
- `mega_blaster`, `kabuterimon_follow_up`, `power_metal` → **no timeline**
- `tentomon_ult` (`Electro Shocker`) exists under the legacy compatibility id and currently has no timeline

Interpretation:
- S09’s roadmap line does **not** require broad new timeline compilation work for every Dorumon/Tentomon-family skill.
- The slice should stay focused on **Blueprint transition migration + deterministic regression coverage**, unless the planner explicitly wants a small timeline add for one slice-specific test.

### 6. Current tests already separate the three concerns nicely

Existing seams:
- **dispatch mapping tests**
  - `tests/dorumon_blueprint.rs`
  - `tests/tentomon_blueprint.rs`
- **runtime state application tests**
  - `tests/dorumon_predator_runtime.rs`
  - `tests/battery_loop_kernel.rs`
- **passive deterministic behavior tests**
  - `tests/passive_reactive_canon.rs`
- **event stream / JSONL style tests**
  - `tests/event_stream.rs`
  - Twin Core / bouncing-fire tests already demonstrate how to assert on generic `Blueprint` transitions

This is a strong task seam: rewrite dispatch first, then runtime consumer, then log/assert surfaces.

## Recommendation

Use the **Twin Core migration template** exactly:

1. **Dorumon path**
   - change `signals.rs` and cycle-hook emission to produce `CombatKernelTransition::Blueprint { owner: "dorumon", name: ..., payload: ... }`
   - update `apply_predator_loop_transitions_system` to decode generic blueprint transitions for `owner == "dorumon"`
   - keep `PredatorLoopState` and its typed resolved event behavior unchanged if possible

2. **Tentomon path**
   - change `tentomon.rs::dispatch()` to emit `CombatKernelTransition::Blueprint { owner: "tentomon", name: ..., payload: ... }`
   - update `battery_loop.rs::apply_battery_loop_transitions_system` to decode generic blueprint transitions for `owner == "tentomon"`
   - keep the passive block-reaction logic untouched except for any necessary import moves

3. **Then remove kernel-local variant ownership**
   - strip `CombatKernelTransition::{BatteryLoop, PredatorLoop}` from `kernel.rs`
   - move `BatteryLoop*` / `PredatorLoop*` transition types to the owning blueprint modules or adjacent blueprint-owned files
   - update `events.rs`/`observability.rs` imports accordingly

4. **Prove the slice with raw transition logging first**
   - first proof should be that Dorumon/Tentomon actions now produce generic `Blueprint` transition writes in the event stream/JSONL, while state application still converges to the same typed resolved events and snapshots

## Natural Task Seams

### T01 — Dorumon raw transition envelope migration
Files:
- `src/combat/blueprints/dorumon/signals.rs`
- `src/combat/blueprints/dorumon/hooks.rs`
- `src/combat/blueprints/dorumon/identity.rs`
- `tests/dorumon_blueprint.rs`
- `tests/dorumon_predator_runtime.rs`

Goal:
- Replace `CombatKernelTransition::PredatorLoop(...)` emission with `CombatKernelTransition::Blueprint { owner: "dorumon", ... }`
- Decode generic transitions in the runtime applier
- Keep `PredatorLoopResolved` and `PredatorLoopState` semantics stable

Why first:
- It directly satisfies the roadmap’s “Predator Loop write in JSONL” requirement.

### T02 — Tentomon raw transition envelope migration
Files:
- `src/combat/blueprints/tentomon.rs`
- `src/combat/battery_loop.rs`
- `tests/tentomon_blueprint.rs`
- `tests/battery_loop_kernel.rs`

Goal:
- Replace `CombatKernelTransition::BatteryLoop(...)` emission with generic `Blueprint` transitions
- Decode generic transitions in `apply_battery_loop_transitions_system`
- Preserve deterministic battery charge/block behavior

### T03 — Kernel/event type cleanup
Files:
- `src/combat/kernel.rs`
- `src/combat/events.rs`
- `src/combat/observability.rs`
- any imports broken by moving transition enums/structs out of `kernel.rs`

Goal:
- remove kernel-local ownership of Dorumon/Tentomon-specific transition envelopes
- keep typed resolved-event formatting intact

### T04 — Slice-level regression assertions
Files:
- `tests/event_stream.rs`
- `tests/dorumon_predator_runtime.rs`
- `tests/battery_loop_kernel.rs`
- optionally a new dedicated Dorumon/Tentomon blueprint-event regression test if current coverage becomes awkward

Goal:
- assert generic `Blueprint` writes appear in raw event stream
- assert resolved state/events remain deterministic

## First Proof

The best first proof is **not** a large cargo test run. It is one narrow runtime assertion per blueprint:

1. Dorumon action with `build_exploit` + `apply_prey_lock`
   - raw `OnKernelTransition` events contain `CombatKernelTransition::Blueprint { owner: "dorumon", ... }`
   - post-application events still contain `PredatorLoopResolved`
   - `PredatorLoopState` snapshot matches current expectations

2. Tentomon action with `build_static_charge`
   - raw `OnKernelTransition` event contains `CombatKernelTransition::Blueprint { owner: "tentomon", name: "build_static_charge", ... }`
   - `BatteryLoopState` still increments once

If those two pass, the rest is mostly cleanup and grep debt removal.

## Risks / Watch-outs

- **Do not rework passive logic while migrating event envelopes.** Tentomon block reaction is already deterministic and shares the `BlockReactionTriggered` surface with generic passive mitigation.
- **Preserve Dorumon target-tracking setup assumptions.** `PredatorLoopState` still requires tracked targets before some transitions; existing tests explicitly seed that state.
- **Expect wide import fallout.** `events.rs`, `observability.rs`, and multiple tests import `BatteryLoop*` / `PredatorLoop*` from `kernel.rs` today.
- **The P001 grep is not a slice-level done signal by itself.** `PrecisionMindGame` and `HolySupport` still legitimately exist outside blueprints before S10, so use slice-local greps carefully.
- **Don’t over-scope into skill timelines.** `dorumon_ult`, `tentomon_ult`, `mega_blaster`, `power_metal`, etc. still lack timelines, but that is not the critical blocker for S09’s stated demo.

## Verification

Recommended narrow commands for the executor:

- `cargo test --test dorumon_predator_runtime`
- `cargo test --test dorumon_blueprint`
- `cargo test --test tentomon_blueprint`
- `cargo test --test battery_loop_kernel`
- `cargo test --test passive_reactive_canon`
- `cargo test --test event_stream`
- `cargo check`
- `cargo check --features windowed`

Useful slice-specific grep checks:
- `rg -n "CombatKernelTransition::PredatorLoop|CombatKernelTransition::BatteryLoop" src tests`
- `rg -n "owner: \"dorumon\"|owner: \"tentomon\"" tests src/combat`

## Planner Notes

This slice is mostly a **mechanical architecture migration with strong existing tests**. The safest decomposition is:
- Dorumon generic envelope
- Tentomon generic envelope
- kernel/event import cleanup
- regression verification

Avoid mixing “make more skills timeline-backed” into the same task unless a failing verification forces it.
