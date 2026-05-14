# M021 — Research & Design

**Status:** consolidato post-spike `M021-timeline-fsm` (33/33 verde). Documento di design corrente, niente storico.

## 1. Target architecture (post-spike)

### 1.1 Kernel core (stabile, generico)

Il kernel `src/combat/` espone una superficie piatta, senza nomi di Digimon e senza trait per skill o blueprint:

- **`Intent` enum (~18 variant)** — unica forma di mutazione: `DealDamage`, `Heal`, `ApplyStatus { mode }`, `ApplyBuff { kind, stack_mode }`, `RemoveStatus`, `Cleanse`, `AdjustToughness`, `Stun`, `ChangeSP`, `ChangeUlt`, `AdvanceTurn(u32)`, `DelayTurn(u32)`, `EnqueueFollowUp`, `BlueprintSignal { owner: &'static str, payload: Box<dyn Any+Send+Sync> }`, `SetBlueprintState { actor, key, value, cast_id }`, `Reject { reason }`.
- **`intent_applier` system** — FIFO drain, route per variant. Unica fonte di mutazione kernel-side. Tutti gli `Intent` con `cast_id` finiscono nel JSONL.
- **`SkillCtx<'a>`** — contesto skill: read-only accessor (`adjacents`, `predict_damage`, `unit_state`, `caster_hp_pct`, `alive_enemies`, `alive_allies`, `peek_pending`, `blueprint_state(actor, key)`, `identity_of(unit)`, `cast_hit_set()`, `skilltree(actor)`, `rng_u32(cast_id, beat, hop, salt)`) + `enqueue(Intent)`. Tri-modale via `SkillCtxMode { DryRun, Execute, Preview }` (D024).
- **`ExtPoint` + `Registry<E>` + `ExtRegistries`** — pattern unificato fn-by-id (D031). Ogni asse = struct marker + impl ExtPoint + campo nello struct aggregato:
  ```rust
  pub trait ExtPoint: 'static { type Fn: Copy; const KIND: &'static str; }
  pub struct Registry<E: ExtPoint> { fns: HashMap<&'static str, E::Fn>, .. }
  ```
  7 assi v0:

  | Asse | Signature |
  |---|---|
  | `HookExt` | `fn(&BeatEvent, &mut SkillCtx)` |
  | `SelectorExt` | `for<'a> fn(&SelectorCtx<'a>) -> Vec<UnitId>` |
  | `PredicateExt` | `fn(&BeatEvent, &SkillCtx) -> bool` |
  | `FormulaExt` | `for<'a> fn(&FormulaCtx<'a>) -> i32` |
  | `TickExt` | `fn(&StatusInstance, &mut SkillCtx)` |
  | `AiUtilityExt` | `for<'a> fn(&AiCtx<'a>) -> f32` |
  | `CueExt` | `for<'a> fn(&CueCtx<'a>) -> CueId` |

- **`CompiledTimeline`** — graph data per ogni skill: `Vec<Beat> + Vec<BeatEdge>`. `BeatKind::{Impact { hook, selector, presentation }, Loop { body: Vec<Beat>, exit_when: PredicateId }, CastStart, CastEnd}`. Loop single-level v0 (verificato sufficiente sul roster).
- **`BeatRunner`** — esegue una `CompiledTimeline`. `LoopFrame` interno mantiene `body_cursor + hop_index`. Emette `BeatEvent { cast_id, beat, hop_index, caster, primary_target, beat_targets }`. `next_from` semantica first-passing edge + fallback unconditional (D029) per evitare halt accidentale.
- **`PassiveRunner`** — variant del runner che consuma `CombatEvent` dal `SignalBus` e driva `CompiledTimeline` listener-shaped. Stessa Intent emission del `BeatRunner`.
- **`SignalBus`** — bus globale per signal cross-blueprint. Taxonomy enum-chiusa registrata a `App::finish()` (D028). Listener attivati per match + `EventFilter::{All, Any, Not, Custom}` (D021).
- **`Clock { HeadlessAuto, Windowed }`** — two-mode (D026). HeadlessAuto consuma BeatEvent immediato; Windowed stalla su `Presentation::Cue(CueId)` finché animation completa. Invariante: stesso Intent stream end-of-cast.
- **`SkillTree { unlocked, ranks }`** — Component per-unit immutable per il run (D033). Letto solo via `ctx.skilltree(actor)`. Sblocchi tra encounter.
- **RNG seeded** — SplitMix64 + FNV. `ctx.rng_u32(cast_id, beat, hop, salt)` deterministico. Seed parte di replay payload.
- **`validate_timeline_refs`** — ricorsiva (incl. `Loop.body` ed `exit_when`), chiamata a `App::finish()`. Ogni id referenziato da timeline (hook, selector, predicate, cue) deve risolvere. Errore = abort boot.

