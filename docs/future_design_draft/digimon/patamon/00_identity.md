# Patamon — Identity & Kit

> **Scope.** Identity sheet **revisionata 2x** (2026-05-12):
> - rev1: allineata a §8 roster minimal (pure healer).
> - rev2: audit canon DAPI + atlas anim → rename skills a **canon Tamers/Adventure** (`tai_atari` / `patapata_hover` / `sparking_air_shot`), ult diventa **hybrid damage AoE + heal AoE** (Sparking Air Shot canon = damage signature, glittering = heal splash). Identity §1 shift da "pure healer" → "support-healer con damage burst ult".

## §0 — Riferimenti

- **Atlas:** `assets/digimon/patamon_atlas.json` v1, 92 frames, frame size 618×732
- **Canon Digimon:** Patamon, Child, Mammal (Holy), Vaccine, fields: Virus Busters / Wind Guardians
- **§8 roster minimal** § Patamon — Support-healer (rev2)
- **Canon move pool (DAPI):** Air Shot (= **Boom Bubble** Adventure dub, signature), Kūchū Air Shot (aerial), **Tai Atari** (Body Blow, body slam), Hane Binta (Wing Slap), **Patapata Hover** (flies), Air Slam, Thousand Wing (spin all-directions), **Sparking Air Shot** (glittering powered Air Shot), Air Gust (multi-cloud), Aero Slash (wind blades), Spin Kick, Glide, Petit Tackle, Tai Atari Bomb, Geigeki Air Shot (counter), Pretty Rush.
- **Rev2 canon selection:** Tai Atari (basic, roll/headbutt anim = canon body slam literal), Patapata Hover (skill, jump+fall reflavor heal-aura), Sparking Air Shot (ult, boom bubble anim = canon glittering hybrid damage+heal). Holy_aegis resta flavor-only passive.
- **Reflavor justification heal:** Patamon = Holy Mammal (Vaccine) + field Virus Busters (anti-Virus = purification = healing canon-adjacent). Wind = breath = life-spirit (mythology universal). Adventure canon = Patamon evolves Angemon (full healer) → Child stage = proto-Angemon, healing-affinity emerging.

## §1 — Identità

Support-healer con damage burst su ult. Heal + cleanse + DR team (sustain primary) + AoE damage Holy burst (ult secondary). **Nessun modifier reattivo lato skill/passive**: la semplicità è la firma sulla lane sustain. Ult `sparking_air_shot` dual-axis damage+heal (rev2). Universale in ogni team con ≥1 spender heavy o team aggressivi.

- **Asse primario:** Sustain (heal skill + heal ult + cleanse + DR passive)
- **Asse secondario:** Damage Holy (basic single + ult AoE burst) — non più solo "filler turno"
- **Vita:** media (HP ~95), squishy ma DR-team copre
- **Stat baseline (proposta):** `hp_max=95`, `speed=105`, `toughness_max=40`, `weakness=Dark`, `ultimate_trigger=100`, `ultimate_cap=150`, `ultimate_charge_per_event=25` (heal counts as event)

## §2 — Atlas mapping

| Slot kit | Atlas clip | Range | Count | Anim type (real) |
|---|---|---|---|---|
| **Idle (loop)** | `idle` | 50–61 | 12 | loop |
| **Basic** (`tai_atari`) | `attack` | 0–8 | 9 | **roll/headbutt** (canon Tai Atari literal match) |
| **Skill** (`patapata_hover`) | `heavy_attack` | 30–43 | 14 | **jump+fall** (reflavor: ascend → radiant descent heal) |
| **Ultimate** (`sparking_air_shot`) | `skill` | 62–76 | 15 | **boom bubble cheek-inflate + projectile** (canon Air Shot literal, Sparking variant via gold VFX) |
| **Hurt** | `hurt` | 44–49 | 6 | one-shot |
| **Block** | `block` | 9–14 | 6 | hold |
| **Death** | `death` | 15–29 | 15 | one-shot |
| **Victory** | `victory` | 77–91 | 15 | loop |

Frame budget FSM: 9 + 14 + 15 = **38 frames** (≈3.2s @12fps).

**Anim ↔ canon match rationale (rev2):**
- `attack` roll/headbutt = **Tai Atari** (Body Blow) canon literal. Zero stretch.
- `heavy_attack` jump+fall ≠ canon literal Patapata Hover (canon = sustained flight). Reframe: jump-apex-descent = "blessing arc" — VFX gold halo at apex + descent blessing trail rende heal-cast leggibile.
- `skill` boom bubble cinematic = **Sparking Air Shot** canon (= "glittering powered-up Air Shot"). Anim base Air Shot literal; "Sparking" distinguer via gold-glitter VFX. Reflavor heal-splash giustificato canon "glittering shrapnel" → mist healing splatter dopo burst.

## §3 — Timing convention

Shared. Healing FSM nodes hanno target shape `Single` (Skill) / `AoE(All)` (Ult) sugli alleati: kernel resta autorità sull'apply, FSM sequenzia VFX.

