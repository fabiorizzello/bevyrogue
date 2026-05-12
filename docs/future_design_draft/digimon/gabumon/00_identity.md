# Gabumon — Identity & Kit

> **Scope.** Identity sheet allineata a §8 roster minimal + Twin Core differenziato vs Agumon. Stesso pattern di `agumon/00_identity.md`.

## §0 — Riferimenti

- **Atlas:** `assets/digimon/gabumon_atlas.json` v1, 81 frames, frame size 747×706
- **Canon Digimon:** Gabumon, Child, Reptile/Beast, Data+Vaccine, fields: Nature Spirits / Wind Guardians / Virus Busters / Metal Empire
- **Canon skill names (dataset DAPI):** signature = Petit Fire (62). Altre: Little Horn (63), Tsuno Kougeki/Horn Attack (65/71), Crush Nail (69), Gabumon Shot (72), Blue Cyclone (76), Claw Attack (77), Upperclaw (78).
- **§8 roster minimal** § Gabumon — Ice bulwark
- **Twin Core:** passive bidirezionale con Agumon (Fire ↔ Ice cross-status)

### Naming policy (round canon-align 2026-05-12)

Canon Gabumon **non ha mosse ice** → reflavor designer-fiction Ice mantenuto, ma **rename ID skill** per match canon names dove semantica clip lo permette:

| Old (designer) | New (canon) | Rationale |
|---|---|---|
| `horn_strike` → `horn_attack` | **`claw_attack`** | canon id 77 "Attacks with its claws". Atlas clip `attack` mostra claw motion (non horn) → anim-canon coerente. Element-neutral. |
| `bubble_blast` | **`gabumon_shot`** | canon id 72 "small blast from mouth", element-neutral → ice reflavor OK |
| `arctic_torrent` | **`blue_cyclone`** | canon id 76 "spins spitting blue fire" → blue=cold reflavor, anim 14f rotation match |
| `fur_cloak` | `fur_cloak` | già canon-lore (indossa pelliccia Garurumon) |

**Effetti meccanici invariati.** Solo ID change + lore flavor. Element Ice + Chilled status restano per Twin Core symmetry con Agumon Heated.

## §1 — Identità

Erosore lento + scudo team. **Chilled stacks** sul nemico, eco sull'adiacente. Sé stesso DR-buffato quando applica status. **Differenziazione vs Agumon:** Agumon esplode al kill (burst, OnKill modifier); Gabumon **eroderà sostenendo** (apply ripetuto, OnStatusApplied echo, DR self). Stesso loop status-stack ma payoff opposto: Agu = detonate; Gabu = persistenza + diffusione laterale.

- **Asse primario:** Sustain DPS Ice, status spread laterale
- **Asse secondario:** Tank-lite (DR self on apply)
- **Vita:** alta (HP ~110), bulwark voluto
- **Stat baseline (proposta):** `hp_max=110`, `speed=92`, `toughness_max=60`, `weakness=Fire`, `ultimate_trigger=100`, `ultimate_cap=150`, `ultimate_charge_per_event=25`

## §2 — Atlas mapping

| Slot kit | Atlas clip | Range | Count |
|---|---|---|---|
| **Idle (loop)** | `idle` | 44–49 | 6 |
| **Basic** (`claw_attack`) | `attack` | 0–8 | 9 |
| **Skill** (`gabumon_shot`) | `heavy_attack` | 27–37 | 11 |
| **Ultimate** (`blue_cyclone`) | `skill` | 50–63 | 14 |
| **Hurt** | `hurt` | 38–43 | 6 |
| **Block / Passive `fur_cloak` proc** | `block` | 9–13 | 5 |
| **Death** | `death` | 14–26 | 13 |
| **Victory** | `victory` | 64–80 | 17 |

Frame budget FSM (basic + skill + ult): 9 + 11 + 14 = **34 frames** (≈2.8s @12fps).

## §3 — Timing convention

Shared con Agumon (§2.2b §G): frame counter logico autoritativo, ms metadata, reference 12fps.

## §4 — Kit shape

| Slot | Skill ID | Target | Costo | Effetto base |
|---|---|---|---|---|
| Basic | `claw_attack` | Single | 0 SP, +1 gen, +25 Ult | Damage piatto Ice; **+1 Chilled** primary |
| Skill | `gabumon_shot` | Single | **1 SP** | Damage medio Ice; **+2 Chilled**; ToughnessHit(8). **Modifier `OnStatusApplied→Echo(Chilled)`** sull'adj lowest-HP |
| Ult | `blue_cyclone` | Single | UltCharge | Damage massivo Ice; +Slowed 2 turni; **OnHit→DR 30% self 1 turno** |
| Passive | `fur_cloak` | listener | — | On `EmitStatus(Chilled)` da Gabumon → DR 20% self 1 turno |
| (Twin Core) | `twin_core_ice` | listener cross | — | Damage Ice +X% se target ha `Heated` (Agumon) |

**Sinergie Twin Core:** Agumon stacca Heated, Gabumon stacca Chilled. Quando entrambi presenti, ogni status applicato da uno **aumenta il damage dell'altro** sul medesimo target → loop `apply A → buff B → apply B → buff A`. Sinergico, non equivalente.

## §5 — Chilled (mechanic, condiviso con Twin Core)

- **Apply:** Basic +1, Skill +2 + echo, Ult Slowed indipendente.
- **Cap:** TBD (proposta: 6 stacks, allineato a Heated).
- **Effect on target:** +X% damage taken da Ice per stack; a soglia ≥3 sblocca `Slowed` (gate Skill).
- **Echo (Skill modifier):** quando Chilled è applicato, il blueprint emette signal `chilled_echo` → +1 Chilled sull'adj con HP% più basso. Non ricorsivo (no chain echo).
- **Twin Core hook:** la passive di Agumon legge `KernelEvent::StatusApplied(Chilled)` → +damage Fire condizionale.

## §6 — Domande chiuse (round 2026-05-12)

- **D1 — DR stack `fur_cloak` (20%) vs Ult `blue_cyclone` (30%).** **Risolto: replace-max.** Se entrambi attivi, il DR effettivo è `max(0.20, 0.30) = 0.30`. Niente additivo, niente moltiplicativo. Allineato a `gabumon/03 §5.3`. La durata di ciascun buff resta indipendente: quando il maggiore decade, il minore (se ancora attivo) riprende. Single-instance per buff_id, refresh durata su re-apply.
- **D2 — Echo target tie-break (Skill `gabumon_shot`).** **Risolto: slot index ascending (deterministic).** Se ≥2 adiacenti hanno lo stesso `hp_pct`, il selector `AdjLowest` sceglie quello con `slot_index` più basso. No RNG. Coerente con politica determinismo headless (`CLAUDE.md` §Convenzioni).
- **D3 — Twin Core: bonus simmetrico o asimmetrico?** **Risolto: simmetrico, +10% damage per status partner presente.** Quando Gabumon colpisce target con `Heated` attivo (da Agumon), damage Ice `×1.10`. Specchio per Agumon su `Chilled`. **Stack:** se entrambi gli status sono presenti sul target (rara ma possibile in combat con Agumon+Gabumon attivi), ogni unit legge solo il proprio status partner — niente cross-stack. **Twin Core ↔ Metal Cannon (Dorumon Dark):** trasparente. Dark non scala su Heated/Chilled, quindi Dorumon non beneficia di Twin Core. Sinergia Dorumon resta HP-threshold based (`dorumon/02 §5` D3). **Inconsistency interna risolta:** `gabumon/04 §6 N3` riportava +15%; identity prevale → **+10%** canon.
