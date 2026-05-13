---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T03: Apply Blessed +1 Ult charge per action in apply_effects

In `src/combat/resolution.rs::apply_effects`, after the existing `match resolved.ult_effect` block, if `attacker_statuses` has `Blessed` AND `resolved.ult_effect != UltEffect::Reset` AND `outcome.succeeded`, call `attacker_ult.try_add(1)` once. Skipping on `Reset` avoids self-feeding the very Ult that is firing (research §Risks — confirm canon interpretation by inline comment citing §H.1). Reuse the `attacker_statuses: Option<&StatusBag>` parameter introduced in T02 — no additional plumbing at the pipeline call sites. Add `tests/status_blessed_ult_charge.rs`: (a) baseline (no Blessed) Basic action → record `attacker_ult.current` delta; (b) Blessed Basic action → assert delta is baseline+1; (c) Blessed Ultimate-cast action (Reset branch) → assert no +1 leak into the post-reset meter. Ensure starting charge is below cap so the +1 isn't clamped away.

## Inputs

- `src/combat/resolution.rs`
- `src/combat/ultimate.rs`
- `src/combat/status_effect.rs`
- `.gsd/milestones/M017/slices/S05/S05-RESEARCH.md`

## Expected Output

- `src/combat/resolution.rs`
- `tests/status_blessed_ult_charge.rs`

## Verification

cargo test --test status_blessed_ult_charge && cargo test
