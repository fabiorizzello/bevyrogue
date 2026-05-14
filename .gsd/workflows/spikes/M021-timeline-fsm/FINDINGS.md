# M021 Timeline-FSM Spike — Findings

**Date:** 2026-05-14
**Status:** validated (33/33 tests passing on pass 3 — extends pass 2 with FSM-gate runtime + roster survey)
**Cmd:** `cargo test --manifest-path .gsd/workflows/spikes/M021-timeline-fsm/Cargo.toml`

## Scope

PoC headless del modello **Timeline-FSM** per M021, in un crate standalone
(zero deps, std-only, fuori dal workspace bevyrogue).

Due passi di validazione:

1. **Pass 1 (5 test):** invarianti strutturali del modello timeline +
   skill-tree patch. Validato il 2026-05-14 (vedi sezione "Pass 1" sotto).
2. **Pass 2 (19 test, questa revisione):** generalizzazione del pattern
   `id → fn` a **tutti gli assi** dove una skill plug-a logica custom —
   hook, selector, predicate, formula, tick, ai utility, cue resolver — sotto
   un unico `Registry<E: ExtPoint>`. Verifica end-to-end che **un singolo**
   blueprint (`agumon::register`) installa l'intera kit, e che la validazione
   prende dangling references su ogni asse con lo stesso code path.

| Invariante | Test | Esito |
|---|---|---|
| **I1.** Determinismo (stesso input ⇒ stesso `Intent`/`CombatEvent` stream) | `determinism_headless_two_runs_identical` | ✅ |
| **I2.** Dry-run ≡ Live (D024) | `dry_run_intent_stream_matches_live` + `bouncing_fire_dry_run_matches_live` | ✅ |
| **I3.** Signal-gating Windowed | `windowed_runner_stalls_until_signal` | ✅ |
| **I4.** Skill-tree patch è graph rewrite a compile-time | `pyromaniac_patch_injects_lingering_burn` | ✅ |
| **I4b.** Gate edge fallisce ⇒ comportamento attuale ben definito | `pyromaniac_gate_halts_when_predicate_fails` | ✅ (rivela F1) |
| **I5.** Validazione strict per ogni asse referenziato dal timeline | `validation_catches_missing_{hook,selector,predicate,cue_resolver}` + `_aggregates_multiple_errors` + `validation_catches_loop_exit_when_unregistered` | ✅ |
| **I5b.** Lookup-time validation per assi off-graph (formula, tick, ai) | `unregistered_{formula,tick,ai_utility}_returns_none_at_lookup` | ✅ |
| **I5c.** Hook/tick/ai/cue agumon-spec invocabili **solo** via registry | `agumon_{formula_used_by_status_tick,ai_utility,cue_resolver_varies_with_state}_via_registry` | ✅ |
| **I6.** Un solo entry point installa tutta la kit Agumon | `single_register_call_installs_every_axis_for_agumon` + `passive_hook_fires_via_registry_without_kernel_changes` | ✅ |
| **I7a.** Bouncing Fire OFF ≡ baseline (skilltree-gate runtime, D033) | `bouncing_fire_off_baseline_identical_to_no_loop` | ✅ |
| **I7b.** Loop tier-N esegue esattamente N hops oppure si ferma per pool exhaustion | `bouncing_fire_tier{1,2,3}_*` (3 test) | ✅ |
| **I7c.** Blueprint state read via predicate + write via `Intent::SetBlueprintState` | `predator_active_predicate_reads_blueprint_state`, `metal_cannon_force_predator_writes_blueprint_state_intent`, `chain_consume_reads_tracked_target_and_resets_state` | ✅ |
| **I7d.** Cross-blueprint identity filter (Twin Core ice) | `twin_core_predicate_{passes,rejects}_*_agumon_caster` | ✅ |
| **I7e.** RNG-gated edge preserva I1 (seeded determinismo) | `rng_predicate_is_deterministic_per_seed` + `rng_predicate_differs_with_different_seed` + `rng_70pct_threshold_skews_higher_than_30pct` | ✅ |
| **I8.** Validator copre `BeatKind::Loop.exit_when` ricorsivamente | `validation_catches_loop_exit_when_unregistered` | ✅ |

## Pattern unificato: `Registry<E: ExtPoint>`

