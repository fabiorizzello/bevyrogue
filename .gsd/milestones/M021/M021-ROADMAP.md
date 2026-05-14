# M021: Skill trait + SkillCtx + Blueprint trait + Plugin self-registration

**Vision:** Sostituire l'attuale schema data-driven `enum Effect` + plugin hardcoded per Digimon con due astrazioni Rust pulite — `trait Ability` (skill attive + passive) e `trait Blueprint` (mechanic franchise) — alimentate da `SkillCtx` (query read-only + enqueue `Intent`) e da un `BlueprintRegistry` startup-frozen. Al termine, il kernel `src/combat/` è completamente generico (zero menzioni di Twin Core / Holy Support / Predator Loop / Battery Loop / Precision Mind Game / Kitsune Grace), aggiungere una nuova skill o un nuovo Digimon non richiede di toccare il kernel, e una sola definizione di skill serve preview, execute, journal, e UI senza riscritture parallele.

## Success Criteria

- Aggiungere un nuovo Digimon richiede solo file nuovi sotto `src/combat/blueprints/<new>/` — zero edit a `kernel.rs`, `intent.rs`, `ctx.rs`, o `combat::api`
- `grep -rE "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/kernel.rs` → 0 righe
- Tutte le 24 skill canon e 6 passive canon eseguono via `Ability::resolve` su `SkillCtx`; `enum Effect` rimosso da `src/data/skills_ron.rs`
- UI combat panel e AI scoring leggono `Ability::resolve(ctx, Mode::DryRun) → ImpactShape` invece di `predict_damage` locale
- `cargo test` verde end-to-end (~74 test integration esistenti + nuovi test per registry, ctx, hooks)
- `cargo check` headless e `--features windowed` senza warning nuovi
- JSONL trace include `CombatKernelTransition::Blueprint { owner, payload }` per ogni `BlueprintSignal` (D008)
- L'invariante kernel ↔ blueprint: `rg "use bevy" src/combat/blueprints/` → 0 righe per blueprint plugin code

## Key Risks / Unknowns

- **Big-bang vs incremental migration** — 16 slice interleaved con dipendenze forti. Rischio di lasciare il main branch in stato "skill metà migrate, metà ancora su Effect" per più giorni. Mitigato dal registry doppio temporaneo (Effect-based + Ability-based convivono fino a S08) e dal commit per-slice con suite verde.
- **Hook cascade semantics divergence preview vs execute** — `preview ≡ execute` (D009) è la promessa più forte verso UI/AI; bug nella simulazione hook contro `FrozenSnapshot` invaliderebbe la fiducia nei numeri preview. Retire in S06 con test parallelo che cast-a una skill in DryRun e Execute e asserisce identità degli `ImpactShape` prodotti.
- **`Intent::BlueprintSignal` payload type-erasure** — `Box<dyn Any>` (D006) ha invariante owner↔type non enforcabile dal compiler. Bug "Agumon emette signal con payload di TwinCore Water" è silenzioso a meno di downcast check. Retire in S03 con `debug_assert` nel dispatcher + test che provoca mismatch e verifica panic in debug.
- **Turn-phase order regression** — D011 introduce 5 step espliciti pre-cast. La pipeline attuale `turn_system/pipeline.rs` (~67 KB) implementa già una variant; rischio di duplicare o di omettere step. Retire in S04 con test esplicito che verifica l'ordine `tick → KO → passive → KO → snapshot → select → cast` su scenario "Agumon OnTurnStart kills Tentomon".
- **AbilityBuilder ergonomia non testata** — il design (`ctx.deal(t).damage(n).tag(...).on_hit(...).done()`) è sulla carta. Rischio di scoprire in S08 (migrazione 24 skill) che il builder non copre un pattern. Retire in S07 costruendo manualmente le 2 skill più complesse (Tohakken multi-hit, Petit Thunder bounce) come fixture e iterando.

## Proof Strategy

