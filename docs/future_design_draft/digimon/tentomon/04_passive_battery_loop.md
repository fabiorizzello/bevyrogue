# Tentomon — Passive: `battery_loop` (existing + tank-lite override)

> **Goal**: passive **esistente** (`src/combat/blueprints/tentomon.rs::battery_loop`) con override identity §4: **+20% block reaction chance** (tank-lite hook).
>
> **Gap §2.2b condivisi:** dual-role (agumon/04), buff/cleanse interactions. Qui solo nuovi.

## §1 — Intent

Battery loop esistente: reactive SP feedback su eventi team. Override: amplia la chance di **block reaction** quando subisce damage (anim clip `block`, frames 9–13).

- **Existing path:** SP grant reattiva quando spese SP team (consultare codice — pattern tracker).
- **Override path:** intercetta `IncomingDamageRequest` (pre-damage) → +20% probability di trigger `BlockReaction` → damage ridotto 50% se SP self ≥3 (identity §5).

## §2 — Blueprint contract (override only)

```rust
impl BlueprintListener for TentomonBlueprint {
    fn on_kernel_event(&self, ev: &CombatEvent, ctx: &mut ListenerCtx) {
        // existing battery_loop logic preserved...

        // override tank-lite
        match ev {
            CombatEvent::IncomingDamage { target, .. }
                if ctx.is_self(target) && ctx.self_sp() >= 3 => {
                // bumpa block reaction probability di +20pp (additivo a base ~10%)
                let roll = ctx.combat_rng().roll();
                if roll < ctx.block_chance_self() + 0.20 {
                    ctx.emit_kernel_effect(KernelEffect::BlockReaction {
                        actor: ctx.self_id(),
                        damage_mult: 0.50,
                    });
                }
            }
            _ => {}
        }
    }
}
```

## §3 — Activation flow

```
nemico colpisce Tentomon → IncomingDamageRequest (pre-resolution)
  └─ battery_loop listener: SP self ≥3? sì → roll RNG → reaction triggered (es)
     └─ KernelEffect::BlockReaction(Tentomon, mult:0.50)
        └─ damage pipeline applica × 0.50
        └─ presentation: FSM Tentomon entra in `block` clip (5f)

durante combat: alleato spende SP
  └─ existing battery_loop path: emit SP grant a self (logica esistente, non modificata)
```

## §4 — Trigger filter precision

- **Self check:** strict `target == Tentomon`. No proc su altri.
- **SP gate:** `self_sp() >= 3`. Sotto, niente bump.
- **RNG:** deterministico via `ctx.combat_rng()` (seeded), test-friendly.
- **Stack con buff DR esistenti:** Block reaction è **separato** dal damage-pipeline DR (es. `holy_aegis`, `dr_self`). Ordine:
  1. Block reaction triggera → damage × 0.50 (pre-DR).
  2. DR buff applica → damage × (1 − DR_total).
  3. Final damage delivered.

## §5 — Open questions (nuovi)

1. **B4 — `CombatEvent::IncomingDamage` esiste?** È un evento **pre-damage**. Verificare `src/combat/events.rs`. Se solo `DamageDealt` (post-fact), serve aggiungere `IncomingDamage` come pre-step. Action item §2.8 (cascade) lo abilita.
2. **B5 — Block reaction emessa via `KernelEffect::BlockReaction` o nuovo verbo?**
   - **A.** Verbo nuovo `BlockReaction { actor, damage_mult }` nel kernel effect set.
   - **B.** Buff temporaneo `dr_block` applicato a self con dur=this-damage-only.
   - **Decisione consigliata:** A. Reaction è effetto puntuale, non state che persiste.
3. **B6 — Block clip FSM trigger.** Quando reaction triggera, l'anim `block` (5f) entra. Chi orchestra?
   - **Proposta:** kernel emette `BlockReactionTriggered` event → presentation listener fa play della clip. Headless lo droppa (cosmetic).
4. **B7 — Battery loop existing logic verifica.** Identity §4 dice "esistente". `src/combat/blueprints/tentomon.rs` ha già il path SP grant; verificare se il design corrente coincide con l'identity sheet (mancano dettagli granulari sul trigger esatto). **Action item:** allineare doc con codice o codice con doc, decidere fonte di verità prima M017.

## §6 — Verdetto

Override tank-lite consolida:
- **Evento `IncomingDamage` pre-step** nel cascade pipeline (B4, gap §2.8).
- **Verbo `BlockReaction`** come kernel effect (B5).
- **RNG seeded shared** tra blueprint (D1 di tentomon/03 e B4 qui).

Battery loop existing logic resta intoccato; override aggiunge **listener path #2** sullo stesso blueprint. Pattern dual-listener identico a Gabumon (`fur_cloak` + `twin_core_ice`).
