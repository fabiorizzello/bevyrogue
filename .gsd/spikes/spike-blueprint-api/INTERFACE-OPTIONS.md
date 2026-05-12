---
spike: SP2
related: RESEARCH.md, DECISION.md
status: complete
created: 2026-05-12
---

# SP2 — Interface options

Three radically different designs for the blueprint extension API. Each is judged on: extensibility (adding a 7th blueprint without touching kernel), test isolation (instantiating one blueprint without the other 5), signal coupling (new `CombatEventKind` variants required), migration cost (lines of churn across the 6 existing blueprints + `mod.rs` + `kernel.rs::register_combat_kernel_runtime` + `observability.rs` + tests), RON-driven control (declarative vs Rust split), and observability (how blueprints surface state to `ValidationSnapshot`).

## Option A — Bevy Plugin per blueprint

Each blueprint implements `impl Plugin for X`. The plugin owns its own state resource registration, its own `apply_*` system, its own kernel-hook registration, and exposes a `Blueprint` impl (a thin marker) so the central dispatcher can route by `BlueprintId`.

### Sketch

```rust
pub struct BatteryLoopPlugin;

impl Plugin for BatteryLoopPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BatteryLoopState>()
            .add_systems(Update, apply_battery_loop_transitions_system);

        // Register dispatch
        let mut registry = app.world_mut().resource_mut::<BlueprintDispatchRegistry>();
        registry.register("tentomon", tentomon_dispatch);

        // Register kernel hook
        let mut kernel = app.world_mut().resource_mut::<CombatKernelRegistry>();
        kernel.register(BatteryLoopHook);

        // Register snapshot extractor
        let mut snap = app.world_mut().resource_mut::<SnapshotRegistry>();
        snap.register("tentomon", |world| {
            world.get_resource::<BatteryLoopState>()
                .map(|s| BlueprintSnapshot::BatteryLoop(s.into()))
        });
    }
}

// In main.rs:
app.add_plugins((
    AgumonPlugin, GabumonPlugin, DorumonPlugin,
    TentomonPlugin, PatamonPlugin, RenamonPlugin,
));
```

### Pros

- Idiomatic Bevy: each blueprint is a self-contained plugin, opt-in/opt-out via `add_plugins`.
- Test isolation: a test can spawn `App::new().add_plugins(BatteryLoopPlugin)` without dragging in the other 5.
- Side-by-side extensibility: a 7th blueprint = one new file + one line in `app.add_plugins`.
- Implicit dependency declaration via Plugin dependencies.

### Cons

- **Three registries** still needed (dispatch, kernel hook, snapshot extractor) — the Plugin doesn't unify them, it just hides the wiring behind `Plugin::build`.
- Inconsistent with current Bevy 0.18 idioms in this codebase: nothing else in `src/combat/` is a `Plugin`; everything is wired manually in `register_combat_kernel_runtime`. Mixing styles.
- Adding the snapshot extractor lambda inside `Plugin::build` is awkward — the closure captures nothing, so it's effectively a static function pointer dressed up.
- Trait surface is **implicit** (Plugin gives no shape to "what is a blueprint"). Discovery requires reading every Plugin impl.
- Doesn't address the new listener half (`on_event(CombatEvent) -> Vec<Effect>`) — that still needs a separate registry inside or outside the Plugin.
- New `CombatEventKind` variants from SP1 (`UnitDied`, `UltimateUsed`, etc.) need a Bevy event-reader system per blueprint, which means 6 near-identical systems (one per blueprint) — high boilerplate.

### Migration cost

- 6 new `Plugin` impls (~30 lines each) = ~180 lines added.
- `register_combat_kernel_runtime` shrinks from 26 lines to ~10 (delete the `init_resource` calls, just call the plugins).
- `blueprints/mod.rs` static `BLUEPRINTS` array deleted (~20 lines). Replaced by registry-resource init (~5 lines).
- `observability.rs::capture_validation_snapshot` per-blueprint reads replaced by snapshot-registry iteration (~30 lines delta).
- Tests: minor — most tests instantiate `App` and manually call `register_combat_kernel_runtime`; they'd switch to `app.add_plugins(...)`. ~20 test files affected, ~3 lines each. ~60 lines.
- **Total: ~290 lines of churn, distributed across many files.**

