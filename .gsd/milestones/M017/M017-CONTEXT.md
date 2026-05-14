# M017: Status taxonomy v0 rewrite (canon §H.1)

**Gathered:** 2026-05-12
**Status:** Ready for planning

## Project Description

Allineare il combat kernel al canon §H.1 sostituendo la tassonomia status legacy (`Burn` / `Freeze` / `Shock` / `DeepFreeze`) con i 5 status canon attivi (`Heated` / `Chilled` / `Paralyzed` / `Slowed` / `Blessed`) più 2 variant reserved gas-era (`Burn` / `Shock`, dichiarate ma non applicabili in v0). Tutti gli status seguono policy single-instance per `(target, kind)` con refresh `replace-max-dur`. Cleanse di default rimuove solo `BuffKind::Debuff`; `Blessed` è cleanse-immune. Le 5 semantiche per-status (DoT 4 dmg/turn Heated, amp +15% Heated/Chilled, skip-turn Paralyzed, delay-on-apply +30% gauge Slowed, dealt-dmg ×1.15 + Ult charge +1 Blessed) sono cablate nelle pipeline esistenti. JSONL log + ValidationSnapshot parlano il vocabolario canon. RON loader rifiuta a load-time ogni id non canon.

## Why This Milestone

Ogni skill identity del roster M017+ si poggia sui 5 status canon (audit SP1, §H.1, §G roster sheets). La tassonomia legacy non mappa più sul design round-3 e ogni `Effect::ApplyStatus` parla ancora la vecchia lingua: ogni milestone successivo (M018 time-manip, M019 DR + heal/cleanse, M020 reactive bus, M021 blueprint trait) eredita debito di traduzione finché questa fondazione non è ricostruita. Va fatto adesso perché blocca strutturalmente la pianificazione di S04 (Paralyzed/Slowed) negli status sheet correnti e perché i log JSONL sono già illeggibili a chi legge il canon.

## User-Visible Outcome

### When this milestone is complete, the user can:

- Lanciare un encounter via `cargo run --bin combat_cli` con skill che applicano `Heated` / `Chilled` / `Paralyzed` / `Slowed` / `Blessed` e vedere i 5 effetti distinti (DoT, amp, skip, delay, buff dealt + Ult charge) nei log JSONL con naming canon.
- Eseguire `cargo test` full headless integration suite e ottenere build verde con zero referenze a `Freeze` / `DeepFreeze` e con `Burn` / `Shock` confinati al solo declaration site nell'enum.
- Editare `assets/data/skills.ron` con id status non canon e ricevere errore di load chiaro al boot (fail-fast).

### Entry point / environment

- Entry point: `cargo test` (suite headless) + `cargo run --bin combat_cli` (scenari scriptati S06)
- Environment: local dev, headless first (no winit/wgpu/egui)
- Live dependencies involved: nessuna (suite deterministica)

## Completion Class

- **Contract complete:** test deterministici per le 5 semantiche (S03 amp/DoT, S04 skip + delay, S05 buff dealt + Ult charge + cleanse-immune), test policy (S02 refresh-max-dur + cleanse Debuff-only), validator RON (S01 fail-fast).
- **Integration complete:** scenario CLI scriptato S06 che applica i 5 status su unit distinti e produce JSONL + ValidationSnapshot canon analizzabili via grep test deterministico.
- **Operational complete:** N/A (milestone interno al combat kernel, no lifecycle esterno).

## Final Integrated Acceptance

A milestone chiuso si deve poter provare:

- `grep -r 'Burn\|Freeze\|Shock\|DeepFreeze' src/ tests/` ritorna match **solo** al declaration site di `Burn` / `Shock` reserved nell'enum (zero match su `Freeze` / `DeepFreeze`, zero match nei test).
- Scenario CLI scriptato applica `Heated` + `Chilled` + `Paralyzed` + `Slowed` + `Blessed` su unit diversi → JSONL log con naming canon, ValidationSnapshot.statuses_per_unit deterministico in fixture.
- Edit di `assets/data/skills.ron` con id `burn` / `shock` / inventato → load fallisce con errore chiaro (no silent no-op).
- `cargo check` + `cargo test` (full headless integration suite) verdi.

## Architectural Decisions

### Status taxonomy v0 (D004)

