# Renamon — Identity & Kit

> **Scope.** Identity sheet **revisionata** (2026-05-12): canon §8 originale era "AoE caster Confused". Decisione user: passa a **Holy + Time Manipulation AoE**, non crit-based, differenziata da Dorumon (single-target Dark threshold). Stesso pattern di `agumon/00_identity.md`.

## §0 — Riferimenti

- **Atlas:** `assets/digimon/renamon_atlas.json` v1, 71 frames, frame size 633×701
- **Canon Digimon:** Renamon, Child, Beast (kitsune-mystic), Data, fields: Nightmare Soldiers / Virus Busters / Wind Guardians
- **§8 roster minimal** § Renamon — AoE caster (**override applicato qui**)
- **Override rationale:** evitare overlap meccanico con Dorumon (entrambi proposti dark/crit nell'esplorazione precedente). Renamon kitsune-flavor → illusion/teleport canon (`Kohenkyo`) → time-manip è naturale.

## §1 — Identità

**Mystica del tempo.** Sweep AoE Holy + manipolazione del turn order. Non infligge crit né scala su stato del nemico; il payoff è **controllo del tempo** (advance alleati, delay nemici) + damage AoE costante. **Differenziazione vs Dorumon:** lane opposte (sweep AoE Holy vs picker single Dark), trigger opposti (turn-order vs HP-threshold), payoff opposti (tempo guadagnato vs nemico morto).

- **Asse primario:** AoE Holy + turn order manipulation
- **Asse secondario:** Buff alleato `Blessed` (status holy, non-cleansable da nemici)
- **Vita:** media (HP ~95), elegante non fragile
- **Stat baseline (proposta):** `hp_max=95`, `speed=115` (la più veloce del roster, coerente con time-theme), `toughness_max=45`, `weakness=Dark`, `ultimate_trigger=120` (più alto: ult forte), `ultimate_cap=150`, `ultimate_charge_per_event=25`

## §2 — Atlas mapping

| Slot kit | Atlas clip | Range | Count |
|---|---|---|---|
| **Idle (loop)** | `idle` | 37–44 | 8 |
| **Basic** (`quick_strike`) | `attack` | 0–9 | 10 |
| **Skill** (`diamond_storm`) | `heavy_attack` | 20–29 | 10 |
| **Ultimate** (`fox_drive`) | `skill` | 45–56 | 12 |
| **Hurt** | `hurt` | 30–36 | 7 |
| **Block** | `block` | 10–15 | 6 |
| **Death** | `death` | 16–19 | 4 |
| **Victory** | `victory` | 57–70 | 14 |

Frame budget FSM: 10 + 10 + 12 = **32 frames** (≈2.7s @12fps). Snappy, coerente con velocità.

## §3 — Timing convention

Shared. Time-manip effects operano su `TurnOrder` (vedi `src/combat/turn_order.rs`) tramite custom signal, **non** alterano frame logici della FSM (FSM determinismo invariato §G).

## §4 — Kit shape

| Slot | Skill ID | Target | Costo | Effetto base |
|---|---|---|---|---|
| Basic | `quick_strike` | Single | 0 SP, +1 gen, +25 Ult | Damage piatto Holy |
| Skill | `diamond_storm` | **AoE(All) enemies** | **1 SP** | Damage medio Holy su tutti (no crit, no status); **`AdvanceTurn(self, 25%)`** — la sua action gauge avanza |
| Ult | `fox_drive` | AoE(All) enemies | UltCharge | Damage alto Holy a tutti; **`DelayTurn(all enemies, 30%)`** — l'action gauge nemico arretra; applica `Blessed` a tutti gli alleati per 2 turni |
| Passive | `kitsune_grace` | listener | — | Quando un alleato consuma Ult, **`AdvanceTurn(self, 10%)`** — Renamon recupera tempo per riapplicare AoE |

**Niente crit, niente Confused random.** L'identità è **tempo**, non caos. Differenziato anche da Tentomon (Bounce + Paralyzed) e da Agumon (Heated burst).

## §5 — Time manipulation (mechanic, NUOVO)

Riferimento codice da costruire: hook su `TurnOrder` via custom signal `advance_turn` / `delay_turn`.

- **AdvanceTurn(target, pct):** sottrae `pct%` al next-action gauge del target → agisce prima.
- **DelayTurn(target, pct):** aggiunge `pct%` al gauge → agisce dopo.
- **Cap:** ogni effetto ±50% per evitare lock infiniti. Stack additivo nello stesso turno, clamp `[0, 200%]`.
- **Determinismo:** modifiche atomiche dopo resolution; nessuna reorder mid-action (eviterebbe race con FSM running). Vedi `src/combat/turn_order.rs::TurnAdvanced` event.
- **UI:** turn order tracker (HSR-style) deve animare lo shift; placeholder per ora (M017 fuori scope).

## §6 — Blessed (status alleato, NUOVO)

- **Apply:** Ult `fox_drive` su tutti gli alleati, 2 turni.
- **Effect on ally:** `+15% damage dealt` + `+1 Ult charge gen per action`.
- **Cleanse:** non-cleansable da effetti nemici (è buff, non debuff). Patamon `holy_breeze` non lo tocca (filtra solo debuff).
- **Stack:** non stacka, refresh durata.

## §7 — Sinergie

- **Renamon → tutti:** AdvanceTurn alleati indirettamente via `kitsune_grace` (recupera turni per riapplicare AoE → più damage globale).
- **Renamon ↛ Dorumon (lane separate):** Renamon non spreada `Confused` (rimosso). Dorumon non dipende più da status setter Renamon. Sinergia debole intenzionale: si **complementano** (AoE+single) senza dipendere uno dall'altro.
- **Renamon ↔ Patamon:** entrambi Holy. Patamon heal, Renamon damage. `Blessed` (Renamon) + `holy_aegis` (Patamon) → team altamente protetto e potenziato per N turni.
- **Renamon ↔ Tentomon:** Tentomon batteria SP → Renamon costa solo 1 SP/skill, quindi può spammare AoE 2/round (cap RoundSpTracker).

## §8 — Domande aperte

- Stat advance/delay: % gauge è la metrica giusta o flat speed? (proposta: % gauge, più leggibile)
- `Blessed` interagisce con Twin Core Agumon/Gabumon? (proposta: no, buff isolato)
- Ult cost UltCharge: 120 trigger vs 100 standard giustificato dalla potenza time-manip? Validare a playtest.
- FoxDrive AoE damage va in `OnBreak→Detonate` come §8 originale prevedeva? **Rimosso** in revisione: niente modifier reattivo, identità = tempo non crit/break. Da confermare.
