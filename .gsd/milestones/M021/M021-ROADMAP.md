# M021: Kernel framework + Timeline FSM + Registry<E>

**Vision:** Un kernel combat che espone solo primitive generiche (`Intent` come unica mutazione, `CompiledTimeline` come unica forma di "skill", `Registry<E: ExtPoint>` come unico asse di estensione, `SkillCtx` come unico contesto, `SignalBus` come bus reattivo, `Clock` two-mode). Niente trait per skill, niente enum effect, niente trait per blueprint. Ogni Digimon = un solo modulo + un solo `register(reg: &mut ExtRegistries)`. Lo skilltree è context immutabile per il run, letto via predicate fn-by-id; abilita/disabilita branch nella FSM senza patch compile-time. Validato da spike standalone (33/33 verde) su 4 pattern architettonicamente distinti.

## Success Criteria

- `rg -E "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/ --glob '!blueprints/**'` → 0 righe
- `rg "enum Effect" src/data/skills_ron.rs` → 0 righe
- `rg "use bevy" src/combat/blueprints/` → 0 righe (i blueprint sono Bevy-agnostic)
- Tutte le 18 active skill canon eseguono via `CompiledTimeline + ExtRegistries`; tutte le 6 passive canon eseguono via `PassiveRunner` listener-driven
- Aggiungere un Digimon scriptato (test fixture) tocca solo `src/combat/blueprints/<new>/` + `assets/data/digimon/<new>/`
- `cargo test` verde end-to-end (~74 test integration esistenti + nuovi test framework + pattern fixture)
- `cargo check` headless e `--features windowed` senza warning nuovi
- JSONL contiene `CombatKernelTransition::Blueprint { owner, payload }` round-trip per ogni `BlueprintSignal`
- `DryRun ≡ Execute` invariante (I2) verde su almeno una skill chain ramificata (Bounce con hook on-hit)
- HeadlessAuto Intent stream end-of-cast ≡ Windowed Intent stream end-of-cast (I3 / D026)
- Determinismo (I1): due run con stesso seed producono stessi `Intent` e `CombatEvent` stream (incluso path RNG-gated)

## Key Risks / Unknowns

- **RON → CompiledTimeline compiler ergonomia non testata su 24 skill**. Spike usa builder Rust diretto. Rischio di scoprire pattern che il compiler RON non esprime cleanly. **Retire in S05** scrivendo Tohakken (Bounce + on_final_hop) e Petit Thunder (AoE + chain) in RON puro; rivedere shape RON se serve.
- **PassiveRunner non spike-validated**. F16 nello spike segnala esplicitamente "fuori scope". Rischio di scoperta tardiva su shape signal→timeline driving. **Retire in S04** con fixture Renamon `kitsune_grace` (passive listener su `UltimateUsed{actor: ally,!self}`) prima di migrare le altre 5 passive.
- **`Intent::BlueprintSignal` payload `Box<dyn Any>` owner↔type invariant** non enforceable dal compiler. **Retire in S04** con `debug_assert` nel dispatcher + test che provoca mismatch e verifica panic in debug.
- **Hook cascade `preview ≡ execute` divergence** (D024 promise). **Retire in S03** con test parallelo che cast-a una skill in DryRun e Execute contro lo stesso `FrozenSnapshot` e asserisce identità degli `Intent` stream.
- **Turn-phase order regression** (D011 5-step). **Retire in S02** con test fixture "Agumon OnTurnStart kills Tentomon" che verifica ordine `tick → KO → passive → KO → snapshot → select → cast`.
- **Big-bang vs incremental**: 12 slice interleaved. Mitigato dal registry doppio temporaneo (Effect-based + Timeline-based convivono fino a fine S06) e dal commit per-slice con suite verde.
- **Ext registry validation per assi off-graph** (formula, tick, ai_utility): nessun grafo li referenzia, validation è "lookup-time-Some". Rischio di blueprint che registra una formula non riferita da nessun hook (dead fn). **Mitigato**: smoke-test per blueprint che esercita ogni hook in S07–S10.

## Proof Strategy

