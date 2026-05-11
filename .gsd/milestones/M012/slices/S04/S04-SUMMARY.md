---
id: S04
parent: M012
milestone: M012
provides:
  - A machine-readable preflight contract for future CLI/windowed adapters and engine validation.
  - Stable action/target/resource/implementation/toughness reason codes for downstream slices.
requires:
  []
affects:
  []
key_files:
  - src/data/skills_ron.rs
  - assets/data/skills.ron
  - src/combat/action_query.rs
  - src/combat/mod.rs
  - tests/action_affordance_query.rs
  - .gsd/PROJECT.md
key_decisions:
  - Expose legality as a pure query surface with stable status/reason enums instead of display-string parsing.
  - Model `TargetHpRule::Any` explicitly in canonical RON while keeping Rust defaults backward-compatible.
  - Use helper-backed toughness visibility and per-target affordance output so target legality and truthful UI surfaces travel together.
  - Preserve ID-addressable snapshots alongside legacy acting/target fields for compatibility while keeping query evaluation deterministic.
patterns_established:
  - DSL-backed legality should be queried from immutable snapshots, not inferred from skill IDs or ECS mutation.
  - Truthful affordance APIs should return both the blocker reason and the inspectable per-target/resource surfaces in one call.
observability_surfaces:
  - Pure query return values act as the diagnostic surface for legality, resource readiness, and toughness truthfulness.
drill_down_paths:
  - .gsd/milestones/M012/slices/S04/tasks/T01-SUMMARY.md
  - .gsd/milestones/M012/slices/S04/tasks/T02-SUMMARY.md
  - .gsd/milestones/M012/slices/S04/tasks/T03-SUMMARY.md
  - .gsd/milestones/M012/slices/S04/tasks/T04-SUMMARY.md
duration: ""
verification_result: passed
completed_at: 2026-05-01T07:29:28.794Z
blocker_discovered: false
---

# S04: Pure legality and affordance query API

**Delivered a pure, DSL-backed combat legality query surface that returns action, target, resource, implementation, and toughness affordances with stable machine-readable reasons.**

## What Happened

S04 introduced the headless legality/affordance boundary that future CLI, windowed, and engine-adapter code can call without mutating ECS state. The slice added the public `src/combat/action_query.rs` module plus snapshot/query types for action, target, resource, implementation, and toughness affordances; extended the DSL legality vocabulary and damaged-target rule; and implemented pure target/action evaluation keyed off `SkillDef.targeting`, `SkillDef.implementation`, and immutable combat snapshots rather than skill-ID tables. The final contract now answers the full preflight question set in one place: whether an action is usable, why it is blocked or deferred, which targets are legal, whether resource requirements are met, and whether toughness is truthfully visible for the target. Test coverage was expanded to pin offensive, revive, heal-like/damaged-target, commander/self/wrong-side, hidden self-only, deferred shapes, SP/ultimate shortfall, no-valid-target, missing-skill, and toughness visibility scenarios. The slice also established the compatibility pattern of carrying both legacy acting/target fields and ID-addressable unit snapshots so the pure query can stay deterministic while older fixtures remain easy to construct.

## Verification

Fresh verification after the final code state: `cargo test-dev --test action_affordance_query` passed (18/18); `cargo test-dev skills_ron` passed; `cargo test-dev --test target_shape_truthfulness --test revive_semantics --test toughness_enemy_only --test skill_legality_contract_docs` passed; `cargo check --features "dev windowed"` passed. A full `cargo test-dev` run still fails in pre-existing `tests/form_identity.rs` regressions (8 failing assertions around Form Identity energy/turn-advance behavior), which were not introduced by S04 and remain outside the legality-query slice.

## Requirements Advanced

- R084 — Advanced the data-driven legality contract by delivering a pure, DSL-backed query surface that tests can call before execution to inspect action and target legality.
- R085 — Advanced the UI-readiness contract by exposing truthful deferred/hidden affordances and enemy-only toughness visibility through queryable statuses rather than hardcoded UI rules.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

A full `cargo test-dev` run still fails in pre-existing `tests/form_identity.rs` regressions unrelated to S04. The slice itself is green under its required focused verification set.

## Follow-ups

S05 should wire the resource affordance data into the real pipeline, enforce Energy caps, and keep the same reason vocabulary so engine validation and UI queries stay aligned.

## Files Created/Modified

None.
