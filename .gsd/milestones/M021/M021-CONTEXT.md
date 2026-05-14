# M021 — Kernel framework + Timeline FSM + Registry<E>

## Obiettivo

Sostituire l'attuale schema "enum `Effect` data-driven + plugin Digimon hardcoded nel kernel" con un **framework piatto e generico** in cui:

1. Il kernel espone solo primitive: `Intent` come unica mutazione, `SkillCtx` come unico contesto, `CompiledTimeline` come unica forma di "skill", `Registry<E: ExtPoint>` come unica forma di "extension axis", `SignalBus` come bus reattivo, `Clock` con due modalità (HeadlessAuto / Windowed).
2. Una skill è **una graph data** (`CompiledTimeline`) che referenzia fn-by-id su 7 assi (`hook`, `selector`, `predicate`, `formula`, `tick`, `ai_utility`, `cue`). Niente trait per skill, niente enum effect.
3. Un blueprint Digimon è **un solo modulo + un solo `register(reg: &mut ExtRegistries)`**. Nessun trait `Blueprint`, nessun registry separato per blueprint, nessun nome Digimon nel kernel.
4. La skilltree è **context immutabile per il run**, letta dai `predicate` di edge-gate; talenti che abilitano/disabilitano branch sono runtime-gate (D033), non patch compile-time (D027 demoted).

Il framework è validato da spike standalone (33/33 verde, 4 pattern architettonicamente distinti: Loop+skilltree-gate, blueprint-state mutabile, cross-blueprint identity filter, RNG-gated edge).

## Scope

### Foundation framework

- **F1. Intent canon**: enum chiuso, ~18 varianti (DealDamage, ApplyStatus, ApplyBuff, AdvanceTurn, DelayTurn, EnqueueFollowUp, BlueprintSignal{owner, payload}, SetBlueprintState{actor, key, value, cast_id}, Reject, …). `intent_applier` FIFO drena la coda e route per variante; nessuna mutazione kernel-side fuori da `intent_applier`.
- **F2. `SkillCtx<'a>`**: contesto skill con accessor read-only (`adjacents`, `predict_damage`, `unit_state`, `alive_enemies`, `peek_pending`, `blueprint_state(actor, key)`, `identity_of(unit)`, `cast_hit_set()`, `skilltree(actor)`, `rng_u32(cast_id, beat, hop, salt)`) + `enqueue(Intent)`. Mode tri-stato: `DryRun / Execute / Preview` (D024), invariante `DryRun ≡ Execute` su Intent stream.
- **F3. `ExtPoint` + `Registry<E>` + `ExtRegistries`**: pattern unificato `id → fn` per ogni asse. ~3 righe per nuovo asse (marker struct + impl ExtPoint + campo aggregato). Lookup via `&'static str`. Built-in fns registrati dal kernel coprono la maggioranza canon (primary, all_enemies, adjacent_to_primary, lowest_hp_pct, atk_scaling, dot_tick, has_target_alive, …).
- **F4. Timeline FSM**: `CompiledTimeline = Vec<Beat> + Vec<BeatEdge>`. `BeatKind::{Impact { hook, selector, presentation }, Loop { body, exit_when }, CastStart, CastEnd}`. `BeatRunner` con `LoopFrame` (single-level v0) emette `BeatEvent { cast_id, beat, hop_index, caster, primary_target, beat_targets }` per ogni step. `validate_timeline_refs` ricorsiva (incl. `Loop.body` ed `exit_when`) a `App::finish()`.
- **F5. `SignalBus` + `PassiveRunner`**: bus globale per signal cross-blueprint enum-chiusi (D028, validation a `App::finish()`). Passive Digimon sono `CompiledTimeline` listener-driven (un `PassiveRunner` separato per FSM passive — Renamon `kitsune_grace`, Gabumon `fur_cloak`, Tentomon `battery_loop`).
- **F6. Skilltree immutable**: `SkillTree { unlocked, ranks }` come Component per-unit immutabile per il run. Predicate gates leggono via `ctx.skilltree(actor)`. Sblocchi tra encounter (Slay-the-Spire-style), mai in-cast.
- **F7. Two-clock model**: `HeadlessAuto` (test) consuma BeatEvent immediato; `Windowed` (gioco) stalla su `Presentation::Cue(CueId)` finché animation completa, poi avanza. Invariante (D026): stesso Intent stream prodotto dai due path.
- **F8. RON → `CompiledTimeline` compiler**: load-time. `skills.ron` v2 = numeri/tag + grafo di beat con string id verso le registry. Typo nel RON = errore al boot. `units.ron` invariato (è già numeri).

