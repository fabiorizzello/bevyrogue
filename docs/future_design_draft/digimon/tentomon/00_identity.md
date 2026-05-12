# Tentomon — Identity & Kit

> **Scope.** Identity sheet **revisionata** (2026-05-12): canon §8 originale era "SP battery puro". Decisione user (Q2 → opzione a): aggiungere ruolo **tank-lite** per coprire l'asse mancante senza introdurre un 7° digimon. Stesso pattern di `agumon/00_identity.md`.

## §0 — Riferimenti

- **Atlas:** `assets/digimon/tentomon_atlas.json` v1, 83 frames, frame size 664×670
- **Canon Digimon:** Tentomon, Child, Insect, Vaccine, fields: Nature Spirits
- **§8 roster minimal** § Tentomon — SP battery (**override applicato: + tank-lite**)
- **Battery Loop:** passive esistente, blueprint già in `src/combat/blueprints/tentomon.rs`

## §1 — Identità

**Battery + Bulwark.** Alimenta SP team + assorbe pressione fisica con corazza chitinosa. Bounce per spreadare Paralyzed. **Differenziazione dal pure tank assente:** non è un tank dedicato (no taunt, no aggro forzato), ma fornisce **mitigation distribuita** + **HP pool alto** + **Block reaction frequente**. Battery resta identità primaria.

**Dual-path passive `battery_loop`** (canon `04_passive_battery_loop.md` §1.5): due meccaniche ortogonali nello stesso blueprint, con boundary chiara:
- **Path A — SP-grant (side-channel listener):** reactive SP feedback su `SpEarned`/`SpSpent` event team. Non-FSM, puro `ctx.notify` + kernel emit (no state transition).
- **Path B — Block-reaction (Full FSM):** sub-variant B Reactive-proc, FSM `Dormant/BlockReady/BlockProc` con clip override `block` 5f + DR 50% applicato pre-DR su `IncomingDamage` quando SP≥3 + RNG roll.

- **Asse primario:** SP feed team (+2 SP gen su basic, vs +1 standard) — corrisponde a Path A side-channel.
- **Asse secondario:** Tank-lite (HP alto, DR su Skill, Block reaction più frequente) — corrisponde a Path B FSM.
- **Vita:** **alta** (HP ~120, la più alta del roster)
- **Stat baseline (proposta):** `hp_max=120`, `speed=85` (lento per design tank), `toughness_max=70` (alto, regge break), `weakness=Fire`, `ultimate_trigger=100`, `ultimate_cap=150`, `ultimate_charge_per_event=25`

## §2 — Atlas mapping

| Slot kit | Atlas clip | Range | Count |
|---|---|---|---|
| **Idle (loop)** | `idle` | 41–48 | 8 |
| **Basic** (`hard_claw`) | `attack` | 0–8 | 9 |
| **Skill** (`petit_thunder`) | `heavy_attack` | 22–33 | 12 |
| **Ultimate** (`electrical_discharge`) | `skill` | 49–64 | 16 |
| **Hurt** | `hurt` | 34–40 | 7 |
| **Block / Passive `battery_loop` reaction proc** | `block` | 9–13 | 5 |
| **Death** | `death` | 14–21 | 8 |
| **Victory** | `victory` | 65–82 | 18 |

Frame budget FSM: 9 + 12 + 16 = **37 frames** (≈3.1s @12fps). Più lungo della media: tank slow-but-impactful.

## §3 — Timing convention

Shared. Battery generation event timing: SP gen emesso su **frame Strike**, non a fine FSM (allinea a Agumon basic).

## §4 — Kit shape

| Slot | Skill ID | Target | Costo | Effetto base |
|---|---|---|---|---|
| Basic | `hard_claw` | Single | 0 SP, **+2 gen** (battery role), +25 Ult | Damage piatto **Electric** (basso). Tag Electric via VFX (claws + static, canon-flavored: Shock Jaw/Rhino Spin Tentomon hanno chele charged with electricity) |
| Skill | `petit_thunder` | **Bounce(3)** | **1 SP** | Damage medio Electric su 3 target (chain lightning, canon: "static electricity amplified by wings"). **Modifier `OnHitN(3)→Apply(Paralyzed)`** — al 3° hit, Paralyzed sul target finale. **Su use, +1 turno DR 25% self** (tank hook) |
| Ult | `electrical_discharge` | AoE(All) enemies | UltCharge | Damage medio su tutta la linea + Paralyzed su 1 random; **+1 SP team** (battery moment). Canon: "discharges electricity from whole body" |
| Passive | `battery_loop` | listener | — | Esistente: SP generation reattiva. **Override: +20% block reaction chance** (tank-lite hook) |

