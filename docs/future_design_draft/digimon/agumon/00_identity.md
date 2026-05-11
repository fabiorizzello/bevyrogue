# Agumon — Identity & Kit (stress test)

> **Scope.** Stress test del design Agumon (Rookie, Fire/Vaccine) contro §2.2b Animation FSM e §8 roster minimal. Obiettivo: scoprire contraddizioni timing / vocabulary / kernel-coupling **prima** di M017.
> **Non vincolante**: questo è un exercise. Tutte le risposte aperte sono raccolte in §06_open_questions del file `04_passive_twin_core_fire.md` (ultimo della serie).

## §0 — Riferimenti

- **Atlas:** `assets/digimon/agumon_atlas.json` v1, 84 frames, frame size 1024×1024
- **§2.2b** Animation FSM (Commands vocabulary, Predicate enum, headless determinism §G)
- **§8 roster minimal** § Agumon — Fire burst
- **Dati legacy:** `assets/data/units.ron` UnitId(1), `assets/data/skills.ron` pepper_breath/agumon_ult

## §1 — Identità (canon §8)

Swing pesante a fuoco. Vive di **Heated stacks**, esplode al kill.

- **Asse primario:** Burst DPS Fire (single → splash on kill)
- **Asse secondario:** Toughness break (modesto, +1 toughness reduce su Heavy)
- **Vita:** medio-alta (HP 100), squishy se ulta sprecata (no sustain)
- **Stat baseline:** `hp_max=100`, `speed=100`, `toughness_max=50`, `weakness=Ice`, `ultimate_trigger=100`, `ultimate_cap=150`, `ultimate_charge_per_event=25`

## §2 — Atlas mapping (canonical)

Atlas Agumon v1 — 8 animazioni nominali. Mapping al kit:

| Slot kit | Atlas clip | Frame range (source) | Count | Note |
|---|---|---|---|---|
| **Idle (loop)** | `idle` | 44–49 | 6 | loop perpetuo fuori skill |
| **Basic** (`claw_strike`) | `attack` | 0–8 | 9 | swing veloce |
| **Heavy Skill** (`pepper_breath`) | `heavy_attack` | 23–36 | 14 | windup-fire-recovery |
| **Ultimate** (`nova_blast`) | `skill` | 50–66 | 17 | charge-blast-detonate |
| **Hurt (reaction)** | `hurt` | 37–43 | 7 | non gestita dalla FSM skill (passive UI) |
| **Block (reaction)** | `block` | 9–13 | 5 | reserved §3.3 §2.2b |
| **Death** | `death` | 14–22 | 9 | terminale, owned dal kernel |
| **Victory** | `victory` | 67–83 | 17 | post-combat |

**Frame budget.** Le 3 clip che usiamo nella FSM sono 9 + 14 + 17 = **40 frames atlas** (≈3.3s @12fps reference). Tutto il resto è sequenze di reaction/state.

## §3 — Timing convention

Per §2.2b §G (headless determinism): **frame counter logico è autoritativo**, ms è metadata.

- Ogni nodo FSM dichiara `frames: N` (autoritativo).
- Ogni command dichiara `at_frame: M` relativo all'inizio del nodo.
- Stretch/compress = aggiungere `Hold { extra_frames }` o `SpeedMul`, mai cambiare ms.
- **Reference framerate:** 12fps (1 frame ≈ 83ms). Ogni tabella include colonna `ms (@12fps ref)` come aiuto designer; non è autoritativa.

> **Conflitto con engine fps?** No: il render legge i frame del clip atlas; se l'engine gira a 60fps, ogni frame logico FSM dura ~5 frame render. Il rendering interpola/duplica, la FSM scandisce sui frame logici.

## §4 — Kit shape (canon §8 + numeri legacy)

| Slot | Skill ID | Target | Costo | Effetto base (intent) |
|---|---|---|---|---|
| Basic | `claw_strike` | Single (Enemy/Alive) | 0 SP, **+1 SP gen**, +25 Ult charge (OnBasicAttack) | Damage piatto Fire `≈8`; **+1 Heated stack** al primary |
| Skill | `pepper_breath` | Single (Enemy/Alive) | **1 SP** | Damage medio Fire `18`; **+2 Heated stacks**; ToughnessHit(10) |
| Ult | `nova_blast` | Single primary + splash adj (Blast) | 0 SP, drena ult bar (off-turn lanciabile anytime, HSR-style) | Damage alto `50` primary, splash 50% sui 2 adj; **modifier-firma `OnKill→Detonate(Heated)`** |
| Passive | `twin_core_fire` | — (listener) | — | +damage condizionale se Gabumon in team applica Chilled |
| Follow-up | (TBD se mantenuto) | — | — | OnEnemyBreak (`agumon_follow_up`) — **da rivalutare**: M017 §8 non lo cita esplicitamente |