- Pattern Loop+skilltree-gate (Bouncing Fire) → retire in **S02+S04** provando che con `rank("bouncing_fire") = 0` lo stream Intent è letteralmente identico al timeline base (fixture `bouncing_fire_off_baseline_identical_to_no_loop` portata in-tree)
- Blueprint state mutabile (Predator Loop) → retire in **S08** provando read via predicate + write via `Intent::SetBlueprintState` end-to-end, JSONL contiene la mutation
- Cross-blueprint identity filter (Twin Core ice) → retire in **S08** provando che lo stesso predicate `is_caster_agumon` passa su cast Agumon e fallisce su cast Gabumon, senza coupling fra i due moduli
- RNG-gated edge (Block Reaction) → retire in **S08** provando determinismo per seed (stesso seed ⇒ stesso draw) + threshold ordering empirico (400 sample, 70% > 30% by ≥100 occorrenze)
- AbilityBuilder→RON convergence → retire in **S05** costruendo Tohakken + Petit Thunder direttamente in RON puro e validando con suite esistente
- Kernel-generic invariant → retire in **S11** con il grep `rg "TwinCore|..." src/combat/ --glob '!blueprints/**'` → 0 righe
- HeadlessAuto≡Windowed Intent stream → retire in **S03** con fixture two-clock (test esegue cast in HeadlessAuto, poi stesso cast in Windowed con clock stub che auto-resolve cue, asserisce Intent stream identico)

## Verification Classes

- **Contract verification**: unit/integration test per ogni asse `Registry<E>` (lookup + validation), `Intent` applier per ogni variant (DealDamage / Heal / ApplyStatus / ApplyBuff / Cleanse / AdjustToughness / Stun / AdvanceTurn / DelayTurn / EnqueueFollowUp / BlueprintSignal / SetBlueprintState / Reject), `BeatRunner` mode parity (DryRun/Execute/Preview), `LoopFrame` (single-level v0)
- **Integration verification**: suite `tests/` esistente (~74 test funzionali) **verde a ogni slice merge**; nuovi test fixture per i 4 pattern del spike (Loop+skilltree-gate / blueprint-state / identity filter / RNG-gated); test cross-blueprint Twin Core paired
- **Operational verification**: `cargo check` (headless) + `cargo check --features windowed` senza warning nuovi a ogni slice; grep verifiers come gate hard a slice di chiusura (`rg "use bevy" src/combat/blueprints/`, `rg "TwinCore|..." src/combat/ --glob '!blueprints/**'`, `rg "enum Effect" src/`)
- **Determinism verification**: replay test con seed fisso, due run, `assert_eq!(stream_a, stream_b)` su Intent + CombatEvent
- **UAT / human verification**: gameplay smoke `cargo run --features windowed` a fine S07 (skill migrate) e fine S11 (kernel digimon-free) — almeno 2 encounter scriptati, comportamento indistinguibile dal pre-refactor

## Milestone Definition of Done

- Tutte le 12 slice in stato `[x]` con SUMMARY.md
- `cargo test` headless verde su 74+ test integration (suite non ridotta)
- `cargo run --features windowed` smoke run su un encounter completo si comporta indistinguibile dal pre-refactor
- I quattro grep invarianti (`rg "use bevy" src/combat/blueprints/`, kernel digimon-free, `enum Effect` assente, `apply_effects` rimossa) restituiscono 0 righe
- JSONL produce `CombatKernelTransition::Blueprint { owner, payload }` su almeno un signal Twin Core round-trip
- `CODEBASE.md` rigenerato e mostra la nuova struttura `src/combat/api/` + `src/combat/blueprints/<x>/` uniformemente directory-shaped
- `KNOWLEDGE.md` aggiornato (P001 rule "skill = `CompiledTimeline` + fn-by-id, niente trait skill, niente enum Effect, mutazione solo via Intent")
- Pre-condizione M024: la `M024-CONTEXT` può citare "files toccati ⊂ `src/combat/blueprints/<new>/` + `assets/data/digimon/<new>/`" come acquisizione M021

## Requirement Coverage

- Covers: N/A — il progetto non ha ancora un `REQUIREMENTS.md` formale
- Leaves for later: M022 (asset pipeline), M023 (visual stack), M024+ (nuovi roster)
- Orphan risks: stack-aware status numerici (D009 storico, indipendente); hot-reload skill logic (out of scope)

## Slices

- [ ] **S01: Kernel framework primitives + CombatPlugin extract** `risk:medium` `depends:[]`
  > After this: `CombatPlugin` non importa `bevy::winit`/`bevy::render`/`bevy_egui`; modulo `src/combat/api/` contiene `Intent` enum + `intent_applier` (FIFO drain) + `SkillCtx<'a>` (read-only + enqueue) + `ExtPoint` trait + `Registry<E>` + `ExtRegistries` Resource + `SignalBus` Resource + `Clock { HeadlessAuto, Windowed }` + RNG seeded (SplitMix64); `rg "use bevy" src/combat/api/` → 0 righe (eccetto `Resource`/`Component` marker); `cast_id: CastId(NonZeroU32)` aggiunto a `CombatEvent` + propagato da `pipeline::step_app` a ~50 callsites