### 1.2 Cosa il kernel **non** sa

- Nomi di Digimon (no `TwinCore`, `BatteryLoop`, `HolySupport`, `PredatorLoop`, `PrecisionMindGame`, `KitsuneGrace`).
- Forma delle skill specifiche (Bounce, Blast, AoE, Heal, Cleanse, Stun…): vivono come `CompiledTimeline` in RON.
- Trait per skill o blueprint (rimossi rispetto al design pre-spike).
- Mutazione diretta del World da hook/predicate/formula/tick — sempre via `Intent`.

### 1.3 Blueprint Digimon

Un blueprint Digimon vive in:

```
src/combat/blueprints/<x>/mod.rs    — un solo modulo, un solo fn register(reg)
assets/data/digimon/<x>/             — RON: identity, skills, signals, talents, default_skilltree
```

Niente codice Digimon-specifico **altrove** (D032). Il kernel non importa `agumon`, `gabumon`, etc.; al boot il `CombatPlugin` chiama `crate::blueprints::register_all(&mut reg)` che itera la lista dei `register` fn.

**Pattern blueprint**:
```rust
pub fn register(reg: &mut ExtRegistries) {
    reg.hooks.register("agumon::on_impact_baby_burner", on_impact_baby_burner);
    reg.selectors.register("agumon::baby_burner_splash", baby_burner_splash);
    reg.predicates.register("agumon::has_pyromaniac", has_pyromaniac);
    reg.formulas.register("agumon::fire_atk_scaling", fire_atk_scaling);
    reg.ticks.register("agumon::heated_tick", heated_tick);
    reg.cues.register("agumon::charge_by_hp", charge_by_hp);
    reg.ai_utilities.register("agumon::burner_utility", burner_utility);
}
```

I `register` fn sono raccolti in un `Vec<fn(&mut ExtRegistries)>` al boot.

### 1.4 Shared-mechanic mini-plugin (D005)

Per mechanic condivise da più Digimon (Twin Core paired Agumon↔Gabumon), un mini-plugin `src/combat/blueprints/twin_core/` espone:
- Helper fn registrabili (es. predicate `twin_core::is_paired_with`, formula `twin_core::ice_arm_dmg`).
- Eventuali listener su `SignalBus` per arming/counter-state.

Nessun Digimon-specific naming nel mini-plugin: rappresenta la mechanic, non gli owner. Aggiungere un terzo Digimon che partecipa = consumer-side registration nel suo modulo.

## 2. Pattern architetturali validati (spike)

I 4 pattern sotto coprono tutti i casi del roster v0; sono stati implementati in `.gsd/workflows/spikes/M021-timeline-fsm/` e portati come fixture in `tests/`.

### 2.1 Loop + skilltree-gate (Agumon Bouncing Fire)

Talenti che abilitano/disabilitano branch nella FSM. Il branch è **sempre presente** nel grafo. Lo skilltree decide a runtime se la gate edge passa.

- `BeatKind::Loop { body, exit_when }`: body = `Vec<Beat>` (hop), `exit_when: PredicateId` valutato dopo ogni hop.
- Edge gate da anchor: `aftermath → bounce_loop` con `gate: Some("agumon::has_bouncing_fire")`.
- Edge fallback: `aftermath → cast_end` senza gate (D029).
- Con `rank("bouncing_fire") = 0`, l'Intent stream è letteralmente identico al timeline base senza branch.

Conseguenza: D027 (compile-time `TimelinePatchOp`) **demoted** dal critical path. Resta disponibile solo per topology rewrite genuini (rari).

### 2.2 Blueprint state mutabile (Dorumon Predator Loop)

State per-unit-per-key, mutato dall'Intent stream.

- **Read**: predicate `dorumon::predator_active` legge `state.blueprint_state[(unit, "dorumon.predator_active")]` via `ctx.blueprint_state(...)`.
- **Write**: hook emette `Intent::SetBlueprintState { actor, key, value, cast_id }` (D034 pending). Il kernel applica nello stream insieme a tutte le altre mutazioni.
- **Replay-reconstructible** end-to-end.

### 2.3 Cross-blueprint identity filter (Gabumon Twin Core ice)

Filtraggio per identità del caster, senza coupling fra blueprint.

- Predicate `gabumon::is_caster_agumon` legge `ctx.identity_of(evt.caster) == "agumon"`.
- Nessun nuovo asse del Registry, nessun import cross-blueprint.
- Il blueprint Gabumon registra la predicate; nessuna modifica al blueprint Agumon.

