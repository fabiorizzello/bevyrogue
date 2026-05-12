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
turno T: Gabumon casta gabumon_shot → EmitStatus(Chilled, primary)
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
- **Basic vs Skill vs Ult:** basic applica Chilled. Identity §1 propone **"DR-self on apply"** sempre, ma 04 dell'01-basic_claw_attack domanda se gate solo su skill/ult. **Decisione operativa:** listener triggera **sempre** (semplicità >> ottimizzazione game-feel). Bilanciare via durata DR (1 turno = 1 colpo subito mitigato), non via gate trigger.
- **Cap:** single instance attiva, refresh durata.

## §5 — Power tuning placeholder

- **Value:** −20% damage taken (allineato a identity §4).
- **Durata:** 1 turno (= fino al prossimo end-of-own-turn).
- **Interazione con Ult `blue_cyclone` DR 30%:** vedi gap S2 (gabumon/03 §5.3). Replace-max consigliato.

## §5b — Presentation (Forma C, §2.2e)

Passive listener-only ⇒ **no FSM**, **no clip dedicata**: Gabumon resta su `idle` in ogni momento dell'arming/aura. La presentation è interamente VFX, splittata sui due path del blueprint dual-role.

### Path A — `fur_cloak` (self DR su outgoing Chilled)

| # | Momento | Channel | Trigger | Preset | Origin | Motion |
|---|---|---|---|---|---|---|
| A1 | Arm flash | 1 | `ctx.notify` da listener su `StatusApplied{status:Chilled, caster:self}` | `fur_cloak_arm` (azure shimmer pop) | `SelfCenter` | `Static` |
| A2 | Active aura | 2 | `Added<Buff_FurCloakDR>` su Gabumon | `fur_cloak_loop` (subtle azure body shimmer) | `SelfCenter` | `Static` |
| A3 | Damage absorbed flash | 1 | `ctx.notify` da listener su `DamageDealt{target:self}` AND `has_buff("fur_cloak_dr")` | `fur_cloak_absorb` (azure spark burst) | `SelfCenter` | `Static` |
| A4 | Aura despawn | 2 | `RemovedComponents<Buff_FurCloakDR>` | — (emitter manager despawned) | — | — |

### Path B — `twin_core_ice` (cross-buff su `StatusApplied{Heated, caster:Agumon}`)

| # | Momento | Channel | Trigger | Preset | Origin | Motion |
|---|---|---|---|---|---|---|
| B1 | Arm flash | 1 | `ctx.notify` da listener su `StatusApplied{status:Heated, caster:agumon}` | `twin_core_ice_ignite` (dual frost-pop) | `SelfCenter` | `Static` |
| B2 | Partner link *(opzionale)* | 1 | stesso trigger di B1 | `twin_core_ice_link` (frost arc verso Agumon) | `SelfCenter` | `Travel { to: EntityCenter(Caster), ease: EaseOut, ms: 200 }` |
| B3 | Active aura | 2 | `Added<Buff_TwinCoreIceActive>` su Gabumon | `twin_core_ice_loop` (subtle frost halo) | `SelfCenter` | `Static` |
| B4 | Boosted hit overlay | 1 | `ctx.notify` da listener su `DamageDealt{caster:self, tag:Ice}` AND buff present | `twin_core_ice_amplify` (frost burst overlay) | `EntityCenter(EventTarget)` | `Static` |
| B5 | Dissipate flash | 1 | `ctx.notify` su `RoundEnded` (buff drop) | `twin_core_ice_dissipate` | `SelfCenter` | `Static` |
| B6 | Aura despawn | 2 | `RemovedComponents<Buff_TwinCoreIceActive>` | — | — | — |

### Note implementative

- **Tag components convenzionali (§2.2e §E):**
  - `Buff_FurCloakDR` — **tag-pure** (presence/absence). Il valore numerico (mult 0.20) resta nella `Buffs` stringy map; l'observer Channel 2 legge solo presence. Vedi gap N5.
  - `Buff_TwinCoreIceActive` — tag-pure (boost +10% gestito via lookup buff stringy nella damage pipeline; allineato a `00_identity.md §6 D3`).
- **`EntityRef::Caster` / `EntityRef::EventTarget`** sono ammessi solo in Channel 1 (listener context, §2.2e §C). Non possono essere usati da observer Channel 2 — che non ha event scope.
- **A3 (`fur_cloak_absorb`)** è guard'd dal listener su `has_buff("fur_cloak_dr")` — il preset si attiva solo quando il DR effettivamente assorbe.
- **B5 dissipate flash** vive su Channel 1 e va emesso **prima** del Removed event che chiude B6 (ordering interno listener vs ECS observer su RoundEnded). Vedi gap N5.5.
- **Headless gating:** entrambi i path sono presentation-only. In `headless` mode (`#[cfg(not(feature = "windowed"))]`) i `ctx.notify` sono no-op e gli observer Channel 2 non sono registrati; il gameplay (buff applicato, multiplier letto in damage pipeline) resta intatto.

