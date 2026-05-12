# Tentomon — Identity & Kit

> **Scope.** Identity sheet **revisionata** (2026-05-12): canon §8 originale era "SP battery puro". Decisione user (Q2 → opzione a): aggiungere ruolo **tank-lite** per coprire l'asse mancante senza introdurre un 7° digimon. Stesso pattern di `agumon/00_identity.md`.

## §0 — Riferimenti

- **Atlas:** `assets/digimon/tentomon_atlas.json` v1, 83 frames, frame size 664×670
- **Canon Digimon:** Tentomon, Child, Insect, Vaccine, fields: Nature Spirits
- **§8 roster minimal** § Tentomon — SP battery (**override applicato: + tank-lite**)
- **Battery Loop:** passive esistente, blueprint già in `src/combat/blueprints/tentomon.rs`

## §1 — Identità

**Battery + Bulwark.** Alimenta SP team + assorbe pressione fisica con corazza chitinosa. Bounce per spreadare Paralyzed. **Differenziazione dal pure tank assente:** non è un tank dedicato (no taunt, no aggro forzato), ma fornisce **mitigation distribuita** + **HP pool alto** + **Block reaction frequente**. Battery resta identità primaria.

- **Asse primario:** SP feed team (+2 SP gen su basic, vs +1 standard)
- **Asse secondario:** Tank-lite (HP alto, DR su Skill, Block reaction più frequente)
- **Vita:** **alta** (HP ~120, la più alta del roster)
- **Stat baseline (proposta):** `hp_max=120`, `speed=85` (lento per design tank), `toughness_max=70` (alto, regge break), `weakness=Fire`, `ultimate_trigger=100`, `ultimate_cap=150`, `ultimate_charge_per_event=25`

## §2 — Atlas mapping

| Slot kit | Atlas clip | Range | Count |
|---|---|---|---|
| **Idle (loop)** | `idle` | 41–48 | 8 |
| **Basic** (`petit_thunder`) | `attack` | 0–8 | 9 |
| **Skill** (`electro_shocker`) | `heavy_attack` | 22–33 | 12 |
| **Ultimate** (`super_shocker`) | `skill` | 49–64 | 16 |
| **Hurt** | `hurt` | 34–40 | 7 |
| **Block** | `block` | 9–13 | 5 |
| **Death** | `death` | 14–21 | 8 |
| **Victory** | `victory` | 65–82 | 18 |

Frame budget FSM: 9 + 12 + 16 = **37 frames** (≈3.1s @12fps). Più lungo della media: tank slow-but-impactful.

## §3 — Timing convention

Shared. Battery generation event timing: SP gen emesso su **frame Strike**, non a fine FSM (allinea a Agumon basic).

## §4 — Kit shape

| Slot | Skill ID | Target | Costo | Effetto base |
|---|---|---|---|---|
| Basic | `petit_thunder` | Single | 0 SP, **+2 gen** (battery role), +25 Ult | Damage piatto Electric (basso) |
| Skill | `electro_shocker` | **Bounce(3)** | **1 SP** | Damage medio su 3 target. **Modifier `OnHitN(3)→Apply(Paralyzed)`** — al 3° hit, Paralyzed sul target finale. **Su use, +1 turno DR 25% self** (tank hook) |
| Ult | `super_shocker` | AoE(All) enemies | UltCharge | Damage medio su tutta la linea + Paralyzed su 1 random; **+1 SP team** (battery moment) |
| Passive | `battery_loop` | listener | — | Esistente: SP generation reattiva. **Override: +20% block reaction chance** (tank-lite hook) |

**SP economy:** Tentomon è l'unica fonte di **+2 SP** su basic. Senza Tentomon, team con 4 unità in mix basic/skill è marginale; con Tentomon, sostenibile a regime.

## §5 — Tank-lite (mechanic, NUOVO override)

Niente sistema taunt/aggro (out of scope §8.7). Tank-lite = mitigation passiva distribuita:

- **HP pool:** 120 vs ~95 media → assorbe ~25% colpi in più.
- **DR self on Skill:** quando usa `electro_shocker`, +25% DR per 1 turno (placeholder vocabulary; allineare a `fur_cloak` di Gabumon).
- **Block reaction:** `block` clip (5f) reagibile più spesso (`battery_loop` passive override). Esce dalla FSM `hurt`, è un edge: damage incoming ridotto del 50% se Tentomon ha SP ≥3 (skin reactive).
- **Trade-off:** Tentomon è **lento** (`speed=85`): paga il tanking con turn order ridotto. Bilanciato dalla passive `kitsune_grace` di Renamon se in team.

## §6 — Sinergie

- **Tentomon → tutti spender:** SP feed è universale. In team Agumon+Gabumon+Renamon+Tentomon, Renamon spamma AoE, Agumon/Gabumon stackano status, Tentomon assorbe e ricarica.
- **Tentomon ↔ Patamon:** doppia mitigation. Patamon DR team (10%) + Tentomon tank-lite. Comp safe a costo di damage.
- **Tentomon ↛ Dorumon (anti-sinergia parziale):** Dorumon vuole turni veloci per inseguire bassi-HP; Tentomon è lento. Compensato dalla SP feed: Dorumon spende skill più spesso.

## §7 — Domande aperte

- Block reaction DR 50% se SP≥3: gate corretto o troppo permissivo? Validare in playtest.
- `super_shocker` "+1 SP team": rompe il cap di RoundSpTracker (`max_non_basic_per_round: 2`)? **Verificare**: l'Ult non passa dal contatore (è fuori dal canale skill), ma SP grant sì.
- Tank-lite vocabulary: serve un componente `DamageReduction` esplicito o si riusa lo status set? (proposta: nuovo componente, allinea a Gabumon `fur_cloak`)
- HP 120 sbilancia il roster? Verificare encounter design con team senza Tentomon.

## §8 — Coverage check (post-override)

Il roster a 6 ora copre **anche** l'asse tank-lite senza aggiungere unità, allineato alla decisione user 2a.
