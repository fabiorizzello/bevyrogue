---
spike: SP2
name: Blueprint extension API stabilization
status: complete
created: 2026-05-12
completed: 2026-05-12
depends_on: SP1, SP3
inputs:
  - src/combat/blueprints/ (6 blueprints: agumon, gabumon, dorumon, tentomon, patamon, renamon)
  - src/combat/kernel.rs (CombatKernelTransition + CombatKernelRegistry + CombatKernelHook)
  - src/combat/{twin_core,predator_loop,battery_loop,holy_support,precision_mind_game}.rs (state resources)
  - src/combat/observability.rs (ValidationSnapshot)
  - src/combat/turn_system/pipeline.rs (transitions_for_action_checked seam)
  - src/data/skills_ron.rs (SkillCustomSignal / CustomSignalPayload envelope)
  - SP1 output (.gsd/spikes/spike-kernel-primitives/RESEARCH.md)
  - SP3 output (.gsd/spikes/spike-skill-dsl-coverage/RESEARCH.md + gaps.md)
outputs:
  - INTERFACE-OPTIONS.md (A/B/C with concrete API sketches)
  - DECISION.md (Option B chosen, rationale)
  - migration_plan.md (order, shims, test impact)
  - sketches/blueprint_trait.rs (non-compiled)
  - sketches/registry.rs (non-compiled)
  - sketches/kitsune_grace.rs (non-compiled)
  - sketches/holy_aegis.rs (non-compiled)
---

# SP2 — Blueprint extension API stabilization

## Goal

Formalize a single, uniform blueprint plugin contract. Six blueprints already exist in `src/combat/blueprints/` and grew opportunistically. Before adding `kitsune_grace` and `holy_aegis` (Renamon/Patamon passives — round-3 canon), and before M017 lands new `CombatEventKind` variants from SP1 (`UnitDied`, `UltimateUsed`, `Healed`, `SpGranted`, `IncomingDamage`, `BlockReactionTriggered`, `AdvanceTurn`, `DelayTurn`) and new RON `Effect`s from SP3 (`ApplyBuff`, `EmitHeal`, `EmitCleanse`, `EmitSpGrant`, `BlockReaction`, `SetBlueprintState`), lock the seam.

## Survey — existing blueprints

All six files were read end-to-end. The shape is uniform: each is a `pub const OWNER: &str = "<name>";` plus a `pub fn dispatch(signal: &SkillCustomSignal, action: &ResolvedAction) -> Result<Vec<CombatKernelTransition>, CustomSignalDispatchError>` function. There is **no trait**; coupling happens by convention.

### Signal envelope status

`CustomSignalPayload` has stabilized to a single generic shape: `{ Empty, Amount { amount: i32 } }` (`src/data/skills_ron.rs:118-121`). All six blueprints consume this envelope; **no blueprint has a Digimon-specific payload variant in the enum.** The historical "legacy Digimon-specific variants" referenced in memory are now collapsed: the discriminator is the `(owner, signal_name)` string pair plus the optional `Amount{i32}` payload. The migration to a generic envelope on the payload side is **complete**.

### Kernel transition output is NOT yet generic

The asymmetry: `CombatKernelTransition` (`src/combat/kernel.rs:890-902`) is still a Rust enum with five Digimon-specific variants — `TwinCore(TwinCoreTransition)`, `BatteryLoop(BatteryLoopTransition)`, `HolySupport(HolySupportTransition)`, `PredatorLoop(PredatorLoopTransition)`, `PrecisionMindGame(PrecisionMindGameTransition)` — plus four kernel-domain variants (`TacticalCycle`, `Strain`, `Flow`, `Fatigue`, `Tag`, `Beat`). Adding a 7th blueprint today requires:

1. Adding a new variant to `CombatKernelTransition`.
2. Adding a new `*State` resource + `*Hook` impl in a sibling module.
3. Adding the resource + hook + system to `register_combat_kernel_runtime` (`kernel.rs:1067-1092`).
4. Adding the blueprint module + dispatcher (`src/combat/blueprints/<name>.rs`).
5. Registering it in the static `BLUEPRINTS` array (`blueprints/mod.rs:107-132`).
6. Adding a snapshot field to `ValidationSnapshot` (`observability.rs:30-44`).
7. Wiring the snapshot read in `capture_validation_snapshot` (`observability.rs:269-307`).
8. Extending `format_validation_snapshot` (`observability.rs:310+`).

That is **eight touch points** for one blueprint. SP2 must collapse this.

### Per-blueprint table

