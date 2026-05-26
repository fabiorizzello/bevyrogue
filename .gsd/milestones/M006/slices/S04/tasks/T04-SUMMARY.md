---
id: T04
parent: S04
milestone: M006
key_files:
  - tests/windowed_only/agumon_module_extraction.rs
  - tests/windowed_only.rs
  - src/windowed/render.rs
  - src/windowed/mod.rs
  - src/windowed/digimon/mod.rs
  - src/windowed/digimon/agumon/mod.rs
key_decisions:
  - Treat the include_str!-based agumon extraction contract as the canonical CI proof for the S04 structural seam, since auto-mode must not launch the windowed binary.
duration: 
verification_result: passed
completed_at: 2026-05-26T13:50:56.481Z
blocker_discovered: false
---

# T04: Verified that the agumon_module_extraction source-contract gate is present, wired into the windowed_only harness, and passing alongside the full S04 verification suite.

**Verified that the agumon_module_extraction source-contract gate is present, wired into the windowed_only harness, and passing alongside the full S04 verification suite.**

## What Happened

I inspected the S04 task inputs and found that `tests/windowed_only/agumon_module_extraction.rs` already existed and `tests/windowed_only.rs` was already wiring it in via the `#[path = ...] mod ...;` harness pattern. I read the source-contract test and confirmed it matched the task contract: it `include_str!`s `src/windowed/render.rs`, `src/windowed/mod.rs`, `src/windowed/digimon/mod.rs`, and `src/windowed/digimon/agumon/mod.rs`; asserts that the engine files contain none of the forbidden Agumon-specific tokens (`AGUMON_`, `fn on_enter_effect_ids`, `fn skill_start_node`, `fn load_agumon_enoki_vfx`, `enoki_effect_path`, and `digimon/agumon_atlas.png`); asserts that the Agumon module owns the required registry/cue/atlas-path tokens; and asserts that the per-Digimon seam exposes `mod agumon` plus `fn register_all`. I then grep-checked the engine files directly and confirmed there was no remaining forbidden token leakage in `src/windowed/render.rs` or `src/windowed/mod.rs`. Since the required artifacts were already present and aligned with the plan, no code edits were necessary in this execution; I proceeded to run the full verification gate and confirmed the source-contract test, full `windowed_only` harness, dependency-gating tests, and warning-free `windowed` build all passed.

## Verification

Ran the targeted source-contract test (`cargo test --features windowed --test windowed_only agumon_module_extraction`), the full windowed-only harness (`cargo test --features windowed --test windowed_only`), the dependency-gating integration test (`cargo test --test dependency_gating`), and a warning gate for the windowed build (`cargo build --features windowed` with output scanned for `warning:`). All commands exited successfully; the targeted test passed 3 assertions, the full harness passed 62 tests, dependency_gating passed 2/2 tests, and the windowed build completed with zero warnings detected.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --features windowed --test windowed_only agumon_module_extraction` | 0 | ✅ pass | 326ms |
| 2 | `cargo test --features windowed --test windowed_only` | 0 | ✅ pass | 360ms |
| 3 | `cargo test --test dependency_gating` | 0 | ✅ pass | 536ms |
| 4 | `cargo build --features windowed && no warning: lines detected in captured output` | 0 | ✅ pass | 273ms |

## Deviations

No code edits were needed because the expected source-contract test and harness registration were already present and matched the task plan; this execution completed by verifying the contract and running the required gates.

## Known Issues

None.

## Files Created/Modified

- `tests/windowed_only/agumon_module_extraction.rs`
- `tests/windowed_only.rs`
- `src/windowed/render.rs`
- `src/windowed/mod.rs`
- `src/windowed/digimon/mod.rs`
- `src/windowed/digimon/agumon/mod.rs`