## §4 — Kit shape

| Slot | Skill ID | Canon JP/EN | Target | Costo | Effetto base |
|---|---|---|---|---|---|
| Basic | `tai_atari` | Tai Atari / Body Blow (canon Adventure) | Single enemy | 0 SP, +1 gen, +25 Ult | Damage piatto Holy (basso, `~6`) |
| Skill | `patapata_hover` | Patapata Hover (canon, reflavor heal) | Single ally | **1 SP** | Heal `~25%` HP max + **Cleanse 1 status** (debuff oldest-first) |
| Ult | `sparking_air_shot` | Sparking Air Shot / glittering powered Air Shot (canon, hybrid damage+heal) | AoE(All) enemies + AoE(All) ally | UltCharge | **Damage Holy ~25 a tutti i nemici** + **Heal ~20% HP max team** + Cleanse 1 status/ally |
| Passive | `holy_aegis` | flavor-only (Holy Mammal aura) | listener | — | Tutti gli alleati: **-10% damage taken** finché Patamon vive |

**Sinergie:** universale. Critico in team con Dorumon (fragile) e Agumon/Renamon (squishy). Aegis DR si stack-additivo (non moltiplica) con Gabumon `fur_cloak` per non sfondare il bilanciamento. **Rev2 cross-roster:** Ult damage scala con `Blessed` (Renamon) → cross-roster combo gratis: Renamon ult applica Blessed → Patamon ult damage +15% sui nemici.

## §5 — Cleanse rules

- **Targets:** rimuove 1 debuff dal target. Priorità: status più "vecchio" (FIFO sui timer), tie-break ID alfabetico.
- **Filter:** cleanse rimuove SOLO debuff (Heated, Chilled, Confused, Paralyzed, Slowed). Mai buff alleati né `Holy` stack (vedi sotto).
- **Vocabolario:** nessuna estensione status set per ora. Cleanse usa toggle binario esistente.

## §6 — Holy element (nota cross-roster)

Patamon e Renamon (post-revisione) sono entrambi Holy. Differenziazione:
- **Patamon:** Holy = heal/buff vector. Damage Holy basso e marginale.
- **Renamon:** Holy = AoE damage vector + time-manip.

Niente competizione meccanica diretta: Patamon agisce sugli alleati, Renamon sui nemici. Sinergia: il `Blessed` di Renamon (vedi `renamon/00_identity.md`) **non interferisce** con cleanse Patamon (Blessed è buff, non debuff).

## §7 — Domande chiuse (round 2026-05-12)

- **Q1 — Heal scaling:** **% HP max target** (no flat). Motivazione: evita inutility su tank vs squishy + uniformità con `sparking_air_shot` ult AoE (`~20%` HP max team). Allinea heal-skill (`~25%`) e ult heal (`~20%`) su stessa metrica. Implementazione: `heal_amount = floor(target.hp_max * 0.25)` per `patapata_hover`, `target.hp_max * 0.20` per `sparking_air_shot`.
- **Q2 — Aegis DR self-include:** **sì**, Patamon ∈ `team_alive`. Motivazione: Patamon squishy (`hp_max=95`) + nessun'altra fonte di mitigation self (no DR buff su skill, no block-reaction tipo Tentomon). Allinea a §4 implicito (passive descrive "Tutti gli alleati", inclusivo). Implementazione `holy_aegis`: `for unit in team_alive { apply_buff(unit, "holy_aegis", Permanent) }` — `team_alive` include Patamon stesso.
- **Q3 — `tai_atari` Holy damage triggera `Blessed` (Renamon)?** **No.** Motivazione: `Blessed` è **buff alleato applicato esplicitamente** da `tohakken` (Renamon ult), non auto-derivato dal damage tag Holy. Conferma §6 cross-roster: il tag `Holy` chiama solo weakness check standard (vs `weakness=Dark`), no Blessed proc. Sinergia damage `Blessed`→Patamon ult avviene solo se Renamon ha **già usato** la sua ult prima (Blessed listener legge `kind:Buff` su ally, non damage tag).

## §7b — Domande aperte (defer playtest / M017+)

- **Q4 — Rev2 Hybrid ult tuning.** 25 damage + 20% heal vs precedente 35% pure heal: total team value cresce ma single-axis-heal cala. Validare se 20% basta a healer-role o serve bump a 25%. **Playtest M017+.**
- **Q5 — Rev2 Sparking variant distinguer.** Visivamente come si distingue `sparking_air_shot` (ult) da `tai_atari` (basic, ex `boom_bubble`)? Risposta corrente: `tai_atari` usa atlas `attack` (roll/headbutt = body slam canon), `sparking_air_shot` usa atlas `skill` (cheek-inflate + projectile = canon Air Shot). Anim diversi = no confusion. Plus glittering gold-VFX su Sparking distinguer. **Validate M017+ quando VFX system live.**