### 2.4 RNG-gated edge (Tentomon Block Reaction)

Edge probabilistici deterministici.

- Predicate `tentomon::rng_below_70` chiama `ctx.rng_u32(cast_id, beat, hop_index, salt)`.
- Threshold parametrica (`threshold_pct`).
- Determinismo I1 preservato: stesso `(rng_seed, cast_id, beat, hop_index, salt)` ⇒ stesso draw.
- Production richiede `state.rng_seed` come parte del replay payload.

## 3. Capability matrix — coverage roster v0

| Capability | Implementazione |
|---|---|
| Damage scaling per ATK/DEF/HP | `FormulaExt` registrato dal kernel (`atk_scaling`, `def_scaling`, `hp_pct_scaling`) |
| Bounce / multi-hit | `BeatKind::Loop` body singolo + `exit_when` (pool exhaustion, tier cap, etc. in OR) |
| AoE / blast | Selector built-in (`all_enemies`, `adjacent_to_primary`) |
| Heal / cleanse | `Intent::Heal`, `Intent::Cleanse` |
| Status apply | `Intent::ApplyStatus { mode: Stack/Refresh/MaxOf }` (D016) |
| Buff apply | `Intent::ApplyBuff { kind, stack_mode: MaxReplace/ProcOnce/Additive }` (D019) |
| Intrinsic status modifier | `StatusDef::intrinsic_modifiers` data-driven (D018) — Heated/Chilled/Blessed simmetria |
| DoT tick | `TickExt` registrato dal kernel/blueprint (`heated_tick`, etc.) |
| Turn manipulation | `Intent::AdvanceTurn(u32)`, `Intent::DelayTurn(u32)` con cap ±50% al emission |
| Follow-up reactions | `Intent::EnqueueFollowUp { skill_id, caster, target }` |
| Passive reattiva | `PassiveRunner` listener su `SignalBus` + `EventFilter` (D021) |
| Modifier pipeline (passive stat boost) | `modifier_aggregator` raccoglie da Ability+Status+Buff con `ModifierCondition` (D017) |
| Talents skilltree | `ctx.skilltree(actor).rank("…")` letto da predicate gates (D033) |
| Cross-blueprint signal | `Intent::BlueprintSignal { owner, payload }` → SignalBus → listener; `CombatKernelTransition::Blueprint` per JSONL |
| Cross-blueprint identity filter | Predicate `ctx.identity_of(unit) == "…"` |
| Stochastic edge | Predicate `ctx.rng_u32(...) < threshold` |
| Blueprint state mutabile | `Intent::SetBlueprintState` + `ctx.blueprint_state(...)` |
| Animation cue dinamico | `Presentation::Dynamic(CueId)` risolto da `CueExt` fn (es. `charge_by_hp`) |
| Multi-FSM per Digimon (Gabumon dual-path) | N `CompiledTimeline` registrate, stesso `register()` |
| Validation per-blueprint | `Registry<ValidationExt>` letto da `ValidationSnapshot` builder |

**Coverage**: 18/18 active skill canon + 6/6 passive canon mappate. Nessun pattern richiede estensioni al framework.

## 4. Invarianti core

| # | Invariante | Verifica |
|---|---|---|
| **I1** | Determinismo: stesso input ⇒ stesso Intent + CombatEvent stream | Replay test seed-fisso, 2 run, `assert_eq` |
| **I2** | DryRun ≡ Execute (D024): stesso skill, stesso ctx, stessa snapshot ⇒ stesso Intent stream | Fixture parallela DryRun vs Execute |
| **I3** | HeadlessAuto.intents ≡ Windowed.intents end-of-cast (D026) | Fixture two-clock con stub auto-resolve cue |
| **I4** | Validation strict per id referenziati dal timeline (D031) | `validate_timeline_refs` a `App::finish()` |
| **I5** | Kernel ignora i nomi di Digimon (P001) | `rg "TwinCore|..." src/combat/ --glob '!blueprints/**'` → 0 righe |
| **I6** | Mutazione solo via `intent_applier` | `rg "world.entity_mut|commands.entity" src/combat/api/` → 0 (escluso intent_applier) |
| **I7** | Skilltree immutable in-cast (D033) | Nessun `Intent::SetSkillTree`; ricerca nel codice = 0 |
| **I8** | Signal taxonomy enum-chiusa (D028) | Validation a `App::finish()`; signal non registrato = panic |

## 5. Slicing M021

Vedi `M021-ROADMAP.md` per slice S01–S12. La struttura riflette la framework-first migration:

