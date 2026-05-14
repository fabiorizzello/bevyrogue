---
estimated_steps: 1
estimated_files: 5
skills_used: []
---

# T03: Pipeline Slowed migration + skills.ron + UI windowed + M017 regression tests

Pipeline branch Slowed in `src/combat/turn_system/pipeline.rs:758` emette ora `DelayTurn{amount_pct: 30}` invece del signed `TurnAdvance{-30}`. Stesso refactor per le copie del match arm a `pipeline.rs:359` e `:666` (log push). Migrare `assets/data/skills.ron`: ogni `TurnAdvance(N)` → `AdvanceTurn(N)`; ogni `TurnAdvance(-N)` → `DelayTurn(N)`. Aggiornare `src/ui/combat_panel.rs:649` (feature `windowed`) per match sui due nuovi `LogEntry::AdvanceTurn` / `DelayTurn`. Aggiornare `tests/status_slowed_delay.rs` e `tests/tempo_resistance.rs` ai nuovi event variant + invariant floor 0 (rimuovere/rinominare `apply_av_change_clamps_to_min_action_threshold` se necessario, mantenendo coverage equivalente).

## Inputs

- `src/combat/turn_system/pipeline.rs`
- `assets/data/skills.ron`
- `src/ui/combat_panel.rs`
- `tests/status_slowed_delay.rs`
- `tests/tempo_resistance.rs`
- `src/combat/events.rs`
- `src/combat/log.rs`

## Expected Output

- `src/combat/turn_system/pipeline.rs`
- `assets/data/skills.ron`
- `src/ui/combat_panel.rs`
- `tests/status_slowed_delay.rs`
- `tests/tempo_resistance.rs`

## Verification

cargo check && cargo check --features windowed && cargo test --test status_slowed_delay && cargo test --test tempo_resistance — entrambi verdi con event match aggiornato ma AV outcome (5000→2000) invariato.