**Drift legacy vs design:**
- `skills.ron` ha `pepper_breath` `sp_cost: 4` — **rotto** rispetto a SP economy reale (§5b). Allineare a **1**.
- `agumon_ult` legacy = "Nova Blast" `damage: 50, ToughnessHit(30)` — **OK**, ma manca il modifier `OnKill→Detonate(Heated)` (oggi è inert).
- `units.ron` ha `basic_skill: pepper_breath` (= heavy) — **separare** in `claw_strike` distinto.
- `agumon_follow_up` esiste in skills.ron + `units.ron.follow_up = OnEnemyBreak`. §8 non lo cita ⇒ decidere se tenerlo (probabilmente sì come retain low-cost).

## §5b — SP Economy reality check (canon shared, vincola tutti i 6 Digimon)

Riferimento codice: `src/combat/sp.rs` → `SpPool { current: 3, max: 5 }`; `RoundSpTracker { max_non_basic_per_round: 2 }`.

| Parametro | Valore canon | Note |
|---|---|---|
| Pool size | max **5 SP** | team-wide, shared |
| Start | 3 SP | encounter open |
| Basic gen | **+1 SP** | per basic action |
| Skill cost | **1 SP** | HSR baseline; ogni Digimon |
| Ult cost | 0 SP | ha bar separata, **lanciabile anytime off-turn** (HSR-style) |
| Non-basic per round cap | **2** | hard ceiling sul churn skill/round |
| Team size | 4 unità |  |

**Pressure check 1 SP/skill, 4 unità, round mix:**
- Tutti basic: +4 SP/round → cap a 5
- 2 basic + 2 skill: 0 net (sostenibile a regime)
- Tutti skill: -4 SP (impossibile → 2/round cap blocca)
- Conclusione: **1 SP è il valore corretto**. Tutti i `sp_cost` heavy/skill nei design draft Digimon = **1**.

**Errori da non rifare nei prossimi 5 Digimon:**
- ❌ Skill cost ≥2 SP — soffoca il team
- ❌ "Ult costa SP" — confondere bar ult con SP pool
- ❌ Bonus SP gen da skill — rompe la tensione (basic deve essere l'unica fonte normale)
- ✅ +1 SP gen da basic, sempre
- ✅ Eventuali talenti/passive che generano SP extra: solo via skill-tree fuori M017

## §5 — Heated (mechanic shared, da formalizzare)

Heated è uno **status fire** che Agumon stacca su target. Definizione minima per i 3 file successivi:

- **Apply:** Basic +1, Heavy +2, Ult applica indirettamente via custom_signal `apply_thermal_spark`.
- **Cap:** TBD (proposta: max 6 stacks)
- **Effect on target:** TBD per M017 — proposta: +`X%` damage taken da Fire per stack, decay -1/turno o on-debuff-removed. **Non è oggetto di stress test FSM**, è oggetto del file `passive_twin_core_fire`.
- **Detonate (Ult OnKill):** se Ult uccide primary, gli `Heated` stack rimanenti vengono "esplosi" sugli adj come damage immediato (proporzionale stacks).

Quesito design da risolvere fuori stress test:
- Heated stacks sono per-target o globali del caster? **Proposta: per-target** (più leggibile, allineato a §K canon).
- Il `custom_signals[apply_heated/apply_thermal_spark]` nei dati attuali è il legacy hook; nel nuovo modello §2.2b il blueprint Agumon ascolterà `EmitStatus(Heated)` dalla FSM, non più signal stringhe.

## §6 — Prossimi file della serie

1. `01_basic_claw_strike.md` — FSM stress base attack (1-2 nodi, low complexity, baseline)
2. `02_skill_pepper_breath.md` — FSM stress heavy skill (3-4 nodi, status apply + tough hit)
3. `03_ult_nova_blast.md` — FSM stress ult (4 nodi, edge reattivo `OnKill→ReactiveDetonate`, QTE Power Charge)
4. `04_passive_twin_core_fire.md` — blueprint listener-only (no FSM), Twin Core con Gabumon

Obiettivo per ogni file: **trovare almeno 1 contraddizione/buco** del design §2.2b prima di M017.
