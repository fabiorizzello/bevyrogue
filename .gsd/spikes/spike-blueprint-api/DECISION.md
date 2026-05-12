---
spike: SP2
related: RESEARCH.md, INTERFACE-OPTIONS.md, migration_plan.md
status: complete
created: 2026-05-12
decision_id: D-M017-BLUEPRINT-API
---

# SP2 — Decision

## Chosen design

**Option B — Trait + central registry.**

A single `Blueprint` trait. A `BlueprintRegistry` resource owning `HashMap<BlueprintId, Box<dyn Blueprint>>`. Three trait methods, each with a `Default::default()`-style empty implementation:

1. `fn id(&self) -> BlueprintId` — identity, used for routing and snapshot keying.
2. `fn commit_signals(&self, signal: &SkillCustomSignal, action: &ResolvedAction) -> Result<Vec<CombatKernelTransition>, CustomSignalDispatchError>` — commit-time effect expansion. Default: `Ok(Vec::new())`. The existing 6 blueprints override this.
3. `fn on_event(&self, event: &CombatEvent, ctx: &BlueprintCtx) -> Vec<Effect>` — event-driven listener half. Default: `Vec::new()`. The new 2 blueprints (kitsune_grace, holy_aegis) override this.
4. `fn snapshot(&self, world: &World) -> BlueprintSnapshot` — observability. Default: `BlueprintSnapshot::Empty`.

## Rationale

### 1. SP3's hybrid partition maps directly onto the trait

SP3 concluded: "Effects in RON, listener filters in Rust" (`spike-skill-dsl-coverage/RESEARCH.md` §134). The `Blueprint` trait expresses exactly this partition: `on_event` returns a `Vec<Effect>` (data) whose body is determined by listener filtering logic (Rust). The signal envelope is RON-declared. We do not invent new abstractions — we surface the partition SP3 already locked in.

### 2. Smallest migration cost given audit findings

The audit (RESEARCH.md §"Survey") shows all 6 existing blueprints already follow a uniform shape: `pub const OWNER: &str` + `pub fn dispatch(...)`. Converting to `impl Blueprint for X { fn id() { ... } fn commit_signals() { /* existing dispatch body */ } }` is a ~10-line-per-blueprint move. ~280 lines net churn (INTERFACE-OPTIONS.md §"Migration cost B"). Option A is ~290 lines but spread across more files. Option C is ~1700 lines and a new interpreter.

### 3. Addresses all five open questions

- **Q1 envelope:** `CustomSignalPayload` stays `{Empty, Amount{i32}}` — unchanged. The trait does not push payload variants.
- **Q2 main.rs registration:** `register_default_blueprints(app)` becomes the single registration site. The static `BLUEPRINTS` array in `blueprints/mod.rs` deletes.
- **Q3 state ownership:** unchanged — state resources remain global `Resource`s, mutated by the existing `apply_*_transitions_system` systems. The trait emits transitions; existing apply pipeline consumes them.
- **Q4 suspend/resume:** trait methods are required to be pure functions of inputs. The trait contract enforces SP1's S03e cascade-suspend safety by construction.
- **Q5 ValidationSnapshot:** registry-driven via `registry.snapshot_all(world) -> HashMap<BlueprintId, BlueprintSnapshot>`. Adding a 7th blueprint adds zero lines to `observability.rs`.

### 4. Default methods accommodate kitsune_grace and holy_aegis cleanly

The two new round-3 passives are listener-only — they have no commit-time signal expansion and (mostly) no state. Under Option B they implement `on_event` and inherit empty defaults for `commit_signals` and `snapshot`. Under Option A they would still need a Plugin shell (`init_resource`-less, system-less) that's mostly boilerplate. The default-methods affordance is the key ergonomic win.

### 5. Stays consistent with existing wiring style

`src/combat/` is manually-wired throughout (no per-module Bevy Plugins). Option A would introduce a Plugin idiom that exists nowhere else. Option B's `register_default_blueprints` follows the pattern of `register_combat_kernel_runtime` already in `kernel.rs:1067`.