1. **Foundation** (S01–S05): kernel framework + timeline FSM + mode parity + signal/passive + built-in fns + RON compiler.
2. **Migration** (S06–S10): skill canon + passive canon + 6 blueprint Digimon (uno per slice o paired) + kernel digimon-free.
3. **Consumers** (S11–S12): UI/AI + roster/validation da registry.

Dipendenze più strette:
- S02 (Timeline FSM) → blocca S03, S05, S06, S07.
- S04 (SignalBus + PassiveRunner + BlueprintSignal dispatcher) → blocca S07–S10.
- S05 (built-in fns + RON compiler) → blocca S06 (migrare skill canon richiede built-in fns).
- S06 (migrate active) + S07 (migrate passive + modifier pipeline) → blocca tutti i blueprint slice S08–S10.

## 6. Decisioni load-bearing (sintesi)

Persistite in `.gsd/DECISIONS.md`:

- **D005** Shared-mechanic mini-plugin (Twin Core paired)
- **D008** `CombatKernelTransition::Blueprint { owner, payload }` (le 5 Digimon-specific eliminate)
- **D009** `cast_id: CastId(NonZeroU32)` su `CombatEvent` + propagazione call-site
- **D010** Ult instant cast (precondition turn-phase order)
- **D011** Turn-phase 5-step esplicito come `SystemSet`
- **D013** Listener ordering tiebreaker `(speed_initiative DESC, slot_index ASC, team_id ASC)`
- **D014** Circuit breaker debug-only @256 evt/cast
- **D016** `StatusApplyMode { Stack, Refresh, MaxOf }`
- **D017** `ModifierCondition` canon list (SourceUnit/SourceTag/TargetHasStatus/TargetIsAlly/Pipeline + All/Any/Not)
- **D018** `StatusDef::intrinsic_modifiers` data-driven (Blessed/Heated/Chilled simmetria)
- **D019** `Intent::ApplyBuff { kind, stack_mode: MaxReplace/ProcOnce/Additive }`
- **D021** `EventFilter::{All, Any, Not, Custom}` per listener compositi
- **D023** Hook come fn-by-id registrato (no trait object)
- **D024** `SkillCtxMode { DryRun, Execute, Preview }`
- **D025** Timeline FSM strato esplicito (`CompiledTimeline`)
- **D026** Two-clock model (HeadlessAuto / Windowed)
- **D028** Signal taxonomy enum chiuso, registrato a `App::finish()`
- **D029** `next_from` first-passing edge + fallback unconditional invariant
- **D030** Selector come fn-by-id registrato (mirror di D023)
- **D031** Pattern unificato `Registry<E: ExtPoint>` (7 assi)
- **D032** Un solo modulo + un solo `register()` per Digimon
- **D033** Skilltree immutable runtime context + `BeatKind::Loop` + `BeatEvent.hop_index` option A

**Demoted**:
- **D027** Compile-time `TimelinePatchOp` — disponibile solo per topology rewrite non rappresentabili come edge-gate. Pyromaniac re-espresso come beat opzionale gated da predicate.

**Pending**:
- **D034** `Intent::SetBlueprintState { actor, key, value, cast_id }` come canonical write-path per state mutabile per-unit-per-key. Pronta per persistenza prima di S09.

**Superseded de facto** (da rivedere prima di S01):
- **D007** (vecchio "trait Blueprint + BlueprintRegistry") — assorbito da D032 (un modulo + un register fn). Nessun trait per blueprint richiesto. Persistenza decisione di supersession raccomandata.

## 7. Gap residui (da decidere prima/dentro M021)

### G-A — `Intent::SetBlueprintState` (D034 pending)

Status: validato dallo spike (Predator Loop fixture). Decisione marginale: scegliere se varianti tipizzate per i casi noti vs `value: i32` generico. Lo spike usa `i32`; sufficiente per il roster v0 (counter, flag). Persistere D034 prima di S09.

### G-B — Passive listener FSM (PassiveRunner) shape

Lo spike F16 segna esplicitamente "fuori scope spike". S04 deve materializzare la shape:
- Driver: `CombatEvent` dal `SignalBus`?
- Lifecycle: persistente per-unit (FSM con state mantenuto tra cast) o ricostruita ogni signal?
- Multi-FSM per Digimon (Gabumon dual-path): N runner indipendenti o singolo runner che route per FSM-id?

**Proposta corrente**: PassiveRunner persistente per-unit-per-FSM, driven da signal match + EventFilter. Validare con fixture `kitsune_grace` in S04 prima delle altre 5 passive.

### G-C — RON shape per `CompiledTimeline`