## §6 — Stress test findings (nuovi)

### ✅ Cosa funziona

- Stesso pattern dual-role di agumon/04: listener vive in Rust, niente RON. **Riusabile** come template per passive listener-only del roster.

### ⚠️ Gap nuovi

1. **N1 — Buff "self-targeting".** ✅ **Chiuso (round-3, 2026-05-12):** alias di `ApplyBuff { target: Self_, kind: DR }` (vedi `02-02b §C2` + `02-08 §H`). Il listener path lo invoca direttamente via `ctx.add_self_buff(...)` (non via FSM Commands), ma il kernel effect è uniforme: stesso `kind`, stesso stacking (intra-unit replace-max). Nessun verbo separato.
2. **N2 — Buff timing race con FSM owner.** Il listener applica buff **mentre Gabumon è ancora in `Burst` FSM**. Il buff è "attivo" già durante il resto della FSM corrente? **Decisione:** sì (state read live), ma il primo damage incoming è del nemico al turno successivo → effetto pratico = mitigation nel turno T+1.
3. **N3 — Twin Core ice-side (specchio Agumon).** Stesso blueprint contiene 2 listener path: `fur_cloak` (self) + `twin_core_ice` (cross). **Action item:** decidere se file separato `04b_passive_twin_core_ice.md` o tutto qui. **Decisione operativa:** tutto qui. Il blueprint Gabumon ha 2 reactive arm:
   - `fur_cloak`: on outgoing Chilled → self DR
   - `twin_core_ice`: on `StatusApplied(Heated, caster:Agumon)` → BuffSelf("twin_core_ice_active", round-scoped) che boosta Ice damage `+10%` next outgoing (canon `00_identity.md §6 D3`, simmetrico con Agumon Twin Core).
4. **N4 — `value_param` schema buff.** Vedi gap S2 (gabumon/03). Buff `value` (mult) vs status `stacks` (count) sono entità distinte nel vocabolario. Cleanse Patamon non li tocca (è solo debuff filter).
5. **N5 — Value-carrying typed-buff component.** ✅ **Chiuso (round-3, 2026-05-12):** decisione A (tag-pure marker) **formalizzata** in `02-02e §E.1` (channel layout) come convenzione canon del roster. `Buff_FurCloakDR` è zero-sized tag (presence-only); il valore numerico vive nella `Buffs` stringy map letta dalla damage pipeline. Observer Channel 2 osserva solo `Added/Removed` per gestire emitter lifecycle. Numeric binding ("VFX intensity proporzionale al mult") resta deferred a `§2.2e §I`.
6. **N5.5 — Ordering Channel 1 vs Channel 2 su buff drop.** ✅ **Chiuso (round-3, 2026-05-12):** ordering **non-normativo**, overlap di ≤1 frame tollerato — codificato in `02-02e §F` (Channel arbitration rules). Sia "flash prima del Removed" sia "flash dopo il Removed" sono accettabili: i due path vivono su channel disgiunti e il player non distingue ≤1f di drift.
7. **N6 — Observer multi-effect (anim + particle + audio).** *Deferred*: codice corrente (idle puro, niente clip dedicata su Added) non richiede observer Channel 2 di emettere `AnimClipPlay`. Gap resta aperto in `02-02e §I` per il primo caso reale (non Gabumon).

## §7 — Verdetto

`fur_cloak` + `twin_core_ice` definiscono il **template buff-applier listener** del roster:
- Self-buff (fur_cloak) e cross-buff (twin_core) condividono lo stesso effect path (`ApplySelfBuff`).
- Cleanse-immune by design (buff, non debuff).
- Listener filter discrimina caster/status precisamente.
- **Presentation Forma C completa:** Channel 1 per eventi puntuali (arm/absorb/dissipate/boosted-hit), Channel 2 per aura state-bound (Added/Removed di buff tag-component). Niente clip anim dedicata — idle puro + VFX.

**Gap nuovi esposti (status post round-3):** N1 ✅ chiuso (alias `ApplyBuff { target: Self_, kind: DR }`, `02-02b §C2`). N5 ✅ chiuso (tag-pure marker formalizzato, `02-02e §E.1`). N5.5 ✅ chiuso (ordering non-normativo, `02-02e §F`). N6 deferred a `02-02e §I` (primo caso reale). S2 ✅ chiuso (alias `ApplyBuff`, vedi gabumon/03 §5).
