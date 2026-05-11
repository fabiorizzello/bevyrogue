# §8 — Roster minimal (canon)

> **Sostituisce** i precedenti `08_skill_designs.md` / `10_full_kit_plan.md` / `11_roster_design_v2.md` (esplorazioni archiviate). Questo è il design **all'osso** validato il 2026-05-11: 6 Rookie, kit identico per slot, identità nei dettagli. Skill-tree, varianti, AoE estesi, status set extension, granted abilities, form: tutti **fuori scope**.

## §8.0 — Costanti del roster

- **Roster size:** 6 Rookie unici.
- **Combat shape:** turn-based line, flat-line (no front/back), HSR-style.
- **Atlas clip pool (uguale per tutti):** `attack`, `block`, `death`, `heavy_attack`, `hurt`, `idle`, `skill`, `victory`.
- **Mapping clip → slot kit (fisso):** `attack` → Basic · `heavy_attack` → Skill · `skill` → Ultimate.
- **Kit shape (uniforme, 4 slot):**

  | Slot | Clip | Costo | SP gen | Note |
  |---|---|---|---|---|
  | Basic | `attack` | 0 SP | +1 | Tap quotidiano, sempre legale |
  | Skill | `heavy_attack` | 3 SP (4 per heavy hitters) | 0 | Identità meccanica |
  | Ultimate | `skill` | charge-gated (no SP) | 0 | Climax, gating su `UltCharge` |
  | Passive | — | — | — | Tratto persistente (anche assente, vedi Renamon v0) |

- **Architettura skill (vincolo §2.2b):** ogni active = **AnimGraph FSM 3-nodi** (Windup → Strike → Recovery), con **nodo opzionale Reactive** se la skill ha modifier-firma. Il kernel resta autorità sull'apply; la FSM sequenzia intent.

## §8.1 — Modifier-firma vocabolario (chiuso v0)

Solo i modifier necessari ai 6 baseline. Vocabolario completo §2.2b deferred.

| Modifier | Trigger kernel | Effetto |
|---|---|---|
| `OnKill→Detonate(status)` | `KernelEvent::UnitDied` su Strike target | Spread dello status sui 2 adiacenti |
| `OnStatusApplied→Echo(status)` | `KernelEvent::StatusApplied` sul target | Re-applica status sull'adiacente più debole |
| `OnKill→Chain` | `KernelEvent::UnitDied` su Strike target (in stato) | Strike ripete su nuovo target (one extra) |
| `OnBreak→Detonate` | `KernelEvent::ToughnessBroken` su Strike target | AoE secondaria, scala su damage accumulato |
| `OnHitN→Apply(status)` | `KernelEvent::DamageDealt` al `N`-esimo hit | Apply status all'ultimo hit della sequenza |

5 modifier. Niente catalogo da 7+. Espansione (Splash, Escalate, ShapeOverride) deferred ad altra milestone.

## §8.2 — Target shape vocabolario (chiuso v0)

| Shape | Definizione |
|---|---|
| `Single` | 1 target preciso |
| `Blast` | target + 2 adiacenti (full damage primary, ridotto adj) |
| `AoE(All)` | tutta la linea, danno full |
| `Bounce(N)` | N hit sequenziali, ogni hit pesca un target diverso |

`Adjacent` shape statico non usato in v0 — gli "adiacenti" arrivano solo via modifier reattivi.

## §8.3 — Roster (6 schede)

### Agumon — Fire burst

- **Identità:** swing pesante a fuoco. Vive di Heated stacks, esplode al kill.
- **Sinergie:** Twin Core con Gabumon (passive bidirezionale già implementata). Tentomon → SP feeder.

| Slot | Skill ID | Target | Costo | Effetto |
|---|---|---|---|---|
| Basic | `claw_strike` | `Single` | 0 SP, +1 gen | Damage piatto, +1 Heated stack al primary |
| Skill | `pepper_breath` | `Single` | 3 SP | Damage medio, +2 Heated, +1 toughness reduce |
| Ultimate | `nova_blast` | `Blast` | UltCharge | Damage alto sul primary, splash adj. **Modifier-firma: `OnKill→Detonate(Heated)`** — se uccide il primary, Heated stacks rimanenti spread sui 2 adiacenti |
| Passive | `twin_core_fire` | — | — | Esistente (vedi `combat_current.md`). +damage se Gabumon in team applica Chilled |

