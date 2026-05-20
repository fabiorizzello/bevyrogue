---
sliceId: S01
uatType: artifact-driven
verdict: PASS
date: 2026-05-20T00:00:00Z
---

# UAT Result — S01

## Checks

| # | Check | Mode | Result | Notes |
|---|-------|------|--------|-------|
| 1 | Full headless suite (`cargo nextest run --profile agent`) | runtime | PASS | 642 tests passed, 1 skipped, 0 failed (UAT expected 601 — suite has grown since UAT was written; zero failures is the binding criterion). |
| 2 | Windowed build (`cargo build --features windowed`) | runtime | PASS | Clean compile. 1 pre-existing future-incompat warning in `src/ui/combat_panel/widgets.rs:181` about `f32` literal fallback — unrelated to S01 scope. |
| 3 | AnimGraph player FSM (`cargo nextest run --profile agent --test anim_player_fsm`) | runtime | PASS | 11/11 tests passed (UAT expected 8 — suite grew). Covers Loop/Hold/SpeedMul, reverse, KernelCue gate, and idle range 54–59 cycle. |
| 4 | Registry hit/miss (`cargo nextest run --profile agent --test anim_registry`) | runtime | PASS | 5/5 tests passed. `resolve()` returns handle on hit, None on miss; SkillGraphRegistry and StanceGraphRegistry independent. |
| 5 | Stance asset load (`cargo nextest run --profile agent --test anim_stance_asset`) | runtime | PASS | 3/3 tests passed. stance.ron parses, agumon_stance resolves via StanceGraphRegistry, validates with zero errors. |
| 6 | Clip↔atlas parity (`cargo nextest run --profile agent --test clip_atlas_parity`) | runtime | PASS | 2/2 tests passed. agumon clip matches atlas with `all` range present; renamon clip also matches. |
| 7 | Gameplay command gate (`cargo nextest run --profile agent --test anim_gameplay_command_forbidden`) | runtime | PASS | 4/4 tests passed (UAT expected 1 — suite grew). Production agumon graph has no gameplay commands; EmitDamage in on_enter / cue presentation rejected; ReleaseKernelCue allowed. |
| 8 | Windowed idle soak (`cargo run --features windowed` — visually verify agumon sprite cycles idle frames 54–59 autonomously via stance graph) | human-follow-up | NEEDS-HUMAN | Requires interactive windowed session and visual judgement. Reproduction: `cargo run --features windowed`, observe agumon sprite for several seconds, confirm frames advance 54 → 55 → 56 → 57 → 58 → 59 → 54 (loop), no panic, no hardcoded frame index. Headless tests in checks 3–5 already prove the FSM derives the correct frame range from data; this check verifies the windowed wiring drives the player. |

## Overall Verdict

PASS — all 7 automatable checks green; the one remaining check (live windowed soak) is honestly human-judgement and is marked NEEDS-HUMAN with reproduction steps.

## Notes

- Tooling: `cargo-nextest` was missing on this machine at UAT start and was installed via `cargo install cargo-nextest --locked` (now `cargo-nextest 0.9.136` in `~/.cargo/bin/`). All checks 1, 3–7 were then re-run with the prescribed `--profile agent` (writes JUnit XML to `target/nextest/agent/junit.xml`).
- An earlier assessment in this session ran `cargo test` as a fallback; that has been superseded by this run, which uses the exact commands from the UAT file.
- Test-count drift: checks 1, 3, and 7 produce more tests than the UAT enumerated, reflecting suite growth since the UAT was written. The binding contract is "0 failures" / specified behaviours, all of which hold.
- Check 8 cannot be honestly automated from a headless agent context (no display, no human visual verifier). Headless tests 3–5 cover the data-driven correctness of the FSM and stance asset; check 8 is the wiring-and-eye verification step.
