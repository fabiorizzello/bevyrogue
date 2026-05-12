# Dorumon — Identity & Kit

> **Scope.** Identity sheet allineata a §8 roster minimal. Stesso pattern di `agumon/00_identity.md`.

## §0 — Riferimenti

- **Atlas:** `assets/digimon/dorumon_atlas.json` v1, 81 frames, frame size 843×661
- **Canon Digimon:** Dorumon, Child, Beast, Data, fields: Unknown / Nightmare Soldiers (X-Antibody lineage)
- **§8 roster minimal** § Dorumon — Executor
- **Predator Loop:** passive esistente, blueprint già in `src/combat/blueprints/dorumon.rs`

## §1 — Identità

Mietitore single-target dark. Apre **Predator window** sui nemici bassi-HP, e dentro la window i suoi finisher chainano. **Differenziazione vs Renamon (post-revisione):** Renamon è AoE holy/time-manip non crit-based; Dorumon è **single-target threshold execute con chain**. Niente sovrapposizione: lane diverse (sweep vs picker), elementi opposti (Holy vs Dark), trigger opposti (status-density vs HP-threshold).

- **Asse primario:** Burst single-target Dark, threshold execute
- **Asse secondario:** Chain on kill dentro Predator state
- **Vita:** medio-bassa (HP ~90), fragile per design
- **Stat baseline (proposta):** `hp_max=90`, `speed=110`, `toughness_max=45`, `weakness=Light`, `ultimate_trigger=100`, `ultimate_cap=150`, `ultimate_charge_per_event=25`

## §2 — Atlas mapping

| Slot kit | Atlas clip | Range | Count |
|---|---|---|---|
| **Idle (loop)** | `idle` | 47–58 | 12 |
| **Basic** (`bite`) | `attack` | 0–8 | 9 |
| **Skill** (`draconic_edge`) | `heavy_attack` | 31–39 | 9 |
| **Ultimate** (`heat_viper`) | `skill` | 59–68 | 10 |
| **Hurt** | `hurt` | 40–46 | 7 |
| **Block** | `block` | 9–13 | 5 |
| **Death** | `death` | 14–30 | 17 |
| **Victory** | `victory` | 69–80 | 12 |

Frame budget FSM: 9 + 9 + 10 = **28 frames** (≈2.3s @12fps). Più snappy di Agumon per fit "executor veloce".

## §3 — Timing convention

Shared. 12fps reference, frame logico autoritativo.

## §4 — Kit shape

| Slot | Skill ID | Target | Costo | Effetto base |
|---|---|---|---|---|
| Basic | `bite` | Single | 0 SP, +1 gen, +25 Ult | Damage piatto Dark |
| Skill | `draconic_edge` | Single | **1 SP** | Damage scaling: ×2 se primary HP <50%, base altrimenti. **Modifier `OnKill→Chain` armato solo in Predator state** (max 1 chain) |
| Ult | `heat_viper` | Single | UltCharge | Damage massivo Dark; bonus +50% se primary <30%; **forza Predator state on hit** |
| Passive | `predator_loop` | listener | — | Esistente: tracking target lowest-HP, attiva Predator state per N turni |

**Sinergie:** Renamon spreada status AoE → Dorumon entra a finire i bassi-HP. Patamon heal lo tiene vivo. Tentomon (battery+tank) lo nutre di SP.

## §5 — Predator state (mechanic, già implementato)

Riferimento: `src/combat/blueprints/dorumon.rs`, `PredatorLoopState`, `PredatorLoopResolved` event.

- **Entry:** quando un nemico cade sotto HP threshold (X%, già configurato).
- **Effect:** dentro Predator state, `draconic_edge` arma `OnKill→Chain`; Ult bonus threshold più aggressivo.
- **Exit:** target tracked muore (chain consumato) o timeout turni.

## §6 — Domande aperte

- Chain target selection: lowest-HP residuo, o stesso ranged group?
- Predator state visibile in UI? (HSR-style debuff badge sul nemico tracked)
- Heat Viper interaction con Twin Core / status altrui (Heated/Chilled/Confused/Holy): bonus o trasparente?
