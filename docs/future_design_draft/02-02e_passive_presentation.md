# §2.2e — Passive presentation: Full FSM mandate + trigger sub-variants + channel layout

> **Scope.** Define HOW passive blueprints (§2.2b §B dual-role, listener side) drive both **gameplay state transitions** and **VFX** through a uniform Full FSM editor-inspectable shape — same `clipmontage.ron` grammar as skills, different trigger semantics.
>
> **Decision round 2026-05-12 (review-3 closure):** Every passive in the M017 roster has Full FSM (3+ nodes + edges) + mandatory anim+VFX. No "listener-only" passives. Editor-inspectable uniformity > shape minimalism. **Forma C "variants by shape" abandoned**; replaced by **trigger sub-variants** (A/B/C below) — all share the same Full FSM shape, differ only in *what fires transitions*.
>
> **Two presentation channels** are output mechanisms of the FSM, not standalone shapes:
>
> - **Channel 1 (Ch1) — trigger-proc visual** (flash/burst on FSM node transition) → `SpawnParticle` Static/Travel via FSM `on_enter`
> - **Channel 2 (Ch2) — persistent-presence visual** (aura/tint while FSM in non-Idle state) → presentation observer on component diff + `VfxEmitter` manager entity (§2.2d §J)
>
> Listener `ctx.notify` (the original "Channel 1" in earlier drafts of this doc) is preserved as **implementation detail** of the Reactive-proc sub-variant — it is the kernel-event entry-point that pushes an FSM transition into the passive blueprint's `BlueprintListener::on_kernel_event`. Observer (the original "Channel 2") is preserved as **implementation detail** of the Aura-loop sub-variant — it owns the `VfxEmitter` lifetime mapped to the FSM's persistent state. Boundary remains strict: listeners write components, observers read diffs.

## §A.0 — Full FSM mandate (Forma C v2)

**Rationale.** Earlier drafts treated passives as "listener-only blueprints with no FSM" because the gameplay logic fits in a single `on_kernel_event` handler. Review-3 of the M017 roster (Agumon `twin_core_fire`, Dorumon `predator_loop`, Gabumon `fur_cloak`, Tentomon `battery_loop`, Renamon `kitsune_grace`, Patamon `holy_aegis`) surfaced two problems with that model:

1. **Editor-inspectable uniformity.** Skill FSMs are editor-inspectable via the same `clipmontage.ron` shape. Passives without FSM require a separate inspection path (read Rust source for the listener handler). Same-tool uniformity beats per-shape special casing.
2. **"Not on-screen = not exists" principle.** A passive that mutates gameplay (DR aura, tracking mark, threshold trigger, partner sync) but has no visual is invisible to the player and to visual tests — the worst possible bug surface. Mandatory anim+VFX makes passives *legible*.

**Mandate:**

| Requirement | Applies to | Rationale |
|---|---|---|
| Full FSM shape (3+ nodes + edges) | every passive in `digimon/<name>/04_passive_*.md` | editor-inspectable parity with skills |
| Anim presence (clip frame range per node) | every passive | makes state machine readable in editor + visible in game |
| VFX presence on at least one channel | every passive | "not on-screen = not exists" |
| Headless determinism (FSM tickable without `windowed`) | every passive | passives can emit gameplay Commands (`ApplyBuff`/`EmitDamage`/`AdvanceTurn`) — must roll in tests |

**Cost vs benefit.** Roughly +20-30 RON lines per passive (vs the previous listener-only minimum). Bought by: editor tooling parity, visual debuggability, test-time gameplay observability. Acceptable.

## §A.1 — Trigger sub-variants (A/B/C)

All three sub-variants share **Full FSM shape**. They differ only in *what scatters the transition from one node to another* — specifically what fires the entry-event into the passive FSM. The FSM topology itself looks identical from the editor.