### Signal coupling

Same as B/C — needs the SP1 `CombatEventKind` extensions regardless. No advantage.

### RON-driven control

Same as B — RON expresses `Effect`s, Rust holds listener filters. Plugin choice is orthogonal.

### Observability

Snapshot registry lambda per plugin. Works, but boilerplate-heavy: each plugin must define its own snapshot extractor closure even when the data is a trivial `From` impl.

---

## Option B — Trait + central registry (RECOMMENDED)

A single `Blueprint` trait. A `BlueprintRegistry` resource owns `HashMap<BlueprintId, Box<dyn Blueprint>>`. Blueprints register via `app.add_blueprint::<T>()` extension method, or via a single `register_default_blueprints(app)` call. Both the commit-time dispatch (existing 6) and the event-driven listening (new 2 — kitsune_grace, holy_aegis) flow through the trait.

### Sketch

```rust
pub trait Blueprint: Send + Sync + 'static {
    fn id(&self) -> BlueprintId;

    /// Default: returns empty. Existing 6 blueprints override this.
    fn commit_signals(
        &self,
        _signal: &SkillCustomSignal,
        _action: &ResolvedAction,
    ) -> Result<Vec<CombatKernelTransition>, CustomSignalDispatchError> {
        Ok(Vec::new())
    }

    /// Default: returns empty. New blueprints (kitsune_grace, holy_aegis) override this.
    fn on_event(
        &self,
        _event: &CombatEvent,
        _ctx: &BlueprintCtx,
    ) -> Vec<Effect> {
        Vec::new()
    }

    /// Default: returns Empty. Blueprints with state override this.
    fn snapshot(&self, _world: &World) -> BlueprintSnapshot {
        BlueprintSnapshot::Empty
    }
}

#[derive(Resource, Default)]
pub struct BlueprintRegistry {
    blueprints: HashMap<BlueprintId, Box<dyn Blueprint>>,
    /// Reverse index: each owner string → BlueprintId, for SkillCustomSignal routing.
    owner_index: HashMap<String, BlueprintId>,
}

impl BlueprintRegistry {
    pub fn register<B: Blueprint>(&mut self, blueprint: B) { /* ... */ }
    pub fn dispatch_commit(&self, signal: &SkillCustomSignal, action: &ResolvedAction)
        -> Result<Vec<CombatKernelTransition>, CustomSignalDispatchError> { /* ... */ }
    pub fn dispatch_event(&self, event: &CombatEvent, ctx: &BlueprintCtx)
        -> Vec<Effect> { /* ... */ }
    pub fn snapshot_all(&self, world: &World) -> HashMap<BlueprintId, BlueprintSnapshot> { /* ... */ }
}

pub enum BlueprintSnapshot {
    Empty,
    TwinCore(TwinCoreSnapshot),
    BatteryLoop(BatteryLoopSnapshot),
    PredatorLoop(PredatorLoopSnapshot),
    HolySupport(HolySupportSnapshot),
    PrecisionMindGame(PrecisionMindGameSnapshot),
    // Future: KitsuneGrace(_), HolyAegis(_) (or BlueprintSnapshot::Stateless for listener-only)
}

// In kernel.rs / register_combat_kernel_runtime:
pub fn register_default_blueprints(app: &mut App) {
    let mut registry = BlueprintRegistry::default();
    registry.register(TwinCoreFire);   // Agumon
    registry.register(TwinCoreIce);    // Gabumon (+ Fur Cloak listener)
    registry.register(PredatorLoop);   // Dorumon
    registry.register(BatteryLoop);    // Tentomon
    registry.register(HolySupport);    // Patamon (+ Holy Aegis listener — same struct or sibling)
    registry.register(PrecisionMindGame); // Renamon (+ Kitsune Grace listener — same struct or sibling)
    app.insert_resource(registry);
}
```