- [ ] **S02: Timeline FSM + validate_timeline_refs** `risk:high` `depends:[S01]`
  > After this: `Beat` + `BeatEdge` + `BeatKind::{Impact { hook, selector, presentation }, Loop { body, exit_when }, CastStart, CastEnd}` + `CompiledTimeline` + `BeatRunner` con `LoopFrame` single-level + `BeatEvent { cast_id, beat, hop_index, caster, primary_target, beat_targets }`; `validate_timeline_refs` ricorsiva (incl. `Loop.body` ed `exit_when`) a `App::finish()`; turn-phase 5-step (`PreTurnTick → PostTickKoResolve → TurnStartHooks → PostHooksKoResolve → BuildFreshSnapshot → SelectActionAndCast`) come `SystemSet` Bevy espliciti; test fixture "OnTurnStart kills target" verde
- [ ] **S03: Mode parity (DryRun ≡ Execute ≡ Preview) + Two-clock invariant** `risk:medium` `depends:[S02]`
  > After this: `SkillCtxMode { DryRun, Execute, Preview }` cabla il `BeatRunner`; `FrozenSnapshot: Arc<...>` backing immutabile per DryRun/Preview; fixture cast eseguita in DryRun, Execute, Preview produce Intent stream identici; clock stub per Windowed con auto-resolve cue, fixture two-clock verifica `HeadlessAuto.intents ≡ Windowed.intents`; circuit breaker debug-only @256 evt/cast (D014)
- [ ] **S04: SignalBus + PassiveRunner + Ult instant + Intent::BlueprintSignal dispatcher** `risk:high` `depends:[S03]`
  > After this: `SignalBus` con signal taxonomy enum-chiusa (D028) validata a `App::finish()`; `PassiveRunner` consuma `CombatEvent` + esegue `CompiledTimeline` listener-driven; `Intent::BlueprintSignal { owner, payload: Box<dyn Any+Send+Sync> }` con `debug_assert` su mismatch owner↔type; `CombatKernelTransition::Blueprint { owner, payload }` variant aggiunta; JSONL subscribe al transition stream; `TacticalCyclePhase::UltInstant` bypassa turn advance (D010); `EventFilter::{All, Any, Not, Custom}` combinatori (D021); fixture Renamon `kitsune_grace` passive (signal-driven) verde
- [ ] **S05: Built-in extension fns + RON → CompiledTimeline compiler** `risk:medium` `depends:[S04]`
  > After this: kernel registra built-in fn-by-id: selectors (`primary`, `all_enemies`, `all_allies`, `adjacent_to_primary`, `lowest_hp_pct_alive`, `self_only`, `random_alive_deterministic`), formulas (`atk_scaling`, `dot_tick`, `dr_aggregate`), predicates (`has_target_alive`, `caster_below_hp_pct`, `target_has_status`), ticks (`heated_tick`, `paralyzed_tick`); RON schema v2 `skills.ron` = numeri/tag + grafo di beat con string id; load-time compiler emette `CompiledTimeline`; typo nel RON = errore al boot; Tohakken (Bounce + on_final_hop) e Petit Thunder (AoE) scritte in RON puro come canary; suite verde
- [ ] **S06: Migrate 18 active skill canon + drop enum Effect** `risk:high` `depends:[S05]`
  > After this: tutte le 18 active skill canon migrate in `assets/data/skills.ron` v2 + 0–2 fn-by-id per blueprint (skill custom); Bounce → `BeatKind::Loop` con `exit_when` (sostituisce `MultiHitOnKO::*` policy); `enum Effect` rimosso da `src/data/skills_ron.rs`; `apply_effects()` rimossa da `resolution.rs`; suite integration ~74 test verde + nuovi test per Loop tier-N (exact-hop / pool-exhaust / talent-off-baseline)
