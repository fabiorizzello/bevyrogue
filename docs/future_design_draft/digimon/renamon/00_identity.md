# Renamon — Identity & Kit

> **Scope.** Identity sheet **revisionata 2x** (2026-05-12):
> - rev1: canon §8 originale "AoE caster Confused" → **Holy + Time Manipulation AoE**, no crit, differenziata da Dorumon.
> - rev2: audit canon DAPI + atlas anim → rename skills a **canon Tamers** (Kokaishū / Koyōsetsu / Tōhakken), atlas clip swap skill↔ult. **Effetti meccanici invariati**, solo naming + clip mapping. Stesso pattern di `agumon/00_identity.md`.

## §0 — Riferimenti

- **Atlas:** `assets/digimon/renamon_atlas.json` v1, 71 frames, frame size 633×701
- **Canon Digimon:** Renamon, Child, Beast (kitsune-mystic), Data, fields: Nightmare Soldiers / Virus Busters / Wind Guardians
- **§8 roster minimal** § Renamon — AoE caster (**override applicato qui**)
- **Override rationale:** evitare overlap meccanico con Dorumon (entrambi proposti dark/crit nell'esplorazione precedente). Renamon kitsune-flavor → illusion/teleport canon (`Kohenkyo`) → time-manip è naturale.
- **Canon move pool (DAPI):** Koyōsetsu (Fox Leaf Arrowheads / **Diamond Storm** Tamers), Kohenkyo (Fox Switch Deception, illusion/teleport), Tōhakken (**Power Paw** Tamers, blue-fire hand strike), Shōda (Palm Strike), Sōzan (Claw Slash), Kokaishū (**Fox Spin Kick**), Kūchū Koyōsetsu (Aerial Diamond Storm), Korenkyaku (Fox Combo Kick), Kosengeki (Fox Flash Attack).
- **Rev2 canon selection:** Kokaishū (basic kick anim), Koyōsetsu (skill, signature Tamers), Tōhakken (ult, Power Paw reskin Holy con AoE VFX shockwave). Kitsune-grace resta flavor-only (no canon move, ability passive).

## §1 — Identità

**Mystica del tempo.** Sweep AoE Holy + manipolazione del turn order. Non infligge crit né scala su stato del nemico; il payoff è **controllo del tempo** (advance alleati, delay nemici) + damage AoE costante. **Differenziazione vs Dorumon:** lane opposte (sweep AoE Holy vs picker single Dark), trigger opposti (turn-order vs HP-threshold), payoff opposti (tempo guadagnato vs nemico morto).

- **Asse primario:** AoE Holy + turn order manipulation
- **Asse secondario:** Buff alleato `Blessed` (status holy, non-cleansable da nemici)
- **Vita:** media (HP ~95), elegante non fragile
- **Stat baseline (proposta):** `hp_max=95`, `speed=115` (la più veloce del roster, coerente con time-theme), `toughness_max=45`, `weakness=Dark`, `ultimate_trigger=120` (più alto: ult forte), `ultimate_cap=150`, `ultimate_charge_per_event=25`

## §2 — Atlas mapping

| Slot kit | Atlas clip | Range | Count | Anim type |
|---|---|---|---|---|
| **Idle (loop)** | `idle` | 37–44 | 8 | loop |
| **Basic** (`kokaishu`) | `attack` | 0–9 | 10 | **kick** (single roundhouse) |
| **Skill** (`koyosetsu`) | `skill` | 45–56 | 12 | **sweep/cast** (diamond storm signature) |
| **Ultimate** (`tohakken`) | `heavy_attack` | 20–29 | 10 | **claw + AoE-cast VFX** (Power Paw reskin Holy) |
| **Hurt** | `hurt` | 30–36 | 7 | one-shot |
| **Block** | `block` | 10–15 | 6 | hold |
| **Death** | `death` | 16–19 | 4 | one-shot |
| **Victory** | `victory` | 57–70 | 14 | loop |

Frame budget FSM: 10 (basic) + 12 (skill) + 10 (ult) = **32 frames** (≈2.7s @12fps). Snappy, coerente con velocità.

**Atlas clip swap rationale (rev2):** atlas `skill` (45–56) ha sweep wide cinematico = match perfetto Diamond Storm signature → assegnato a skill slot. Atlas `heavy_attack` (20–29) è claw frontale single-strike → ult Tōhakken (Power Paw, hand-ignition) con VFX shockwave radiale che vendi visualmente AoE.

## §3 — Timing convention

Shared. Time-manip effects operano su `TurnOrder` (vedi `src/combat/turn_order.rs`) tramite custom signal, **non** alterano frame logici della FSM (FSM determinismo invariato §G).

## §4 — Kit shape

| Slot | Skill ID | Canon JP/EN | Target | Costo | Effetto base |
|---|---|---|---|---|---|
| Basic | `kokaishu` | Kokaishū / Fox Spin Kick | Single | 0 SP, +1 gen, +25 Ult | Damage piatto Holy |
| Skill | `koyosetsu` | Koyōsetsu / Diamond Storm (Tamers) | **AoE(All) enemies** | **1 SP** | Damage medio Holy su tutti (no crit, no status); **`AdvanceTurn(self, 25%)`** — la sua action gauge avanza |
| Ult | `tohakken` | Tōhakken / Power Paw (Tamers, Holy reskin) | AoE(All) enemies | UltCharge | Damage alto Holy a tutti; **`DelayTurn(all enemies, 30%)`** — l'action gauge nemico arretra; applica `Blessed` a tutti gli alleati per 2 turni |
| Passive | `kitsune_grace` | flavor-only (kitsune-mystic) | listener | — | Quando un alleato consuma Ult, **`AdvanceTurn(self, 10%)`** — Renamon recupera tempo per riapplicare AoE |

**Niente crit, niente Confused random.** L'identità è **tempo**, non caos. Differenziato anche da Tentomon (Bounce + Paralyzed) e da Agumon (Heated burst).

## §5 — Time manipulation (mechanic, NUOVO)

Riferimento codice da costruire: hook su `TurnOrder` via custom signal `advance_turn` / `delay_turn`.

- **AdvanceTurn(target, pct):** sottrae `pct%` al next-action gauge del target → agisce prima.
- **DelayTurn(target, pct):** aggiunge `pct%` al gauge → agisce dopo.
- **Cap:** ogni effetto ±50% per evitare lock infiniti. Stack additivo nello stesso turno, clamp `[0, 200%]`.
- **Determinismo:** modifiche atomiche dopo resolution; nessuna reorder mid-action (eviterebbe race con FSM running). Vedi `src/combat/turn_order.rs::TurnAdvanced` event.
- **UI:** turn order tracker (HSR-style) deve animare lo shift; placeholder per ora (M017 fuori scope).

## §6 — Blessed (status alleato, NUOVO)

- **Apply:** Ult `tohakken` su tutti gli alleati, 2 turni.
- **Effect on ally:** `+15% damage dealt` + `+1 Ult charge gen per action`.
- **Cleanse:** non-cleansable da effetti nemici (è buff, non debuff). Patamon `patapata_hover` non lo tocca (filtra solo debuff).
- **Stack:** non stacka, refresh durata.

## §7 — Sinergie

- **Renamon → tutti:** AdvanceTurn alleati indirettamente via `kitsune_grace` (recupera turni per riapplicare AoE → più damage globale).
- **Renamon ↛ Dorumon (lane separate):** Renamon non spreada `Confused` (rimosso). Dorumon non dipende più da status setter Renamon. Sinergia debole intenzionale: si **complementano** (AoE+single) senza dipendere uno dall'altro.
- **Renamon ↔ Patamon:** entrambi Holy. Patamon heal, Renamon damage. `Blessed` (Renamon) + `holy_aegis` (Patamon) → team altamente protetto e potenziato per N turni.
- **Renamon ↔ Tentomon:** Tentomon batteria SP → Renamon costa solo 1 SP/skill, quindi può spammare AoE 2/round (cap RoundSpTracker).

## §8 — Domande chiuse (round 2026-05-12)

**D1 — Metrica time-manip: `% gauge` (chiusa).**
- AdvanceTurn/DelayTurn operano su **% del next-action gauge** del target, non su flat speed.
- Rationale: leggibilità HSR-style (turn order tracker mostra shift % del gauge, non delta speed astratto); evita interazioni opache con `speed` stat (che resta invariante di unit, non field-per-event).
- Conforme a §5 spec già scritta (`AdvanceTurn(target, pct)` / `DelayTurn(target, pct)`).

**D2 — `Blessed` × Twin Core: no interaction (chiusa).**
- `Blessed` (buff Renamon su alleati) è **isolato**: non triggera né è triggerato da Twin Core Agumon/Gabumon.
- Rationale: Twin Core legge esclusivamente lo status del **partner Agumon ↔ Gabumon** (Heated/Chilled, gabumon/00 §6 D3); `Blessed` non è status partner-side, non entra nel loop.
- Stack: `Blessed` può coesistere con `Heated`/`Chilled` su un alleato (buff + debuff separati), ma non somma né triggera nulla cross-buff.

**D4 — FoxDrive `OnBreak→Detonate`: rimosso definitivo (chiusa).**
- Niente modifier reattivo crit/break. **Identità Renamon = tempo, non break-payoff.**
- Rationale: introdurrebbe un asse meccanico secondario (break reattivo) che diluisce la focalizzazione time-manip; sovrapposto a Dorumon hp-threshold trigger (lane già differenziata in §1).
- Conferma definitiva: nessuna FoxDrive mechanic nel kit. Riaprire solo se time-manip si dimostra sotto-power a playtest.

## §8b — Domande aperte (defer playtest/M017+)

- **D3 — Ult charge 120 vs 100 standard.** Trigger più alto giustificato dalla potenza AoE+delay+Blessed combinati? Validare a playtest (defer post-M015).
- **D5 — VFX Tōhakken ult (rev2).** Canon = blue-fire hand ignition. Reskin Holy = `holy_fire_ignite` (gold/azure) su hand → `gold_shockwave` radial expand → `phantom_paw_burst` per ogni enemy hit + `blessed_motes` rise su allies. Vendi AoE pur con anim single-strike claw. Validare a M017+ VFX pass.