`BlueprintCtx` is a read-only handle to the resources a listener may consult (turn_order, team membership, status registry, etc.).

### Pros

- **One trait, one registry, three responsibilities** — commit-time dispatch, event listening, snapshot extraction — all flow through the same `Box<dyn Blueprint>`.
- **Default methods**: a stateless listener-only blueprint (kitsune_grace, holy_aegis) overrides only `on_event`. A state-machine blueprint (battery_loop, predator_loop) overrides only `commit_signals` + `snapshot`. A hybrid blueprint (twin_core when it gets a listener for "partner status applied") overrides all three.
- **`ValidationSnapshot` becomes registry-driven**: `snapshot.blueprints = registry.snapshot_all(world)`. Adding a 7th blueprint = zero lines in `observability.rs`.
- **Single registration site** in `kernel.rs::register_default_blueprints`. Plus the `BlueprintRegistry` resource itself. Total wiring = 2 lines per blueprint.
- **Test isolation**: a test can build `BlueprintRegistry { blueprints: { id: Box::new(BatteryLoop) }, ... }` directly without the App at all (for unit tests on `commit_signals`/`on_event`).
- **Maps cleanly onto SP3's hybrid conclusion**: listener filter = `on_event` body in Rust; effect cascade = the returned `Vec<Effect>` (data, eventually loaded from RON).
- **No new Bevy Plugin idiom needed** — stays consistent with the rest of `src/combat/` which is manually-wired.
- **Easy back-compat shim**: existing `blueprints::transitions_for_action_checked` becomes a one-line forwarder to `registry.dispatch_commit`. The static `BLUEPRINTS` array deletes; tests that use `transitions_for_action_checked` continue to compile.

### Cons

- `Box<dyn Blueprint>` introduces dynamic dispatch — measurable cost is **trivial** (one virtual call per signal, single-digit per action), but worth noting.
- The trait has three methods, each potentially a no-op default — discoverability slightly worse than three separate traits (`SignalDispatcher`, `EventListener`, `SnapshotProvider`). **Mitigation**: doc-comment heavily; SP2 sketch shows the three responsibilities.
- `BlueprintSnapshot` enum is still a closed sum type; adding a new variant for a 7th blueprint means editing the enum. **Mitigation**: this is one line in `observability.rs`; alternative `BlueprintSnapshot::Custom(Box<dyn Any + Serialize>)` adds type-erasure complexity that isn't worth it for one line saved.
- Listener-only blueprints (kitsune_grace) and existing commit-dispatch blueprints (twin_core_fire) share a trait that semantically does two different things. Acceptable: the trait is the **stable seam**, not the abstraction.

### Migration cost

- 6 new `Blueprint` impls (~25 lines each, mostly forwarding to existing apply functions and snapshot extractors) = ~150 lines.
- `BlueprintRegistry` + `BlueprintSnapshot` + `BlueprintCtx` types = ~120 lines (new file `src/combat/blueprint_registry.rs`).
- `blueprints/mod.rs::dispatch_custom_signal` becomes one-line forwarder: `registry.dispatch_commit(signal, action)`. The static `BLUEPRINTS` array deletes (~25 lines removed).
- `register_combat_kernel_runtime` adds one line `register_default_blueprints(app)`; the 5 `init_resource::<*State>()` calls **stay** (state resources still exist) and the 5 `apply_*_transitions_system` registrations **stay** (apply systems still exist). No change to existing apply-side wiring.
- `observability.rs::capture_validation_snapshot`: replace 5 hardcoded resource reads with one `registry.snapshot_all(world)` call (~30 lines simplified).
- Tests: existing tests use `transitions_for_action_checked` (one call site, one-line forwarder) and `capture_validation_snapshot` (return type changes — `twin_core: ValidationTwinCoreSnapshot` → `blueprint_snapshots: HashMap<BlueprintId, BlueprintSnapshot>` requires test updates). **~5 tests touch snapshot fields directly**; the rest are insulated.
- **Total: ~200 lines added, ~80 lines removed = ~280 lines net churn, concentrated in 2 new files + 3 modified files.**

