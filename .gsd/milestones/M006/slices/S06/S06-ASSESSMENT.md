---
sliceId: S06
uatType: artifact-driven
verdict: PASS
date: 2026-05-27T00:00:00.000Z
---

# UAT Result — S06

## Checks

| Check | Mode | Result | Notes |
|-------|------|--------|-------|
| A1 — Starvation regression test (`cargo test --test animation -- registry_starvation`) | runtime | PASS | 1/1 passed: `registry_starvation::populate_graph_registries_starves_second_event_when_first_matches` ok; exit 0 |
| A2 — Full headless suite (`cargo test`) | runtime | PASS | All test binaries exit 0, 0 failures; all integration and lib targets green |
| M1 — Renamon idle sprite present in windowed run | human-follow-up | NEEDS-HUMAN | Cannot launch windowed binary in auto-mode. Human must run `cargo run --features windowed`, wait for encounter scene, and confirm both Agumon and Renamon idle sprites are visible. |
| M2 — No spurious warn-once in happy path | human-follow-up | NEEDS-HUMAN | Same windowed run as M1. Human must confirm no `animation graph loaded but no registry entry could be built` warnings appear in console for Agumon or Renamon assets. |

## Overall Verdict

PASS — All automatable checks pass (starvation regression test green, full headless suite 0 failures); two windowed visual checks (M1, M2) require human sign-off with `cargo run --features windowed`.

## Notes

- `cargo test --test animation -- registry_starvation`: 1 test, exit 0, ~0.00s (binary already compiled).
- `cargo test` (full headless suite): all test binaries pass, exit 0. Includes animation, combat, timeline, ui, and windowed_only targets.
- Manual windowed checks M1 and M2 cannot be automated. To complete sign-off:
  1. Run `cargo run --features windowed` (requires display / `cargo winx` alias).
  2. Confirm Renamon's idle sprite renders alongside Agumon's.
  3. Confirm no `animation graph loaded but no registry entry could be built` log lines for either Digimon.
- If only one sprite appears, check console for the warn-once message and verify `StanceGraphPaths`/`SkillGraphPaths` entries for Renamon.