- [ ] **S07: Modifier pipeline + Migrate 6 passive canon** `risk:high` `depends:[S04,S06]`
  > After this: `ModifierCondition` canon list (D017: `SourceUnit/SourceTag/TargetHasStatus/TargetIsAlly/Pipeline + All/Any/Not`); `StatusDef::intrinsic_modifiers: Vec<ModifierTemplate>` data-driven (D018 — Heated/Chilled/Blessed simmetria); `Intent::ApplyBuff { kind, stack_mode }` con `BuffStackMode { MaxReplace, ProcOnce, Additive }` (D019); modifier_aggregator raccoglie da Ability+Status+Buff; tutte le 6 passive canon migrate come `PassiveRunner` listener-driven; fixture Block Reaction (RNG-gated edge, Tentomon battery_loop) verde
- [ ] **S08: Agumon + Gabumon migrated (Twin Core paired)** `risk:medium` `depends:[S06,S07]`
  > After this: `src/combat/blueprints/agumon/` + `src/combat/blueprints/gabumon/` ognuno con un solo `register(reg: &mut ExtRegistries)`; mini-plugin `src/combat/blueprints/twin_core/` (D005 shared-mechanic) gestisce entrambi gli owner; cross-blueprint identity filter (predicate `is_caster_agumon`) verde su Twin Core ice (Gabumon arma su Heated by Agumon); test fixture Agumon Bouncing Fire (skilltree-gate, OFF≡baseline / tier1 / tier2 / tier3-exhaust) verde; nessun coupling fra Agumon e Gabumon code
- [ ] **S09: Dorumon + Tentomon migrated (Predator Loop + Battery Loop)** `risk:medium` `depends:[S08]`
  > After this: `src/combat/blueprints/dorumon/` + `src/combat/blueprints/tentomon/` directory-shaped; Predator Loop usa `Intent::SetBlueprintState` (D034) per write-deferred + predicate read via `ctx.blueprint_state(actor, key)`; Battery Loop RNG-gated edge usa `ctx.rng_u32(cast_id, beat, hop, salt)` SplitMix64; flat-file legacy `src/combat/blueprints/tentomon.rs` rimosso
- [ ] **S10: Patamon + Renamon migrated + kernel digimon-free** `risk:medium` `depends:[S09]`
  > After this: `src/combat/blueprints/patamon/` (Holy Aegis) + `src/combat/blueprints/renamon/` (Kitsune Grace + Precision Mind Game); le 5 variant Digimon-specific di `CombatKernelTransition` (`TwinCore`, `BatteryLoop`, `HolySupport`, `PredatorLoop`, `PrecisionMindGame`) rimosse da `kernel.rs`; sub-snapshot Digimon-specific in `observability.rs` rimossi (sostituiti da optional injection via hook registry); `rg "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/ --glob '!blueprints/**'` → 0 righe; smoke `cargo run --features windowed` su 2 encounter scriptati indistinguibile dal pre-refactor
- [ ] **S11: UI/AI consumers via SkillCtx::Mode::Preview** `risk:low` `depends:[S06,S07]`
  > After this: `query_skill_preview` thin wrapper che esegue il runner in `Preview` mode e ritorna `Vec<Intent>` astratti; `combat_panel.rs` consuma direttamente lo stream Intent (no più `predict_damage` UI-side); AI scoring usa helper esterno sullo stesso stream; `Mode::Preview` cabla anche hover-senza-target
- [ ] **S12: RosterEntry blueprint-keyed + ValidationSnapshot from registry** `risk:low` `depends:[S10]`
  > After this: `UnitDef` senza field hardcoded Digimon-specific (`twin_core`, `holy_support`, …); ogni blueprint dichiara validation rule via fn registrata in `Registry<ValidationExt>`; `ValidationSnapshot` popolata dall'iter del registry; criterio falsificabile M021 verificato (aggiungere un Digimon scriptato tocca solo `blueprints/<new>/` + `data/digimon/<new>/`); `KNOWLEDGE.md` aggiornato

## Horizontal Checklist

- [ ] D### M021 rilette a fine S07 e S10 — ancora valide a nuovo scope?
- [ ] D034 (`Intent::SetBlueprintState`) persistita in `.gsd/DECISIONS.md` prima di S09
- [ ] K-P001 aggiornato in `KNOWLEDGE.md` (rule "skill = CompiledTimeline + fn-by-id, niente trait skill, niente enum Effect, mutazione solo via Intent")
- [ ] `CODEBASE.md` rigenerato dopo S10 e S12 per riflettere la nuova struttura `src/combat/api/` + `src/combat/blueprints/<x>/` uniforme
- [ ] Suite integration deterministica preservata — nessun nuovo test introduce wall-clock o RNG senza seed
- [ ] `cargo check --features windowed` verificato a ogni slice (non solo headless)
- [ ] JSONL backward-compatibility valutata a S04 — `CombatKernelTransition::Blueprint` aggiunto, eventuali tool downstream parsano JSONL avvisati
- [ ] Spike `M021-timeline-fsm` portato in-tree come fixture deterministica (33/33 verde) o cleanup `rm -rf .gsd/workflows/spikes/M021-timeline-fsm/` dopo che i 4 pattern sono coperti dai test integration

