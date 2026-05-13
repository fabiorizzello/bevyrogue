---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: AV applicatore: cap ±50% + clamp [0, 2*MAX_AV], split Advance/Delay pure-logic

Riscrivere il livello di AV math come funzioni pure (no Bevy, no event bus): split `apply_advance(av, pct)` / `apply_delay(av, pct, resistance)` con cap interno ≤50 (defensive) e clamp finale [0, 20_000]. Floor 0 rimpiazza `MIN_ACTION_THRESHOLD_AV`. `TempoResistance` curva resta delay-only. `ActionValue::advance` ceiling sale da MAX_AV a 2*MAX_AV; `is_ready()` invariato (>= MAX_AV). Inline `#[cfg(test)] mod tests` con casi boundary basici.

## Inputs

- `src/combat/resistance.rs`
- `src/combat/av.rs`
- `.gsd/milestones/M018/slices/S01/S01-RESEARCH.md`

## Expected Output

- `src/combat/resistance.rs`
- `src/combat/av.rs`

## Verification

cargo check && cargo test --lib resistance && cargo test --test tempo_resistance (after T03 update); pure-logic boundaries verified via inline #[cfg(test)] tests in resistance.rs.
