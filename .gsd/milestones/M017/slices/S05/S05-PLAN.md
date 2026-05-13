# S05: Blessed — buff dealt + Ult charge + cleanse-immune

**Goal:** Wire two attacker-side Blessed hooks (dmg-dealt ×1.15 + Ult charge +1 per action) and lock in the cleanse-immune regression guard. Blessed is already a first-class StatusEffectKind classified as BuffKind::Buff (S02), so cleanse-immunity is structurally satisfied — S05 adds the offensive amp and Ult-charge bump in apply_effects/calculate_damage plus three DoD integration tests.
**Demo:** Test `status_blessed_offensive.rs`: unit Blessed colpisce → dmg ×1.15. Test `status_blessed_ult_charge.rs`: unit Blessed esegue azione → +1 Ult charge oltre baseline. Test `status_blessed_cleanse_immune.rs`: cleanse non rimuove Blessed.

## Must-Haves

- `cargo check` clean.
- `cargo test --test status_blessed_cleanse_immune` green (StatusBag::cleanse_debuffs preserves Blessed).
- `cargo test --test status_blessed_offensive` green (Blessed attacker deals round(base*tag*tri*break*1.15) vs control 1.0×).
- `cargo test --test status_blessed_ult_charge` green (Blessed attacker gains baseline+1 ult charge per non-Reset action; Reset action does not double-feed).
- Full `cargo test` suite remains green (no regressions in damage_tests, resolution_tests, follow_up_chains, combat_coherence).

## Proof Level

- This slice proves: integration — three new integration tests in `tests/` exercise the real damage and ult-charge pipelines through `apply_effects`; no fixture-only proofs.

## Integration Closure

Both `apply_effects` call sites in `src/combat/turn_system/pipeline.rs` (~lines 280, 576) are updated to pass the attacker's `&StatusBag` so the Blessed mult and +1 ult charge fire on real basics/skills/ultimates. No new entrypoints; the per-Digimon blueprint seam from M015 is untouched.

## Verification

- No new observability surfaces required — existing CombatEvent damage payloads naturally reflect the post-mult dmg. JSONL log/ValidationSnapshot canon naming is owned by S06.

## Tasks

- [x] **T01: Add cleanse-immune regression test for Blessed** `est:15m`
  Lock in the §H.1 cleanse-immune line for Blessed as a slice-level regression guard. S02 already wired BuffKind::Buff classification and cleanse_debuffs() excludes it (see status_effect.rs:42, 197-209). This task only adds a new test file matching the DoD-mandated name. Zero src/ changes.
  - Files: `tests/status_blessed_cleanse_immune.rs`
  - Verify: cargo test --test status_blessed_cleanse_immune

- [x] **T02: Thread attacker_dmg_mult through apply_effects and apply Blessed ×1.15** `est:1h`
  Add an `attacker_dmg_mult: f32` parameter to `calculate_damage` in `src/combat/damage.rs` (folded into the final product as another factor; default 1.0 from call sites without Blessed context). In `src/combat/resolution.rs::apply_effects` accept an `attacker_statuses: Option<&StatusBag>` and compute `1.15 if has Blessed else 1.0`, passing it to every `calculate_damage` call. Update the two `apply_effects` call sites in `src/combat/turn_system/pipeline.rs` (~280, ~576) to fetch the attacker StatusBag from the existing tuple and pass it through. Insert `None` at all other call sites in `tests/resolution_tests.rs` mechanically — they pass no buff context today. Add `tests/status_blessed_offensive.rs`: spawn attacker with/without Blessed, fire a Basic, assert the dmg event shows `round(base*tag*tri*break*1.15)` vs `round(base*tag*tri*break)`.
  - Files: `src/combat/damage.rs`, `src/combat/resolution.rs`, `src/combat/turn_system/pipeline.rs`, `tests/resolution_tests.rs`, `tests/damage_tests.rs`, `tests/status_blessed_offensive.rs`
  - Verify: cargo check && cargo test --test status_blessed_offensive && cargo test --test damage_tests && cargo test --test resolution_tests

- [x] **T03: Apply Blessed +1 Ult charge per action in apply_effects** `est:45m`
  In `src/combat/resolution.rs::apply_effects`, after the existing `match resolved.ult_effect` block, if `attacker_statuses` has `Blessed` AND `resolved.ult_effect != UltEffect::Reset` AND `outcome.succeeded`, call `attacker_ult.try_add(1)` once. Skipping on `Reset` avoids self-feeding the very Ult that is firing (research §Risks — confirm canon interpretation by inline comment citing §H.1). Reuse the `attacker_statuses: Option<&StatusBag>` parameter introduced in T02 — no additional plumbing at the pipeline call sites. Add `tests/status_blessed_ult_charge.rs`: (a) baseline (no Blessed) Basic action → record `attacker_ult.current` delta; (b) Blessed Basic action → assert delta is baseline+1; (c) Blessed Ultimate-cast action (Reset branch) → assert no +1 leak into the post-reset meter. Ensure starting charge is below cap so the +1 isn't clamped away.
  - Files: `src/combat/resolution.rs`, `tests/status_blessed_ult_charge.rs`
  - Verify: cargo test --test status_blessed_ult_charge && cargo test

## Files Likely Touched

- tests/status_blessed_cleanse_immune.rs
- src/combat/damage.rs
- src/combat/resolution.rs
- src/combat/turn_system/pipeline.rs
- tests/resolution_tests.rs
- tests/damage_tests.rs
- tests/status_blessed_offensive.rs
- tests/status_blessed_ult_charge.rs