Il pass 2 riscrive le tre registry separate del pass 1 (hooks/predicates +
implicit-selector-as-enum) sotto un singolo concept:

```rust
pub trait ExtPoint: 'static {
    type Fn: Copy;
    const KIND: &'static str;
}

pub struct Registry<E: ExtPoint> { fns: HashMap<&'static str, E::Fn>, .. }
```

Sette istanze, una per asse, ~3 righe di dichiarazione ciascuna:

```rust
pub struct HookExt;       impl ExtPoint for HookExt      { type Fn = fn(&BeatEvent, &mut SkillCtx);          const KIND: &'static str = "hook"; }
pub struct SelectorExt;   impl ExtPoint for SelectorExt  { type Fn = for<'a> fn(&SelectorCtx<'a>) -> Vec<UnitId>; const KIND: &'static str = "selector"; }
pub struct PredicateExt;  impl ExtPoint for PredicateExt { type Fn = fn(&BeatEvent, &SkillCtx) -> bool;       const KIND: &'static str = "predicate"; }
pub struct FormulaExt;    impl ExtPoint for FormulaExt   { type Fn = for<'a> fn(&FormulaCtx<'a>) -> i32;       const KIND: &'static str = "formula"; }
pub struct TickExt;       impl ExtPoint for TickExt      { type Fn = fn(&StatusInstance, &mut SkillCtx);      const KIND: &'static str = "tick"; }
pub struct AiUtilityExt;  impl ExtPoint for AiUtilityExt { type Fn = for<'a> fn(&AiCtx<'a>) -> f32;           const KIND: &'static str = "ai_utility"; }
pub struct CueExt;        impl ExtPoint for CueExt       { type Fn = for<'a> fn(&CueCtx<'a>) -> CueId;        const KIND: &'static str = "cue"; }
```

`ExtRegistries` aggrega le sette in un unico struct, registrato come
`Resource` Bevy in produzione. **Conseguenza pratica**: aggiungere un nuovo
asse è una dichiarazione di ~3 righe + un campo nel struct aggregato; tutto
il resto (validation, lookup, lifecycle) è già fatto dal generic.

## Single-entry-point per Digimon

`src/agumon.rs` è la prova di concetto: **un solo file**, **una sola fn
`register(reg: &mut ExtRegistries)`**, esercita tutti i 7 assi:

- 5 hooks (impact, splash, aftermath, lingering_burn, twin_core passive)
- 1 custom selector (`baby_burner_splash` — adjacenti ordinati per HP, ≤2)
- 2 predicates (`has_adjacent_targets`, `has_two_alive_allies`)
- 2 formulas (`fire_atk_scaling`, `heated_dot`)
- 1 status tick (`heated_tick`)
- 1 AI utility (`burner_utility`)
- 1 cue resolver (`charge_by_hp` — anim varia con HP%)

Il **kernel** (`lib.rs`) non menziona mai "agumon" o "fire". Il timeline
RON-equivalent (in Rust per il spike) referenzia solo string ID. Aggiungere
Gabumon = clonare il modulo + una nuova riga nel builder.

## Findings nuovi (pass 2)

### F6 — Predicate gate ha bisogno del `beat_targets` corrente

Il primo run di `pyromaniac_patch_injects_lingering_burn` falliva: il gate
`has_adjacent_targets` su `splash_adj → lingering_burn` leggeva un
`beat_targets` vuoto perché `next_from` ricostruiva il `BeatEvent` da
`base_event` (che è "stato iniziale del cast", non "stato dell'ultimo beat").

Fix nel runner (`BeatRunner::last_beat_targets: Vec<UnitId>`): ogni beat
Impact salva i targets risolti dal selector, e `next_from` carries-through
quel set per la valutazione del gate. ~6 righe in totale.

**Promozione**: in produzione il `BeatEvent` corrente è il context naturale
per i gate, non un clone di base_event. Va modellato esplicitamente come
"running beat event" che il runner aggiorna ad ogni step.

### F7 — `SkillCtx` flat non basta per "ctx.formula(id)" naturale

I blueprint vogliono scrivere:

```rust
fn on_impact_main(evt: &BeatEvent, ctx: &mut SkillCtx) {
    let dmg = ctx.formula("agumon::fire_atk_scaling")(&fctx);
    ctx.enqueue(Intent::DealDamage { ... });
}
```