- Hook cascade `preview ≡ execute` → retire in **S06** provando che, su una skill chain ramificata (es. Koyosetsu con `on_hit → Heal`), `Mode::DryRun` e `Mode::Execute` producono `ImpactShape` strutturalmente identici contro la stessa `FrozenSnapshot`
- BlueprintSignal owner↔type invariant → retire in **S03** provando che un mismatch owner→payload type panicca in `debug_assertions` con messaggio actionable, e che il signal corretto round-trip via `CombatKernelTransition::Blueprint` arriva al plugin owner intatto
- Turn-phase order → retire in **S04** provando con uno scenario fixture che (a) status DoT applicato a Tentomon lo uccide prima dello step 6, (b) la skill selezionata dal giocatore non può targettare Tentomon perché filtrato dalla snapshot fresca
- AbilityBuilder copertura → retire in **S07** costruendo Tohakken (multi-hit Bounce con `MultiHitOnKO::Redirect`) e Petit Thunder (AoE+chain) via builder e validando con suite esistente
- Kernel-generic invariant → retire in **S13** con il grep `rg "TwinCore|..." src/combat/kernel.rs` → 0 righe, dimostrando che i 5 famiglie enum Digimon-specific sono migrate fuori dal kernel

## Verification Classes

- **Contract verification**: nuovi unit/integration test per `AbilityRegistry::lookup`, `BlueprintRegistry::get`, `SkillCtx::drain_intents`, `intent_applier` per ogni `Intent` variant (Damage / Heal / ApplyStatus / FollowUp / AdvanceTurn / DelayTurn / Cleanse / BlueprintSignal / Reject), `MultiHitOnKO` policy (Skip/Redirect/Stop), listener ordering tiebreaker (`speed_initiative DESC, slot_index ASC, team_id ASC`)
- **Integration verification**: suite `tests/` esistente (~74 test funzionali) **deve restare verde a ogni slice merge** — Bounce, Blast, AoE, Heal, Cleanse, status apply, Ult instant; nuovi test per turn-phase ordering, hook cast-scoped vs listener bus-globale, JSONL transition stream contiene `Blueprint` variant
- **Operational verification**: `cargo check` (headless) + `cargo check --features windowed` senza warning nuovi a ogni slice; grep verifiers (`rg "use bevy" src/combat/blueprints/`, `rg "TwinCore" src/combat/kernel.rs`, `rg "Effect::" src/`) come gate hard a slice di chiusura
- **UAT / human verification**: gameplay smoke run con `cargo run --features windowed` a fine S08 (skill migrate) e fine S13 (kernel digimon-free) — verificare che le ability si comportano identico al pre-refactor su almeno 2 encounter scriptati

## Milestone Definition of Done

This milestone is complete only when all are true:

- Tutte le 16 slice sono in stato `[x]` con SUMMARY.md scritto
- `cargo test` headless verde su 74+ test integration (suite esistente non ridotta)
- `cargo run --features windowed` smoke run su un encounter completo (player vs enemy team, almeno una Ult, una passive trigger, un Bounce) si comporta indistinguibile dal pre-refactor
- I tre grep invarianti (`rg "use bevy" src/combat/blueprints/`, `rg "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/kernel.rs`, `rg "enum Effect" src/data/skills_ron.rs`) restituiscono 0 righe
- JSONL produce `CombatKernelTransition::Blueprint { owner, payload }` per almeno un signal su un encounter scriptato (replay-friendly forensics)
- `CODEBASE.md` rigenerato e mostra la nuova struttura `src/combat/api/`
- `KNOWLEDGE.md` aggiornato con la nuova architecture rule "skill = `trait Ability`, niente `enum Effect`, niente mutazione diretta del World"
- Pre-condizione M024 (primo roster successivo): la `M024-CONTEXT` può citare l'invariante "files toccati ⊂ `src/combat/blueprints/<new>/`" come acquisizione M021

## Requirement Coverage

- Covers: N/A — il progetto non ha ancora un `REQUIREMENTS.md` formale (capability contract pre-1.0)
- Partially covers: nessuno
- Leaves for later: M022 (asset pipeline), M023 (visual stack), M024+ (nuovi roster)
- Orphan risks: stack-aware status numerici (deferred D009 pre-esistente, non in M021); hot-reload skill logic (out of scope, vincolo utente esplicito)

## Slices

- [ ] **S01: CombatPlugin extract** `risk:low` `depends:[]`
  > After this: `cargo check` headless + windowed verde; `register_combat_kernel_runtime` sostituito da `app.add_plugins(CombatPlugin)`; nessuna logica cambiata; main.rs/headless.rs/windowed.rs ricomposti via plugin
- [ ] **S02: `combat::api` facade — domain-pure types** `risk:low` `depends:[S01]`
  > After this: modulo `src/combat/api/` esiste con `Ability`, `Intent`, `SkillCtx`, `BlueprintState`, `ImpactShape` come tipi placeholder; `rg "use bevy" src/combat/api/` → 0 righe; `static_assertions::assert_obj_safe!(Ability)` compila
