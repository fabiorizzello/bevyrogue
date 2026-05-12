# Dorumon — Skill: `draconic_edge` (threshold scaling + chain in Predator state)

> **Goal**: stress test **damage scaling condizionato a HP%** + **modifier reattivo `OnKill→Chain` gated dallo state Predator**. Caso più complesso del kit Dorumon, perché l'edge esiste **solo se Predator state attivo**.
>
> **Gap §2.2b condivisi:** params G1, source kind G5 (event payload), multi-target G6, ordering G4. Qui solo nuovi.

## §1 — Intent

- **Cost:** **1 SP** — **Gen:** +25 Ult
- **Effect base:** Damage Dark `≈16` su primary; **×2 multiplier se primary HP <50%**.
- **Modifier-firma:** **`OnKill→Chain`** — se primary muore E Predator state attivo → +1 hit su nuovo target lowest-HP residuo (max 1 chain).
- **Atlas clip:** `heavy_attack` (frames 31–39, count 9)

## §2 — FSM topology

4-nodo + edge reattivo: `Wind → Cleave → ChainStrike? → Recovery`.

```
commit → Wind(2f) → Cleave(3f) → Recovery(2f) → exit
                      │            │ (no-chain path)
                      │ on_enter:
                      │   // damage scaling — risolto al commit (snapshot G5)
                      │   // se primary.hp_pct < 0.50 → mul = "skill_mul_threshold"
                      │   // else                     → mul = "skill_mul_base"
                      │   EmitDamage { hits:1, mul_param:"$chosen_mul", target:Single(primary) }
                      │   SpawnParticle("dark_slash","claw")
                      │   Shake { intensity:2, duration_ms:100 }
                      │
                      ├── edge A (Predator-gated): predicate(
                      │       KernelEvent(UnitDied{unit: primary}) AND BlueprintState(predator_active)
                      │   ) prio:10
                      │   ──▶ ChainStrike(2f)
                      │       on_enter:
                      │         EmitDamage { hits:1, mul_param:"chain_mul",
                      │                      target:LowestHpPctAlive(scope:EnemyTeam) }
                      │         SpawnParticle("chain_arc","new_target_pivot")
                      │
                      └── edge B (default): TimeInNode prio:0 ──▶ Recovery
```

## §3 — Nodes table

| Node | frames | atlas | on_enter |
|---|---|---|---|
| `Wind` | 2 | 31–32 | `SpawnParticle("dark_charge","horn")` |
| `Cleave` | 3 | 33–35 | EmitDamage(primary, mul-scaled) + particle + shake |
| `ChainStrike` | 2 | 36–37 | (solo edge A) EmitDamage(new_lowest) + particle |
| `Recovery` | 2 | 38–39 (no-chain) / 38–39 (chain) | — |

Frame budget: con chain 2+3+2+2=9 = atlas. ✅ Senza chain 2+3+4 padded a 9 (Recovery 4f); usare variant Recovery via edge priority (vedi agumon/03 G7).

## §4 — Kernel events expected

```
Cleave.on_enter
  ├─ DamageDealt(primary, ≈16 or ≈32, Dark)
  └─ se primary muore: KernelEvent::UnitDied { unit: primary }

[edge resolution next tick]
  if predator_active AND UnitDied(primary):
    edge A → ChainStrike.on_enter
      └─ DamageDealt(new_lowest_alive, ≈chain_mul, Dark)

else:
  edge B → Recovery
```

## §5 — Open questions (nuovi)

1. **F1 — Damage scaling pre-emit (`$chosen_mul`) richiede "conditional param resolution" al commit.** Modello attuale (G1) ha `params: HashMap<String, Value>` statica. Qui serve **condizionale**:
   - **A.** Inline expression nel `mul_param`: `"if primary.hp_pct < 0.5 then skill_mul_threshold else skill_mul_base"` (mini-DSL).
   - **B.** Due Commands con predicate inline `EmitDamage { mul_param:"skill_mul_threshold", predicate:"primary.hp_pct < 0.5" }` + `EmitDamage { mul_param:"skill_mul_base", predicate:"primary.hp_pct >= 0.5" }`. Engine emette solo quello che matcha.
   - **C.** Blueprint risolve in Rust al commit, emette **un solo** `EmitDamage` con `mul_param` già selezionato (param map letta dal blueprint, no DSL).
   - **Decisione consigliata:** C. Niente nuovo DSL; blueprint Dorumon ha 4 righe Rust che scelgono il param. Coerente con G5 (snapshot at commit). FSM resta dichiarativa, scelta logica vive nel blueprint.
2. **F2 — `BlueprintState(predator_active)` predicate.** Estensione vocabolario predicate (§2.2b §D non lo include). Schema:
   - `BlueprintState { state_key: string, expected: Value }` — il blueprint owner espone state queryable.
   - Headless: deterministico, state read live.
3. **F3 — `TargetShape::LowestHpPctAlive { scope: EnemyTeam }`.** Già discusso in gabumon/02 (`AdjLowest`) e dorumon stesso. Vocabolario `TargetShape` cresce; consolidare in famiglia unica `Selector`:
   - `Selector::LowestHp { scope, exclude: [self, dead], metric: HpPct | HpAbs }`
4. **F4 — Chain consume Predator state?** Identity §5: "Exit: target tracked muore (chain consumato)". → Sì, chain consuma. Action item: blueprint Dorumon su `ChainStrike.on_enter` chiama `ctx.set_blueprint_state("predator_active", false)`.

## §6 — Verdetto

Draconic Edge è il **caso più ricco** del roster su edge reattivi:
- Edge gated da **blueprint state** (F2), non solo da kernel event.
- Damage scaling condizionato → decisione tra DSL vs blueprint Rust → consigliato Rust (semplifica vocabolario).
- Conferma necessità di `Selector::LowestHp` (allinea a gabumon/02 `AdjLowest`).

3 estensioni vocabolario: `BlueprintState` predicate, `Selector::LowestHp`, conditional param (path C senza DSL).