## Boundary Map

### S01 → S02, S03, S04, S05

Produces:
- `CombatPlugin: Plugin` wrappa `register_combat_kernel_runtime`; nessun import winit/wgpu/egui (D008)
- `src/combat/api/` con `Intent` enum (~18 variant, including `BlueprintSignal` + `SetBlueprintState` + `Reject`), `intent_applier` (FIFO drain, route per variant), `SkillCtx<'a>` (read-only accessor + `enqueue(Intent)` + `cast_hit_set()` + `skilltree(actor)` + `blueprint_state(actor, key)` + `identity_of(unit)` + `rng_u32(...)`), `ExtPoint` trait + `Registry<E>` + `ExtRegistries` Resource (7 assi), `SignalBus` Resource, `Clock { HeadlessAuto, Windowed }`, RNG seeded
- `CastId(NonZeroU32)` aggiunto a `CombatEvent`; propagato in `pipeline::step_app` e tutti i call-site emit
- Re-export plugin pubblico da `src/lib.rs`

Consumes:
- nothing (first slice)

### S02 → S03, S05, S06, S07

Produces:
- `Beat { id, kind: BeatKind, presentation: Presentation, … }` + `BeatEdge { from, to, gate: Option<PredicateId> }` + `BeatKind::{Impact { hook, selector }, Loop { body: Vec<Beat>, exit_when: PredicateId }, CastStart, CastEnd}` + `CompiledTimeline` + `BeatRunner` con `LoopFrame` (single-level v0) + `BeatEvent { cast_id, beat, hop_index, caster, primary_target, beat_targets }`
- `validate_timeline_refs` ricorsiva (incl. `Loop.body` ed `exit_when`) chiamata a `App::finish()` (D031)
- `next_from` first-passing edge + fallback unconditional invariant (D029)
- Turn-phase 5-step come `SystemSet` Bevy espliciti (D011)
- Fixture "OnTurnStart kills target" verde

Consumes:
- `Intent`, `SkillCtx`, `Registry<E>` da S01

### S03 → S04, S06, S11

Produces:
- `SkillCtxMode { DryRun, Execute, Preview }` parametrico nel `BeatRunner`
- `FrozenSnapshot: Arc<...>` backing immutabile per DryRun/Preview
- Fixture parallela `DryRun ≡ Execute ≡ Preview` su skill chain ramificata verde (I2 / D024)
- Two-clock fixture (HeadlessAuto vs Windowed con clock stub) Intent stream identico verde (I3 / D026)
- Circuit breaker debug-only @256 evt/cast (D014)

Consumes:
- Timeline FSM da S02

### S04 → S05, S07, S08

Produces:
- `SignalBus` con signal taxonomy enum-chiusa registrata a `App::finish()` (D028)
- `PassiveRunner` driven da `CombatEvent` → `CompiledTimeline` (listener variant del runner; share Intent emission con `BeatRunner`)
- `Intent::BlueprintSignal { owner: &'static str, payload: Box<dyn Any+Send+Sync> }` + `debug_assert` su mismatch owner↔type
- `CombatKernelTransition::Blueprint { owner, payload }` variant + JSONL subscribe al transition stream
- `TacticalCyclePhase::UltInstant` bypassa turn advance (D010)
- `EventFilter::{All, Any, Not, Custom}` combinatori (D021)
- Fixture Renamon `kitsune_grace` passive (signal `UltimateUsed { actor: ally, !self }` → buff) verde

Consumes:
- Mode parity da S03

### S05 → S06, S07

