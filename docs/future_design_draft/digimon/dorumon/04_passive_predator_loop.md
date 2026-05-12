# Dorumon — Passive: `predator_loop` (existing — tracking + state entry/exit)

> **Goal**: passive **già implementata** (`src/combat/blueprints/dorumon.rs::PredatorLoopState`, `PredatorLoopResolved` event). Allineamento del design doc al codice esistente; identificare gap se l'identity sheet diverge dal comportamento attuale.
>
> **Gap §2.2b condivisi:** dual-role (agumon/04). Memory note: "PredatorLoopState must explicitly track a target before Dorumon transitions are emitted in a headless runtime test". Qui solo nuovi gap.

## §1 — Intent

- **Tracking:** scan continuo lowest-HP% enemy alive; aggiorna `tracked_target`.
- **Entry:** quando `tracked_target.hp_pct < threshold` → `predator_active = true` per N turni.
- **Exit:** `tracked_target` muore (chain consumato in `draconic_edge`) o N turni expira.
- **Effect:** abilita edge A su `draconic_edge` (chain on kill); cambia threshold ult bonus a `<30%`.

## §2 — Blueprint contract

```rust
impl BlueprintListener for DorumonBlueprint {
    fn on_kernel_event(&self, ev: &CombatEvent, ctx: &mut ListenerCtx) {
        match ev {
            // tracking: ogni damage o death, ricomputa lowest
            CombatEvent::DamageDealt { target, .. }
            | CombatEvent::UnitDied { unit: target } => {
                self.state.recompute_tracked(ctx);
                if let Some(t) = self.state.tracked_target {
                    if ctx.unit_hp_pct(t) < self.config.entry_threshold {
                        self.state.predator_active = true;
                        self.state.expires_in = self.config.duration_turns;
                        ctx.emit_kernel_event(CombatEvent::PredatorLoopResolved {
                            tracked: t,
                            active: true,
                        });
                    }
                }
            }
            // turn-end tick: decrementa durata
            CombatEvent::TurnEnded { .. } if self.state.predator_active => {
                self.state.expires_in -= 1;
                if self.state.expires_in == 0 {
                    self.state.predator_active = false;
                }
            }
            _ => {}
        }
    }
}
```

(Pseudocode; reale è in `src/combat/blueprints/dorumon.rs`.)

## §3 — Activation flow

```
nemico subisce damage (qualsiasi caster)
  └─ DamageDealt event
     └─ predator_loop listener: recompute tracked = lowest-HP%
        └─ if tracked.hp_pct < threshold: predator_active = true (N turni)
           └─ emit PredatorLoopResolved event

Dorumon casta draconic_edge
  └─ edge A predicate: BlueprintState(predator_active) AND UnitDied(primary)
     └─ se entrambi: ChainStrike fires → consume state

oppure: ult heat_viper forza state on hit (vedi 03 F5)
```

## §4 — Trigger filter precision

- **Tracking scope:** EnemyTeam alive.
- **Entry:** condition gating, no manual override (eccetto F5 ult force).
- **Exit:** condizioni:
  - `tracked_target.died` ed era il target → reset state e ricalcola lowest.
  - `expires_in == 0` → state off.
  - Manual force off da blueprint? No (decisione: solo timeout o consume).
- **Memory note constraint:** PredatorLoopState **deve** tracciare un target prima di emettere transizioni. Headless test deve setup il tracking explicitly o il kernel rifiuta con `InvalidTarget`.

## §5 — Open questions (nuovi)

1. **G1 — Allineamento doc vs codice.** Identity §5 dice "Predator state" e "Exit: target tracked muore o timeout turni". Verificare in `src/combat/blueprints/dorumon.rs` se entrambe le condizioni di exit sono implementate o solo una. **Action item:** se manca, aggiungere; se non manca, sync identity.
2. **G2 — UI visibility (identity §6).** "Predator state visibile in UI?" — HSR debuff badge sul tracked target. Out of scope M017 ma documentare hook event `PredatorLoopResolved` come signal per UI.
3. **G3 — Threshold value.** Identity §5 dice "X%". Codice esistente probabilmente ha valore default (es. 50%). Confermare e documentare nel config.
4. **G4 — Force-state via Ult (F5) interagisce con timeout?** Se Ult forza `predator_active=true` con `dur:N`, e tracking lowest-HP% già attivo con `dur:M`, qual è la durata finale? Max-replace? Refresh? **Decisione consigliata:** max(N, M) — più generoso al player.
5. **G5 — Chain interaction con Twin Core / Heated / Chilled.** Heat Viper interaction (identity §6): "Heat Viper interaction con Twin Core / status altrui — bonus o trasparente?" **Decisione consigliata:** trasparente. Dark damage non scala su Heated/Chilled di Agumon/Gabumon. Mantiene Dorumon **single-target executor pure**, non status-dipendente. Niente sinergie cross-roster.

## §6 — Verdetto

`predator_loop` è il **template "blueprint state machine listener"** del roster:
- Listener mantiene state interno (`PredatorLoopState`).
- State è interrogabile via predicate (`BlueprintState`, F2).
- State è mutabile via Command (`SetBlueprintState`, F5).
- Eventi del kernel (`DamageDealt`, `UnitDied`, `TurnEnded`) sono trigger.

Pattern **generalizzabile** ad altri Digimon che richiedono state machine interna (es. futuri form-change Renamon/Kyubimon, evoluzioni). Vocabolario `BlueprintState` + `SetBlueprintState` formalizza il contratto.

**Allineamento doc-codice è action item pratico**, non gap architetturale.