**SP economy:** Tentomon è l'unica fonte di **+2 SP** su basic. Senza Tentomon, team con 4 unità in mix basic/skill è marginale; con Tentomon, sostenibile a regime.

## §5 — Tank-lite (mechanic, NUOVO override)

Niente sistema taunt/aggro (out of scope §8.7). Tank-lite = mitigation passiva distribuita:

- **HP pool:** 120 vs ~95 media → assorbe ~25% colpi in più.
- **DR self on Skill:** quando usa `petit_thunder`, +25% DR per 1 turno (placeholder vocabulary; allineare a `fur_cloak` di Gabumon).
- **Block reaction:** `block` clip (5f) reagibile più spesso (`battery_loop` passive override). Esce dalla FSM `hurt`, è un edge: damage incoming ridotto del 50% se Tentomon ha SP ≥3 (skin reactive).
- **Trade-off:** Tentomon è **lento** (`speed=85`): paga il tanking con turn order ridotto. Bilanciato dalla passive `kitsune_grace` di Renamon se in team.

## §6 — Sinergie

- **Tentomon → tutti spender:** SP feed è universale. In team Agumon+Gabumon+Renamon+Tentomon, Renamon spamma AoE, Agumon/Gabumon stackano status, Tentomon assorbe e ricarica.
- **Tentomon ↔ Patamon:** doppia mitigation. Patamon DR team (10%) + Tentomon tank-lite. Comp safe a costo di damage.
- **Tentomon ↛ Dorumon (anti-sinergia parziale):** Dorumon vuole turni veloci per inseguire bassi-HP; Tentomon è lento. Compensato dalla SP feed: Dorumon spende skill più spesso.

## §7 — Domande chiuse (round 2026-05-12)

- **D1 — `electrical_discharge` "+1 SP team" vs `RoundSpTracker` cap.** ✅ **Chiuso:** l'Ult **non passa** dal contatore `max_non_basic_per_round` (è fuori dal canale Skill: SP grant da Ult è economy-side, non spend-side). La SP grant è cap-aware sul lato ricevente (`SpPool.add` clamp al cap 5, vedi `src/combat/sp.rs`), non lato spender-tracker. Allineato al pattern Gabumon ult (`blue_cyclone` `EmitSpGrant`). **`EmitSpGrant` formalizzato in `02-02b §C2`** come verbo kernel-known (gap S1 ✅ chiuso round-3 2026-05-12). Patamon heal usa `EmitHeal` (verbo separato, già in §C2), non `EmitSpGrant`.
- **D2 — Tank-lite vocabulary.** ✅ **Chiuso:** **nuovo componente** `Buff_*DR` tag-pure (presence-only) + valore in stringy `Buffs` map, allineato a Gabumon `fur_cloak`. Formalizzato in `02-02e §E.1` (channel layout, tag-component convention) + `02-08 §H` (DR taxonomy, stacking rules intra-unit max-replace / cross-unit additive clamp 0.5). Niente status set riutilizzato: status hanno `stacks`, buff DR hanno `value` mult.

## §7b — Domande aperte (defer playtest/M015+)

- **B1 — Block reaction DR 50% se SP≥3.** Gate corretto o troppo permissivo? Validare in playtest (M015+ balance pass). Concern: rende Tentomon "always-on tank" se team gira high-SP — possibile soglia da raise a SP≥4 o togliere gate del tutto.
- **B2 — HP 120 sbilancia il roster?** Verificare encounter design con team senza Tentomon (M015+ encounter validation). Se squad balance "no-Tentomon" risulta troppo fragile, considerare bumping HP altri tank-capable (Gabumon? Patamon?) o ridurre HP Tentomon a 110.

## §8 — Coverage check (post-override)

Il roster a 6 ora copre **anche** l'asse tank-lite senza aggiungere unità, allineato alla decisione user 2a.
