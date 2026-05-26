---
sliceId: S04
uatType: artifact-driven
verdict: PASS
date: 2026-05-26T10:00:00.000Z
---

# UAT Result — S04

## Checks

| Check | Mode | Result | Notes |
|-------|------|--------|-------|
| Dep-gating (R005/R016): `cargo test --test dependency_gating` | runtime | PASS | exit 0; `2 passed` — `bevy_enoki_absent_from_headless_graph` ok, `bevy_enoki_present_in_windowed_graph` ok |
| Windowed build: `cargo build --features windowed` | runtime | PASS | exit 0; `Finished dev profile` — no errors or enoki-related warnings |
| Windowed contract + parse tests: `cargo test --features windowed --test windowed_only` | runtime | PASS | exit 0; **46 passed, 0 failed** — includes `impact_effect_parses_into_enoki_schema`, `enoki_plugin_is_registered`, `spawn_effect_by_id_enoki_branch_spawns_correct_components`, `spawn_effect_by_id_quad_loop_unchanged_for_other_effects` |
| Full headless suite: `cargo test` | runtime | PASS | exit 0; all test targets clean, 0 failed — representative counts: 72/53/52/52/51/50 passed across integration targets; dep-gating binary 2 passed |
| baby_flame.impact renders through enoki (`cargo winx`) | human-follow-up | NEEDS-HUMAN | Deferred to S05 / K001 visual sign-off. K001 forbids running windowed binary in auto-mode. |
| Other effects unaffected (Sharp Claws, Baby Burner) | human-follow-up | NEEDS-HUMAN | Deferred to S05 / K001 visual sign-off. |
| Fail-loud diagnostic on missing asset | human-follow-up | NEEDS-HUMAN | Deferred — manual destructive test (rename particle.ron path, observe WARN on `windowed.agumon_playback`). |

## Overall Verdict

PASS — all 4 automatable checks pass with 0 failures; 3 visual/destructive manual checks are deferred to S05 / K001 as designed.

## Notes

- Check 1 (dep-gating): both `bevy_enoki_absent_from_headless_graph` and `bevy_enoki_present_in_windowed_graph` pass immediately from the already-compiled binary (0.29 s); proves R005/R016 headless isolation invariant holds.
- Check 2 (windowed build): fully cached incremental build, exits clean — no enoki render-stack symbols leaking into compilation errors.
- Check 3 (windowed_only, 46 passed): S04-specific key tests all present and green alongside all prior S01–S03 regression tests.
- Check 4 (headless, 0 failures across all targets): full headless suite ran in ~2.3 s with no regressions introduced by the enoki wiring. Total passing tests across all headless targets exceeds 500+.
- Manual UAT steps 5, 6, and 7 require a GPU/display context (K001 constraint) and are intentionally deferred to S05's visual sign-off phase. The automated source-contract tests (`enoki_impact_render.rs`) provide the closest available substitute — they prove the spawn branch is wired correctly at the code level.