### Signal coupling

Same as A/C — SP1 `CombatEventKind` extensions are independent. `on_event` reads from existing `MessageReader<CombatEvent>` via a single registry-driven `blueprint_event_listener_system` (registered once, dispatches to all blueprints).

### RON-driven control

Same as A — RON expresses `Effect`s, Rust holds listener filters inside `on_event`. The trait is the carrier for the Rust half; the RON DSL is unchanged.

### Observability

Registry-driven. Adding a 7th blueprint adds zero lines in `observability.rs` (the enum variant is a one-line addition in `BlueprintSnapshot`; alternative is a stringly-keyed map of `serde_json::Value`, which we reject for type safety).

---

## Option C — Pure-data + generic dispatch

No per-blueprint Rust at all. State machines expressed in RON (e.g. `blueprints/battery_loop.ron`). A generic interpreter system reads the RON, evaluates the listener predicate against `CombatEvent`, and emits `Effect`s into the cascade.

### Sketch

```rust
// assets/data/blueprints/battery_loop.ron
Blueprint(
    id: "battery_loop",
    state_keys: [
        ("static_charge", U8(0), U8(3)),     // current, cap
        ("circuit_charge", U8(0), U8(2)),
    ],
    listeners: [
        Listener(
            on: SpGranted(target: Self),
            when: AlwaysTrue,
            effects: [
                IncrementState(key: "static_charge", by: 1),
            ],
        ),
        Listener(
            on: IncomingDamage(target: Self),
            when: StateAtLeast(key: "circuit_charge", value: 3),
            effects: [
                BlockReaction(kind: All, damage_mult: 0.5),
                DecrementState(key: "circuit_charge", by: 1),
            ],
        ),
    ],
)

// src/combat/blueprint_interpreter.rs (new — ~600 lines)
pub struct BlueprintInterpreter { /* ... */ }
impl BlueprintInterpreter {
    pub fn on_event(&mut self, event: &CombatEvent) -> Vec<Effect> { /* eval predicate, emit effects */ }
}
```

### Pros

- **Maximum extensibility**: a 7th blueprint = one new `.ron` file, zero Rust.
- **Hot-reloadable balance**: tweak state thresholds without recompile.
- **Designer-friendly**: a non-programmer can add a blueprint.
- **Audit-friendly**: all blueprint logic in one place (the RON files).

### Cons

- **Predicate language is unbounded.** SP3 lists specific listener filters: "caster==gabumon && status==Chilled && same_round" (Twin Core Fire trigger), "actor != self && team == self.team" (Kitsune Grace), "HpPctBelow(0.50)" (Predator Loop entry threshold). Encoding these requires a non-trivial expression DSL (variable references, comparisons, boolean operators). Either:
  - The DSL is **closed and limited** (a fixed set of predicate shapes) — fine for current 6+2 but every new pattern adds a new variant.
  - The DSL is **open** (mini-language) — requires a parser, evaluator, and **tests for the language itself**. Significant infrastructure.
