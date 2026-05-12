# Gabumon — Passive: `fur_cloak` (listener-only)

> **Goal**: passive **self-targeting** triggered on outgoing `StatusApplied`. Mirror funzionale ma asse opposto al `twin_core_fire` di Agumon (che è **outgoing damage scaling** vs `fur_cloak` = **incoming damage mitigation**). Insieme al Twin Core ice-side definisce il dual-listener Gabumon.
>
> **Gap §2.2b condivisi:** dual-role (vedi agumon/04 §6), pre-damage vs post-event (G9), RoundId (G10). Qui solo nuovi.

## §1 — Intent

- **Direzione:** self-mitigation reattiva. Quando Gabumon applica Chilled (qualsiasi target), arma DR self.
- **Effect:** `BuffSelf { id:"fur_cloak_dr", value:0.20, dur:1 }` — −20% damage taken, dura 1 turno (fino al prossimo damage incoming risolto, o tier end).
- **Scope:** listener-only. No FSM extra.

## §2 — Blueprint contract

```rust
impl BlueprintListener for GabumonBlueprint {
    fn on_kernel_event(&self, ev: &CombatEvent, ctx: &mut ListenerCtx) {
        match ev {
            CombatEvent::StatusApplied { caster, status: Chilled, .. }
                if ctx.is_self(caster) && self.fur_cloak_arming_allowed() => {
                ctx.add_self_buff(BuffId("fur_cloak_dr"), value: 0.20, dur: Turns(1));
            }
            _ => {}
        }
    }
}

// Twin Core ice-side (file separato logico, stesso blueprint)
impl BlueprintListener {
    // on StatusApplied(Heated, caster:Agumon) → arma TwinCoreIceActive su self
    // Vedi agumon/04 specchio
}
```

## §3 — Activation flow

```
turno T: Gabumon casta bubble_blast → EmitStatus(Chilled, primary)
  └─ CombatEvent::StatusApplied { caster:Gabumon, target:primary, status:Chilled }
     └─ fur_cloak listener cattura → add_self_buff(fur_cloak_dr, dur:1)

turno T+1 (nemico): nemico colpisce Gabumon
  └─ damage pipeline legge has_buff(fur_cloak_dr) → multiplier (1 − 0.20) = 0.80
  └─ DamageDealt(Gabumon, scaled)
  └─ buff decrementa / expira a end_of_owner_next_turn
```

## §4 — Trigger filter precision

- **Caster check:** `caster == self` (Gabumon stesso). Non Tentomon che applica Paralyzed, non altri.
- **Status check:** `status == Chilled`. Strict.
- **Basic vs Skill vs Ult:** basic applica Chilled. Identity §1 propone **"DR-self on apply"** sempre, ma 04 dell'01-basic_horn_strike domanda se gate solo su skill/ult. **Decisione operativa:** listener triggera **sempre** (semplicità >> ottimizzazione game-feel). Bilanciare via durata DR (1 turno = 1 colpo subito mitigato), non via gate trigger.
- **Cap:** single instance attiva, refresh durata.

## §5 — Power tuning placeholder

- **Value:** −20% damage taken (allineato a identity §4).
- **Durata:** 1 turno (= fino al prossimo end-of-own-turn).
- **Interazione con Ult `arctic_torrent` DR 30%:** vedi gap S2 (gabumon/03 §5.3). Replace-max consigliato.

## §6 — Stress test findings (nuovi)

### ✅ Cosa funziona

- Stesso pattern dual-role di agumon/04: listener vive in Rust, niente RON. **Riusabile** come template per passive listener-only del roster.

### ⚠️ Gap nuovi

1. **N1 — Buff "self-targeting" require nuovo verbo o riusa `ApplySelfBuff` di gabumon/03 §5.S2.** Decisione: **stesso verbo** (`ApplySelfBuff` o equivalente). Listener path lo invoca direttamente (non via FSM Commands), ma il kernel effect è uniforme.
2. **N2 — Buff timing race con FSM owner.** Il listener applica buff **mentre Gabumon è ancora in `Burst` FSM**. Il buff è "attivo" già durante il resto della FSM corrente? **Decisione:** sì (state read live), ma il primo damage incoming è del nemico al turno successivo → effetto pratico = mitigation nel turno T+1.
3. **N3 — Twin Core ice-side (specchio Agumon).** Stesso blueprint contiene 2 listener path: `fur_cloak` (self) + `twin_core_ice` (cross). **Action item:** decidere se file separato `04b_passive_twin_core_ice.md` o tutto qui. **Decisione operativa:** tutto qui. Il blueprint Gabumon ha 2 reactive arm:
   - `fur_cloak`: on outgoing Chilled → self DR
   - `twin_core_ice`: on `StatusApplied(Heated, caster:Agumon)` → BuffSelf("twin_core_ice_active", round-scoped) che boosta Ice damage `+15%` next outgoing.
4. **N4 — `value_param` schema buff.** Vedi gap S2 (gabumon/03). Buff `value` (mult) vs status `stacks` (count) sono entità distinte nel vocabolario. Cleanse Patamon non li tocca (è solo debuff filter).

## §7 — Verdetto

`fur_cloak` + `twin_core_ice` definiscono il **template buff-applier listener** del roster:
- Self-buff (fur_cloak) e cross-buff (twin_core) condividono lo stesso effect path (`ApplySelfBuff`).
- Cleanse-immune by design (buff, non debuff).
- Listener filter discrimina caster/status precisamente.

**Nessun gap architetturale nuovo oltre quelli ereditati.** Il design pattern è solido; il gap reale è S2 (verbo `ApplySelfBuff` formalizzato).
