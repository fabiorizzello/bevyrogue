# Renamon — Passive: `kitsune_grace` (ally-Ult listener → self AdvanceTurn)

> **Goal**: passive reattiva a **evento alleato** (`UltimateUsed` by ally). Primo listener cross-team che modifica `TurnOrder` di self.
>
> **Gap §2.2b condivisi:** dual-role (agumon/04), time-manip T1 (02_skill_koyosetsu.md). Qui nuovi.

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
turno T: Agumon casta baby_burner → CombatEvent::UltimateUsed { actor:Agumon }
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

## §4b — VFX (Forma C — single-channel variant)

`kitsune_grace` è **listener-only**, edge-event (`UltimateUsed` by ally). Nessuno stato persistente lato Renamon, nessun `Buff_*` da osservare via `Added/Removed`. Forma C qui collassa su **Channel 1 only** (nessun Channel 2).

| Momento | Channel | Trigger | Preset | Origin | Motion |
|---|---|---|---|---|---|
| **Grace flash** | 1 | `ctx.notify` su `CombatEvent::UltimateUsed { actor }`, filter `is_ally(actor) && !is_self(actor)` | `kitsune_grace_flash` (golden chime burst) | `SelfCenter` (Renamon) | `Static` |
| **Time-link** *(opt.)* | 1 | stesso trigger sopra | `kitsune_grace_link` (chime arc, "ruba tempo") | `EntityCenter(EventTarget)` (ally caster) | `Travel { to: EntityCenter(Self), ease: EaseOut, ms: 200 }` |

**Note implementative:**

- **Niente Channel 2.** L'effetto è istantaneo (single `AdvanceTurn` kernel effect). Nessuna aura persistente, nessun `Added/Removed` da osservare. Variante "edge-only" della Forma C — vedi K5.
- **`EntityRef::EventTarget`** mappa l'`actor` del `CombatEvent::UltimateUsed` → ally caster, naturale per il listener Channel 1 (allineato §2.2e §C tabella).
- **Travel self-ward.** Direzione `→ EntityCenter(Self)` (verso Renamon), opposto ai Travel link precedenti (Tentomon battery, Gabumon twin_core, che vanno outward). Semantica directional intenzionale: "Renamon ruba tempo dal caster". Niente nuovo verbo — `EntityRef::Self` esiste.
- **Headless gating.** Sia il listener Channel 1 che il listener kernel (`AdvanceTurn` emit) sono distinti: il kernel listener è gameplay-canonical (sempre on), il presentation listener è `#[cfg(feature = "windowed")]`.

## §5 — Open questions (nuovi)

1. **K1 — Self-Ult triggera kitsune_grace?**
   - **A.** Sì (Renamon ult charge sé stessa) → loop infinito se non capped. Cap 50% lo blocca, ma è UX strana.
   - **B.** No (escluso da filter) → identity §1 ("recupera tempo per riapplicare AoE") implica reagire **ad altri**.
   - **Decisione consigliata:** B. Filter `!ctx.is_self(actor)`.
2. **K2 — `UltimateUsed` event quando è emesso?** Al `commit_action(Ult)` o a `Ult.Strike.on_enter` (consumo bar)? **Coerenza:** allineare a "consumo bar" → solo se l'Ult andata davvero. Cancellazioni (es. invalid target) non triggerano.
3. **K3 — Bound check.** Cap 50% per call (T1). 10% × 5 alleati Ult in stesso round → 50%, ok. Reale: 1-2 ult/round max. Safe.
4. **K4 — Compatibilità con `Blessed`.** Blessed (03_ult_tohakken.md) buffa damage e Ult charge gen. Niente double-dip con `kitsune_grace`: K1 reagisce all'**uso** dell'ult, non al **charge**. Distinto.
5. **K5 — Forma C single-channel variant.** §2.2e §B (Forma C) presume coppia Channel 1 + Channel 2 (edge-flash + state-aura). `kitsune_grace` ha solo Channel 1 (listener edge-only, nessuno stato persistente da osservare). Action item: formalizzare in §2.2e §B "**Forma C ammette varianti: full (Ch1+Ch2), edge-only (Ch1 only), state-only (Ch2 only quando lo stato è già setup altrove); minimo 1 channel, max 2**". Gap N7 nuovo, trivial.
6. **K6 — Travel self-ward semantica directional.** Il `kitsune_grace_link` punta `to: EntityCenter(Self)` invece di `EntityCenter(EventTarget)`. Niente nuovo verbo grammatica (`EntityRef::Self` già definito). Solo nota convenzionale: la direction del Travel è **semantically meaningful** ("steal" vs "grant") e i preset designer devono poterlo scegliere liberamente. Annotare in §2.2d esempi: i Travel link cross-unit non sono sempre outward dal caster.

## §6 — Verdetto

`kitsune_grace` consolida:
- **Event canonico `UltimateUsed`** (verificare o aggiungere).
- **Listener emette `KernelEffect::AdvanceTurn`** — primo caso di listener che produce kernel effect (non solo buff/state mutation).
- **Forma C single-channel** (Channel 1 only, no state-aura) — variante listener-only edge-driven.

Pattern: listener può **emit kernel effect**, non solo applicare buff. Generalizzazione utile per altri passive futuri.