- **No type checking.** Typos in state keys (`"static_chrge"`) fail at runtime, not compile time. RON-side validators help but don't catch logic errors.
- **Performance cost**: each `CombatEvent` is matched against every registered listener via runtime predicate evaluation. Negligible at current scale (~10 events/turn × ~10 listeners) but worse than typed Rust dispatch.
- **No staged migration path.** All 6 existing blueprints would need to be re-expressed in RON simultaneously, plus the kernel transition emission semantics. Their state machines (BatteryLoopState's `threshold_grant_emitted_this_cycle` guard, PredatorLoopState's cap/decay logic, TwinCoreState's `active_thermal_spark_targets` list) are not trivially expressible as keyed state values.
- **Tests would need to change shape**: today integration tests assert specific `BatteryLoopTransition` enum variants; under C they'd assert RON-derived state changes (less ergonomic, more brittle to RON refactors).
- **SP3 explicitly says this is overkill**: "the round-3 design doesn't require RON-driven listener predicates; it requires Effect-driven *consequences*" (`spike-skill-dsl-coverage/RESEARCH.md` §136). Pushing for option C is materially more expensive than B with no design-canon mandate.

### Migration cost

- 6 blueprints re-expressed in RON (~80 lines each) = ~480 lines of RON.
- Interpreter (~600 lines of Rust, with predicate evaluator + state-mutation primitives + observability hooks).
- Existing apply systems (`apply_battery_loop_transitions_system` × 5) **all deleted** — replaced by generic interpreter system. ~1500 lines removed.
- Existing state resources collapse into one generic `BlueprintStateRegistry { HashMap<BlueprintId, HashMap<String, ParamValue>> }`. Most apply-side code disappears.
- **However**: every state-machine invariant currently enforced by typed Rust methods (`gain_static_charge` clamps to cap, `consume_prey_lock_payoff` is idempotent within a turn) must be re-expressed declaratively. The "threshold_grant_emitted_this_cycle" guard pattern in BatteryLoop is non-trivial to express in pure data without growing the DSL.
- Tests: ~20 integration tests rewritten to assert on the generic state-key/value map. ~600 lines.
- **Total: ~1700 lines added (interpreter + RON + tests), ~1500 lines removed (apply systems + typed state). Net +200 lines but the *complexity* shifts from "well-typed Rust" to "untyped RON + interpreter", and the work is roughly 10× the engineering effort of B.**

### Signal coupling

Listener predicates over `CombatEventKind` must be reified into the RON DSL. New SP1 events (`UnitDied`, `UltimateUsed`, etc.) require new predicate-language tags. This **couples the RON DSL to the kernel event vocabulary**, which is a longer-term coupling than B's (where Rust code naturally evolves with the event enum).

### RON-driven control

100% RON. SP3 explicitly does not require this and lists "trigger half stays in Rust under either option B or option C-minus" (`spike-skill-dsl-coverage/RESEARCH.md` §136).

### Observability

Generic. The interpreter exposes a single `HashMap<BlueprintId, HashMap<String, ParamValue>>` snapshot. Loses the structured types (no more `BatteryLoopSnapshot { static_charge: u8, ... }` — instead `{ "static_charge": Int(2), ... }`). Less type-safe for validators that check specific invariants.

---

## Comparison matrix

| Criterion | A (Plugin) | B (Trait+Registry) | C (Pure-data) |
|---|---|---|---|
| Extensibility (7th blueprint) | One Plugin file + one `add_plugins` line | One trait impl + one register line | One RON file |
| Test isolation | Good (per-Plugin App) | Excellent (no App needed for unit tests on trait methods) | Good (interpreter only) |
| Signal coupling to SP1 events | Identical (event-reader in Plugin) | Identical (event-reader in registry) | Requires RON DSL extension |
| Migration cost | ~290 lines, many files | ~280 lines, 5 files | ~1700 lines, big infra |
| RON-driven control | Status quo (Effects in RON) | Status quo | 100% RON |
| Observability | Registry of lambdas | Registry of trait objects (typed snapshot) | Generic key/value map (untyped) |
| Listener-only blueprints (kitsune_grace, holy_aegis) | Plugin with no state — wasteful | `on_event` override, default everything else — clean | RON file with only `listeners:` block |
| Aligns with SP3 conclusion | Yes, but adds Bevy ceremony | Yes, directly maps the hybrid partition | Overshoots — SP3 says not needed |
| Consistency with existing `src/combat/` | Diverges (no other module is a Plugin) | Matches (manual wiring like the rest) | Diverges heavily |
| Type safety of snapshot | Strong | Strong | Weak (key/value map) |

## Recommendation

**Option B.** It maps directly onto SP3's hybrid partition, has the smallest migration cost, preserves type safety, addresses all five open questions (registry-driven `ValidationSnapshot`, single trait covering commit + listen + snapshot, default methods cover the listener-only case for kitsune_grace/holy_aegis), and stays consistent with the manual-wiring idiom used elsewhere in `src/combat/`. See `DECISION.md` for the formal rationale.