| Blueprint | File | Signals in | State held | Kernel events out | Envelope shape |
|---|---|---|---|---|---|
| `agumon` | `agumon.rs` (63) | `apply_heated`, `apply_meltdown_crack`, `apply_thermal_spark` | `TwinCoreState` (shared with gabumon, **global resource**) | `CombatKernelTransition::TwinCore(_)` | generic `Amount{i32}` |
| `gabumon` | `gabumon.rs` (63) | `apply_chilled`, `apply_deep_crack`, `apply_thermal_spark` | shares `TwinCoreState` | `CombatKernelTransition::TwinCore(_)` | generic `Amount{i32}` |
| `dorumon` | `dorumon.rs` (55) | `build_exploit`, `apply_prey_lock`, `consume_prey_lock_payoff`, `enter_berserk` | `PredatorLoopState` (global resource) | `CombatKernelTransition::PredatorLoop(_)` | generic `Amount{i32}` + 2 `Empty` |
| `tentomon` | `tentomon.rs` (70) | `build_static_charge`, `build_circuit_charge`, `spend_circuit_charge` | `BatteryLoopState` (global resource) | `CombatKernelTransition::BatteryLoop(_)` | generic `Amount{i32}` |
| `patamon` | `patamon.rs` (57) | `build_holy_support_grace` | `HolySupportState` (global resource) | `CombatKernelTransition::HolySupport(_)` | generic `Amount{u8}` |
| `renamon` | `renamon.rs` (40) | `open_momentum_window`, `commit_precision_press`, `reveal_bait`, `resolve_precision_success` | `PrecisionMindGameState` (global resource) | `CombatKernelTransition::PrecisionMindGame(_)` | all 4 `Empty` |

**Audit findings (cross-cutting):**

- **Envelope is uniform.** Every blueprint uses the `SkillCustomSignal { owner, signal, payload: CustomSignalPayload }` envelope. Memory note about "Digimon-specific variants kept as compatibility shims alongside the generic envelope" appears to refer to the kernel-transition output enum (still Digimon-specific) plus the per-blueprint *internal* `*Signal` enum (Rust-side parse target, file-local). The signal *input* envelope has fully converged.
- **No trait.** Dispatch is a static `&[BlueprintRegistration]` array of `fn(&SkillCustomSignal, &ResolvedAction) -> Result<Vec<CombatKernelTransition>, _>` pointers. There is no trait abstraction.
- **State ownership: 100% global resources.** Every blueprint state (`TwinCoreState`, `PredatorLoopState`, `BatteryLoopState`, `HolySupportState`, `PrecisionMindGameState`) is a `Resource`, **not** a `Component`. This is because round-2 design treated each Digimon archetype as a singleton in the party (one Agumon at most). For round-3 enemy reuse (an enemy Agumon could exist), this assumption will need revision — flagged for SP1 follow-up, see "Surprises" below.
- **Listener filters live in apply systems**, not in blueprints. The blueprint files are pure parser+adapter: they take a `SkillCustomSignal` and emit a `CombatKernelTransition`. The actual reaction logic (e.g. "thermal_spark cross-resonance increment when partner is chilled") lives in `apply_twin_core_transitions_system` in `src/combat/twin_core.rs`. The blueprint module is the **mouth**, not the **brain**.
- **No Bevy `Plugin` per blueprint.** All blueprint registration is centralized in `register_combat_kernel_runtime`. There is no per-blueprint `Plugin` impl.
- **`ValidationSnapshot` is statically wired.** `observability.rs` imports each blueprint's `*Snapshot` type by name and reads the corresponding resource. No registry indirection.

## Open questions — answered

### Q1. Has `CustomSignalPayload::*` enumeration stabilized?

**Yes, fully.** Two variants total: `Empty` and `Amount { amount: i32 }`. All six blueprints share these. New blueprints **do not push new variants** — they reuse `Amount` or `Empty`. The only blueprint-specific thing on the input side is the `signal` string discriminator.

If a future blueprint needs a richer payload (e.g. multiple int fields, an enum tag), the natural extension is to add one more variant (e.g. `Tagged { tag: String, amount: i32 }`) or to encode the payload as multiple sequential `SkillCustomSignal` entries. SP2 does not need to expand the enum today; SP3 explicitly defers `multiplier_chain: Vec<ParamRef>` and `ParamRef::EventPayload` until a second skill needs them.

### Q2. Plugin registration in `src/main.rs`: trait+registry or match per blueprint?

**Neither in `main.rs`.** Registration is centralized in `register_combat_kernel_runtime` (`src/combat/kernel.rs:1067-1092`), called from `main.rs:64`. It is a flat sequence of:

- `registry.register(<Hook>)` × 5 (the typed-hook registry from `CombatKernelRegistry`).
- `app.init_resource::<*State>()` × 5.
- `add_systems(Update, (apply_*_transitions_system, ...))` × 5.

