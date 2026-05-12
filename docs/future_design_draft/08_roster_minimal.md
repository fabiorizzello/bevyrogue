# §8 — Roster minimal (canon)

> **Sostituisce** i precedenti `08_skill_designs.md` / `10_full_kit_plan.md` / `11_roster_design_v2.md` (esplorazioni archiviate). Design **all'osso** validato il 2026-05-11, **revisionato 2026-05-12** su Renamon e Tentomon. 6 Rookie, kit identico per slot, identità nei dettagli. Skill-tree, varianti, AoE estesi, status set extension, granted abilities, form: tutti **fuori scope**.
>
> **Stat baselines** (HP/speed/toughness/weakness/ult) vivono nelle identity sheet per Digimon in `docs/future_design_draft/digimon/<name>/00_identity.md`. Qui solo kit shape.
>
> **Revisione 2026-05-12:**
> - **Renamon**: da "AoE caster Confused/crit" → **Holy + Time Manipulation** (AdvanceTurn/DelayTurn, buff `Blessed`). No crit, no `OnBreak→Detonate`. Skill 1 SP. Aggiunto passive `kitsune_grace`.
> - **Tentomon**: da "SP battery puro" → **battery + tank-lite** (HP alto, DR su Skill, block reaction +20%). Skill 1 SP. Battery resta primaria.
> - **Motivazione**: differenziare Renamon da Dorumon (lane opposte) e coprire asse tank senza 7° unità.

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

- **Architettura skill (vincolo §2.2b):** ogni active = **AnimGraph FSM 3-nodi** (Windup → Strike → Recovery), con **nodo opzionale Reactive** se la skill ha reactive signature. Il kernel resta autorità sull'apply; la FSM sequenzia intent.

## §8.1 — Reactive signature vocabolario (chiuso v0)

Solo le reactive signature necessarie ai 6 baseline. **Mapping FSM canon: vedi `02-02b §C4`** (round-3, 2026-05-12, X5 — ogni reactive signature è shorthand per pattern FSM edge + Command, NON un Command runtime).

> **Glossario "modifier" (X16, 2026-05-12).** Il termine "modifier" nei doc copre tre sensi distinti, **non intercambiabili**: (1) **playhead modifier** (`Hold`, `SpeedMul`, `Loop`) — opera sull'animazione, `02-02 §A` / `02-02b §A`; (2) **damage/stat modifier** (`AttributeSet`, multiplier Twin Core ×1.15) — opera sulla damage pipeline, `02-02b §C` / `agumon/04`; (3) **reactive signature** — pattern reattivo design-side (es. `OnKill→Detonate`), shorthand per FSM edge + Command, **mai un Command runtime**. La canon è: usare "reactive signature" per il senso (3); "modifier" solo per (1) e (2). Mai "modifier" da solo per indicare la firma reattiva.

| Reactive signature | Trigger kernel | Effetto | FSM mapping |
|---|---|---|---|
| `OnKill→Detonate(status)` | `KernelEvent::UnitDied` su Strike target | Spread dello status sui 2 adiacenti | `02-02b §C4` riga 1 |
| `OnStatusApplied→Echo(status)` | `KernelEvent::StatusApplied` sul target | Re-applica status sull'adiacente più debole | `02-02b §C4` riga 2 |
| `OnKill→Chain` | `KernelEvent::UnitDied` su Strike target (in stato) | Strike ripete su nuovo target (one extra) | `02-02b §C4` riga 3 |
| `OnHitN→Apply(status)` | `KernelEvent::DamageDealt` al `N`-esimo hit | Apply status all'ultimo hit della sequenza | `02-02b §C4` riga 4 |

4 modifier attivi in v0. `OnBreak→Detonate` rimosso (Renamon revisione 2026-05-12). Espansione (Splash, Escalate, ShapeOverride) deferred.

## §8.2 — Target shape vocabolario (chiuso v0)

| Shape | Definizione |
|---|---|
| `Single` | 1 target preciso |
| `Blast` | target + 2 adiacenti (full damage primary, ridotto adj) |
| `AoE(All)` | tutta la linea, danno full |
| `Bounce(N)` | N hit sequenziali, ogni hit pesca un target diverso |

`Adjacent` shape statico non usato in v0 — gli "adiacenti" arrivano solo via reactive signature.

## §8.3 — Roster (6 schede)

### Agumon — Fire burst

- **Identità:** swing pesante a fuoco. Vive di Heated stacks, esplode al kill.
- **Sinergie:** Twin Core con Gabumon (passive bidirezionale già implementata). Tentomon → SP feeder.

| Slot | Skill ID | Target | Costo | Effetto |
|---|---|---|---|---|
| Basic | `sharp_claws` | `Single` | 0 SP, +1 gen | Damage piatto, +1 Heated stack al primary |
| Skill | `baby_flame` | `Single` | 3 SP | Damage medio, +2 Heated, +1 toughness reduce |
| Ultimate | `baby_burner` | `Blast` | UltCharge | Damage alto sul primary, splash adj. **Reactive signature: `OnKill→Detonate(Heated)`** — se uccide il primary, Heated stacks rimanenti spread sui 2 adiacenti |
| Passive | `twin_core_fire` | — | — | Esistente (vedi `combat_current.md`). +damage se Gabumon in team applica Chilled |

