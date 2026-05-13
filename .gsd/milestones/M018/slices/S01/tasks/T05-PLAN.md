---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T05: CLI scenario advance-delay-cap + full-suite regression gate

Estendere `src/bin/combat_cli.rs` con scenario `--scenario advance-delay-cap`: spawna 2 unità, esegue una sequenza scriptata (e.g. AdvanceTurn(50), AdvanceTurn(50), DelayTurn(80), DelayTurn(50)) e stampa AV gauge step-by-step pre/post + JSONL one-entry-per-applicazione (event kind + amount_pct capped + AV pre/post). Eseguire `cargo test` full suite come gate finale: zero regressioni sui 40 binari esistenti, status taxonomy M017 invariata, TTK scenarios stabili.

## Inputs

- `src/bin/combat_cli.rs`
- `src/combat/events.rs`
- `src/combat/jsonl_logger.rs`

## Expected Output

- `src/bin/combat_cli.rs`

## Verification

cargo run --bin combat_cli -- --scenario advance-delay-cap (esit 0 + JSONL leggibile su stdout con cap visibile); cargo test full passes; cargo check --features windowed passes.
