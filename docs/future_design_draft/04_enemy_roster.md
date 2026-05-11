# §4 — Enemy roster

## 4.1 Stato attuale

`units.ron` ha 3 enemy unit hand-crafted: Devimon (101), Goblimon (102), Ogremon (103). Skill referenziano `enemy_ult_fire`, `ogremon_ult`. Encounter preset bind hard-coded.

**Naming convention:** il roster del player usa **nomi JP canonici** (V-mon, Plotmon, Tentomon, Patamon, Gabumon, Agumon). I dati skill di partenza vengono dalla skill `digimon` (K001), via `python3 .claude/skills/digimon/scripts/query.py skills NAME` per estrarre il kit franchise come base di design (non copia 1:1 — i numeri sono originali, ma i nomi e i temi delle skill seguono il canon JP).

**Audit roster da fare in S01 cleanup-task:** verificare i 6 OWNER attuali (`agumon`, `gabumon`, `patamon`, `tentomon`, `dorumon`, `renamon`) — quali devono passare a JP. `Dorumon` e `Renamon` sono già JP canon. `Agumon`, `Gabumon`, `Patamon`, `Tentomon` sono già JP. Quindi nessun rinome immediato sui 6 — la regola JP vale soprattutto per evoluzioni future (Champion: Greymon, Devimon, …) e per le skill (es. `pepper_breath` → forma JP se diverge).

## 4.2 Alleati come template di nemici nerfed

**Pros:**
- 6 enemy unit "gratis" dal punto di vista design (refusi dei Rookie alleati)
- Tematicamente coerente: "Wild Agumon", "Corrupted Patamon" — coerente con il lore Digimon dove gli stessi Digimon esistono in versione amica/ostile
- Le AI per gli enemy possono riusare i kit alleati, semplificando `enemy_ai.rs`

**Cons:**
- Player conosce già il kit avversario → riduce sorpresa
- Serve nerf bilanciato (-X% stats, AI degradata) altrimenti gli encounter diventano specchi

**Proposta concreta (encounter parametrico, 1-4 wild):**

| Tier encounter | Source | Nerf per wild | Encounter use |
|---|---|---|---|
| Wild pack (1-4) | `Vec<UnitId>` di 1-4 Rookie | scala dinamica con la dimensione del pack | Encounter 1-4 |
| Hand-crafted boss | Devimon/Goblimon/Ogremon esistenti | nessuno | Encounter 4-5 |

**Encounter shape:**

```rust
pub enum EncounterKind {
    WildPack(Vec<UnitId>),       // 1..=4 elementi
    Handcrafted(EncounterPreset),
}
```

**Curva nerf dinamica — definita in `encounter_balance.ron` (§2.5):**

Valori in `assets/data/encounter_balance.ron` (vedi forma in §2.5). Tabella **proposta iniziale**, raffinabile in S07 senza ricompilare:

| `pack_size` | HP mult | ATK mult | Ult |
|---|---|---|---|
| 1 | 0.65 | 0.75 | suppresso |
| 2 | 0.75 | 0.85 | suppresso |
| 3 | 0.85 | 0.90 | charge 50% iniziale, 1 sola volta |
| 4 | 0.95 | 0.95 | normale |

**Level scaling:** ogni wild scala col livello medio del party tramite `level_track` in `encounter_balance.ron` — `final_hp = base_hp(unit_stats.growth, level) * pack_nerf.hp_mult * (1 + level_track.hp_per_level * level_offset)`. Stesso per ATK. Questo evita di duplicare la curva di crescita: i wild **riusano** la `GrowthCurve` del Rookie corrispondente da `unit_stats.ron` e applicano il moltiplicatore di nerf sopra.

Razionale: 1 wild vs 4 player = HP boost servirebbe ma rompe il fantasy ("è un Rookie selvatico, non un boss"); nerfarlo invece premia il "trash mob veloce". 4 wild vs 4 player è specchio → nerf minimo, encounter di stress test.

**Validità input:** `WildPack` deve avere `1..=4` elementi e ogni `UnitId` deve esistere nel blueprint registry. Validato al bootstrap (panic in debug, log+skip in release).

**Impl:** invece di duplicare unit in `units.ron`, aggiungere un modificatore `WildVariant { hp_mult, atk_mult, suppress_ult, ult_initial_charge_pct, ult_max_casts }` calcolato a bootstrap leggendo `encounter_balance.ron` + `unit_stats.ron`. Funzione `apply_wild_variant(pack_size, level)` in `bootstrap.rs`. **Zero numeri hardcoded in Rust** — sono tutti in RON.

I 3 enemy hand-crafted (Devimon/Goblimon/Ogremon) restano come boss/elite — la varietà nel run viene dai wild pack di dimensione variabile.

## 4.3 Verifica

Test `tests/enemy_variants.rs`: bootstrap di encounter `WildPack(vec![AGUMON])`, `WildPack(vec![AGUMON, GABUMON])`, `WildPack(vec![AGUMON, GABUMON, PATAMON])`, `WildPack(vec![AGUMON, GABUMON, PATAMON, TENTOMON])` produce Unit con HP/ATK secondo curva, e suppress_ult corretto per ogni size. Test edge: `WildPack(vec![])` e `WildPack(vec![…; 5])` falliscono validazione bootstrap.
