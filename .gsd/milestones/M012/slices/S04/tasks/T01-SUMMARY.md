---
id: T01
parent: S04
milestone: M012
key_files:
  - src/data/skills_ron.rs
  - assets/data/skills.ron
  - src/combat/action_query.rs
  - src/combat/mod.rs
  - tests/action_affordance_query.rs
key_decisions:
  - Use a borrowed `ActionQueryKind<'a>::Skill(&'a SkillDef)` so the query vocabulary stays pure and DSL-driven without skill-ID tables.
  - Model enabled/disabled/deferred/hidden as reason-bearing status enums so later slices can return stable machine-readable codes instead of display strings.
  - Make `TargetHpRule::Any` explicit in canonical RON while keeping the Rust-side `SkillTargeting` default backward-compatible for existing fixtures.
duration: 
verification_result: mixed
completed_at: 2026-05-01T06:46:13.744Z
blocker_discovered: false
---

# T01: Added pure action-query vocabulary, damaged-target DSL rule, and canonical RON defaults for legality preflight.

**Added pure action-query vocabulary, damaged-target DSL rule, and canonical RON defaults for legality preflight.**

## What Happened

Implemented the contract-first vocabulary for the upcoming legality query layer. `src/data/skills_ron.rs` now exposes `TargetHpRule::{Any, Damaged}`, extends `LegalityReasonCode` with the missing preflight reasons, and gives `SkillTargeting` a defaulted `target_hp_rule` so canonical data can spell out the default explicitly in `assets/data/skills.ron`. Added the pure `src/combat/action_query.rs` module with the snapshot, affordance, status, and toughness types the later query logic will use, exported it from `src/combat/mod.rs`, and added `tests/action_affordance_query.rs` to pin the public vocabulary and fixture shape. Because the new `SkillTargeting` field is now part of the struct contract, I also updated existing inline `SkillTargeting` fixtures across the codebase to opt into `..Default::default()` for the new field without changing their existing semantics.

## Verification

Task-specific verification passed: `cargo test-dev --test action_affordance_query` passed (3/3 tests), `cargo test-dev skills_ron` passed (canonical DSL round-trip/validation suite), and `cargo test-dev --test target_shape_truthfulness --test revive_semantics --test toughness_enemy_only --test skill_legality_contract_docs` passed (contract and legality regressions). Full-suite verification `cargo test-dev` still fails in existing `tests/form_identity.rs` with 8 unrelated assertions about Form Identity energy/turn-advance behavior; that failure reproduces in isolation with `cargo test-dev --test form_identity` and appears outside the scope of this slice.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test-dev --test action_affordance_query` | 0 | ✅ pass | 7500ms |
| 2 | `cargo test-dev skills_ron` | 0 | ✅ pass | 12900ms |
| 3 | `cargo test-dev --test target_shape_truthfulness --test revive_semantics --test toughness_enemy_only --test skill_legality_contract_docs` | 0 | ✅ pass | 7100ms |
| 4 | `cargo test-dev` | 101 | ❌ fail | 6300ms |

## Deviations

Had to mass-update existing `SkillTargeting` literals across tests and source to preserve compilation after adding the new DSL field.

## Known Issues

`cargo test-dev` currently fails in `tests/form_identity.rs` (8 failing assertions) even when run in isolation; this appears unrelated to the action-query slice and was not changed here.

## Files Created/Modified

- `src/data/skills_ron.rs`
- `assets/data/skills.ron`
- `src/combat/action_query.rs`
- `src/combat/mod.rs`
- `tests/action_affordance_query.rs`