### 6. Why NOT Option C

Two reasons:
1. **SP3 explicitly rejects it.** "The round-3 design doesn't require RON-driven listener predicates; it requires Effect-driven *consequences*" (`spike-skill-dsl-coverage/RESEARCH.md` §136). Building a predicate DSL when none of the canonical patterns mandate it is gold-plating.
2. **Loss of type safety.** Every state-machine invariant currently checked by typed Rust (`BatteryLoopState::gain_static_charge` clamps to cap, `PredatorLoopState::consume_prey_lock_payoff` is idempotent within a turn) becomes a runtime predicate in the interpreter. The existing typed transitions catch bugs at compile time; the RON DSL would catch them at runtime via validators or, worse, not catch them at all.

### 7. Why NOT Option A

Per-blueprint Plugins do not solve the underlying coordination problem (three registries: dispatch, kernel hook, snapshot extractor) — they just hide it inside `Plugin::build`. Option B unifies the three registries into one `BlueprintRegistry` resource. Plugin gives no shape; trait gives a shape.

## Persisted decision

Recommend the following entry in `.gsd/DECISIONS.md`:

> **D-M017-BLUEPRINT-API** (2026-05-12, owner: GSD M017 prep).
>
> Adopt Option B — Trait + central registry — for the blueprint extension API. Define a `Blueprint` trait with four methods: `id() -> BlueprintId`, `commit_signals(signal, action) -> Vec<KernelTransition>` (default empty), `on_event(event, ctx) -> Vec<Effect>` (default empty), `snapshot(world) -> BlueprintSnapshot` (default empty).
>
> A `BlueprintRegistry` resource owns `HashMap<BlueprintId, Box<dyn Blueprint>>` and exposes `dispatch_commit`, `dispatch_event`, and `snapshot_all` methods. Registration happens once in `register_default_blueprints(app)` at kernel runtime setup.
>
> The existing `CustomSignalPayload` envelope (`{Empty, Amount{i32}}`) is preserved unchanged. The existing `CombatKernelTransition` enum keeps its 5 typed Digimon variants for back-compat; new blueprints (kitsune_grace, holy_aegis) emit `Effect` cascades, not new typed kernel transitions.
>
> `ValidationSnapshot` gains a `blueprint_snapshots: HashMap<BlueprintId, BlueprintSnapshot>` field, replacing the 5 hardcoded snapshot fields. The 5 typed fields are kept as derived getters during migration for test compat.
>
> Migration sequencing in `migration_plan.md`. Out of scope for this decision: state-resource-per-UnitId refactor (deferred to M018+ as `D-M017-BLUEPRINT-STATE-PER-UNIT`).

## What this decision does NOT do

- Does **not** change `CustomSignalPayload`. Two-variant envelope stays.
- Does **not** add new `CombatKernelTransition` variants for kitsune_grace or holy_aegis. They are listener-only blueprints; their effects flow through the SP1 `Effect` cascade (RON-declared), not through new typed kernel transitions.
- Does **not** convert blueprint state resources to per-UnitId components. Deferred to M018+ (see RESEARCH.md §"Surprises" #2).
- Does **not** modify the `Effect` enum scope set by SP3. SP3's "add-now" list is the canonical M017 schema delta.
- Does **not** force migration of all 6 blueprints in a single PR. `migration_plan.md` sequences them.

## Risks

- **Risk:** `Box<dyn Blueprint>` virtual dispatch. **Mitigation:** measured — single-digit calls per action — negligible at this scale.
- **Risk:** trait grows. **Mitigation:** review at each blueprint addition; if a method has more than 2 distinct override patterns, split it.
- **Risk:** `BlueprintSnapshot` sum type churn. **Mitigation:** acceptable; one line per new blueprint is cheaper than the type-erasure alternative.
- **Risk:** existing tests assert on typed `ValidationSnapshot.twin_core: TwinCoreSnapshot`. **Mitigation:** keep typed getters as helpers; deprecate the fields gradually. `migration_plan.md` lists every affected test.
