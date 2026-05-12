# Renamon — Passive: `kitsune_grace` (ally-Ult listener → self AdvanceTurn)

> **Goal**: passive reattiva a **evento alleato** (`UltimateUsed` by ally). Primo listener cross-team che modifica `TurnOrder` di self.
>
> **Gap §2.2b condivisi:** dual-role (agumon/04), time-manip T1 (renamon/02). Qui nuovi.

## §1 — Intent

- **Trigger:** un alleato (qualsiasi, incluso Renamon stessa? vedi §5) consuma Ult.
- **Effect:** `AdvanceTurn(self, 10%)` — Renamon avanza il proprio gauge.
- **Scope:** listener-only, no FSM.

## §2 — Blueprint contract

```rust
impl BlueprintListener for RenamonBlueprint {
    fn on_kernel_event(&self, ev: &CombatEvent, ctx: &mut ListenerCtx) {
        match ev {
            CombatEvent::UltimateUsed { actor }
                if ctx.is_ally(actor) && !ctx.is_self(actor) => {
                ctx.emit_kernel_effect(KernelEffect::AdvanceTurn {
                    actor: ctx.self_id(),
                    pct: 10,
                });
            }
            _ => {}
        }
    }
}
```

## §3 — Activation flow

```
turno T: Agumon casta nova_blast → CombatEvent::UltimateUsed { actor:Agumon }
  └─ Renamon listener: actor != self → trigger
     └─ KernelEffect::AdvanceTurn(Renamon, 10%)
        └─ TurnGauge Renamon −10%

turno successivo: gauge Renamon ridotto → agisce prima
  └─ se Renamon Ult: niente self-trigger (vedi §5)
```

## §4 — Trigger filter precision

- **Actor check:** alleato vivo, NON self (escluso per evitare loop).
- **Event:** `UltimateUsed` (verificare esiste in `src/combat/events.rs`. Probabilmente `CombatEventKind::UltimateUsed` o derivato da `UltimateCharge` consumption. **Action item se mancante:** aggiungere evento canonico `UltimateUsed { actor }` dopo consumo.
- **Cap:** stack additivo nel turno (Patamon ult + Agumon ult = 20% advance). Identity §5 dice cap ±50% per effetto, stack additivo clamp `[0, 200%]`.

## §5 — Open questions (nuovi)

1. **K1 — Self-Ult triggera kitsune_grace?**
   - **A.** Sì (Renamon ult charge sé stessa) → loop infinito se non capped. Cap 50% lo blocca, ma è UX strana.
   - **B.** No (escluso da filter) → identity §1 ("recupera tempo per riapplicare AoE") implica reagire **ad altri**.
   - **Decisione consigliata:** B. Filter `!ctx.is_self(actor)`.
2. **K2 — `UltimateUsed` event quando è emesso?** Al `commit_action(Ult)` o a `Ult.Strike.on_enter` (consumo bar)? **Coerenza:** allineare a "consumo bar" → solo se l'Ult andata davvero. Cancellazioni (es. invalid target) non triggerano.
3. **K3 — Bound check.** Cap 50% per call (T1). 10% × 5 alleati Ult in stesso round → 50%, ok. Reale: 1-2 ult/round max. Safe.
4. **K4 — Compatibilità con `Blessed`.** Blessed (renamon/03) buffa damage e Ult charge gen. Niente double-dip con `kitsune_grace`: K1 reagisce all'**uso** dell'ult, non al **charge**. Distinto.

## §6 — Verdetto

`kitsune_grace` consolida:
- **Event canonico `UltimateUsed`** (verificare o aggiungere).
- **Listener emette `KernelEffect::AdvanceTurn`** — primo caso di listener che produce kernel effect (non solo buff/state mutation).

Pattern: listener può **emit kernel effect**, non solo applicare buff. Generalizzazione utile per altri passive futuri.