Lo spike costruisce timeline in Rust diretto. Per S05 servono:
- Schema RON v2 (estensione di `skills.ron`): grafo di beat + edge + presentation + selector/hook/exit_when id.
- Compiler load-time `RonSkill → CompiledTimeline` (validation strict).
- Backward-compat con `skills.ron` v1 durante la migration S06 (registry doppio temporaneo).

Decisione concreta da prendere in S05: una skill = un file RON, o `skills.ron` resta single-file? Roster v0 fit-check raccomanda single-file per ora (74 skill ≪ soglia di split).

### G-D — ID interning: `&'static str` vs `SkillId(Interned<str>)`

Lo spike usa `&'static str`. Production-grade: probabilmente `Interned<Symbol>` per:
- Confronti O(1) e debug-friendly.
- Stable id che sopravvive a `Box::leak` hack.

**Proposta**: tenere `&'static str` per S01–S07 (semplice, zero deps); decidere se introdurre interning in S11 quando UI consuma frequentemente per id.

### G-E — Loop nesting depth

Spike implementa single-level `BeatKind::Loop`. Roster v0 verificato sufficiente (nessuna skill nest-loops). Decisione: documentare il limite esplicitamente o introdurre depth check. **Proposta**: depth check (`debug_assert!(loop_depth <= 1)` in `BeatRunner`) + error in `validate_timeline_refs`. Costo: ~5 righe, evita scoperta tardiva.

### G-F — D008 final shape

Pre-spike D008 prevedeva `CombatKernelTransition::Blueprint { owner, payload }` come double-layer routing via `Intent::BlueprintSignal`. Post-spike il flow è:
1. Hook emette `Intent::BlueprintSignal { owner, payload }`.
2. `intent_applier` route a `SignalBus` + emette `CombatKernelTransition::Blueprint { owner, payload }` per JSONL.
3. Listener consuma da `SignalBus`.

Il "double-layer" originale resta valido, ma il routing kernel-side è unificato in `intent_applier` (non in un dispatcher separato). Aggiornare D008 con la shape definitiva durante S04.

### G-G — Cue duration mapping in Windowed clock

Lo spike usa stub auto-resolve. Production Windowed: mapping `CueId → animation duration` letto da asset metadata. Out-of-scope M021 strict (asset pipeline = M022), ma il `Clock { Windowed }` deve esporre l'API per ricevere callback "cue completed". **Proposta**: ship M021 con stub fisso (es. 200ms/cue) + TODO marker per M022.

## 8. Out-of-scope esplicito

- Scripting embedded / hot-reload skill logic.
- Stack-aware status numerici (D009 storico).
- `Intent::SetSkillTree` (skilltree mutabile in-cast) — vietato da D033.
- Save mid-combat con `Box<dyn Trait>` serializzabile — `typetag` ha limitazione documentata su `Send + Sync` ([serde#384](https://github.com/serde-rs/serde/issues/384)); D008 JSONL output-only è safe.
- Multi-level Loop nesting.
- Talenti che riscrivono topologia della timeline (`TimelinePatchOp`, D027 demoted) — disponibile ma fuori critical path v0.

## 9. Fonti

- Spike: `.gsd/workflows/spikes/M021-timeline-fsm/FINDINGS.md` (33/33 verde, 4 pattern fixture — Loop+skilltree-gate, blueprint-state mutabile, cross-blueprint identity filter, RNG-gated edge)
- Roster v0 reference: `docs/future_design_draft/digimon/<x>/04_passive_*.md` (5 passive — Agumon, Dorumon, Gabumon, Patamon, Renamon, Tentomon) — base empirica del pattern survey nello spike
- [Bevy 0.17→0.18 migration](https://bevy.org/learn/migration-guides/0-17-to-0-18/)
- [Bevy required components issues #16406](https://github.com/bevyengine/bevy/issues/16406), [#16645](https://github.com/bevyengine/bevy/issues/16645) — preferire `#[require]` static su `BlueprintMarker` vs runtime registration
- [bevy-trait-query benchmarks](https://docs.rs/bevy-trait-query/latest/bevy_trait_query/) — 10× più lento delle query concrete; NON usare nell'hot path (lookup blueprint resta `HashMap` su `Resource` frozen-after-startup)
- [Veloren combat module](https://docs.veloren.net/veloren_common/combat/index.html) — pattern Attack-as-struct + ECS system (alternativa data-only valida; il nostro framework prende la stessa filosofia "data + fn-by-id" un livello sopra)
- [typetag](https://github.com/dtolnay/typetag) + [serde#384](https://github.com/serde-rs/serde/issues/384) — limite `Send + Sync` su `Box<dyn Trait>` serializzabile