**AnimGraph nota:** `baby_burner` ha 4 nodi (Windup → Strike → ReactiveDetonate → Recovery). Edge `Strike→ReactiveDetonate` su `KernelEvent::UnitDied(primary)`; fallback `TimeInNode→Recovery`.

---

### Gabumon — Ice bulwark

- **Identità:** erosore lento + scudo team. Chilled stacka, l'eco diffonde.
- **Sinergie:** Twin Core con Agumon. Pelliccia frusta = mitigation passiva per chi gli sta accanto in formazione (futuro: niente formation ora).

| Slot | Skill ID | Target | Costo | Effetto |
|---|---|---|---|---|
| Basic | `claw_attack` | `Single` | 0 SP, +1 gen | Damage piatto, +1 Chilled |
| Skill | `gabumon_shot` | `Single` | 3 SP | Damage medio, +2 Chilled (Slowed 1 turno se Chilled ≥3). **Reactive signature: `OnStatusApplied→Echo(Chilled)`** — il `Chilled` applicato eco sull'adiacente lowest-HP |
| Ultimate | `blue_cyclone` | `Single` | UltCharge | Damage massivo singolo, +Slowed 2 turni |
| Passive | `fur_cloak` | — | — | Quando applica Chilled, +1 turno di DR 20% su sé stesso (mitigation tank-lite) |

---

### Dorumon — Executor

- **Identità:** mietitore. Apre window con Predator Loop esistente, finisce chi sta morendo.
- **Sinergie:** Renamon → status setter → Dorumon capitalizza. Patamon → heal (Dorumon è fragile).

| Slot | Skill ID | Target | Costo | Effetto |
|---|---|---|---|---|
| Basic | `bite` | `Single` | 0 SP, +1 gen | Damage piatto |
| Skill | `dash_metal` | `Single` | 3 SP | Damage alto se primary < 50% HP, normale altrimenti. **Reactive signature in Predator state: `OnKill→Chain`** — se uccide, Strike ripete su nuovo target (max 1 chain) |
| Ultimate | `metal_cannon` | `Single` | UltCharge | Damage massivo executor (bonus se primary < 30%) |
| Passive | `predator_loop` | — | — | Esistente (vedi `combat_current.md`) |

**AnimGraph nota:** `dash_metal` ha edge `Strike→ChainStrike` gated su `Predicate::And(KernelEvent::UnitDied, PassiveActive::PredatorLoop)`. Fuori Predator state, `OnKill→Chain` non si arma.

---

### Patamon — Support-healer con damage burst ult

- **Identità:** manutentore. Heal + cleanse base + DR aura. Ult **hybrid damage+heal** AoE (rev2: canon Sparking Air Shot = "glittering powered-up Air Shot" giustifica dual-axis).
- **Sinergie:** universale (cura chi serve). Critico in team aggressivi. Cross-roster: ult damage scala con `Blessed` Renamon.

| Slot | Skill ID | Canon EN/JP | Target | Costo | Effetto |
|---|---|---|---|---|---|
| Basic | `tai_atari` (Body Blow) | Tai Atari | `Single` enemy | 0 SP, +1 gen | Damage piatto Holy ~6 |
| Skill | `patapata_hover` (canon, reflavor heal) | Patapata Hover | `Single` ally | **1 SP** | Heal `~25% HP max` ally + Cleanse 1 debuff |
| Ultimate | `sparking_air_shot` (canon, hybrid) | Sparking Air Shot | `AoE(All)` enemies + `AoE(All)` ally | UltCharge | **Damage Holy ~25 a tutti i nemici** + **Heal ~20% HP max team** + Cleanse 1/ally |
| Passive | `holy_aegis` | flavor-only | — | — | Tutti gli alleati: -10% damage taken finché Patamon vive |

**Note:** Patamon è l'unico digimon **senza reactive signature reattivo**. Identità = "support affidabile" (rev2: + damage burst ult). Ult hybrid risolve "ult dead" su team full-HP (damage AoE resta valore anche senza heal).

---

### Renamon — Holy time-manip AoE

- **Identità:** sweep AoE Holy + manipolazione del turn order. No crit, no scaling su status nemico. Payoff = **tempo guadagnato** + damage AoE costante. Buff `Blessed` agli alleati.
- **Sinergie:** Renamon ↛ Dorumon (lane separate, complementari non dipendenti). Renamon ↔ Patamon (entrambi Holy: `Blessed` + `holy_aegis`). Renamon ↔ Tentomon (SP feed → AoE spammabile).
- **Vedi:** `digimon/renamon/00_identity.md` per stats e dettaglio time-manip mechanic.