**AnimGraph nota:** `nova_blast` ha 4 nodi (Windup → Strike → ReactiveDetonate → Recovery). Edge `Strike→ReactiveDetonate` su `KernelEvent::UnitDied(primary)`; fallback `TimeInNode→Recovery`.

---

### Gabumon — Ice bulwark

- **Identità:** erosore lento + scudo team. Chilled stacka, l'eco diffonde.
- **Sinergie:** Twin Core con Agumon. Pelliccia frusta = mitigation passiva per chi gli sta accanto in formazione (futuro: niente formation ora).

| Slot | Skill ID | Target | Costo | Effetto |
|---|---|---|---|---|
| Basic | `horn_strike` | `Single` | 0 SP, +1 gen | Damage piatto, +1 Chilled |
| Skill | `bubble_blast` | `Single` | 3 SP | Damage medio, +2 Chilled (Slowed 1 turno se Chilled ≥3). **Modifier-firma: `OnStatusApplied→Echo(Chilled)`** — il `Chilled` applicato eco sull'adiacente lowest-HP |
| Ultimate | `arctic_torrent` | `Single` | UltCharge | Damage massivo singolo, +Slowed 2 turni |
| Passive | `fur_cloak` | — | — | Quando applica Chilled, +1 turno di DR 20% su sé stesso (mitigation tank-lite) |

---

### Dorumon — Executor

- **Identità:** mietitore. Apre window con Predator Loop esistente, finisce chi sta morendo.
- **Sinergie:** Renamon → status setter → Dorumon capitalizza. Patamon → heal (Dorumon è fragile).

| Slot | Skill ID | Target | Costo | Effetto |
|---|---|---|---|---|
| Basic | `bite` | `Single` | 0 SP, +1 gen | Damage piatto |
| Skill | `draconic_edge` | `Single` | 3 SP | Damage alto se primary < 50% HP, normale altrimenti. **Modifier-firma in Predator state: `OnKill→Chain`** — se uccide, Strike ripete su nuovo target (max 1 chain) |
| Ultimate | `heat_viper` | `Single` | UltCharge | Damage massivo executor (bonus se primary < 30%) |
| Passive | `predator_loop` | — | — | Esistente (vedi `combat_current.md`) |

**AnimGraph nota:** `draconic_edge` ha edge `Strike→ChainStrike` gated su `Predicate::And(KernelEvent::UnitDied, PassiveActive::PredatorLoop)`. Fuori Predator state, `OnKill→Chain` non si arma.

---

### Patamon — Pure healer

- **Identità:** manutentore. Heal + cleanse base + buff piccolo. Nessun modifier reattivo, è semplicità voluta.
- **Sinergie:** universale (cura chi serve). Critico in team aggressivi.

| Slot | Skill ID | Target | Costo | Effetto |
|---|---|---|---|---|
| Basic | `boom_bubble` | `Single` enemy | 0 SP, +1 gen | Damage piatto (basso) |
| Skill | `holy_breeze` | `Single` ally | 3 SP | Heal alleato target, +Cleanse 1 status |
| Ultimate | `celestial_light` | `AoE(All)` ally | UltCharge | Heal team intero (full row) |
| Passive | `holy_aegis` | — | — | Tutti gli alleati: -10% damage taken finché Patamon vive |

**Note:** Patamon è l'unico digimon **senza modifier-firma**. Identità = "support puro affidabile", non "trickster reattivo".

---

### Renamon — AoE caster

- **Identità:** sweep AoE + capitalizer sullo stato del nemico. L'Ult scala con la quantità di status sul target.
- **Sinergie:** Renamon → Dorumon (set status, Dorumon esegue). Renamon + Tentomon = wave clear.

| Slot | Skill ID | Target | Costo | Effetto |
|---|---|---|---|---|
| Basic | `quick_strike` | `Single` | 0 SP, +1 gen | Damage piatto |
| Skill | `diamond_storm` | `AoE(All)` | 4 SP | Damage medio su tutti, +1 Confused random |
| Ultimate | `fox_drive` | `Single` | UltCharge | Damage scala lineare con N° status diversi sul primary. **Modifier-firma: `OnBreak→Detonate`** — se rompe la toughness, AoE secondaria 50% damage |
| Passive | — | — | — | (assente in v0) |

