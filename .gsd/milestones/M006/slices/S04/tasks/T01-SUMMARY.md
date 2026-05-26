---
id: T01
parent: S04
milestone: M006
key_files:
  - src/windowed/digimon/mod.rs
  - src/windowed/digimon/agumon/mod.rs
  - src/windowed/mod.rs
  - tests/windowed_only/digimon_sprite_cue_dispatch.rs
key_decisions:
  - Engine (UiPlugin) owns the CueRegistry resource init; the agumon module only populates it via its register() entry point
  - register_all is pub(super) and agumon::register is pub(in crate::windowed), keeping the per-Digimon seam internal to the windowed binary crate
  - Updated the S03 source-contract test to follow the registration into the agumon module rather than deleting it — the location-pin intent survives the extraction
duration: 
verification_result: passed
completed_at: 2026-05-26T11:53:07.107Z
blocker_discovered: false
---

# T01: Scaffolded src/windowed/digimon/agumon/ module tree and moved register_agumon_cues into it, wired via register_all from UiPlugin

**Scaffolded src/windowed/digimon/agumon/ module tree and moved register_agumon_cues into it, wired via register_all from UiPlugin**

## What Happened

Established the per-Digimon module + register(app) seam (mirroring blueprints/<name>/register_*; MEM018/MEM106/MEM109, D049) and validated the wiring end-to-end with the smallest extraction — the cue registration.

Created src/windowed/digimon/mod.rs exposing `pub(super) fn register_all(app: &mut App)` (calls `agumon::register(app)`) plus `pub(in crate::windowed) mod agumon;`. Created src/windowed/digimon/agumon/mod.rs exposing `pub(in crate::windowed) fn register(app: &mut App)` that adds the moved `register_agumon_cues` Startup system; the function body (hit_flash Flash + hit_shake SpriteShake + camera_impact CameraShake registrations) moved verbatim from mod.rs:136-163.

In src/windowed/mod.rs: added `mod digimon;`, deleted `register_agumon_cues` and its `.add_systems(Startup, register_agumon_cues)` line, kept `.init_resource::<CueRegistry>()` in UiPlugin (engine owns the resource; agumon only populates it), and call `crate::windowed::digimon::register_all(app)` exactly once from UiPlugin::build after the system-chain setup (after CueRegistry init_resource).

Deviation: the S03 source-contract test `mod_registers_the_three_agumon_cue_ids` pinned the cue registration to mod.rs, which directly contradicts S04's extraction goal. Updated it (renamed to `agumon_module_registers_the_three_agumon_cue_ids`) to follow the registration to its new home: it now include_str!s the agumon module source and asserts the three cue ids live there, asserts mod.rs still inits CueRegistry, and asserts `fn register_agumon_cues` is absent from mod.rs. The test's intent (pinning the three cue ids are registered) is preserved; only the asserted location moved.

## Verification

cargo build --features windowed with RUSTFLAGS=-D warnings finished green (zero warnings). cargo test --features windowed --test windowed_only passed 59/59. grep confirms register_agumon_cues no longer appears in src/windowed/mod.rs (0 hits) and now lives in src/windowed/digimon/agumon/mod.rs (2 hits: definition + add_systems).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `RUSTFLAGS="-D warnings" cargo build --features windowed` | 0 | pass | 86000ms |
| 2 | `grep -c register_agumon_cues src/windowed/mod.rs (expect 0)` | 0 | pass | 50ms |
| 3 | `cargo test --features windowed --test windowed_only` | 0 | pass (59 passed; 0 failed) | 1245ms |

## Deviations

Updated tests/windowed_only/digimon_sprite_cue_dispatch.rs: the S03 source-contract test pinned the cue registration to mod.rs, contradicting S04's extraction. Renamed/updated it to assert the three cue ids now register in the agumon module while mod.rs retains CueRegistry init and no longer defines register_agumon_cues.

## Known Issues

none

## Files Created/Modified

- `src/windowed/digimon/mod.rs`
- `src/windowed/digimon/agumon/mod.rs`
- `src/windowed/mod.rs`
- `tests/windowed_only/digimon_sprite_cue_dispatch.rs`
