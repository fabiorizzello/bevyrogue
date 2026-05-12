# Gabumon — Identity & Kit

> **Scope.** Identity sheet allineata a §8 roster minimal + Twin Core differenziato vs Agumon. Stesso pattern di `agumon/00_identity.md`.

## §0 — Riferimenti

- **Atlas:** `assets/digimon/gabumon_atlas.json` v1, 81 frames, frame size 747×706
- **Canon Digimon:** Gabumon, Child, Reptile/Beast, Data+Vaccine, fields: Nature Spirits / Wind Guardians / Virus Busters / Metal Empire
- **§8 roster minimal** § Gabumon — Ice bulwark
- **Twin Core:** passive bidirezionale con Agumon (Fire ↔ Ice cross-status)

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
| **Basic** (`horn_strike`) | `attack` | 0–8 | 9 |
| **Skill** (`bubble_blast`) | `heavy_attack` | 27–37 | 11 |
| **Ultimate** (`arctic_torrent`) | `skill` | 50–63 | 14 |
| **Hurt** | `hurt` | 38–43 | 6 |
| **Block** | `block` | 9–13 | 5 |
| **Death** | `death` | 14–26 | 13 |
| **Victory** | `victory` | 64–80 | 17 |

Frame budget FSM (basic + skill + ult): 9 + 11 + 14 = **34 frames** (≈2.8s @12fps).

## §3 — Timing convention

Shared con Agumon (§2.2b §G): frame counter logico autoritativo, ms metadata, reference 12fps.

## §4 — Kit shape

| Slot | Skill ID | Target | Costo | Effetto base |
|---|---|---|---|---|
| Basic | `horn_strike` | Single | 0 SP, +1 gen, +25 Ult | Damage piatto Ice; **+1 Chilled** primary |
| Skill | `bubble_blast` | Single | **1 SP** | Damage medio Ice; **+2 Chilled**; ToughnessHit(8). **Modifier `OnStatusApplied→Echo(Chilled)`** sull'adj lowest-HP |
| Ult | `arctic_torrent` | Single | UltCharge | Damage massivo Ice; +Slowed 2 turni; **OnHit→DR 30% self 1 turno** |
| Passive | `fur_cloak` | listener | — | On `EmitStatus(Chilled)` da Gabumon → DR 20% self 1 turno |
| (Twin Core) | `twin_core_ice` | listener cross | — | Damage Ice +X% se target ha `Heated` (Agumon) |

**Sinergie Twin Core:** Agumon stacca Heated, Gabumon stacca Chilled. Quando entrambi presenti, ogni status applicato da uno **aumenta il damage dell'altro** sul medesimo target → loop `apply A → buff B → apply B → buff A`. Sinergico, non equivalente.

## §5 — Chilled (mechanic, condiviso con Twin Core)

- **Apply:** Basic +1, Skill +2 + echo, Ult Slowed indipendente.
- **Cap:** TBD (proposta: 6 stacks, allineato a Heated).
- **Effect on target:** +X% damage taken da Ice per stack; a soglia ≥3 sblocca `Slowed` (gate Skill).
- **Echo (Skill modifier):** quando Chilled è applicato, il blueprint emette signal `chilled_echo` → +1 Chilled sull'adj con HP% più basso. Non ricorsivo (no chain echo).
- **Twin Core hook:** la passive di Agumon legge `KernelEvent::StatusApplied(Chilled)` → +damage Fire condizionale.

## §6 — Domande aperte (raccolte in file futuri)

- DR stack rules: `fur_cloak` DR si rinnova o si sovrappone con DR Ult?
- Echo target tie-break: se 2 adj hanno stesso HP%, regola deterministica?
- Twin Core: bonus simmetrico o asimmetrico? (proposta: simmetrico, +10% damage per status presente)