**Note:** Skill costa 4 SP (non 3) perché `AoE(All)` con status apply è high-value. Calibrazione bilanciamento da fissare in playtest.

---

### Tentomon — SP battery

- **Identità:** alimentatore della squadra. Bounce per spreadare paralisi.
- **Sinergie:** universale (alimenta Agumon/Gabumon/Dorumon/Renamon). Battery Loop esistente.

| Slot | Skill ID | Target | Costo | Effetto |
|---|---|---|---|---|
| Basic | `petit_thunder` | `Single` | 0 SP, +2 gen | Damage piatto, **+2 SP** invece di +1 (battery role) |
| Skill | `electro_shocker` | `Bounce(3)` | 3 SP | Damage medio su 3 target. **Modifier-firma: `OnHitN(3)→Apply(Paralyzed)`** — al 3° hit, Paralyzed sul target finale |
| Ultimate | `super_shocker` | `AoE(All)` | UltCharge | Damage medio su tutta la linea, +1 Paralyzed su 1 random |
| Passive | `battery_loop` | — | — | Esistente (vedi `combat_current.md`) |

---

## §8.4 — Sinergie team (graph minimale)

```
                  Tentomon ─── SP feed ───┬──▶ Agumon ◀── Twin Core ──▶ Gabumon
                                          │
                                          ├──▶ Renamon ── status setter ──▶ Dorumon (capitalize/execute)
                                          │                                     ▲
                                          └──▶ Dorumon                          │
                                                                                │
                  Patamon ── heal/cleanse/DR ── tutti ─────────────────────────┘
```

Hub: **Tentomon** (SP), **Patamon** (sustain). Coppia: **Agumon↔Gabumon** (Twin Core). Asse: **Renamon→Dorumon** (setup → execute).

## §8.5 — Coverage check

| Asse | Copertura | Note |
|---|---|---|
| Burst single-target | Agumon, Dorumon | Due profili: burst-prep (Agu) vs executor-threshold (Doru) |
| Sustain DPS | Gabumon, Renamon | Erosione (Gabu) vs scaling-status (Reno) |
| AoE | Renamon (`AoE(All)`), Tentomon (`Bounce`+`AoE(All)` ult), Agumon (`Blast` ult) | Wave clear copertura ok |
| Sustain/heal | Patamon | Unico, niente backup |
| SP economy | Tentomon battery, Patamon basso uso | Tentomon indispensabile in team con ≥2 spender |
| Tank-lite | Gabumon (Fur Cloak DR), Patamon (Aegis DR team) | Niente tank dedicato. Mitigation distribuita |
| Status apply | Agumon (Heated), Gabumon (Chilled/Slowed), Renamon (Confused), Tentomon (Paralyzed) | 4 axes attive |
| Status capitalize | Renamon (Fox Drive scale-w-status), Dorumon (threshold HP) | 2 lane di payoff |

## §8.6 — Scope architetturale (cosa serve per implementare)

1. **`clip.ron`** per ogni Digimon (lossless dal `_atlas.json`). § 2.2 invariato.
2. **`animation_fsm.ron`** per ogni Digimon: 3 active × FSM 3 o 4 nodi (4 se ha modifier-firma). § 2.2b.
3. **`skills.ron`** entries per le 18 skill (3 per digimon). Numeri base, niente logica condizionale.
4. **Blueprint listener** per ogni Digimon: minimal (Patamon = nessun listener). Twin Core/Predator Loop/Battery Loop riusano il blueprint esistente.
5. **Kernel events necessari:** `UnitDied`, `StatusApplied`, `ToughnessBroken`, `DamageDealt` — tutti già emessi.

## §8.7 — Fuori scope di questa fase (per chiarezza)

- Skill tree, varianti skill, unlock nodes
- Catalogo modifier completo (7+) — solo 5 modifier listati in §8.1
- Status set extension (Stealth, Cleanse altri, Frostbite, ecc) — Cleanse di Patamon usa il toggle binario esistente
- AoE shape extra (Adjacent statico, Bounce parametrico, ShapeOverride conditional)
- Passive Renamon (deferred)
- Multiple actives heterogeneous (es. 4 skill Patamon) — kit shape uniforme
- Form/Digivolution, Champion stage
- Tank dedicato (niche distribuito su Gabumon Fur Cloak + Patamon Aegis)
