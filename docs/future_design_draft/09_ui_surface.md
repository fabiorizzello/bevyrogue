# §9 — UI Surface (allineato §8 roster minimal)

> Pannelli del combat panel `bevy_egui` (feature `windowed`) e contratto headless. La UI legge da `CombatEvent` + `ValidationSnapshot`; **non muta lo stato di gameplay** (`combat_current.md` §Boundaries). Allineato a §8 (kit uniforme Basic/Skill/Ult/Passive, 5 reactive signature chiusi, niente skill-tree, niente QTE in v0).

## §9.1 — Principio guida

**Tutto ciò che modifica il payload di una skill o ne abilita/blocca il cast deve essere visibile prima del cast.** Niente "damage scala su X" se X non è leggibile. Niente cast illegale senza ragione esposta. Il combat è turn-based: il player pianifica 2-3 turni avanti.

## §9.2 — Pannelli persistenti

### §9.2.1 — Unit roster panel (per unit attiva, ally e enemy)

- **Nome + sprite** (clip `idle` dall'atlas).
- **HP**: barra + numero, color shift su low-HP (≤30%).
- **Toughness**: barra se `toughness_max > 0`. A 0 → badge `BROKEN` + freeze frame.
- **SP**: numero (team-shared pool, 1 volta per team).
- **Ultimate charge**: barra dedicata + marker della threshold.
- **Status badges** (icon list su target / caster, vedi §8.1 + §8.3):
  - `Heated`, `Chilled`, `Slowed`, `Paralyzed`, `Blessed` — stack count + durata residua (es. `2t`) + tooltip. `Confused` rimosso (round-3 2026-05-12, dropped da Renamon).
  - Mark blueprint esistenti: `PreyLock(turns)` (Dorumon), `TwinCore` shared meter (Agumon/Gabumon), `StaticCharge` (Tentomon, esistente come passive Battery Loop).
  - Tooltip on hover: nome status, effetto, fonte (chi l'ha applicato), interazione con reactive signature se rilevante (es. "Heated: Baby Burner con kill spread Heated agli adiacenti").

### §9.2.2 — Caster identity panel (per ally selezionato)

Resource per Digimon — solo quelle realmente esistenti in v0:

| Digimon | Resource visualizzata | Stato |
|---|---|---|
| Agumon | Heated stacks sul target (tag su enemy, non self) | letto da `ValidationSnapshot` |
| Gabumon | Chilled stacks sul target + Fur Cloak DR flag self | DR window indicator |
| Patamon | (nessuna resource — support puro) | passive `Holy Aegis` icon team-wide |
| Dorumon | `Predator Loop` state (active/idle) + `PreyLock` su target | flag self + per-target |
| Tentomon | `Battery Loop` state (esistente) | indicatore on/off |
| Renamon | (nessuna resource v0 — niente passive) | — |

Passive icon w/ tooltip se attiva:
- "Twin Core: damage bonus quando il partner applica lo status complementare."
- "Fur Cloak: +DR 20% per 1 turno dopo aver applicato Chilled."
- "Holy Aegis: ally team prendono -10% damage finché Patamon vive."
- "Predator Loop: in stato Predator, Dash Metal usa `OnKill→Chain`."
- "Battery Loop: condizioni esistenti del blueprint."

### §9.2.3 — Turn order timeline (top bar)

- Card orizzontali ordinate per `TurnOrder` priority + speed-stat.
- Ally cards full color; enemy red-tinted.
- Per card: portrait + speed + queued action preview (se telegraphed).
- **Telegraph badge**: enemy con `charged_attack.lead_turns > 0` → big yellow icon + "Xt to {skill name}".

## §9.3 — Pre-cast preview overlay

Quando il player seleziona una skill (prima di confermare il target):

### §9.3.1 — Skill panel

- Skill name + icona + damage tag (Fire/Ice/Dark/Holy/Electric/Physical).
- Cost: SP o "Ult" se ult.
- Target shape (§8.2): `Single` / `Blast` / `AoE(All)` / `Bounce(N)` — disegno minimal sulla linea nemica.
- Description (1-2 frasi) con esplicito reactive signature se presente.

### §9.3.2 — Target selector

- Highlight valid targets per shape.
  - `Single` → 1 target clickable.
  - `Blast` → primary clickable, 2 adiacenti auto-highlight in tinta secondaria.
  - `AoE(All)` → tutta la linea highlighted, primary = quello mostrato per damage projection.
  - `Bounce(N)` → primary clickable, N target highlighted con numerazione hop.
- Per primary highlighted:
  - **Damage projection** (post-mod): "Base 17 (Dark) × 1.12 PredatorLoop = 19".
  - **Status che sarà applicato**: "+ Heated 3t" o "+ Blessed 2t (alleato)".
  - **Toughness delta**: "−12 → BREAK!" se il break proc.
  - **Reactive signature preview** (se armable): "If kill → Detonate Heated to adj" / "If primary breaks → AoE 50%".

### §9.3.3 — Legality reason

Skill non legale → hover mostra reason:
- `"Out of SP (need 3)."`
- `"Target is KO."`
- `"Ultimate not charged."`
- `"Dash Metal: target valido ma OnKill→Chain non si arma fuori Predator state."` (info, non block)

Reason stringa esposta dal `legality()` del trait (in §8 deferred a slice S03b).

## §9.4 — During-cast feedback

### §9.4.1 — Phase indicator (small banner top-center)

Allineato all'AnimGraph FSM 3/4-node (§8.0):
- `Windup` → "Charging..." (durata variabile)
- `Strike` → no banner (è il colpo)
- `ReactiveDetonate/Echo/Chain` → "Reactive: {modifier-name}"
- `Recovery` → no banner

Solo phase con durata >300ms mostrano banner.

### §9.4.2 — Headless equivalence

In `headless` (no `windowed`), tutto il rendering è soppresso. La FSM emette comunque i `Commands` (§2.2b) e i `CombatEvent` per logging/JSONL via `jsonl_logger.rs`.

**Niente QTE in v0**: la FSM minimal non usa `Suspend(QTE)`. Riservato a milestone successiva.

## §9.5 — Post-hit feedback

### §9.5.1 — Damage numbers (floating)

- Numero grande sopra il colpito.
- Color by tag (Fire orange, Ice cyan, Dark purple, Holy yellow, Electric yellow-white, Physical white).
- **Breakdown** opzionale (small under main): "base 17 × 1.12 PredatorLoop = 19".
- **Tag secondari**: `BREAK!` quando rompe toughness; `DETONATE!` / `CHAIN!` / `ECHO!` / `PARALYZE!` quando un reactive signature proc'a.

### §9.5.2 — Status change popup

- "+ Heated 3t" appear-and-fade vicino al target.
- "Chilled echoed → adj" se `OnStatusApplied→Echo` triggera.
- "Heated spread → adj" se `OnKill→Detonate` triggera.

### §9.5.3 — Resource change cue

- SP gained/spent: "+1 SP" / "−3 SP" floating verso pool icon.
- Ult charge: bar increment animation + percent.
- Twin Core meter: shared bar pulse quando entrambi i partner contribuiscono.

### §9.5.4 — Break event

- Big screen-wide flash + "BREAK!" banner + freeze frame ~200ms.
- Trigger: `CombatEvent::ToughnessBroken` (kernel canon).

### §9.5.5 — Reactive signature cue (per skill che ne ha uno)

Una sola voce per modifier; arco visivo che parte dallo Strike target verso i target reattivi:

| Modifier | Cue visiva | Banner |
|---|---|---|
| `OnKill→Detonate(Heated)` (Agumon Baby Burner) | Esplosione + spread heat sui 2 adj | `DETONATE: HEATED SPREAD` |
| `OnStatusApplied→Echo(Chilled)` (Gabumon Bubble Blast) | Onda fredda verso adj lowest-HP | `ECHO: CHILLED` |
| `OnKill→Chain` (Dorumon Dash Metal in Predator) | Arrow dal target morto al nuovo target | `PREDATOR CHAIN` |
| `OnHitN(3)→Apply(Paralyzed)` (Tentomon Electro Shocker) | Lampo finale sul 3° hop | `PARALYZE` |

Patamon: nessuna cue di reactive signature (kit puramente lineare).

## §9.6 — Combat log (text-side)

Headless-friendly, una linea per `CombatEvent`. Esempi allineati ai kit §8.3:

- `"Agumon casts Baby Flame → Goblimon takes 18 (Fire). Heated 3t applied."`
- `"Agumon casts Baby Burner → Goblimon takes 42 (Fire). Goblimon DIES. DETONATE: Heated spreads to 2 adjacent."`
- `"Gabumon casts Bubble Blast → Wolfmon takes 14 (Ice). Chilled 2t applied. ECHO: Chilled applied to Slimon (lowest HP adj)."`
- `"Dorumon casts Dash Metal (Predator state) → Goblimon takes 50 (Dark). Goblimon DIES. CHAIN: Dash Metal fires on Slimon, takes 38."`
- `"Patamon casts Patapata Hover on Renamon → +30 HP, Heated cleansed."`
- `"Renamon casts Tōhakken → all enemies take 60 (Holy). Blessed 2t to all allies. AdvanceTurn 25% to Agumon."`
- `"Tentomon casts Electro Shocker → 3 bounces. 3rd hit: Slimon PARALYZED 2t."`

Color-coded by tag. Scrollable. Persistito su JSONL.

## §9.7 — Settings / accessibility

- Animation speed slider (1× / 2× / instant) — headless test mode = instant.
- Damage number toggle.
- Status badge tooltip always-on (vs hover).

## §9.8 — Pannello UI matrix (M017 minimal scope)

| Pannello | Componenti | Bloccante per kit minimal? |
|---|---|---|
| Unit roster + HP/Toughness/SP/Ult | esistente in `combat_panel.rs` | base |
| Status badges (Heated/Chilled/Slowed/Paralyzed/Blessed) con stack + durata | parziale | **bloccante** (Heated scaling, status apply) |
| Twin Core tag visibility | esistente | parziale |
| PreyLock mark + Predator state self-flag | nuovo | **bloccante** Dorumon |
| Turn order timeline w/ telegraph | parziale | **bloccante** charged_attack enemy |
| Pre-cast damage projection + reactive signature preview | nuovo | desiderabile |
| Pre-cast legality_reason hover | nuovo | desiderabile (v0: nessuna gate complessa) |
| Reactive signature cue (5 voci) | nuovo | **bloccante** (4 dei 6 Digimon usano reactive signature) |
| Combat log line per event | esistente (`log.rs`) | base |

## §9.9 — Cosa NON va in UI v0

- Skill-tree node panel — deferred.
- Mind-Game phase strip (Renamon) — deferred (passive Renamon assente in v0).
- Grace counter team-wide — deferred (Patamon è support binario, niente Grace).
- StaticCharge / CircuitCharge bar dedicate — Tentomon non ha resource ulteriore oltre Battery Loop esistente.
- QTE prompt overlay (4 kinds) — deferred (niente QTE in v0).
- `PlayerChoice` overlay (intercept, stance) — deferred.
- Cinematica ult completa — sprite + atlas clip sufficient.
- VFX particle complete — deferred.
- Form swap animation — deferred (no Digivolution in v0).

## §9.10 — Headless test contract

Per ogni kit minimal, `cargo test` deve provare:

- Skill cast end-to-end senza UI.
- Reactive signature armed/non-armed per ognuna delle 5 reactive signature (gate condition).
- Cleanse di Patamon rimuove ≥1 status binario.
- All visible state derivabile da `CombatEvent` stream (no UI-only state).
- Snapshot test sul `CombatEvent` order + payload finale.
