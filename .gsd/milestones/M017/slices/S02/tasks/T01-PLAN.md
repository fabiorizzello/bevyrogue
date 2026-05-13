---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T01: StatusBag + BuffKind types and policy

## Inputs

- None specified.

## Expected Output

- `src/combat/status_effect.rs`

## Verification

Inline `#[cfg(test)] mod tests` in `src/combat/status_effect.rs` covers: refresh-max-dur math, multi-kind coexistence, classify_buff_kind totality, cleanse_debuffs leaving Blessed intact. Run `cargo test --lib combat::status_effect` (the rest of the tree will not compile until T02-T04, which is expected).
