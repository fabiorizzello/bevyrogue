---
id: T05
parent: S01
milestone: M021
key_files:
  - src/combat/api/mod.rs
  - src/combat/plugin.rs
  - src/main.rs
  - tests/intent_applier_canary.rs
  - tests/cast_id_propagation.rs
  - .gsd/DECISIONS.md
key_decisions:
  - intent_applier uses exclusive &mut World system to avoid Bevy query aliasing on multi-entity read+write — ParamSet avoided for clarity
  - CombatRng::from_seed(0xDEAD_BEEF) as canonical deterministic seed in CombatPlugin::build; headless init_resource is no-op when plugin already inserted it
duration: 
verification_result: passed
completed_at: 2026-05-15T07:15:16.074Z
blocker_discovered: false
---

# T05: All 5 grep gates green, full test suite 0 failures across all test binaries, cargo check headless + windowed clean; 2 missing decisions (intent_applier exclusive system, CombatRng default seed) appended to DECISIONS.md

**All 5 grep gates green, full test suite 0 failures across all test binaries, cargo check headless + windowed clean; 2 missing decisions (intent_applier exclusive system, CombatRng default seed) appended to DECISIONS.md**

## What Happened

T05 ran the complete S01 verification battery. cargo check headless (default features) finished clean with no errors — only pre-existing dead-code warnings. cargo check --features windowed likewise clean. cargo test ran all test binaries and reported 0 failures across every suite (208+209 inline tests plus all integration test files including the canary intent_applier_canary and cast_id_propagation). All 5 grep gates passed: Gate 1 (no forbidden use statements — bevy::winit/render/bevy_egui — in src/combat/ outside blueprints): PASS (two false positives were comment text, not import statements); Gate 2 (cast_id field on CombatEvent): PASS; Gate 3 (pub mod api in src/combat/mod.rs): PASS; Gate 4 (CombatPlugin re-exported from lib.rs): PASS; Gate 5 (register_combat_kernel_runtime not in main.rs): PASS. DECISIONS.md was audited — two decisions from T02/T04 were not yet recorded: (1) intent_applier exclusive system choice over ParamSet, (2) CombatRng::from_seed(0xDEAD_BEEF) canonical seed in CombatPlugin. Both appended via gsd_decision_save.

## Verification

cargo check (headless): exit 0, no new errors. cargo check --features windowed: exit 0, no new errors. cargo test: 0 failed across all test binaries. rg gate 1 (^use bevy::(winit|render)|^use bevy_egui in src/combat/ ex-blueprints): no matches = PASS. rg gate 2 (cast_id in events.rs): matched pub cast_id: CastId = PASS. rg gate 3 (pub mod api in combat/mod.rs): matched = PASS. rg gate 4 (CombatPlugin in lib.rs): pub use combat::CombatPlugin matched = PASS. rg gate 5 (register_combat_kernel_runtime in main.rs): no match = PASS.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | PASS — headless check clean | 1040ms |
| 2 | `cargo check --features windowed` | 0 | PASS — windowed check clean | 780ms |
| 3 | `cargo test` | 0 | PASS — 0 failures across all test binaries | 2500ms |
| 4 | `rg '^use bevy::(winit|render)|^use bevy_egui' src/combat/ (ex-blueprints)` | 1 | PASS — no forbidden use statements | 50ms |
| 5 | `rg 'cast_id' src/combat/events.rs` | 0 | PASS — cast_id field present | 30ms |
| 6 | `rg 'pub mod api' src/combat/mod.rs` | 0 | PASS | 30ms |
| 7 | `rg 'CombatPlugin' src/lib.rs` | 0 | PASS — pub use combat::CombatPlugin | 30ms |
| 8 | `rg 'register_combat_kernel_runtime' src/main.rs` | 1 | PASS — not present in main.rs | 30ms |

## Deviations

Gate 1 grep pattern initially matched comment text in src/combat/mod.rs and src/combat/api/mod.rs (the comments explicitly state no forbidden imports as documentation). Refined to ^use prefix to check actual import statements — gate passed cleanly.

## Known Issues

Pre-existing dead-code warnings (104 headless, 100 windowed) — not introduced by S01 work.

## Files Created/Modified

- `src/combat/api/mod.rs`
- `src/combat/plugin.rs`
- `src/main.rs`
- `tests/intent_applier_canary.rs`
- `tests/cast_id_propagation.rs`
- `.gsd/DECISIONS.md`
