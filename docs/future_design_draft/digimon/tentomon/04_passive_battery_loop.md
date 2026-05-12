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

## §4b — Anim hook (block clip)

Passive listener-only ⇒ no FSM proprio. Un momento richiede comunque clip override (non solo VFX):

```
Signal: KernelEffect::BlockReaction { actor: self, damage_mult: 0.50 }
  └─ AnimPlayer.play_one_shot(clip:"block", count:5f, ~0.4s @12fps)
     ├─ Origin: self
     ├─ Layer: above idle, preempts hurt clip (reaction wins over flinch)
     └─ Fallback completion → idle
```

`block` clip (5f): chitin guard sideways + wings fold. Match canon-flavor (Rolling Guard 374 "curls in ball to raise defense" — semantica leggermente diversa ma più vicino disponibile).

**Headless:** clip override skip (presentation-only).

## §4c — VFX (Forma C: Channel 1 + Channel 2, §2.2e)

Passive listener-only ⇒ niente `SpawnParticle` Command (canale FSM). VFX viaggiano su:

- **Channel 1** = `ListenerCtx::notify(NotifyParticle)` per one-shot event-bound (§2.2e §C).
- **Channel 2** = presentation observer su component diff (`Added`/`Removed`) per persistent state-bound (§2.2e §D).

| # | Momento | Channel | Trigger | Preset | Origin | Motion |
|---|---|---|---|---|---|---|
| 1 | **SP grant proc** (existing battery path) | 1 | `on_kernel_event(SpEarned { source: "battery_loop", actor: ally })` | `battery_pulse` (brief yellow shimmer) | `SelfCenter` (Tentomon) | `Static` |
| 2 | **SP grant link** *(opt.)* | 1 | stesso trigger di #1 | `battery_link_zap` (short bolt) | `SelfCenter` (Tentomon) | `Travel { to: EntityCenter(EventTarget), ease: EaseOut, ms: 150 }` |
| 3 | **Block-ready aura (entry)** | 2 | `Added<BuffComponent_BlockReady>` su Tentomon | `battery_ready_loop` (subtle pulsing yellow halo) | `SelfCenter` | `Static` |
| 4 | **Block reaction flash** | 1 | `on_kernel_event(BlockReactionTriggered { actor: self, .. })` | `static_shield` (short chitin-spark burst) | `SelfCenter` | `Static` |
| 5 | **Block-ready aura despawn** | 2 | `RemovedComponents<BuffComponent_BlockReady>` | — (manager despawned) | — | — |

### Note implementative

- **`BuffComponent_BlockReady`** — indicator-component non-buff (vedi §5 gap B8). Inserito/rimosso dal kernel SP-watcher quando `Tentomon.sp` attraversa la soglia `>=3 / <3`. Pure marker presentation, non c'è `BuffId` stringa equivalente (gameplay non lo ignora — non esiste lato gameplay).
- **Riga 1+2** sono entrambe **Channel 1** e ricevono `EventTarget` (l'alleato che ha speso SP) tramite `EntityRef::EventTarget` (§2.2e §C tabella, listener-side only).
- **Riga 2 opzionale:** se il link visivo è troppo rumoroso con SP frequenti, droppare e tenere solo riga 1.
- **Riga 4** richiede `CombatEvent::BlockReactionTriggered` (vedi §5 gap B9). Oggi c'è solo `KernelEffect::BlockReaction` Command, non l'event speculare consumato dal presentation observer.
- **Headless:** Channel 1 = `notify` no-op se `cfg(not(feature = "windowed"))`. Channel 2 = system gated dietro feature flag, mai schedulato in headless.

## §5 — Open questions (nuovi)

1. **B4 — `CombatEvent::IncomingDamage` esiste?** È un evento **pre-damage**. Verificare `src/combat/events.rs`. Se solo `DamageDealt` (post-fact), serve aggiungere `IncomingDamage` come pre-step. Action item §2.8 (cascade) lo abilita.
2. **B5 — Block reaction emessa via `KernelEffect::BlockReaction` o nuovo verbo?**
   - **A.** Verbo nuovo `BlockReaction { actor, damage_mult }` nel kernel effect set.
   - **B.** Buff temporaneo `dr_block` applicato a self con dur=this-damage-only.
   - **Decisione consigliata:** A. Reaction è effetto puntuale, non state che persiste.
3. **B6 — Block clip FSM trigger.** Quando reaction triggera, l'anim `block` (5f) entra. Chi orchestra?
   - **Proposta:** kernel emette `BlockReactionTriggered` event → presentation listener fa play della clip. Headless lo droppa (cosmetic).
4. **B7 — Battery loop existing logic verifica.** Identity §4 dice "esistente". `src/combat/blueprints/tentomon.rs` ha già il path SP grant; verificare se il design corrente coincide con l'identity sheet (mancano dettagli granulari sul trigger esatto). **Action item:** allineare doc con codice o codice con doc, decidere fonte di verità prima M017.
5. **B8 — Indicator-component senza `BuffId` (presentation-only marker).** `BuffComponent_BlockReady` esiste solo come marker presentation (osservato da Channel 2 §2.2e §D), inserito quando SP self attraversa la soglia ≥3. La convenzione §2.2e §E presume mirror 1:1 `BuffId` ↔ `Buff_*` typed-component. Qui non c'è `BuffId` gameplay-side: la soglia SP non è uno stato del modello (è un derived predicate). Opzioni:
   - **A.** Estendere convenzione §2.2e §E ammettendo prefix dedicato `BuffComponent_*` per indicator-only senza `BuffId`. Esplicita che è presentation-pure, niente accesso gameplay.
   - **B.** Promuovere "block_ready" a `BuffId` fittizio, applicato dal kernel SP-watcher con multiplier 1.0 e dur `Permanent` (no-op gameplay-side). Convenzione §2.2e §E resta intatta.
   - **Decisione consigliata:** A. È più onesto — il marker esiste solo per pilotare presentation, formalizzare un buff fittizio è una bugia mantenuta da entrambi i lati. Collegare a §2.2e §I gap 1 (numeric-binding) come caso adiacente. **Action item:** decisione architetturale dopo, file Tentomon resta valido in entrambi gli scenari (riga 3+5 di §4c referenziano il nome typed-component, non `BuffId`).
6. **B9 — `CombatEvent::BlockReactionTriggered` non esiste.** §4c riga 4 (block reaction flash) richiede event speculare al `KernelEffect::BlockReaction` Command per la presentation observer Channel 1. Oggi c'è solo il Command (input al damage pipeline), niente event di out-side. **Proposta:** aggiungere `CombatEvent::BlockReactionTriggered { actor, damage_mult, attacker }` emesso dal kernel dopo aver applicato il mult. Coerente con il pattern Command→Event esistente (es. `ApplyBuff` → `BuffApplied`). Da formalizzare in §02-02b §G-Events. **Action item:** patch §02-02b §G-Events con il nuovo event quando si chiude il giro del roster (deferred). Specchio gameplay-side: `agumon/04` §9 G-Verbs ha `BlockReaction` Command, manca event speculare anche lì.

## §6 — Verdetto

Override tank-lite consolida:
- **Evento `IncomingDamage` pre-step** nel cascade pipeline (B4, gap §2.8).
- **Verbo `BlockReaction`** come kernel effect (B5).
- **RNG seeded shared** tra blueprint (D1 di tentomon/03 e B4 qui).

Battery loop existing logic resta intoccato; override aggiunge **listener path #2** sullo stesso blueprint. Pattern dual-listener identico a Gabumon (`fur_cloak` + `twin_core_ice`).