- [ ] **S03: `trait Blueprint` + `BlueprintRegistry` + dispatcher** `risk:high` `depends:[S01,S02]`
  > After this: `register_blueprint::<B>(app)` atomico (D012), dispatcher route `Intent::BlueprintSignal` per owner, debug panic su mismatch owner↔type (D006), JSONL contiene `CombatKernelTransition::Blueprint { owner, payload }` (D008) round-trip via test
- [ ] **S04: Interprete + `AbilityRegistry` + observer wiring + turn-phase order** `risk:high` `depends:[S02,S03]`
  > After this: `AbilityRegistry` come `Resource`, `CombatAppExt::register_ability` cabla observer per `AbilityHook` (cast-scoped) e `BlueprintListener` (bus-globale) con tiebreaker D013; pipeline turn-start applica i 5 step D011 e un test fixture verifica che una passive `OnTurnStart` può uccidere un target prima dello step 6
- [ ] **S05: `Intent` canon + chain linkage + applier** `risk:high` `depends:[S04]`
  > After this: enum `Intent` con ~17 variant canon (D002) + `BlueprintSignal` + `Reject`; `intent_applier` route per variant, drain FIFO, dry-run mode (no apply) testato; `Mode::DryRun` + `Mode::Execute` producono ImpactShape identici su skill banale
- [ ] **S06: `SkillCtx` + `FrozenSnapshot` + hook cascade cast-scoped** `risk:medium` `depends:[S05]`
  > After this: `SkillCtx` espone accessor mirati (D015 Q3), `Arc<FrozenSnapshot>` come backing immutabile per dry-run, `AbilityHook` filtrato sul `cast_id` corrente vs `BlueprintListener` su bus globale (D009); test parallelo dimostra `preview ≡ execute` su chain ramificata (`on_hit → Heal`)
- [ ] **S07: `AbilityBuilder` + typed tuning eager injection** `risk:medium` `depends:[S06]`
  > After this: builder fluente costruisce Tohakken (MultiHit Bounce con `MultiHitOnKO::Redirect(LowestHpAlive)`, D011) e Petit Thunder (AoE) come fixture; `struct *Numbers: Deserialize` per ogni skill, typo nel RON = errore al boot; circuit breaker debug-only @256 evt/cast (D014) testato con loop deliberato
- [ ] **S08: Migrate 24 skill canon + drop `enum Effect`** `risk:medium` `depends:[S07]`
  > After this: tutte 24 skill canon migrate in `blueprints/<x>/abilities/<name>.rs`; `enum Effect` rimosso da `src/data/skills_ron.rs`; `skills.ron` contiene solo numeri/tag; suite integration verde, gameplay smoke run su 1 encounter scriptato indistinguibile dal pre-refactor
- [ ] **S09: Agumon migrated to `Blueprint` trait** `risk:low` `depends:[S03,S08]`
  > After this: Agumon usa `register_blueprint::<Agumon>` con `type State = TwinCoreFireState`, signal Twin Core emessi via `Intent::BlueprintSignal { owner: "twin_core", ... }` round-trippano via JSONL; passive Twin Core Fire ancora su shim legacy (migrata in S11)
- [ ] **S10: Gabumon migrated paired Twin Core** `risk:low` `depends:[S09]`
  > After this: Gabumon usa stesso registry, Twin Core mini-plugin gestisce entrambi gli owner (D005) senza tocchi a `blueprints/twin_core/` per aggiungere Gabumon; suite verde
- [ ] **S11: Migrate 6 passive canon via `AbilityHook` + `BlueprintListener`** `risk:high` `depends:[S07,S10]`
  > After this: Twin Core Fire/Water, Holy Aegis, Predator Loop, Battery Loop, Precision Mind Game tutte come `kind: Passive` con `hooks()` o `listeners()`; modifier pipeline (§5.12 research) cabla i passive stat boost; observer test verdi per ogni passive reattiva
- [ ] **S12: Dorumon + Tentomon migrated** `risk:low` `depends:[S11]`
  > After this: Dorumon (Predator Loop) e Tentomon (Battery Loop) usano `register_blueprint`; due moduli flat (`blueprints/tentomon.rs`) ristrutturati a directory simmetrica `blueprints/tentomon/`
- [ ] **S13: Patamon + Renamon migrated, kernel Digimon-free** `risk:medium` `depends:[S12]`
  > After this: Patamon (Holy Aegis) + Renamon (Kitsune Grace + Precision Mind Game) migrati; tutti i 5 variant Digimon-specific di `CombatKernelTransition` rimossi da `kernel.rs`; `rg "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/kernel.rs` → 0 righe