Produces:
- Built-in fn-by-id registrate dal kernel: selectors (`primary`, `all_enemies`, `all_allies`, `adjacent_to_primary`, `lowest_hp_pct_alive`, `self_only`, `random_alive_deterministic`), formulas (`atk_scaling`, `dot_tick`, `dr_aggregate`), predicates (`has_target_alive`, `caster_below_hp_pct`, `target_has_status`, `target_has_status_from`), ticks (`heated_tick`, `paralyzed_tick`, `chilled_tick`)
- RON schema v2 `skills.ron`: numeri/tag + grafo di beat con string id verso le registry
- Load-time compiler `RonSkill → CompiledTimeline` (validation strict, typo = errore boot)
- Tohakken + Petit Thunder come canary in RON puro

Consumes:
- Timeline FSM da S02, Mode parity da S03, SignalBus/PassiveRunner da S04

### S06 → S08, S09, S10, S11

Produces:
- 18 active skill canon in `assets/data/skills.ron` v2 + 0–2 fn-by-id custom per blueprint
- Bounce come `BeatKind::Loop` con `exit_when` (sostituisce `MultiHitOnKO::*` policy)
- `enum Effect` rimosso da `src/data/skills_ron.rs`
- `apply_effects()` rimossa da `resolution.rs`
- Suite integration verde + nuovi test Loop tier-N

Consumes:
- Built-in fns + RON compiler da S05

### S07 → S08, S09, S10, S11

Produces:
- `ModifierCondition` canon list ricco (D017)
- `StatusDef::intrinsic_modifiers: Vec<ModifierTemplate>` data-driven (D018)
- `Intent::ApplyBuff { kind, stack_mode }` con `BuffStackMode { MaxReplace, ProcOnce, Additive }` (D019)
- `modifier_aggregator` system che raccoglie da Ability+Status+Buff
- 6 passive canon migrate come `PassiveRunner` listener-driven
- Fixture Block Reaction (Tentomon battery_loop, RNG-gated edge, threshold parametrica) verde

Consumes:
- SignalBus/PassiveRunner da S04, built-in fns da S05, skill migrate da S06

### S08 → S09, S10

Produces:
- `src/combat/blueprints/agumon/mod.rs` + `src/combat/blueprints/gabumon/mod.rs` con un solo `register(reg)`
- `src/combat/blueprints/twin_core/` mini-plugin (D005) gestisce paired owner senza modifiche per aggiungere Gabumon
- Fixture Bouncing Fire (skilltree-gate OFF≡baseline / tier1 / tier2 / tier3-exhaust / dry-run-parity) verde
- Cross-blueprint identity filter `is_caster_agumon` verde su Twin Core ice

Consumes:
- Pattern blueprint + Intent::BlueprintSignal da S04, modifier pipeline da S07, skill canon da S06

### S09 → S10

Produces:
- `src/combat/blueprints/dorumon/mod.rs` (Predator Loop con `Intent::SetBlueprintState` write-deferred + predicate read)
- `src/combat/blueprints/tentomon/mod.rs` (Battery Loop RNG-gated edge, threshold)
- Flat-file legacy `src/combat/blueprints/tentomon.rs` rimosso (struttura directory uniforme)

Consumes:
- Pattern Agumon/Gabumon da S08

### S10 → S11, S12

Produces:
- `src/combat/blueprints/patamon/mod.rs` (Holy Aegis) + `src/combat/blueprints/renamon/mod.rs` (Kitsune Grace + Precision Mind Game)
- 5 variant `CombatKernelTransition` Digimon-specific rimosse da `kernel.rs`
- Sub-snapshot Digimon-specific in `observability.rs` rimossi (optional injection via hook registry)
- Kernel grep invariant verde: `rg "TwinCore|..." src/combat/ --glob '!blueprints/**'` → 0 righe

Consumes:
- Pattern blueprint completo da S08–S09

### S11 → (consumer-side, no downstream)

Produces:
- `query_skill_preview` thin wrapper su runner `Mode::Preview` → `Vec<Intent>`
- `combat_panel.rs` consuma `Intent` stream diretto, `predict_damage` UI-side rimosso
- AI scoring esterno su Intent stream
- `Mode::Preview` cabla hover-senza-target

Consumes:
- Skill migrate (S06) + Passive (S07) + Modifier pipeline (S07)

### S12 → (terminal)

Produces:
- `UnitDef` senza field Digimon-specific; ogni blueprint dichiara validation via `Registry<ValidationExt>` fn
- `ValidationSnapshot` popolata da iter del registry, no campi hardcoded
- Criterio falsificabile M021 verificato: aggiungere un Digimon scriptato tocca solo `src/combat/blueprints/<new>/` + `assets/data/digimon/<new>/`

Consumes:
- Kernel digimon-free da S10