A typed registry (`CombatKernelRegistry`) **does exist** for the kernel-hook side, but it is **not** parameterised over blueprints — each `*Hook` impl is a hardcoded type. The blueprint-dispatch side (`blueprints/mod.rs::BLUEPRINTS`) is a separate static array. The two indices are not unified.

### Q3. Direct state write vs kernel-command emission?

**Direct state write, on the apply-system side.** The blueprint module emits a `CombatKernelTransition`; the corresponding `apply_<x>_transitions_system` consumes the transition and mutates the `*State` resource directly via `state.gain_static_charge(amount)` style methods. No further kernel-command indirection. The source of truth for blueprint state is the `Resource`; the transition is the **command** in CQRS terms.

This is clean and works. SP2 should preserve it: blueprints **return commands** (data), they do not mutate state directly.

### Q4. Suspend/resume cascade contract on the blueprint side?

**Not yet exercised.** The current code does not have a mid-cascade suspend mechanism — `transitions_for_action_checked` is called synchronously in `dispatch_blueprint_transitions` (`pipeline.rs:106-139`) and all transitions are emitted in one batch per action. Suspend/resume is M017 S03e scope (Follow-up FIFO subsume into generic `KernelEffect::EnqueueAction` cascade per SP1 §Follow-up).

**SP2 implication:** the blueprint contract must be **synchronous-safe and idempotent** — given the same `(SkillCustomSignal, ResolvedAction)` pair, the output `Vec<CombatKernelTransition>` must be deterministic and side-effect-free. When M017 S03e introduces cascade suspend, the engine will replay the transition emission on resume; blueprints with internal mutable state would break this. The current design satisfies this because blueprints are pure functions of their inputs.

### Q5. How does `ValidationSnapshot` know which blueprints to query?

**Static, hardcoded.** `capture_validation_snapshot` (`observability.rs:158-308`) literally lists each blueprint state by name:

```rust
let twin_core = world.get_resource::<TwinCoreState>()...
let holy_support = world.get_resource::<HolySupportState>().map(HolySupportSnapshot::from);
let predator_loop = world.get_resource::<PredatorLoopState>().map(|s| s.snapshot());
let precision_mind_game = world.get_resource::<PrecisionMindGameState>().map(PrecisionMindGameSnapshot::from);
let battery_loop = world.get_resource::<BatteryLoopState>().map(BatteryLoopSnapshot::from);
```

Adding a 7th blueprint requires editing this function. The `ValidationSnapshot` struct itself has a fixed field per blueprint (`twin_core`, `holy_support`, `predator_loop`, `battery_loop`, `precision_mind_game`). SP2 must address this via a `HashMap<BlueprintId, BlueprintSnapshotValue>` field or equivalent registry-driven approach.

## Interface options

See `INTERFACE-OPTIONS.md` for concrete API sketches of:

- **Option A — Bevy Plugin per blueprint.** Each blueprint implements `impl Plugin for X` and registers its own state, systems, and dispatch hook.
- **Option B — Trait + central registry.** A single `Blueprint` trait + `BlueprintRegistry` resource; blueprints register via `app.add_blueprint::<T>()`. Dispatch keyed by `BlueprintId`. State + snapshot exposure flows through the trait.
- **Option C — Pure-data + generic dispatch.** No per-blueprint Rust; state machines expressed in RON; generic interpreter system in kernel.

## Decision

See `DECISION.md`. **Option B (Trait + central registry).** Rationale in one sentence: SP3's hybrid conclusion (Effects in RON, listener filters in Rust) maps directly onto a `Blueprint` trait where the `dispatch` half is the listener filter and the returned `Vec<KernelTransition>` half is the effect cascade, and the audit shows that Option A's per-plugin model adds Bevy ceremony without solving the `ValidationSnapshot` registry gap while Option C's pure-data ambitions would require building a state-machine interpreter that SP3 explicitly does not need.

## Surprises that should update SP1/SP3

1. **The "Digimon-specific compat shims" memory note refers to the kernel-transition output enum, not the input envelope.** SP1's D-M017-EVENTS-BUS decision ("single bus") does not directly address whether `CombatKernelTransition` becomes an opaque `BlueprintTransition { blueprint_id: BlueprintId, payload: BlueprintPayload }` envelope or keeps its 5 typed variants. **SP2 recommendation: keep the typed variants for now** (zero code churn) and add a generic `Blueprint(BlueprintId, OpaquePayload)` variant later if a 7th blueprint with non-trivial typed output appears. The five existing variants are the closed canonical set; new blueprints in M017 (`kitsune_grace`, `holy_aegis`) emit only `CombatEvent`s and `Effect` cascades, not new typed kernel transitions, so no enum extension is needed for them. This **shrinks** SP1's scope: no new `CombatKernelTransition` variants needed in M017.