- [ ] **S14: UI/AI consumers riscritti via `Ability::resolve(Mode::DryRun)`** `risk:low` `depends:[S08,S11]`
  > After this: `query_skill_preview` chiama `Ability::resolve(ctx, Mode::DryRun) → ImpactShape`, `combat_panel.rs` consuma direttamente, `predict_damage` rimosso da UI; AI scoring usa helper esterno su `ImpactShape`; `Mode::DryRunNoTarget` cablato per hover-senza-target
- [ ] **S15: `RosterEntry` blueprint-keyed payload** `risk:low` `depends:[S13]`
  > After this: `UnitDef` perde i field hardcoded (`twin_core`, `holy_support`); ogni blueprint dichiara la propria validation rule via trait method; aggiungere un Digimon nuovo non richiede di editare `units_ron.rs`
- [ ] **S16: `ValidationSnapshot` field-from-registry** `risk:low` `depends:[S15]`
  > After this: `ValidationSnapshot` popolata dal `BlueprintRegistry` (i 5 sub-snapshot hardcoded eliminati); criterio falsificabile M021 (M024 puramente blueprint-locale) verificato a tavolino su 1 Digimon scriptato; `KNOWLEDGE.md` aggiornato

## Horizontal Checklist

- [ ] Ogni D### M021 (D002–D015) ri-letto a fine S08 e S13 — ancora valido a nuovo scope?
- [ ] K-P001 aggiornato in `KNOWLEDGE.md` con la nuova rule "skill = `trait Ability`, niente `enum Effect`, niente mutazione diretta del World"
- [ ] `CODEBASE.md` rigenerato dopo S13 e S16 per riflettere la nuova struttura `src/combat/api/`
- [ ] Suite integration deterministica preservata — nessun nuovo test deve introdurre wall-clock o RNG senza seed
- [ ] `cargo check --features windowed` verificato a ogni slice (non solo headless) per intercettare regressioni egui
- [ ] JSONL backward-compatibility valutata a S03 — `CombatKernelTransition::Blueprint` aggiunto, eventuali tool downstream che parsano JSONL avvisati

## Boundary Map

### S01 → S02

Produces:
- `CombatPlugin: Plugin` wrappa `register_combat_kernel_runtime`; non importa winit / wgpu / bevy_egui (vincolo D008)
- Re-export plugin pubblico da `src/lib.rs`

Consumes:
- nothing (first slice)

### S02 → S03, S04

Produces:
- Modulo `src/combat/api/` con i tipi: `Ability` (trait object-safe), `BlueprintState` (trait), `Intent` (enum scaffold), `SkillCtx` (struct scaffold), `ImpactShape` (struct), `Mode { DryRun, Execute, DryRunNoTarget }`
- Invariante grep: `rg "use bevy" src/combat/api/` → 0 righe (eccetto `Resource` / `Component` come marker neutri)

Consumes:
- nothing oltre `S01`

### S03 → S04, S09, S10, S12, S13

Produces:
- `trait Blueprint { type State: BlueprintState; const ID: &'static str; fn build(app: &mut App); }`
- `BlueprintRegistry: Resource` startup-frozen (D007) con `get(owner) / iter()`
- `register_blueprint::<B>(app)` atomico: registra type + state + entry registry (D012)
- `CombatKernelTransition::Blueprint { owner: &'static str, payload: Box<dyn Any + Send + Sync> }` variant
- Dispatcher `BlueprintSignal → CombatKernelTransition::Blueprint → JSONL` (D008)

Consumes:
- `Ability` trait scaffold da S02 (per parametrizzare il signal payload)

### S04 → S05, S06, S08, S11

Produces:
- `AbilityRegistry: Resource` + `CombatAppExt::register_ability::<A>()`
- Observer cabling per `AbilityHook` (cast-scoped, filter `cast_id == current`) e `BlueprintListener` (bus-globale) con tiebreaker `(speed_initiative DESC, slot_index ASC, team_id ASC)` (D013)
- Turn-phase pipeline 5-step: `TurnStart → tick DoT/status → resolve KO → apply turn-start passive → resolve KO → build TargetableSnapshot fresca → select skill+target → cast` (D011)

Consumes:
- `BlueprintRegistry` da S03 per discovery dei `BlueprintListener`

### S05 → S06, S07, S08, S11, S14