| Sub-variant | Trigger semantics | Listener-side entry | FSM topology (typical) | Roster examples |
|---|---|---|---|---|
| **A. Aura-loop** | continuous while alive | `on_kernel_event(CombatStarted)`/`(UnitSpawned)` → start FSM; `UnitDied { self }` → stop FSM | `Idle ↔ AuraTick (loop)` — 2-3 nodes cycling, no `on_event` edges, only `TimeInNode` | `holy_aegis` Patamon, `fur_cloak` Ch2 block-ready, `predator_loop` Ch1 idle scan |
| **B. Reactive-proc** | event-driven discrete | `on_kernel_event(specific_event)` → push FSM transition via internal signal | `Dormant → Proc → Resolve → Dormant` — edges include `KernelEvent(...)` or internal signal predicate | `kitsune_grace` (UltimateUsed by ally), `fur_cloak` Ch1 (block-reaction), `battery_loop` Ch1 (block-reaction) |
| **C. State-watch** | state predicate change | `on_kernel_event(any)` reads predicate (hp pct, partner status, etc.); if predicate transitions, push FSM transition | `Idle ↔ Active` with edges gated on `KernelEvent` filtered by predicate-true | `twin_core_fire` (partner Heated/Chilled), `predator_loop` Ch2 (hp-threshold), `holy_aegis` cleanse-tick (sub-state) |

**Boundary between sub-variants:** A passive can mix sub-variants across its channels (e.g. `fur_cloak` Ch1 = Reactive-proc on block-reaction, Ch2 = Aura-loop on block-ready presence; `predator_loop` Ch1 = Aura-loop idle scan, Ch2 = State-watch hp-threshold). The FSM topology shows this as nested or parallel sub-states; the doc lists each channel's sub-variant explicitly.

**FSM ↔ listener integration:** the listener (`BlueprintListener::on_kernel_event`) is the **transition pusher**, not the FSM itself. The FSM lives in `self.fsm_rt` exactly like skill FSMs (§2.2b §H) — it ticks frame-by-frame, emits Commands `on_enter`, evaluates edges. The listener observes kernel events and translates them into FSM transitions by writing to a small `pending_signals` queue that edges with `KernelEvent(...)` consume. Headless tests drive the listener directly; the FSM ticks as a unit-under-test.

```rust
pub trait BlueprintListener {
    fn on_kernel_event(&mut self, ev: &CombatEvent, ctx: &mut ListenerCtx);
    /// Passives only: tick the FSM after listener has had a chance to push signals.
    fn tick_passive_fsm(&mut self, ctx: &mut PassiveTickCtx);
}
```

The kernel calls `on_kernel_event` for every relevant event, then `tick_passive_fsm` after the event drain — single integration point per passive per frame. Both run headless (FSM emits gameplay Commands → `KernelEffect`); only cosmetic Commands no-op.

## §A — Problem (legacy framing, preserved for reference)

§2.2b defines an FSM that emits `SpawnParticle` Commands during a skill, scoped to a clipmontage. §2.2d defines `VfxLocus` + `VfxMotion` for those Commands. Neither covers **passive** Digimon (Twin Core, Predator Loop, Battery Loop, Fur Cloak, Holy Aegis, Kitsune Grace, …). Passives:

