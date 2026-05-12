# Dorumon — Ult: `heat_viper` (single-target burst + threshold bonus + force Predator state)

> **Goal**: ult single-target con **scaling threshold più aggressivo** + **forza l'entry in Predator state on hit**. Caso "ult come setup", non solo finisher.
>
> **Gap §2.2b condivisi:** params G1, source kind G5, F1 (conditional param via blueprint), F2 (BlueprintState write), ordering G4. Qui solo nuovi.

## §1 — Intent

- **Cost:** 0 SP, consuma ult bar. Anytime off-turn.
- **Effect base:** Damage Dark massivo `≈60` su primary; **+50% damage se primary HP <30%** (threshold più aggressivo del skill `<50%`).
- **Modifier:** **forza `predator_active=true`** on hit (anche se l'HP threshold del passive non è triggerato). Setup per chain successivo.
- **Atlas clip:** `skill` (frames 59–68, count 10)

## §2 — FSM topology

3-nodo: `Charge → Spit → Recovery`.

```
commit → Charge(3f) → Spit(4f) → Recovery(3f) → exit
                        │
                        │ on_enter:
                        │   // F1 path C — blueprint sceglie mul al commit
                        │   EmitDamage { hits:1, mul_param:"$chosen_ult_mul", target:Single(primary),
                        │                tough_break_param:"ult_tough_break" }
                        │   // F2 — blueprint mutate state esplicito
                        │   SetBlueprintState { state_key:"predator_active", value:true,
                        │                       dur_param:"predator_force_dur" }
                        │   SpawnParticle("infernal_viper","primary_pivot")
                        │   Shake { intensity:4, duration_ms:200 }
```

## §3 — Nodes table

| Node | frames | atlas | on_enter |
|---|---|---|---|
| `Charge` | 3 | 59–61 | `SpawnParticle("dark_breath_charge","mouth")` |
| `Spit` | 4 | 62–65 | EmitDamage(threshold-scaled) + SetBlueprintState + particle + shake |
| `Recovery` | 3 | 66–68 | — |

Frame budget: 10 = atlas. ✅

## §4 — Kernel events expected

```
Spit.on_enter
  ├─ DamageDealt(primary, ≈60 or ≈90, Dark)
  ├─ ToughnessReduced(primary, X)
  └─ BlueprintStateChanged(Dorumon, predator_active = true, dur:N turns)

Listener side: Predator Loop passive registra state change (vedi 04).
```

## §5 — Open questions (nuovi)

1. **F5 — `SetBlueprintState` command nuovo.** Schema:
   - `SetBlueprintState { state_key: string, value: Value, dur_param: Option<string> }`
   - Headless: gameplay (no drop).
   - Cleanup: `dur` expira → state revert (false / default).
2. **F6 — Force-Predator interaction con passive auto-entry.** Il passive (`predator_loop`) entra automaticamente quando un nemico cade sotto threshold. Ult forza entry **anche se** no threshold soddisfatto. Edge case:
   - Ult colpisce primary, primary muore al colpo → state forzato attivo, ma `lowest_hp_target` tracking è ora un altro nemico. Coerente: Predator state riarmato sul nuovo lowest.
   - **Decisione:** force entry imposta state attivo ma il target tracked è ricalcolato dal passive normalmente (next tick). Niente conflitto.
3. **F7 — Threshold ult <30% vs skill <50% asimmetrico.** Game-design: skill è "soft setup" (×2 sotto 50%), ult è "executioner" (×1.5 sotto 30%, su damage base alto). Numeri coerenti con identity §4.
4. **F8 — Ult charge accumulo trigger.** G11 ereditato — verificare che skill/basic ricarichino. Dorumon è fragile e veloce: deve poter ulta-re spesso.

## §6 — Verdetto

Heat Viper introduce **1 verbo nuovo**: `SetBlueprintState` (F5). Pattern: ult può **modificare lo state del proprio blueprint**, abilitando setup/payoff combos (ult → predator → chain successivo).

Nessun nuovo concetto architetturale; estensione del modello "blueprint state queryable + mutable".