### Plugin & lifecycle

- **P1. `CombatPlugin` extract**: separazione da `register_combat_kernel_runtime`. Vincolo: nessun import `bevy::winit`/`bevy::render`/`bevy_egui` nel plugin core (verificato da `cargo check`).
- **P2. `cast_id: CastId(NonZeroU32)`**: aggiunto a `CombatEvent` + `BeatEvent` + propagato da `pipeline::step_app` a tutti i call-site (D009 precondition).
- **P3. Ult instant cast**: separazione `TacticalCyclePhase::UltInstant` come bypass turn advance (D010 precondition).
- **P4. Turn-phase 5-step**: `PreTurnTick → PostTickKoResolve → TurnStartHooks → PostHooksKoResolve → BuildFreshSnapshot → SelectActionAndCast` come `SystemSet` Bevy espliciti (D011).

### Migration

- **M1. Built-in extension fns**: kernel registra il set canonico (selector, predicate, formula, tick) prima della migration skill, così le skill canon si esprimono in RON+grafo senza blueprint code aggiuntivo.
- **M2. Migrate 18 active skill canon** → `CompiledTimeline` in `skills.ron`. Drop `enum Effect`, drop `apply_effects()` in `resolution.rs`. Bounce → `BeatKind::Loop` con `exit_when` (sostituisce `MultiHitOnKO::*` policy).
- **M3. Migrate 6 passive canon** come `PassiveRunner` driven da `SignalBus`. Modifier pipeline aggregator (intrinsic-modifier dei status + Ability modifier + Buff modifier) con `ModifierCondition` ricco (D017, D018) e `EventFilter::{All, Any, Not, Custom}` (D021).
- **M4. Migrate 6 blueprint Digimon** = 6 moduli `src/combat/blueprints/<x>/` con un solo `register(reg)`. Mini-plugin `twin_core` per shared-mechanic Agumon↔Gabumon (D005). Niente trait `Blueprint`.
- **M5. Cleanup kernel**: rimozione delle 5 variant Digimon-specific da `CombatKernelTransition` (`TwinCore`, `BatteryLoop`, `HolySupport`, `PredatorLoop`, `PrecisionMindGame`) e dei sub-snapshot in `observability.rs`. Sostituiti da `CombatKernelTransition::Blueprint { owner, payload }` generico.

### Consumers

- **C1. UI/AI riscritti via `SkillCtx::Mode::Preview`**: `query_skill_preview` chiama il runner in `Preview` (no-apply) e legge l'`Intent` stream prodotto; `combat_panel.rs` consuma direttamente, `predict_damage` UI-side rimosso. AI scoring usa helper esterno sullo stesso stream.
- **C2. `RosterEntry` blueprint-keyed payload**: `UnitDef` perde field hardcoded Digimon-specific (`twin_core`, `holy_support`, …). Ogni blueprint dichiara la propria validation rule via fn registrata in `Registry<ValidationExt>`.
- **C3. `ValidationSnapshot` field-from-registry**: popolata da `ExtRegistries` iter. Aggiungere un Digimon non richiede edit a `units_ron.rs`.

## Vincoli

- **P001 (Kernel generico, K001 in KNOWLEDGE)**: il kernel non menziona mai nomi di Digimon. `rg "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/` → 0 righe a fine M021 (escluse le subdir `blueprints/`).
- **Intent come unica mutazione**: hooks/predicate/formula/tick **non mutano stato direttamente**. Producono `Intent` via `ctx.enqueue(...)`. Il kernel li applica nel pipeline. Replay-reconstructible end-to-end.
- **Determinismo (I1)**: stesso input ⇒ stesso `Intent` stream. RNG seeded via `(rng_seed, cast_id, beat, hop_index, salt)`. Nessun `HashMap` iteration order leak, nessun wall-clock.
- **DryRun ≡ Execute (I2 / D024)**: il runner in `DryRun` produce lo stesso Intent stream di `Execute`, senza applicare. UI/AI usano questo path.
- **Signal-gating Windowed (I3 / D026)**: la timeline FSM stalla solo su `Presentation::Cue`, mai su altri beat. Intent stream end-of-cast identico tra HeadlessAuto e Windowed.
- **Validation strict a boot (I5 / D031)**: ogni id referenziato da `CompiledTimeline` (hook, selector, predicate, cue, `Loop.exit_when`) deve risolvere in `ExtRegistries`. Mancanza = errore a `App::finish()`, mai a runtime.
- **Skilltree immutabile per il run (D033)**: nessun `Intent` muta lo skilltree in-cast. Sblocchi avvengono tra encounter.
- **Headless first**: `CombatPlugin` gira senza feature `windowed`. Gating `#[cfg(feature = "windowed")]` solo per egui/winit.
- **No-DSL / no-scripting**: niente Turing-completeness in RON, niente Rhai/Rune (D010 originale). Logica vive in fn Rust registrate.

