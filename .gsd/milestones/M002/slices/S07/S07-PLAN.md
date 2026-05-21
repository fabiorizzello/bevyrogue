# S07: Energy-backed ult gauge runtime migration (Agumon)

**Goal:** Chiudere il loop energy-backed di Agumon nel runtime: ult readiness e drain devono leggere/scrivere il gauge effettivo da UltGaugeMetadata + Energy invece di UltimateCharge, lasciando i Digimon non-opted-in (metadata vuota) invariati sul path legacy.
**Demo:** cargo run --features windowed --bin bevyrogue — barra ult Agumon sale solo da energy, Ultimate si abilita esattamente quando energy=max, fire ult azzera la barra

## Must-Haves

- "Ultimate" si abilita per Agumon esattamente quando Energy.current >= Energy.max, non prima.
- Cast dell'ultimate Agumon azzera Energy.current (oltre a UltimateCharge per back-compat).
- Digimon senza metadata (Gabumon, Patamon, Dorumon, ecc.) restano sul gauge legacy UltimateCharge e non rompono test esistenti.
- Test dedicato Agumon energy-loop passa: basic riempie energy, ult resta lock finche energy<max, ult drena energy.
- cargo test complessivo verde su windowed_only, assets_data, action_query, damage_resolution, bootstrap_encounter, digimon_kits.

## Proof Level

- This slice proves: integration

## Integration Closure

Slice chiusa quando il loop Agumon basic->energy->ult->drain e osservabile sia headless (test integration) sia windowed (demo manuale), senza regressioni sui Digimon legacy.

## Verification

- Nessun nuovo log/telemetria; il drain energy verra coperto dal CombatEvent::EnergyDrained se gia emesso, altrimenti restiamo sui contatori esistenti. Il test integration e la spia primaria.

## Tasks

- [x] **T01: Plumbing UltGaugeMetadata + Energy in snapshot/query** `est:M`
  Estendere ResolveActorsQuery/UnitQuerySnapshot per esporre Option<UltGaugeMetadata> e Option<Energy>; aggiornare build_snapshot_from_ecs_with_sp e tutti i callsite units_data (UI combat panel, CLI bootstrap/dashboard, preview cache, follow_up). Non cambia ancora il comportamento - solo plumbing.
  - Files: `src/combat/action_query/types.rs`, `src/combat/action_query/mod.rs`, `src/combat/turn_system/types.rs`, `src/ui/combat_panel/mod.rs`, `src/ui/combat_panel/render.rs`, `src/bin/combat_cli/bootstrap.rs`, `src/bin/combat_cli/dashboard.rs`, `src/combat/preview.rs`
  - Verify: cargo check --features windowed && cargo test --features windowed --test action_query --test windowed_only

- [x] **T02: Ult readiness via effective_ult_gauge** `est:M`
  In legality/action.rs, quando snapshot.gauge_meta indica energy-backed, calcolare readiness ult da effective_ult_gauge(meta, energy, ult) invece di leggere actor.ultimate_ready. Aggiornare anche resources.rs per ResourceKind::Ultimate details (current/max/ready). Path legacy invariato quando metadata vuota.
  - Files: `src/combat/action_query/legality/action.rs`, `src/combat/action_query/legality/resources.rs`, `src/combat/action_query/legality/mod.rs`
  - Verify: cargo test --features windowed --test action_query

- [x] **T03: Drain Energy su UltEffect::Reset per energy-backed** `est:M`
  In resolution/apply.rs e turn_system/pipeline/timeline_exec.rs (finalize path), quando l'attacker e energy-backed (metadata = energy) azzerare anche Energy.current oltre a attacker_ult.current. Mantenere UltimateCharge=0 per back-compat finche legacy non viene smantellato.
  - Files: `src/combat/resolution/apply.rs`, `src/combat/turn_system/pipeline/timeline_exec.rs`
  - Verify: cargo test --features windowed --test damage_resolution --test windowed_only

- [x] **T04: Test integrazione Agumon energy-loop** `est:S`
  Nuovo test in tests/digimon_kits/ che esercita il loop completo headless: spawn Agumon vs dummy, lancia sharp_claws N volte, verifica energy sale, ult resta locked finche energy<max, ult si abilita a energy=max, lancia ult, verifica energy.current==0 dopo cast. Usa harness esistenti e seeded RNG (R004).
  - Files: `tests/digimon_kits/agumon_energy_gauge.rs`, `tests/digimon_kits.rs`
  - Verify: cargo test --features windowed --test digimon_kits agumon_energy_gauge

- [x] **T05: Fixture sweep UnitQuerySnapshot callsite nei test** `est:M`
  Aggiornare le fixture e i test esistenti che costruiscono UnitQuerySnapshot a mano o hard-codano la tuple units_data per il nuovo shape (gauge_meta + energy opzionali). Mantenere None/None nelle fixture legacy per non opt-in nessun comportamento nuovo.
  - Files: `tests/action_query/action_affordance_consumers.rs`, `tests/action_query/action_affordance_query.rs`
  - Verify: cargo test --features windowed

## Files Likely Touched

- src/combat/action_query/types.rs
- src/combat/action_query/mod.rs
- src/combat/turn_system/types.rs
- src/ui/combat_panel/mod.rs
- src/ui/combat_panel/render.rs
- src/bin/combat_cli/bootstrap.rs
- src/bin/combat_cli/dashboard.rs
- src/combat/preview.rs
- src/combat/action_query/legality/action.rs
- src/combat/action_query/legality/resources.rs
- src/combat/action_query/legality/mod.rs
- src/combat/resolution/apply.rs
- src/combat/turn_system/pipeline/timeline_exec.rs
- tests/digimon_kits/agumon_energy_gauge.rs
- tests/digimon_kits.rs
- tests/action_query/action_affordance_consumers.rs
- tests/action_query/action_affordance_query.rs