Per farlo, `SkillCtx` deve borrow-are `&ExtRegistries` e `&CombatStateMock`.
Lo spike ha aggirato il problema con un `FormulaContextGuard` thread-local
(brutto ma esplicito) per non gonfiare le firme. **In produzione**:
`SkillCtx<'a>` con campi `pub registries: &'a ExtRegistries, pub state: &'a
CombatStateMock` — niente RAII, niente raw pointer. Il pattern Bevy 0.18
naturale è `SystemParam` che combina i due come SkillCtx tipo via
`Res<ExtRegistries>` + `Query<...>`.

### F8 — Validation è uniforme **per gli assi referenziati dal grafo**

`validate_timeline_refs` itera beat ed edges e controlla 4 assi (hook,
selector, predicate, cue) con lo stesso pattern `errs.push(...)`. Per gli
altri 3 (formula, tick, ai utility), la validation è "lookup at invocation
site returns Some" — esposto dal test `unregistered_*_returns_none_at_lookup`.

In produzione conviene aggiungere validation strict anche per:
- **Tick**: ogni `StatusEffect` RON referenzia un `tick: TickId` ⇒ scan
  della `StatusLibrary` per riferimenti non registrati.
- **Formula**: harder — i formula reference vivono inside hook bodies, non
  in RON. Soluzione realista: **convention** di registrazione (ogni blueprint
  registra le sue formula via `register()`), e *unit test del blueprint*
  che esercita ciascun hook per smoke-test della lookup.
- **AI utility**: il `SkillTimeline` RON espone `ai: Some(AiSpec(utility:
  AiUtilityId))` ⇒ scan analogo a quello del timeline.

### F9 — Cue resolver dinamico è il punto più potente per polish

Sembra zucchero ma non lo è: il `Presentation::Dynamic(CueId)` permette al
blueprint di emettere cue diversi senza moltiplicare beat. Esempio testato:
`charge_by_hp` torna anim diversa sotto/sopra 30% HP. Niente if/else nei
beat statici, niente patch del timeline, niente "split cast into two
branches". È **estensibilità senza enum bloat** dello stesso tipo che ti
preoccupava lato selector.

## Pass 1 findings (still valid, included for completeness)

### F1 — Edge gating "first-passing" richiede fallback edge

`InsertBeatAfter` deve produrre due edge da anchor (gated + unconditional
fallback) per evitare halt accidentale quando il gate fallisce. Lo spike
documenta il comportamento attuale in `pyromaniac_gate_halts_when_predicate_fails`
ma il fix è ~10 righe in `compile_timeline`.

### F2 — `BeatKind::Loop` non implementato (deferred)

Bounce/loop richiede `BeatKind::Loop { body: SubTimeline, exit_when:
Predicate }`. Spike successivo o slice dedicata.

### F3 — Signal taxonomy non chiusa

`SignalName = &'static str` libero. In produzione enum chiuso registrato in
`App::finish()`. **D028 load-bearing.**

### F4 — `BeatEvent.beat` viene assegnato dal runner

Pattern già pulito: il blueprint scrive solo `BeatEvent { caster,
primary_target, beat_targets: [], cast_id, beat: <ignored> }` come "base
event" e il runner lo riscrive per ogni beat.

### F5 — Multi-hook per beat: non necessario

`Option<HookId>` basta — composizione di effetti avviene *all'interno della
singola fn* via N `ctx.enqueue`.

## Pass 3 findings (FSM-gate runtime + roster survey)

Survey dei 5 Digimon doc (`docs/future_design_draft/digimon/<x>/04_passive_*.md`)
ha isolato **4 pattern architettonicamente distinti** che lo spike doveva
stress-testare prima di promuovere D033. Ognuno ha le sue fixture e i suoi
test verdi:

| Pattern | Origine doc | Test |
|---|---|---|
| **Bouncing Fire** (Loop + skilltree-gate) | Agumon talents | 5 (off/tier1/tier2/tier3-exhaust/dry-run) |
| **Predator Loop** (mutable blueprint state) | Dorumon 04 | 3 (predicate read / hook write / chain consume) |
| **Twin Core ice** (cross-blueprint identity filter) | Gabumon 04 / Agumon 04 | 2 (pass / reject) |
| **Block Reaction** (RNG-gated edge) | Tentomon 04 | 3 (determinism / seed diff / threshold ordering) |
| **Loop validator** | structural | 1 (dangling exit_when) |