## Demo

A milestone closed:

- `rg "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/` (escluse `blueprints/`) → 0 righe.
- `rg "enum Effect" src/data/skills_ron.rs` → 0 righe; `skills.ron` contiene solo numeri + grafo + string id.
- 24 skill canon eseguono via `CompiledTimeline` su `SkillCtx`; suite `tests/` (74+) verde end-to-end.
- 6 passive canon eseguono via `PassiveRunner` su `SignalBus`.
- 6 blueprint Digimon = 6 file `src/combat/blueprints/<x>/mod.rs` + 6 dir RON `assets/data/digimon/<x>/`. Nessun codice Digimon-specific altrove.
- JSONL contiene `CombatKernelTransition::Blueprint { owner, payload }` per ogni `BlueprintSignal` round-trip su encounter scriptato.
- Aggiungere un Digimon nuovo (test scriptato) tocca solo `src/combat/blueprints/<new>/` + `assets/data/digimon/<new>/`.
- `cargo run --features windowed` smoke su un encounter completo: comportamento indistinguibile dal pre-refactor.

## Riferimenti

- **D005** — Shared-mechanic mini-plugin (Twin Core paired).
- **D008** — `CombatKernelTransition::Blueprint { owner, payload }` come unica variant blueprint-side (le 5 Digimon-specific eliminate).
- **D009** — `cast_id` su `CombatEvent` + `BeatEvent` (precondition cast-scoped hooks).
- **D010** — Ult instant cast (precondition turn-phase order).
- **D011** — Turn-phase 5-step esplicito come `SystemSet`.
- **D017** — `ModifierCondition` canon list ricco.
- **D018** — `StatusDef::intrinsic_modifiers` data-driven (Blessed/Heated/Chilled symmetry).
- **D021** — `EventFilter::{All, Any, Not, Custom}` per listener compositi.
- **D023** — Hook come fn-by-id registrato (no trait object).
- **D024** — `SkillCtxMode { DryRun, Execute, Preview }`.
- **D025** — Timeline FSM strato esplicito (`CompiledTimeline`).
- **D026** — Two-clock model (`HeadlessAuto` / `Windowed`).
- **D028** — Signal taxonomy enum chiuso, registrato a `App::finish()`.
- **D029** — `next_from` first-passing edge + fallback unconditional invariant.
- **D030** — Selector come fn-by-id registrato (mirror di D023).
- **D031** — Pattern unificato `Registry<E: ExtPoint>` (7 assi).
- **D032** — Un solo modulo + un solo `register()` per Digimon.
- **D033** — Skilltree immutable runtime context + `BeatKind::Loop` + `BeatEvent.hop_index` option A.
- **D027** — *Demoted post-spike*: compile-time `TimelinePatchOp` resta disponibile solo per topology rewrite non rappresentabili come edge-gate; non sul critical path v0.
- **D034** *(pending)* — `Intent::SetBlueprintState` come canonical write-path per state per-unit-per-key.
- **K001 / P001** — Kernel generico, specifiche fuori dal kernel.
- **Spike `M021-timeline-fsm`** — 33/33 verde, `FINDINGS.md` con 17 finding + 4 pattern fixture (Loop+skilltree-gate / blueprint-state mutabile / cross-blueprint identity filter / RNG-gated edge).
- **`M021-RESEARCH.md`** — capability matrix, design surface, gap analysis.

## Dipendenze

- **Richiede chiuso**: M018 (target shape, Bounce, selectors), M019 (DR pipeline), M020 (reactive bus uniforme + shim removal kernel-side).
- **Sblocca**: M022 (asset pipeline, ortogonale), M023+ (visual stack), M024–M028 (nuovi roster — nascono direttamente sopra il framework, niente debito strutturale).

## Non-scope (deferred)

- Multi-level `BeatKind::Loop` nesting (v0 single-level basta — verificato sul roster v0).
- Scripting embedded / hot-reload skill logic (vincolo utente esplicito, post-1.0).
- Stack-aware status numerici (Heated × N DoT scaling) — D009 deferred storico, indipendente da M021.
- `Intent::SetSkillTree` (skilltree mutabile in-cast) — esplicitamente fuori scope (D033).
- Passive `PassiveRunner` con timeline ramificate complesse → v0 ammette grafi line-shaped; ramificazione cross-signal differita.
