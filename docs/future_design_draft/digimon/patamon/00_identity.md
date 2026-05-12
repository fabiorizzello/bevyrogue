# Patamon — Identity & Kit

> **Scope.** Identity sheet allineata a §8 roster minimal. Stesso pattern di `agumon/00_identity.md`.

## §0 — Riferimenti

- **Atlas:** `assets/digimon/patamon_atlas.json` v1, 92 frames, frame size 618×732
- **Canon Digimon:** Patamon, Child, Mammal (Holy), Vaccine, fields: Virus Busters / Wind Guardians
- **§8 roster minimal** § Patamon — Pure healer

## §1 — Identità

Manutentore puro. Heal + cleanse + DR team. **Nessun modifier reattivo**: la semplicità è la firma. È l'unico Digimon senza modifier-firma; identità = "support affidabile". Universale in ogni team con ≥1 spender heavy o team aggressivi.

- **Asse primario:** Sustain (heal/cleanse/DR)
- **Asse secondario:** Damage Holy basso, presente solo per non lasciarlo "skip turn" senza basic
- **Vita:** media (HP ~95), squishy ma DR-team copre
- **Stat baseline (proposta):** `hp_max=95`, `speed=105`, `toughness_max=40`, `weakness=Dark`, `ultimate_trigger=100`, `ultimate_cap=150`, `ultimate_charge_per_event=25` (heal counts as event)

## §2 — Atlas mapping

| Slot kit | Atlas clip | Range | Count |
|---|---|---|---|
| **Idle (loop)** | `idle` | 50–61 | 12 |
| **Basic** (`boom_bubble`) | `attack` | 0–8 | 9 |
| **Skill** (`holy_breeze`) | `heavy_attack` | 30–43 | 14 |
| **Ultimate** (`celestial_light`) | `skill` | 62–76 | 15 |
| **Hurt** | `hurt` | 44–49 | 6 |
| **Block** | `block` | 9–14 | 6 |
| **Death** | `death` | 15–29 | 15 |
| **Victory** | `victory` | 77–91 | 15 |

Frame budget FSM: 9 + 14 + 15 = **38 frames** (≈3.2s @12fps).

## §3 — Timing convention

Shared. Healing FSM nodes hanno target shape `Single` (Skill) / `AoE(All)` (Ult) sugli alleati: kernel resta autorità sull'apply, FSM sequenzia VFX.

## §4 — Kit shape

| Slot | Skill ID | Target | Costo | Effetto base |
|---|---|---|---|---|
| Basic | `boom_bubble` | Single enemy | 0 SP, +1 gen, +25 Ult | Damage piatto Holy (basso, `~6`) |
| Skill | `holy_breeze` | Single ally | **1 SP** | Heal `~25%` HP max + **Cleanse 1 status** (debuff random oldest-first) |
| Ult | `celestial_light` | AoE(All) ally | UltCharge | Heal `~35%` team intero + Cleanse 1 status a tutti |
| Passive | `holy_aegis` | listener | — | Tutti gli alleati: **-10% damage taken** finché Patamon vive |

**Sinergie:** universale. Critico in team con Dorumon (fragile) e Agumon/Renamon (squishy). Aegis DR si stack-additivo (non moltiplica) con Gabumon `fur_cloak` per non sfondare il bilanciamento.

## §5 — Cleanse rules

- **Targets:** rimuove 1 debuff dal target. Priorità: status più "vecchio" (FIFO sui timer), tie-break ID alfabetico.
- **Filter:** cleanse rimuove SOLO debuff (Heated, Chilled, Confused, Paralyzed, Slowed). Mai buff alleati né `Holy` stack (vedi sotto).
- **Vocabolario:** nessuna estensione status set per ora. Cleanse usa toggle binario esistente.

## §6 — Holy element (nota cross-roster)

Patamon e Renamon (post-revisione) sono entrambi Holy. Differenziazione:
- **Patamon:** Holy = heal/buff vector. Damage Holy basso e marginale.
- **Renamon:** Holy = AoE damage vector + time-manip.

Niente competizione meccanica diretta: Patamon agisce sugli alleati, Renamon sui nemici. Sinergia: il `Blessed` di Renamon (vedi `renamon/00_identity.md`) **non interferisce** con cleanse Patamon (Blessed è buff, non debuff).

## §7 — Domande aperte

- Heal scaling: % HP max target o flat? (proposta: % max, evita inutility su tank vs squishy)
- Aegis DR si applica anche a Patamon stesso? (proposta: sì, fonte unica di mitigation self)
- Boom Bubble damage Holy contribuisce a status `Blessed` di Renamon? (proposta: no — separato)