**Decision:** Sostituire `Burn` / `Freeze` / `Shock` / `DeepFreeze` con i 5 status canon `Heated` / `Chilled` / `Paralyzed` / `Slowed` / `Blessed` + 2 reserved `Burn` / `Shock`. Single-instance per `(target, kind)`. Refresh `replace-max-dur` (no additive). `Confused` dropped.

**Rationale:** Canon §H.1 lock-in da round-3; ogni roster sheet M017+ presuppone questo vocabolario. Traduzione perpetua sarebbe peggio del refactor.

### Slowed semantica = one-shot delay-on-apply

**Decision:** All'`EmitStatus(Slowed)` il gauge target è pushed +30% una volta sola. Lo status persiste in `statuses[]` per `dur=2` solo a fini refresh-policy + observability. Nessun recurring tick di gauge. Nessun clamp in M017.

**Rationale:** Coerente con `replace-max-dur` (niente accumulo per re-apply). Clamp `[0,200]` + cap `±50%` sono territorio M018 (split `AdvanceTurn` / `DelayTurn`). Mantenere M017 puro su semantica status, non su time-manip primitive.

**Alternatives Considered:**
- Recurring speed multiplier mentre Slowed attivo — scartato: confonde Slowed con Chilled (che già modula speed −20% turno corrente) e duplica gauge-math che M018 deve possedere.

### Source attribution differita a M020

**Decision:** In M017 `StatusInstance` resta keyed su `(target, kind)` puro. Nessun campo `source: Entity` / `source_blueprint_id`. `OnStatusApplied` payload usa il nuovo `StatusKind` senza source field.

**Rationale:** Roadmap esplicita "no nuove varianti reactive — quello è M020". Aggiungere `source` ora produce superficie morta per 3 milestone; l'uniqueness §H.2 punto 4 multi-source rimane teorica fino al bus reactive M020. Costo accettato: secondo refactor di test in M020.

**Alternatives Considered:**
- Aggiungere `source` adesso "per stabilità API" — scartato: nessun consumer attuale, e il rischio del doppio refactor è inferiore al rischio di committare uno shape sbagliato a 3 milestone di distanza.

### Heated single-instance in M017

**Decision:** Heated segue `refresh_max_dur` come gli altri 4 status canon attivi (single-instance per `(target, kind)`). Il "multi-stack cap-per-skill" citato in D004 è forward-looking.

**Rationale:** §H.5 deferra esplicitamente "Stack-aware status (Heated × N)" post-M017 (D009). Il test S02 ("apply Heated dur=2, re-apply dur=1, check dur=2") è autoritativo.

**Alternatives Considered:**
- Multi-stack già in M017 con counter — scartato: sfora scope e contraddice §H.1 single-instance rule.

### Validator RON fail-fast su id non canon

**Decision:** Il loader `src/data/skills_ron.rs` accetta solo `heated` / `chilled` / `paralyzed` / `slowed` / `blessed`. Qualsiasi altro id (incluso `burn` / `shock` reserved) produce errore di load chiaro.

**Rationale:** `burn` / `shock` reserved sono dichiarati nell'enum per stabilità futura ma non applicabili in v0; un no-op silenzioso renderebbe `assets/data/skills.ron` autoritativo su comportamento non implementato.

### Test legacy: delete-and-rewrite-fresh

**Decision:** I 4 file `tests/status_effect_apply.rs`, `tests/status_effect_turn_tick.rs`, `tests/status_effect_integration.rs`, `tests/status_accuracy.rs` sono cancellati in S01. S02-S05 scrivono test fresh sulla semantica canon.

**Rationale:** Il modello legacy non ha 1:1 con il canon (Confused dropped, DeepFreeze → Slowed con semantica diversa, accuracy mod assente nel canon). Migrare per `#[ignore]` + inventario produce debito di pulizia senza beneficio.

## Error Handling Strategy

- **RON load:** validator status id fail-fast con messaggio che cita id offending + lista id accettati.
- **Apply:** `EmitStatus(kind)` su target già portatore di `kind` invoca refresh `max(dur_old, dur_new)`; mai panic, mai duplicate instance.
- **Cleanse:** filtra per `BuffKind::Debuff`; `Blessed` (Buff cleanse-immune) sopravvive; nessun side-effect su target privo di status (no-op silenzioso, non errore).
- **Tick:** scaduto `dur == 0` rimuove l'instance senza emettere `OnStatusExpired` event (event tipizzato è M020).
- Coerente con D008 headless-first: ogni branch sopra deve girare senza feature `windowed`.