Produces:
- `enum Intent` canon completo con ~17 variant (D002): `DealDamage`, `Heal`, `ApplyStatus`, `RemoveStatus`, `Cleanse`, `ChangeSP`, `ChangeUlt`, `AdjustToughness`, `Stun`, `AdvanceTurn(u32)`, `DelayTurn(u32)`, `EnqueueFollowUp`, `BlueprintSignal { owner, payload }`, `Reject { reason }`, ...
- `intent_applier` che drena FIFO la coda e route per variant
- `Mode::DryRun` (no apply) vs `Mode::Execute` parallel paths

Consumes:
- `CombatKernelTransition::Blueprint` da S03 (per il dispatch di `BlueprintSignal`)

### S06 → S07, S08, S11, S14

Produces:
- `SkillCtx<'w>` con accessor mirati (`target_attribute`, `target_affinities`, `caster_hp_pct`, `alive_enemies`, `peek_pending`, ...) e `ctx.enqueue(Intent)`
- `FrozenSnapshot` (`Arc<...>`) come backing immutabile per `Mode::DryRun`
- `AbilityHook` cast-scoped semantics, garantito `preview ≡ execute`

Consumes:
- `Intent` enum da S05 per il drain

### S07 → S08, S11

Produces:
- `AbilityBuilder` fluente (`ctx.deal(t).damage(n).tag(...).on_hit(...).done()`)
- Loader RON typed (`struct *Numbers: Deserialize`) con eager injection al construction (typo = errore al boot)
- Circuit breaker debug-only @256 evt/cast (D014)
- 2 skill di prova migrate (Tohakken, Petit Thunder) come fixture di sanity

Consumes:
- `SkillCtx` + builders da S06

### S08 → S09, S14

Produces:
- 24 skill canon migrate in `src/combat/blueprints/<x>/abilities/<name>.rs`
- `enum Effect` rimosso da `src/data/skills_ron.rs`
- `skills.ron` contiene solo `id`, numeri, `target_shape` base, tag (niente logica)

Consumes:
- AbilityBuilder + tuning da S07

### S09 → S10, S11

Produces:
- Agumon usa `register_blueprint::<Agumon>` con `type State = TwinCoreFireState`
- Pattern `Intent::BlueprintSignal { owner: "twin_core", payload }` round-trip via JSONL

Consumes:
- BlueprintRegistry + dispatcher da S03; skill Agumon migrate da S08

### S10 → S11

Produces:
- Gabumon usa stesso registry, Twin Core mini-plugin (D005) gestisce entrambi gli owner senza modifiche a `blueprints/twin_core/`

Consumes:
- Pattern Agumon da S09

### S11 → S12, S13, S14

Produces:
- 6 passive canon migrate come `kind: Passive` con `hooks()` (intra-skill) o `listeners()` (cross-Digimon)
- Modifier pipeline (§5.12 research) cabla passive stat boost

Consumes:
- AbilityBuilder da S07, pattern Twin Core da S10

### S12 → S13

Produces:
- Dorumon + Tentomon via `register_blueprint`; struttura blueprint flat (`blueprints/tentomon.rs`) ristrutturata a directory `blueprints/tentomon/`

Consumes:
- Passive migrate da S11

### S13 → S15, S14

Produces:
- Patamon + Renamon migrati
- 5 variant Digimon-specific di `CombatKernelTransition` rimossi da `kernel.rs`
- Kernel grep invariant verde: `rg "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/kernel.rs` → 0 righe

Consumes:
- Pattern blueprint completo da S12

### S14 → (consumer-side, no downstream)

Produces:
- `query_skill_preview` thin wrapper su `Ability::resolve(ctx, Mode::DryRun)`
- `combat_panel.rs` consuma `ImpactShape` direttamente; `predict_damage` UI-side rimosso
- AI scoring esterno su `ImpactShape`

Consumes:
- Skill (S08) + Passive (S11) migrate

### S15 → S16

Produces:
- `UnitDef` senza field hardcoded; ogni blueprint dichiara validation rule via trait method
- Aggiungere un Digimon nuovo non richiede di editare `src/data/units_ron.rs`

Consumes:
- Kernel digimon-free da S13

### S16 → (terminal)

Produces:
- `ValidationSnapshot` popolata dal `BlueprintRegistry`, no campi hardcoded
- Criterio falsificabile M021 verificato: aggiungere un Digimon scriptato tocca solo `src/combat/blueprints/<new>/`

Consumes:
- RosterEntry blueprint-keyed da S15