2. **State ownership = global resource is wrong for enemy reuse.** Every existing state resource assumes "at most one of this archetype in combat" (one Agumon, one Renamon, ...). Round-3 canon explicitly allows enemy Digimon to share archetypes with party Digimon. When this lands (post-M017), `TwinCoreState`/`PredatorLoopState`/etc. must become `HashMap<UnitId, _State>` or be migrated to per-unit `Component`s. **SP1 should record this as a deferred refactor** under D-M017-STATUS-REWRITE or a new decision (`D-M017-BLUEPRINT-STATE-PER-UNIT`). Not in M017 scope; flag for M018+.

3. **`kitsune_grace` and `holy_aegis` do not need new `CombatKernelTransition` variants.** Their behaviour is fully expressible as: (listener on `CombatEventKind::UltimateUsed` / `CombatStarted` / `UnitDied`) → (emit `Effect::AdvanceTurn` or `Effect::ApplyBuff` or `Effect::EmitCleanse` via the kernel cascade). They do not need a `KitsuneGrace(_)` or `HolyAegis(_)` kernel-transition variant. **The blueprint contract for them is "listener + effect cascade", with no state machine and no typed kernel transition.** This is materially simpler than Twin Core / Predator Loop / Battery Loop. **SP3 already concluded this**; SP2 confirms it from the API-design side.

4. **The `dispatch` half of the blueprint contract is mis-named.** Current `dispatch(signal, action) -> Vec<Transition>` is a **commit-time effect expansion**, not a **listener**. The kitsune_grace/holy_aegis additions are pure **listeners** (`on_event(CombatEvent, ...) -> Vec<Effect>`). Existing blueprints don't listen to `CombatEvent` at all — they only translate `SkillCustomSignal` into `KernelTransition`. The trait must cover **both** the synchronous commit-time expansion (existing 6) AND event-driven listening (new 2). SP2's `Blueprint` trait sketch has two methods: `commit_signals(...) -> Vec<KernelTransition>` and `on_event(...) -> Vec<Effect>`. Both are pure functions; either can be a no-op default.

## Method

1. Read all 6 blueprint files + `mod.rs` + `kernel.rs::register_combat_kernel_runtime` + `observability.rs::capture_validation_snapshot` + `turn_system/pipeline.rs::dispatch_blueprint_transitions` + `data/skills_ron.rs::CustomSignalPayload`. Done.
2. Cross-checked SP1 + SP3 conclusions. Done.
3. Wrote three concrete API sketches in `INTERFACE-OPTIONS.md`. Done.
4. Wrote `kitsune_grace.rs` and `holy_aegis.rs` non-compiled sketches under chosen option. Done.
5. Wrote migration plan. Done.

## Decision summary (proposed for `.gsd/DECISIONS.md`)

- **D-M017-BLUEPRINT-API**: "Adopt Option B (Trait + central registry). Define a `Blueprint` trait with `id() -> BlueprintId`, `commit_signals(signal, action) -> Vec<KernelTransition>` (default: empty), `on_event(event, ctx) -> Vec<Effect>` (default: empty), `snapshot(world) -> BlueprintSnapshot` (default: empty). A `BlueprintRegistry` resource owns `HashMap<BlueprintId, Box<dyn Blueprint>>`. The existing `CustomSignalPayload` envelope is preserved unchanged. The existing `CombatKernelTransition` enum keeps its 5 typed Digimon variants for back-compat; new blueprints (kitsune_grace, holy_aegis) emit `Effect` cascades, not new typed transitions. `ValidationSnapshot` gains a `blueprint_snapshots: HashMap<BlueprintId, BlueprintSnapshot>` field replacing the 5 hardcoded fields (compat-aliased during migration)."

## Out of scope (confirmed)

- Modifying `src/` (sketches only, throwaway, in `.gsd/spikes/spike-blueprint-api/sketches/`).
- Touching `docs/future_design_draft/` (canon lock-in).
- Implementing the migration (M017 slice work).
- Touching existing tests.
- Modifying SP1/SP3 outputs.

## See also

- `INTERFACE-OPTIONS.md` — three concrete API sketches with pros/cons.
- `DECISION.md` — chosen option (B) with rationale.
- `migration_plan.md` — sequencing for the 6 existing blueprints.
- `sketches/blueprint_trait.rs` — trait surface.
- `sketches/registry.rs` — registry plumbing.
- `sketches/kitsune_grace.rs` — Renamon passive under chosen API.
- `sketches/holy_aegis.rs` — Patamon passive under chosen API.
