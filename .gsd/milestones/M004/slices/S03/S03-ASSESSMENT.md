---
sliceId: S03
uatType: artifact-driven
verdict: PASS
date: 2026-05-25T14:55:00Z
---

# UAT Result — S03

## Checks

| Check | Mode | Result | Notes |
|-------|------|--------|-------|
| `cargo test --test animation` passes (≥110 tests, 0 failures) — covers variant-selection determinism, detonate/flash load + curve assertions, validate_effects with DanglingVariant | artifact | PASS | Ran `cargo test --test animation` via `gsd_exec` (`c1fd6431-c636-449e-8a13-eeb8c61795b5`). Observed `test result: ok. 110 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s`. |
| `cargo build --features windowed` compiles clean (no windowed leak from VfxContext or variant map) | artifact | PASS | Ran `cargo build --features windowed` via `gsd_exec` (`ee977cc5-af77-4891-9faa-ff9e9e0e6072`). Exit code 0; stderr ended with `Finished 'dev' profile [optimized + debuginfo] target(s) in 0.56s`. |
| `cargo test --features windowed --test windowed_only` passes (≥32 tests) — windowed detonate spawn contract holds | artifact | PASS | Ran `cargo test --features windowed --test windowed_only` via `gsd_exec` (`55beb3b3-c9ae-4c7f-a642-cab99f1cafe2`). Observed `test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s`. |
| Run `cargo run --features windowed` (alias `cargo winx`) | human-follow-up | NEEDS-HUMAN | Manual visual runtime launch required; auto-mode in artifact-driven UAT cannot honestly sign off interactive windowed execution. |
| Trigger Baby Burner detonate in-game | human-follow-up | NEEDS-HUMAN | Requires a human to exercise the in-game action after launching the windowed build. |
| Confirm: multi-shard outward burst at target, followed by a bright central flash pop | human-follow-up | NEEDS-HUMAN | Subjective visual verification; objective automated artifact checks above confirm authored detonate/flash data and windowed spawn contract, but not final perceived effect quality. |
| Sign off: visual quality acceptable for milestone review | human-follow-up | NEEDS-HUMAN | Human reviewer must approve visual quality after observing the live effect. |

## Overall Verdict

PASS — all automatable artifact-driven checks passed; only the explicitly manual visual sign-off remains as NEEDS-HUMAN follow-up.

## Notes

Automated evidence came from fresh `gsd_exec` runs in this verification lane. Human follow-up should launch the windowed build locally, cast Baby Burner, and confirm the detonate burst plus flash match the expected visual quality described in the UAT file.