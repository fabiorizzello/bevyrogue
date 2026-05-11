---
id: T04
parent: S07
milestone: M012
key_files:
  - tests/action_affordance_consumers.rs
key_decisions:
  - Use exact-literal source scans as the negative regression for reintroducing local KO/team/skill-ID hardcoding in CLI and windowed adapters.
duration: 
verification_result: passed
completed_at: 2026-05-01T14:12:52.839Z
blocker_discovered: false
---

# T04: Added a windowed source-scan regression to keep S07 legality affordances query-backed across CLI and UI.

**Added a windowed source-scan regression to keep S07 legality affordances query-backed across CLI and UI.**

## What Happened

I audited the CLI and windowed combat affordance consumers against the shared legality-query contract and found no remaining runtime legality hardcoding to remove. To lock that in, I added a regression test that scans both `src/bin/combat_cli.rs` and `src/ui/combat_panel.rs` for the forbidden KO/skill-ID literals that would reintroduce local legality decisions. I then ran the full S07 verification chain; the behavioral affordance tests, legality integration tests, full cargo test suite, and the `dev windowed` check all passed.

## Verification

Fresh verification after the last code edit completed successfully. The final gate ran `cargo test-dev --test action_affordance_consumers && cargo test-dev --test action_affordance_query && cargo test-dev --test engine_legality_integration && cargo test-dev && cargo check --features "dev windowed"` and exited 0. The combined run passed the consumer regression tests (including the new windowed-source scan), the affordance query tests, the engine legality integration tests, the full Rust test suite, and the windowed feature check.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test-dev --test action_affordance_consumers && cargo test-dev --test action_affordance_query && cargo test-dev --test engine_legality_integration && cargo test-dev && cargo check --features "dev windowed"` | 0 | ✅ pass | 6000ms |

## Deviations

No runtime source changes were needed; the adapters were already query-backed, so the only update was a regression test to protect against reintroducing local legality hardcoding.

## Known Issues

None.

## Files Created/Modified

- `tests/action_affordance_consumers.rs`
