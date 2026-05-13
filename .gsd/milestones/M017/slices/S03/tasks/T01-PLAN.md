---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: status_amp_pct lookup helper + unit tests

Add a pure lookup `status_amp_pct(bag: &StatusBag, tag: DamageTag) -> i32` in `src/combat/status_effect.rs` returning 115 when (Heated && tag=Fire) or (Chilled && tag=Ice), else 100. Zero coupling to damage/turn pipelines. Covers the canon §H.1 amp% rule for Heated/Chilled. Add 4 unit tests in the existing `#[cfg(test)] mod tests` block: non-Heated→100, Heated+Fire→115, Heated+Ice→100 (wrong tag), Chilled+Ice→115. Skills: tdd, verify-before-complete.

## Inputs

- `.gsd/milestones/M017/slices/S03/S03-RESEARCH.md`
- `src/combat/status_effect.rs`
- `src/combat/types.rs`

## Expected Output

- `src/combat/status_effect.rs`

## Verification

cargo test combat::status_effect::tests::status_amp -- --nocapture && cargo check