| Slot | Skill ID | Target | Costo | Effetto |
|---|---|---|---|---|
| Basic | `kokaishu` (Fox Spin Kick) | `Single` | 0 SP, +1 gen | Damage piatto Holy |
| Skill | `koyosetsu` (Diamond Storm) | `AoE(All)` enemies | **1 SP** | Damage medio Holy su tutti; **`AdvanceTurn(self, 25%)`** |
| Ultimate | `tohakken` (Power Paw, Holy reskin) | `AoE(All)` enemies | UltCharge | Damage alto Holy a tutti; **`DelayTurn(all enemies, 30%)`**; applica `Blessed` agli alleati per 2 turni |
| Passive | `kitsune_grace` | listener | — | Quando un alleato consuma Ult, **`AdvanceTurn(self, 10%)`** |

**Note:** Skill costa **1 SP** (non 3-4) perché il payoff primario è time-manip, non damage. `Blessed` = `+15% damage dealt`, `+1 Ult charge gen per action`, non-cleansable da nemici.

---

### Tentomon — Battery + tank-lite

- **Identità:** alimentatore SP + bulwark distribuito. HP alto, DR su Skill, block reaction frequente. Bounce per spreadare Paralyzed. Battery resta primaria; tank-lite copre l'asse mancante senza 7° unità.
- **Sinergie:** universale (alimenta Agumon/Gabumon/Dorumon/Renamon). Battery Loop esistente.
- **Vedi:** `digimon/tentomon/00_identity.md` per stats (HP 120, speed 85) e dettaglio block reaction.

| Slot | Skill ID | Target | Costo | Effetto |
|---|---|---|---|---|
| Basic | `hard_claw` | `Single` | 0 SP, **+2 gen** | Damage piatto Electric (claw + static VFX, canon-flavored) |
| Skill | `petit_thunder` | `Bounce(3)` | **1 SP** | Damage medio Electric su 3 target (chain lightning, canon: "static electricity amplified by wings"). **Modifier: `OnHitN(3)→Apply(Paralyzed)`**. **+1 turno DR 25% self** (tank hook) |
| Ultimate | `electrical_discharge` | `AoE(All)` | UltCharge | Damage medio su tutta la linea + Paralyzed su 1 random; **+1 SP team** (battery moment). Canon: "discharges electricity from whole body" |
| Passive | `battery_loop` | listener | — | Esistente: SP gen reattiva. **Override: +20% block reaction chance** (tank-lite) |

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
| Burst single-target | Agumon, Dorumon | Burst-prep (Agu) vs executor-threshold (Doru) |
| Sustain DPS | Gabumon | Erosione Chilled |
| AoE | Renamon (Holy AoE × 2), Tentomon (`Bounce`+`AoE(All)` ult), Agumon (`Blast` ult) | Wave clear ok |
| Sustain/heal | Patamon | Unico, niente backup |
| SP economy | Tentomon battery (+2 gen), Renamon spender (1 SP/skill) | Tentomon indispensabile con ≥2 spender |
| Tank-lite | Tentomon (HP 120 + DR + block +20%), Gabumon (Fur Cloak DR) | Mitigation distribuita su 2 unità. Patamon `holy_aegis` (-10% DR team) **non conta come tank-lite**: è framed come *support mitigation aura* sotto l'asse sustain primario (vedi `patamon/00 §1`), non come contributo all'asse tank. |
| Time manipulation | Renamon (AdvanceTurn/DelayTurn, `Blessed`) | Lane unica, nessun overlap |
| Status apply | Agumon (Heated), Gabumon (Chilled/Slowed), Tentomon (Paralyzed) | 3 axes (Confused rimosso da Renamon) |
| Status capitalize | Dorumon (threshold HP via Predator Loop) | 1 lane (Renamon non capitalizza più) |

## §8.6 — Scope architetturale (cosa serve per implementare)

1. **`clip.ron`** per ogni Digimon (lossless dal `_atlas.json`). § 2.2 invariato.
2. **`animation_fsm.ron`** per ogni Digimon: 3 active × FSM 3 o 4 nodi (4 se ha reactive signature). § 2.2b.
3. **`skills.ron`** entries per le 18 skill (3 per digimon). Numeri base, niente logica condizionale.
4. **Blueprint listener** per ogni Digimon: minimal (Patamon = nessun listener). Twin Core/Predator Loop/Battery Loop riusano il blueprint esistente.
5. **Kernel events necessari:** `UnitDied`, `StatusApplied`, `ToughnessBroken`, `DamageDealt` — tutti già emessi.

## §8.7 — Fuori scope di questa fase (per chiarezza)

- Skill tree, varianti skill, unlock nodes
- Catalogo modifier completo (7+) — solo 5 modifier listati in §8.1
- Status set extension (Stealth, Cleanse altri, Frostbite, ecc) — Cleanse di Patamon usa il toggle binario esistente
- AoE shape extra (Adjacent statico, Bounce parametrico, ShapeOverride conditional)
- Multiple actives heterogeneous (es. 4 skill Patamon) — kit shape uniforme
- Form/Digivolution, Champion stage
- Tank dedicato (niche distribuito su Tentomon + Gabumon; Patamon `holy_aegis` è support mitigation, non tank-lite — vedi §8.5)
- Turn-order UI animato (placeholder per Renamon time-manip, M017 fuori scope)
