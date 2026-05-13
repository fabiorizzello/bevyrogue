---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T04: Boundary test suite: tests/turn_advance_split.rs

Nuovo file di integration test (naming funzionale, vedi CLAUDE.md). Casi deterministici headless: (a) `DelayTurn(80)` enforcement → cap a 50 → AV change `-5000` con AV iniziale `MAX_AV` → AV 5000; (b) `AdvanceTurn(80)` → cap 50 → AV change `+5000`; (c) doppio `AdvanceTurn(50)` su AV=10_000 → AV 20_000 (ceiling); (d) terzo `AdvanceTurn(50)` non muove AV oltre 20_000 (no overflow); (e) `DelayTurn(50)` su AV=2000 → AV 0 (no negative); (f) `DelayTurn(50)` su boss con `TempoResistance(0.25)` riduce raw delay del 75% (curva preservata); (g) cap NON applicato a livello evento — verificare che l'event emesso ha già `amount_pct ≤ 50`. Usare `Messages::get_cursor_current()` pattern come in `status_slowed_delay.rs:158`.

## Inputs

- `src/combat/events.rs`
- `src/combat/resistance.rs`
- `src/combat/av.rs`
- `src/combat/resolution.rs`
- `tests/status_slowed_delay.rs`

## Expected Output

- `tests/turn_advance_split.rs`

## Verification

cargo test --test turn_advance_split — tutti i casi verdi, deterministico su 3 run consecutivi.
