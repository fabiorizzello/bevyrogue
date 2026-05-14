# M020: Reactive bus uniforme + shim removal

**Vision:** Completare il bus eventi combat con i variant mancanti (UltimateUsed, UnitDied con payload) ed eliminare i tre re-export shim Digimon-specific da `src/combat/mod.rs`, così che M021 trovi il kernel già privo di alias legacy e il bus già informativo per ogni listener downstream.

## Success Criteria

- CombatEventKind::UltimateUsed emesso ogni volta che un'unità usa la ultimate; listener esistenti non rotti
- OnKO rinominato UnitDied con campi status_remaining: Vec<StatusEffectKind> e heated_remaining: u32; tutti i siti emit/match aggiornati
- I tre pub use shim (twin_core, holy_support, predator_loop) rimossi da combat/mod.rs; tutti i siti d'importazione aggiornati ai path blueprint canonici
- cargo test green (72+ test)
- cargo check headless e windowed senza warning nuovi

## Slices

- [ ] **S01: S01** `risk:low` `depends:[]`
  > After this: cargo test passa con un nuovo test che verifica UltimateUsed emesso e un test che verifica UnitDied porta i campi corretti

- [ ] **S02: Rimozione shim pub use legacy (twin_core / holy_support / predator_loop)** `risk:low` `depends:[S01]`
  > After this: cargo test passa; grep ricorsivo mostra zero occorrenze di combat::twin_core, combat::holy_support, combat::predator_loop fuori dai file blueprint stessi

## Boundary Map

Not provided.