- have no clipmontage (they don't drive an animation),
- are listener-only blueprints (§2.2b §B): `BlueprintListener::on_kernel_event` is their only entry point,
- own state that lives outside any single skill's resolution (round-scoped buffs, multi-turn tracking, threshold-gated booleans).

The FSM `SpawnParticle` channel is not reachable from a listener — it is owned by a `SkillBehavior::execute` call, not by `on_kernel_event`. And even if it were, FSM `SpawnParticle` has **preset-driven lifecycle** (§2.2d §J): not the right shape for FX whose lifetime is "as long as buff X is on the unit".

Hence: a passive presentation channel that is **explicitly two-pronged**, because the two cases are genuinely different problems.

## §B — Two channels

```
       ┌──────────────────────────────────────────────────────────────────┐
       │ Listener (passive blueprint, headless-safe Rust)                 │
       │                                                                  │
       │   on_kernel_event(ev, ctx) {                                     │
       │     match ev {                                                   │
       │       StatusApplied { caster: gabumon, status: Chilled } => {    │
       │         ctx.add_self_buff("twin_core_fire_active", UntilRoundEnd)│
       │         ctx.notify(NotifyParticle { preset: "twin_core_ignite",  │
       │                                     origin: SelfCenter,          │
       │                                     motion: Static })  ─────┐    │
       │       }                                                     │    │
       │     }                                                       │    │
       │   }                                                         │    │
       └─────────────────────────────────────────────────────────────┼────┘
                                                                     │
                                          one-shot, event-driven     │
                                                                     ▼
                                                ┌─────────────────────────────┐
                                                │ PresentationBus             │
                                                │   - NotifyParticle (drop    │
                                                │     in headless, §2.2b §G)  │
                                                │   - NotifyShake / Sfx       │
                                                └─────────────────────────────┘
                                                                     ▲
                                                                     │
                                          one-shot, event-driven     │
       ┌─────────────────────────────────────────────────────────────┼────┐
       │ Presentation observer (windowed-only system)                │    │
       │                                                             │    │
       │  fn observe_twin_core_aura(                                 │    │
       │      mut commands: Commands,                                │    │
       │      added:   Query<Entity, Added<Buff_TwinCoreFireActive>>,│    │
       │      mut removed: RemovedComponents<Buff_TwinCoreFireActive>│    │
       │  ) {                                                        │    │
       │      for e in &added {                                      │    │
       │          commands.entity(e).insert(VfxEmitter { ... });  ───┘    │
       │      }                                                           │
       │      for e in removed.read() {                                   │
       │          commands.entity(e).remove::<VfxEmitter>();              │
       │      }                                                           │
       │  }                                                               │
       └──────────────────────────────────────────────────────────────────┘
```

The **listener channel** (`ctx.notify`) handles transitions: instants when state crosses a boundary. The **observer channel** handles duration: the time between transitions, when state is held.

## §C — Channel 1: `ListenerCtx::notify`

Mirror of `SkillExecCtx::notify` (§2.2b §C cosmetic Commands). Listener gets a presentation-bus handle through `ListenerCtx`; pushing on it is a no-op in headless (`#[cfg(feature = "windowed")]` gate inside the bus, listener writes unconditionally).

```rust
pub trait BlueprintListener {
    fn on_kernel_event(&mut self, ev: &CombatEvent, ctx: &mut ListenerCtx);
}

pub struct ListenerCtx<'w> {
    // ...existing gameplay methods (add_self_buff, emit_kernel_event, etc.)

    /// Push a cosmetic notify onto the presentation bus. No-op headless.
    pub fn notify(&mut self, n: PresentationNotify) { /* ... */ }
}

pub enum PresentationNotify {
    Particle(NotifyParticle),
    Shake(NotifyShake),
    Sound(NotifySound),
}

pub struct NotifyParticle {
    pub preset: String,
    pub origin: VfxLocus,           // §2.2d
    pub motion: VfxMotion,           // §2.2d (typically Static for listener notifies)
    pub cancel_id: Option<NotifyId>, // optional: lets a follow-up notify supersede in flight
}
```

**`EntityRef` context.** Because the notify is emitted from inside `on_kernel_event(ev, ctx)`, the listener channel **uniquely** supports `EntityRef::Caster` and `EntityRef::EventTarget` — they resolve against `ev`. FSM-emitted `SpawnParticle` does **not** have this context (validator rejects per §2.2d §H.5).

**Listener-only refs:**

| Ref | Listener channel | FSM channel |
|---|---|---|
| `EntityRef::Self_` | ✅ | ✅ |
| `EntityRef::Primary` | ⚠️ requires the listener to have stashed it (rare) | ✅ |
| `EntityRef::Caster` | ✅ | ❌ rejected |
| `EntityRef::EventTarget` | ✅ | ❌ rejected |
| `EntityRef::FromBlueprintState(k)` | ✅ | ✅ |
| `EntityRef::FromParamSnapshot(k)` | ⚠️ rare (listeners don't usually snapshot) | ✅ |

**Use this channel for:** arm flashes, dissipate poofs, entry locks, hit overlays, one-shot reaction visuals. Anything that is *emitted in response to one event*.

**Do not use this channel for:** persistent auras, marks that track a unit over multiple turns, link visuals that must live as long as a partner is alive. Those go through §D.

## §D — Channel 2: presentation observer

A `#[cfg(feature = "windowed")]` Bevy system that observes **component diff** — `Added<T>`, `RemovedComponents<T>`, `Changed<T>` — and maintains a 1:1 mapping between a gameplay state and a `VfxEmitter` manager entity (§2.2d §J).

```rust
// src/presentation/passive_observers.rs (new file, M017 target)
#[cfg(feature = "windowed")]
pub fn observe_twin_core_aura(
    mut commands: Commands,
    added:   Query<Entity, Added<Buff_TwinCoreFireActive>>,
    mut removed: RemovedComponents<Buff_TwinCoreFireActive>,
) {
    for e in &added {
        commands.entity(e).insert(VfxEmitter {
            preset:  "twin_core_fire_loop".into(),
            origin:  VfxLocus::SelfCenter,
            motion:  VfxMotion::Static,
            cadence: EmitCadence::Continuous,
        });
    }
    for e in removed.read() {
        commands.entity(e).remove::<VfxEmitter>();
    }
}

#[cfg(feature = "windowed")]
pub fn observe_predator_mark(
    mut commands: Commands,
    state: Query<(Entity, &DorumonBlueprint), Changed<DorumonBlueprint>>,
    mut marks: Local<HashMap<Entity, Entity>>, // dorumon -> mark manager entity
) {
    for (dorumon_e, bp) in &state {
        let want = bp.predator_loop.tracked_target;
        let have = marks.get(&dorumon_e).copied();
        if want != have {
            // tracked target changed: despawn old mark, spawn new
            if let Some(old) = have { commands.entity(old).despawn(); }
            if let Some(new_target) = want {
                let mark_e = commands.spawn(VfxEmitter {
                    preset:  "predator_mark_loop".into(),
                    origin:  VfxLocus::EntityCenter(EntityRef::FromBlueprintState(
                                 "predator_loop.tracked_target".into())),
                    motion:  VfxMotion::Static,
                    cadence: EmitCadence::Continuous,
                }).id();
                marks.insert(dorumon_e, mark_e);
            } else {
                marks.remove(&dorumon_e);
            }
        }
    }
}
```

**Rules:**

1. **One function per persistent FX.** Boilerplate proportional to FX count. For a 6-Digimon roster with ~6 persistent passives total, this is ~30 LOC per observer × 6 = under 200 LOC of presentation glue. Accepted.
2. **Observer reads component diff, never kernel events.** The listener writes the component; the observer reads the diff. If you find yourself wanting `EventReader<CombatEvent>` in an observer, stop — that work belongs in §C.
3. **Observer is windowed-only.** Whole module behind `#[cfg(feature = "windowed")]`. Headless build does not compile observers.
4. **Mapping table is in Rust, not RON.** Modding implication: a third-party data-only mod cannot ship a passive that has its own persistent VFX without also shipping the Rust observer. Accepted trade-off for v0 (no mod system yet); revisit if/when modding becomes a goal.
5. **Migrating marks** (predator_mark when tracking changes): observer detects via `Changed<DorumonBlueprint>` (or a finer-grained `Changed<PredatorLoopState>` if extracted), diffs current vs desired target, despawns old manager + spawns new. The `VfxEmitter.origin` re-resolves every emission tick (§2.2d §J), so a single emitter following a migrating target is also legal — the choice between "one emitter, re-resolved" vs "despawn-respawn" is per-FX, driven by whether the preset's loop state should reset on retarget.

## §E — Naming convention for buff components

For Channel 2 to work via `Added<T>` / `RemovedComponents<T>`, each presentation-bound buff must be a **typed Rust component**, not just a `BuffId` string in a `Buffs` collection.

**Rule:** Buff IDs that have a persistent VFX counterpart are convention-named `Buff_PascalCase` in Rust:

| `BuffId` (gameplay, stringy) | Component (presentation hookable) |
|---|---|
| `"twin_core_fire_active"` | `Buff_TwinCoreFireActive` |
| `"twin_core_ice_active"` | `Buff_TwinCoreIceActive` |
| `"fur_cloak_active"` | `Buff_FurCloakActive` |
| `"holy_aegis_active"` | `Buff_HolyAegisActive` |

The gameplay layer continues to read/write via `BuffId` (string) — `add_self_buff(BuffId("twin_core_fire_active"), ...)` — but the buff-add code in the kernel also inserts the typed component when the BuffId is in a registered presentation-bound list. A small `BuffComponentRegistry` (Rust constant) maps `BuffId → fn(&mut EntityCommands)` to do the typed insert.

**Buffs with no persistent FX** (purely numeric, listener-only, off-screen) do not need a typed component — they live as `BuffId` strings in the `Buffs` collection only. This keeps the typed-component sprawl bounded.

## §E.1 — Channel layout convention (N5/N5.5/N6 closure)

Round-3 review accumulated three naming conventions across passive docs:
- **N5** — channels MUST be enumerated explicitly in the passive doc §VFX section as `Ch1` / `Ch2`.
- **N5.5** — channel ID is also the implementation layer: Ch1 → listener `notify` (trigger-proc); Ch2 → observer + `VfxEmitter` (persistent-presence).
- **N6** — a passive may have Ch1 only, Ch2 only, or both. **Reactive-proc** sub-variant typically only Ch1. **Aura-loop** sub-variant typically only Ch2. **State-watch** typically both (arm flash + persistent tint).

Codified table (use as authoring template for `digimon/<name>/04_passive_*.md` §VFX):

| Sub-variant | Ch1 (trigger-proc) | Ch2 (persistent-presence) | Both? |
|---|---|---|---|
| A. Aura-loop | rare (start/stop flash on combat begin/end) | **mandatory** (the aura itself) | uncommon |
| B. Reactive-proc | **mandatory** (proc flash) | optional (briefly held armed state) | depends on tell needs |
| C. State-watch | **mandatory** (state-transition flash) | **mandatory** (tint/glow while state held) | **standard** |

**Empty channels are not allowed.** Every passive has VFX on at least one channel (§A.0 mandate). A passive doc that lists "no VFX" fails review.

## §F — Channel selection rule (Full FSM era)

Decision rule for an FX in a passive design doc, post-Full-FSM mandate:

```
What is the trigger semantic of this passive (or sub-channel)?
  Aura-loop      → Ch2 mandatory (persistent), Ch1 optional (start/stop flash)
  Reactive-proc  → Ch1 mandatory (proc flash), Ch2 optional (armed state hold)
  State-watch    → Ch1 + Ch2 both mandatory (transition flash + held tint)
```

Then for each FX, pick `SpawnParticle` motion (§2.2d §C) per its semantic intent (§2.2d §C.2):
- proc flash on FSM node entry → `Static` on `EntityCenter(Self_)` (Ch1)
- aura on persistent state → `Static` motion, observer-managed `VfxEmitter` (Ch2)
- partner-link beam → `Travel` from caster to self with `EntityCenter` endpoints (Ch1 or Ch2 depending on whether it's instant or held)
- proc on event target → `Static` on `EntityCenter(EventTarget)` (Ch1 only; Ch2 would not have event context — observer reads state, not events)

A passive doc (`digimon/<name>/04_passive_*.md`) §VFX section is expected to list each FX with its channel + motion + sub-variant explicit, so the implementer reads off the contract without re-deciding.

## §G — Determinism and headless

Full FSM mandate (§A.0) makes passive FSMs **gameplay-relevant** — they can emit Commands like `ApplyBuff`/`EmitDamage`/`AdvanceTurn`/`EmitSpGrant` (§2.2b §C2) just like skill FSMs. Therefore the passive FSM ticks headless. Only the cosmetic Commands (`SpawnParticle`/`Shake`/`Sound`) no-op headless, per §2.2b §G.

- **Passive FSM** ticks under `cargo test` headless. `tick_passive_fsm` is pure rel. (rt, events_since_last_tick) — replay-stable. Tests in `tests/` exercise passive gameplay (Twin Core boost, Predator Loop trigger, Holy Aegis DR application) directly through the FSM tick. ✅
- **Channel 1 (listener `notify`)** is no-op in headless: `ListenerCtx::notify` pushes to a presentation bus that is `#[cfg(feature = "windowed")]`-gated. Listener gameplay logic (buff application, kernel event emission) is unaffected. ✅
- **Channel 2 (observer)** is not compiled in headless (entire `presentation/passive_observers.rs` is `#[cfg(feature = "windowed")]`). The component diff it observes (`Added<Buff_X>` / `RemovedComponents<Buff_X>`) is still inserted/removed by the kernel headless — only the observer that reads the diff is gated. ✅
- **`Buff_*` typed components** are inserted by the kernel regardless of feature flag (uniform code path between headless and windowed). Observer (windowed-only) consumes the diff. Decision per §E note B (alternative B selected) — headless build does the typed-component bookkeeping anyway because some gameplay code may want `Added<Buff_X>` for non-VFX reasons (e.g. test assertions). Negligible perf cost; uniform.
- **Validator** (§2.2b §L) extended to passive FSMs: same checks (entry exists / reachability / dangling edges / frame-range in-bounds / Command params reference exist). Passive-specific check: every passive FSM has at least one VFX-emitting `on_enter` across all reachable nodes (enforces §A.0 mandate).

## §H — Cross-refs

- §2.2b `02-02b_animation_fsm.md` §B: dual-role blueprint (listener side is what this doc serves).
- §2.2b §C: `SpawnParticle` Command (FSM channel — what this doc complements for passives).
- §2.2b §G: cosmetic vs gameplay tagging — `notify`-emitted FX is cosmetic, headless-safe.
- §2.2d §B: `VfxLocus` + `EntityRef` — Channel 1 and Channel 2 both consume these.
- §2.2d §J: `VfxEmitter` component contract — Channel 2 spawns it.
- `digimon/agumon/04_passive_twin_core_fire.md` §VFX: Twin Core fire-side as worked example (Channels 1 + 2).
- `digimon/dorumon/04_passive_predator_loop.md` §VFX: Predator Loop as worked example (Channels 1 + 2, with migrating mark).

## §I — Open gaps

1. **Stack-aware persistent FX.** If a `BuffId` can stack (e.g. Heated × N), the typed component is either non-counting or carries a stack count. `Added<T>` fires only on 0→1 transition; stack 1→2 would not re-trigger. Decision: presentation FX driven by stack count needs `Changed<T>` + a count field on the typed component, not `Added<T>`. Document case-by-case when it first arises.
2. **Multiple persistent FX on the same buff.** If a buff wants both an aura on self and a mark on target, the observer spawns two `VfxEmitter` entities — one parented to self, one to the marked target. The current `VfxEmitter` component is single-emitter per entity, so additional FX live on additional entities. No schema change required.
3. **Numeric-binding FX** (e.g. "aura intensity proportional to remaining buff duration"). Neither channel binds presentation to a continuous numeric value. Workaround for v0: discrete tiers via separate components (`Buff_X_Low` / `Buff_X_High`). Real solution would be a shader uniform pipeline; **deferred** until a passive concretely needs it.
4. **Observer ordering.** Multiple observers reacting to the same component-diff are ordered by Bevy system ordering, which is non-deterministic across builds unless explicitly chained. For pure FX (no state mutation), order is cosmetic — no determinism risk. If an observer ever mutates gameplay state, it has been mis-placed (move to listener). Validator: any observer that writes outside `Commands` for `VfxEmitter` / sprite components fails review.