## Risks and Unknowns

- **Cascata test legacy:** la cancellazione di 4 file legacy + rename in `follow_up_chains` / `combat_coherence` / `form_identity` può smascherare assunzioni implicite sulla vecchia tassonomia (es. test che si appoggiavano a Freeze come "skip turn" ora trovano Chilled che non skippa). Mitigation: S01 deve verificare `cargo test` verde prima di chiudere; S02-S05 ricostruiscono coverage semantica.
- **JSONL log compatibility:** consumer esterni (sprite_pipeline?, analytics?) che parsano vecchi nomi → si rompono. Nessun consumer noto a oggi. Risk acceptance: rompiamo adesso, non c'è ecosistema esterno.
- **Slowed senza clamp in M017:** repeated apply prima di M018 può portare gauge fuori `[0,200]`. Mitigation: M017 single-instance + `refresh_max_dur` significa che re-apply non somma (one-shot pushed solo al primo apply), quindi il rischio è limitato a multi-source su target diversi — non un caso del roster M017.
- **`source_blueprint_id` rinviato:** quando M020 lo introduce, qualche test esistente sui buff potrebbe necessitare refactor. Tracked: aggiornare M020 context con questo follow-up esplicito.

## Existing Codebase / Prior Art

- `src/combat/status_effect.rs` — enum + componente + apply/refresh/tick attuali (legacy); target principale del rewrite.
- `src/combat/damage.rs` — pipeline danno; ospita hook amp% Heated/Chilled in S03.
- `src/combat/speed.rs` — `SpeedModifier`; ospita Chilled −20% turno corrente in S03.
- `src/combat/sp.rs` / `src/combat/ultimate.rs` — Blessed +1 Ult charge in S05.
- `src/combat/turn_system/pipeline.rs` — Paralyzed skip-turn + Slowed delay-on-apply in S04.
- `src/combat/events.rs` — payload `OnStatusApplied` (field type → nuovo `StatusKind`).
- `src/combat/observability.rs`, `src/combat/log.rs`, `src/combat/jsonl_logger.rs` — naming canon in S06.
- `src/data/skills_ron.rs` — schema `Effect::ApplyStatus` + validator id canon.
- `assets/data/skills.ron` — RON content da migrare con mappa default (`Burn→Heated`, `Freeze→Chilled`, `Shock→Paralyzed`, `DeepFreeze→Slowed`).
- `docs/combat_current.md` — sezione status da allineare al canon.

## Relevant Requirements

- Vedi `.gsd/REQUIREMENTS.md` per il register completo; M017 avanza in particolare la capacità "status taxonomy canon §H.1" che fonda l'identity di tutte le skill roster M017+.

## Scope

### In Scope