Totale: **14 nuovi test** sopra ai 19 pass-2 → **33 verdi**.

### F10 — `BeatKind::Loop` + `hop_index` su `BeatEvent` (option A) regge

Loop body è un `Vec<Beat>`; `LoopFrame` nel runner mantiene `body_cursor`
+ `hop_index`. `BeatEvent` esporta `hop_index: u32` (sempre 0 fuori Loop,
0..N dentro). Una sola fn signature predicate `(&BeatEvent, &SkillCtx) ->
bool` regge sia per gate non-loop sia per `exit_when` di Loop. `Registry<E>`
non si frammenta: stesso axis per ogni predicate.

**Trade-off documentato** (rispetto a opzione B `LoopCtx` separato): `hop_index`
=0 fuori Loop è convenzione semantica, non type-safe. Predicate non-Loop
semplicemente non lo leggono. Costo zero, beneficio: un solo axis nel
Registry.

### F11 — Skilltree-gate runtime (D033) sostituisce compile-time patch per la maggior parte dei casi

`agumon::base_timeline_with_bouncing_fire()` ha il branch Loop **sempre
presente** nel grafo. Skilltree `rank("agumon::bouncing_fire")` decide al
runtime se l'edge `aftermath → bounce_loop` passa. Fallback edge `aftermath
→ cast_end` (no gate) evita F1.

Test `bouncing_fire_off_baseline_identical_to_no_loop` prova che, con
talento OFF, lo stream Intent è **letteralmente identico** al timeline base
senza branch. Quindi:

- Talenti che **abilitano/disabilitano** branch ⇒ runtime gate (D033 primary).
- Talenti che **inseriscono beat nuovi non rappresentabili come edge** (es.
  Pyromaniac add-on lingering burn) ⇒ compile-time patch (D027 secondary).

In pratica: **D027 è demoted** dal critical path v0. Pyromaniac stesso si
riesprime come beat opzionale gated da predicate; D027 resta disponibile
per topology rewrite genuini (rari).

### F12 — Blueprint state pattern richiede `Intent::SetBlueprintState`

`predator_loop` valida due movements:
1. **Read**: predicate `dorumon::predator_active` legge
   `state.blueprint_state[(unit, "dorumon.predator_active")]` via
   `ctx.blueprint_state(...)`.
2. **Write**: hook `on_metal_cannon_force_predator` emette `Intent::
   SetBlueprintState { actor, key, value, cast_id }` — il kernel applica
   nello stream (D008 transition stream coerente con le altre mutazioni).

L'invariante "**niente write diretto da hook → mock state**" regge: tutte
le mutazioni passano dall'Intent stream. Replay reconstructible.

### F13 — Cross-blueprint identity filter: zero costo

Twin Core ice (Gabumon arma su Heated by Agumon) si esprime con un singolo
predicate fn-by-id che legge `ctx.identity_of(evt.caster) == "agumon"`.
Nessun nuovo asse del Registry, nessun coupling tra blueprint. Test
`twin_core_predicate_passes_for_agumon_caster` /
`twin_core_predicate_rejects_non_agumon_caster` confermano lo split.

Rilettura post-survey: D005 (shared-mechanic mini-plugin) resta valida, ma
in molti casi una predicate per identità + un hook arming sono sufficienti.
Lo shared-mechanic plugin va dove c'è **stato condiviso** (Twin Core con
contatori), non dove c'è solo signal-listening (filter + arming).

### F14 — RNG-gated edge preserva determinismo (I1)

Block Reaction (Tentomon battery_loop) usa `RngRollBelow` su edge `BlockReady
→ BlockProc`. Implementato nel spike come predicate `tentomon::rng_below_*`
che chiama `ctx.rng_u32(cast_id, beat, hop_index, salt)` — SplitMix64 + FNV
deterministico.

- `rng_predicate_is_deterministic_per_seed`: stesso (seed,cast,beat,hop) ⇒
  stesso draw. **I1 preservato**.
- `rng_predicate_differs_with_different_seed`: variando il seed, almeno un
  draw cambia (sanity check non-stuck).
- `rng_70pct_threshold_skews_higher_than_30pct`: la threshold parametrica
  funziona — 400 sample, 70% fires almeno +100 volte di 30%.

**Conseguenza**: l'invariante I1 vale anche con predicate stocastiche, *a
patto che* il seed venga incluso nello stato deterministico del run
(replay seed-carrying). Production deve esporre `state.rng_seed` come
parte del replay payload.

### F15 — Pool exhaustion come exit condition esplicita

Memory note pre-esistente: "Bounce pool exhaustion breaks the hop loop
silently (no OnActionFailed event emitted)". Lo spike formalizza:
`bounce_should_stop` ha **tre clausole OR** — tier cap, pool exhaustion,
talent missing. Test `bouncing_fire_tier1_runs_exactly_one_hop` esercita
proprio pool exhaustion: tier-1 ⇒ vorrebbe 1 hop ma gli avversari del
fixture sono tutti già coperti da primary+splash ⇒ selector ritorna empty
⇒ nessun DealDamage ⇒ exit_when prossima iterazione vede `pool ⊆ hit_set`
⇒ exit.

Pattern **uniforme** per ogni Loop: l'autore della skill scrive un solo
predicate `*_should_stop` che combina condizioni in OR. Niente
hardcoding kernel-side.

### F16 — Cross-blueprint synergy via listener FSM è fuori scope spike

Pattern visto in Renamon `kitsune_grace` (passive triggera su
`UltimateUsed{actor:ally,!self}`): è una FSM **passive**, non legata al
cast del proprio Digimon. Vive su un signal-bus globale che listena
`CombatEvent` e pusha signal nella propria FSM. Lo spike non implementa
questo path (è una runner variant — il `BeatRunner` modella un cast, non
una listener-FSM).

**Action item M021 roadmap**: aggiungere `PassiveRunner` analogo nello
slice "timeline runner + signal bus". L'API extension è identica
(`Registry<E>`, hook fn-by-id), cambia solo il driving event.

### F17 — Multi-FSM blueprint (Gabumon dual-path) confermato come array di timeline

Gabumon ospita 2 FSM paralleli (`fur_cloak_fsm` + `twin_core_ice_fsm`).
Implementazione naturale: il blueprint registra **N timeline** distinte e
N entry-point listener nel signal bus. `register()` torna `()` e installa
ogni timeline come una unità separata. Coerente con D032 (un solo modulo
+ un solo register).

## Decisioni da scrivere/aggiornare

Lo spike ora valida i seguenti nodi decisionali. Pronti per
`.gsd/DECISIONS.md`:

### Dal pass 1

- **D025** — Timeline FSM come strato esplicito (`CompiledTimeline` =
  graph di `Beat` + `BeatEdge` + registry).
- **D026** — Two-clock model (`HeadlessAuto` / `Windowed`), invariante
  Intent stream uguale tra i due.
- **D027** — Skill-tree come `TimelinePatchOp` applicato a compile-time —
  **DEMOTED post pass 3**: idioma secondario, disponibile solo per
  topology rewrite non-edge-gate. Critical path v0 usa D033 (runtime gate).
- **D028** — Signal taxonomy enum-chiuso, registrato in `App::finish()`
  (promossa load-bearing da F3).
- **D029** — `next_from` ha semantica *first-passing edge*; `InsertBeatAfter`
  produce gated + fallback edge per evitare halt accidentale (F1).

### Nuove dal pass 2

- **D030** — Selector come fn-by-id registrato (mirror di D023). Sostituisce
  `enum ImpactSelector` con `Registry<SelectorExt>`. Built-in selectors
  (`primary`, `all_enemies`, `all_allies`, `adjacent_to_primary`,
  `self_only`) registrati dal kernel; selettori signature-skill registrati
  dal blueprint.
- **D031** — Pattern unificato "extension point": un singolo `Registry<E:
  ExtPoint>` per ogni asse fn-by-id. Si applica a hook, predicate, selector,
  formula, tick, ai_utility, cue. Validation centralizzata in
  `validate_timeline_refs` per gli assi grafo-referenziati; per gli altri,
  "lookup-time-Some" + smoke test del blueprint.
- **D032** — Blueprint per Digimon = **un solo modulo** con **un solo
  `register(reg: &mut ExtRegistries)`** entry point. Nessun codice
  Digimon-specifico altrove. Kernel non menziona mai nomi di Digimon.

### Nuove dal pass 3

- **D033** — **Skilltree come context input immutabile per gate runtime
  della FSM**, `BeatKind::Loop { body, exit_when }` come canonica iterazione
  bounded, `BeatEvent.hop_index` come counter uniforme. Pattern primary per
  talenti che abilitano/disabilitano branch (Bouncing Fire, Pyromaniac
  re-espresso). Skilltree load-time-immutable per il run, mutato solo tra
  encounter.
- **D034 (proposta, lasciata aperta)** — `Intent::SetBlueprintState` come
  canonical write-path per state mutabile per-unit-per-key (Predator Loop,
  Twin Core counter, Battery armed). Mutuazione passa dall'Intent stream,
  mai diretta da hook → state. Garantisce replay reconstructibility.

### Aggiornamenti

- **D020** — `on_final_hop` si dissolve in `BeatKind::Loop` body.
- **D023** — confermata, slot è `Beat.hook` (non `Ability.on_hit`).
- **D024** — confermata, `SkillCtxMode` regge sia per hook che per tick e
  AI utility (tutti gli assi che producono `Intent` rispettano l'invariante
  Dry-run ≡ Live). Estende a Loop body (test
  `bouncing_fire_dry_run_matches_live`).
- **D027** — **demoted**: compile-time patch resta available ma non è più
  critical path. Pyromaniac re-espresso come beat opzionale gated da
  predicate (runtime gate D033). Costo: meno superficie da spiegare al
  prossimo agente; un solo idioma per talenti edge-gate.

## Costi confermati

- **`Registry<E>` generic + ExtPoint trait**: ~40 righe in kernel, una volta.
- **Per nuovo asse**: ~3 righe (struct marker + impl ExtPoint + campo in
  ExtRegistries). Stessa API per ogni axis.
- **Per nuovo Digimon**: ~1 modulo + ~1 riga nel builder. Zero kernel changes.
- **Validation centralizzata** per assi referenziati dal timeline: ~30 righe
  totali (`validate_timeline_refs`).
- **+1 slice** in M021 roadmap: timeline runner + signal bus + presentation
  cue events + registry framework.
- **Refactor Bounce** in `BeatKind::Loop`: deferred, da fare in slice
  dedicata (F2).

## Promozione

Confermata: **opzione 1 della conversation** (adopt pieno timeline-FSM +
Registry<E> pattern) regge end-to-end.

- I 6 invarianti chiave (I1-I6) reggono in headless puro.
- Il modello hook-fn flat che avremmo dovuto scrivere comunque (D023+D024)
  è esattamente lo stesso, generalizzato a 7 assi sotto un unico pattern.
- Lo skill-tree estendibile costa una sola volta come `TimelinePatchOp`.
- L'enum bloat che preoccupava lato selector (e che sarebbe arrivato anche
  lato formula, tick, ai, cue) viene **eliminato strutturalmente** dal
  registry pattern.

## Cleanup

Spike rimovibile in un comando: `rm -rf .gsd/workflows/spikes/M021-timeline-fsm/`.
Nessun impatto sul workspace o sul build principale.

## File del spike

```
.gsd/workflows/spikes/M021-timeline-fsm/
  Cargo.toml          — standalone, zero deps
  src/
    lib.rs            — Intent (+SetBlueprintState), CombatEvent, ExtPoint,
                        Registry<E>, ExtRegistries, BeatKind (+Loop),
                        Beat/BeatEdge/CompiledTimeline, SignalBus, Clock,
                        BeatRunner (+LoopFrame +cast_hit_set), runtime::
                        RuntimeGuard, validate_timeline_refs (recursive)
    agumon.rs         — Agumon blueprint + bouncing_fire talent (Loop,
                        skilltree-gate, bounce selector, exit_when)
    dorumon.rs        — predator_loop pattern (blueprint state read+write)
    gabumon.rs        — twin_core_ice (cross-blueprint identity predicate)
    tentomon.rs       — block_reaction (RNG-gated edge, deterministic)
  tests/
    validation.rs     — 33 tests: I1-I8 + 4 pattern fixtures
  FINDINGS.md         — this file
```
