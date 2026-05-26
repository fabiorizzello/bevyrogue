---
sliceId: S04
uatType: artifact-driven
verdict: PASS
date: 2026-05-26T14:04:30.552890Z
---

# UAT Result — S04

## Checks

| Check | Mode | Result | Notes |
|-------|------|--------|-------|
| Run `RUSTFLAGS='-D warnings' cargo build --features windowed`. | artifact | PASS | Ran via `gsd_exec`: `RUSTFLAGS='-D warnings' cargo build --features windowed`. Build finished successfully in `dev` profile with warnings denied: `Finished 'dev' profile [optimized + debuginfo] target(s) in 0.26s`. Evidence: `.gsd/exec/5a938c17-883d-4086-97bb-fdd8d0722564.stderr`. |
| Run `cargo test --features windowed --test windowed_only agumon_module_extraction -- --nocapture`. | artifact | PASS | Ran via `gsd_exec`: `cargo test --features windowed --test windowed_only agumon_module_extraction -- --nocapture`. Observed `running 3 tests` and all 3 passed: `digimon_module_exposes_the_register_all_seam`, `agumon_module_owns_the_registry_population_tokens`, and `engine_files_no_longer_embed_agumon_specific_tokens`. Evidence: `.gsd/exec/628802eb-802c-46b9-8548-3bfcc3e0224c.stdout`. |
| Run `cargo test --features windowed --test windowed_only`. | artifact | PASS | Ran via `gsd_exec`: `cargo test --features windowed --test windowed_only`. Observed `running 62 tests` and `test result: ok. 62 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`. Evidence: `.gsd/exec/2e3e7f4b-bf62-47c8-83f3-86a9a2bbd1e4.stdout`. |
| Run `cargo test --test dependency_gating`. | artifact | PASS | Ran via `gsd_exec`: `cargo test --test dependency_gating`. Observed both gating assertions pass: `bevy_enoki_absent_from_headless_graph` and `bevy_enoki_present_in_windowed_graph`; final result `2 passed; 0 failed`. Evidence: `.gsd/exec/4e0e1296-2d03-4f33-97c2-eeebe34c745b.stdout`. |
| For manual K001 sign-off, launch `cargo run --features windowed` and exercise Agumon presentation flows (idle/skill/hurt/death, hit flash/shake, Baby Flame charge/projectile/impact, Baby Burner detonate, Sharp Claws slash). | human-follow-up | NEEDS-HUMAN | This check is explicitly a live visual/runtime equivalence review outside auto-mode. In artifact-driven mode I did not invent a visual PASS. Human reviewer should launch `cargo run --features windowed` and confirm the listed Agumon flows remain behaviorally equivalent after extraction, with special attention to projectile arrival and detonate paths. |

## Overall Verdict

PASS — all automatable S04 artifact/build/test gates passed, and the only remaining item is the explicitly manual K001 runtime visual sign-off.

## Notes

- Build/test verification was executed in the repository root with persisted evidence under `.gsd/exec/`.
- Test commands briefly contended on Cargo package/build locks because they were run in parallel after the build gate; all still completed successfully.
- This assessment truthfully leaves pixel/runtime equivalence as a human follow-up rather than collapsing it into a structural pass.