- Enum `StatusKind` canon (5 attive + 2 reserved) + componente `StatusInstance` single-instance per `(target, kind)`.
- Policy apply `replace-max-dur` + cleanse filter `BuffKind::Debuff` (con `Blessed` cleanse-immune).
- Semantica per-status: Heated (DoT 4/turno + amp +15% fire dealt-on-target), Chilled (amp +15% ice dealt-on-target + speed −20% turno corrente), Paralyzed (skip turn), Slowed (one-shot +30% gauge delay all'apply), Blessed (×1.15 dmg dealt + +1 Ult charge per azione + cleanse-immune).
- Migrazione RON con mappa default + validator fail-fast.
- JSONL log + ValidationSnapshot vocabolario canon.
- Cascata test: rename non-status, cancellazione 4 file legacy, scrittura fresh test in S02-S05.
- Update `docs/combat_current.md` sezione status.

### Out of Scope / Non-Goals

- DR pipeline `BuffKind::DR` + clamp 0.5 → **M019**.
- `Effect::EmitHeal` / `Effect::EmitCleanse` come variant tipizzate → **M019** (M017 usa cleanse esistente con filtro per kind).
- Nuove reactive event variants (`StatusApplied` tipizzato, `UltimateUsed`, `UnitDied` extended payload, source attribution su `OnStatusApplied`) → **M020**.
- `AdvanceTurn` / `DelayTurn` split + cap ±50% + gauge clamp `[0,200]` → **M018**.
- TargetShape resolver expansion → **M018**.
- Blueprint trait/registry refactor (`trait Blueprint` + `BlueprintRegistry`) → **M021**.
- UI / sprite render canon naming → **M023+**.
- Stack-aware status (Heated × N con cap-per-skill) → post-M017 (D009).
- Implementazione behavior delle variant reserved `Burn` / `Shock` → fuori M017.

## Technical Constraints

- **Headless first (D008):** nessuna dipendenza winit/wgpu/egui introdotta; tutto compila con `cargo check` default.
- **Determinismo (CLAUDE.md):** nessun wall-clock, nessun RNG senza seed nei test S02-S06.
- **Eventi single-source-of-truth:** `CombatEvent` resta il bus; il rename del payload `OnStatusApplied` non introduce canali paralleli.
- **D004 + D009:** 5 variant attive single-instance; `Confused` dropped; multi-stack deferred.
- **Naming canon:** `grep` per legacy non deve trovare match nei test né altrove in `src/` (eccetto declaration site `Burn` / `Shock` reserved).
- **Verifica di chiusura (slice e milestone):** `cargo check` + `cargo test` verdi.

## Integration Points

- `assets/data/skills.ron` — riscritto in place; consumer principale del nuovo validator.
- `src/data/skills_ron.rs` — schema `Effect::ApplyStatus` + validator (loosen su variant attivi, tighten su fail-fast id non canon).
- `src/combat/events.rs` — payload `OnStatusApplied` field type → nuovo `StatusKind`; nessuna nuova event variant.
- `src/combat/turn_system/pipeline.rs` — hook Paralyzed (skip) + Slowed (delay-on-apply) integrati nella pipeline turn esistente, non in moduli paralleli.
- `src/combat/damage.rs` / `speed.rs` / `sp.rs` / `ultimate.rs` — hook amp/speed/charge cablati nei rispettivi moduli senza branching character-specific.

## Testing Requirements

- **S01:** `cargo check` + `cargo test` full suite verdi dopo enum rewrite + RON migration + cascade test rename + delete legacy.
- **S02:** test policy deterministici (apply / re-apply max-dur / cleanse Debuff-only / Blessed cleanse-immune).
- **S03:** `tests/status_amp_pipeline.rs` (Heated +15% fire, Chilled +15% ice), Heated DoT 4/turno a turn-end visibile in log.
- **S04:** `tests/status_paralyzed_skip.rs` (skip determinato con seed fisso su N turni), `tests/status_slowed_delay.rs` (timeline push visibile via gauge snapshot pre/post apply).
- **S05:** `tests/status_blessed_offensive.rs` (×1.15 dmg dealt), `tests/status_blessed_ult_charge.rs` (+1 Ult charge per azione), `tests/status_blessed_cleanse_immune.rs`.
- **S06:** scenario CLI scriptato + grep test su JSONL log per zero leak vocabolario legacy + ValidationSnapshot fixture deterministica.
- **Gate verifica:** ogni slice chiude solo se `cargo test` verde su full suite (non solo target locale).

## Acceptance Criteria

- Per-slice "After this" criteria definiti nel roadmap M017-ROADMAP.md restano autoritativi.
- Milestone-level: ricapitola "Final Integrated Acceptance" sopra (grep clean + scenario CLI + load fail-fast + suite verde).

## Open Questions

- Nessuna aperta al kickoff.
- Risolte in fase di interview:
  - Slowed semantica → **one-shot delay-on-apply** (no recurring tick, no clamp in M017).
  - Source attribution (`StatusKind`, `source_blueprint_id`) → **deferita a M020** (M017 keyed `(target, kind)` puro).
  - Heated multi-stack → **single-instance pura in M017** (multi-stack cap-per-skill deferred post-M017, §H.5 / D009).
  - RON id non canon (incluso `burn` / `shock` reserved) → **fail-fast a load-time**.
  - Mappa rename legacy → **default** (`Burn→Heated`, `Freeze→Chilled`, `Shock→Paralyzed`, `DeepFreeze→Slowed`).
  - Test legacy → **delete-and-rewrite-fresh** in S02-S05.
