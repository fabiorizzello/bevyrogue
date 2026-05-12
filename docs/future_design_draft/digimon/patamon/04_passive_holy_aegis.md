# Patamon — Passive: `holy_aegis` (always-on aura)

> **Goal**: passive **non-reattiva**, sempre-attiva finché Patamon vive. Caso "no edge, no trigger" — il buff esiste come state.
>
> **Gap §2.2b condivisi:** dual-role pattern (agumon/04). Qui nuovi.

## §1 — Intent

- **Effect:** −10% damage taken per **tutti gli alleati vivi** finché Patamon è vivo.
- **Trigger:** assente. Buff esiste come **aura state**, applicato/rimosso ai death/spawn events del team.
- **Self-included:** sì (vedi identity §7).
- **Atlas:** nessuno (no FSM, no animation).

## §2 — Blueprint contract

```rust
impl BlueprintListener for PatamonBlueprint {
    fn on_kernel_event(&self, ev: &CombatEvent, ctx: &mut ListenerCtx) {
        match ev {
            // Patamon morto → rimuove l'aura da tutti gli alleati
            CombatEvent::UnitDied { unit } if ctx.is_self(unit) => {
                for ally in ctx.team_alive() {
                    ctx.remove_buff(ally, "holy_aegis");
                }
            }
            // Alleato spawnato/revive (futuro) o combat start → applica aura se Patamon vivo
            CombatEvent::CombatStarted | CombatEvent::UnitSpawned { .. }
                if ctx.is_self_alive() => {
                for ally in ctx.team_alive() {
                    ctx.add_buff(ally, "holy_aegis", value: 0.10, dur: Permanent);
                }
            }
            _ => {}
        }
    }
}
```

## §3 — Activation flow

```
combat start
  └─ CombatStarted event
     └─ Patamon listener applica holy_aegis (Permanent) a tutti gli alleati vivi (incluso self)

durante combat
  └─ tutti i damage incoming agli alleati: kernel legge buff "holy_aegis" → × 0.90

Patamon muore
  └─ UnitDied(self)
     └─ rimuove holy_aegis da tutti gli alleati
     └─ damage incoming torna 100%
```

## §4 — Trigger filter precision

- **Apply trigger:** `CombatStarted`, `UnitSpawned`, futuro `UnitRevived`.
- **Remove trigger:** `UnitDied { unit: Patamon }`.
- **Filter:** strict `caster == self`. Niente "altri Patamon" (canon: solo 1 per team).

## §5 — Power tuning

- **Value:** 10% DR (allineato a identity §4).
- **Stack rules:** additivo con `fur_cloak` (Gabumon DR 20%) → −30% se entrambe attive, **non moltiplicativo** (identity §4 lo specifica esplicitamente).
- **Cap totale DR:** 50% suggerito (evita "muri").

## §6 — Open questions (nuovi)

1. **A1 — Buff "Permanent" semantica.** Vocabolario buff `dur` ha `Turns(n) | RoundEnd | Permanent | UntilCondition`?
   - **Decisione:** `Permanent` ammesso come variante; cleanup esplicito via `remove_buff` su death event.
   - Cleanse Patamon (`holy_breeze`) **non rimuove** holy_aegis (è buff alleato, non debuff).
2. **A2 — Patamon revive (futuro fuori scope).** Se in futuro arriva revive, listener deve riapplicare aura. Hook `UnitRevived`. **Skip M017.**
3. **A3 — Stack additivo con `fur_cloak`.** Implementazione: damage pipeline somma valori DR di buff `kind:DR` attivi, clamp 0.5. **Action item §2.8 (effect cascade):** definire stacking rules per `kind` di buff.
4. **A4 — `CombatStarted` event esiste?** Verificare `src/combat/events.rs::CombatEvent`. Se no, **action item:** aggiungerlo (utile anche per ogni init-time listener).

## §7 — Verdetto

`holy_aegis` è il **caso degenere passive**: nessun edge predicate, solo state-applied/state-removed. Espone il bisogno di:
- `dur: Permanent` come variante di durata buff.
- DR stacking rules formalizzati (A3).
- `CombatStarted` come kernel event (A4).

**Nessun gap architetturale duro** — sono estensioni minor del vocabolario buff e dell'event set.
