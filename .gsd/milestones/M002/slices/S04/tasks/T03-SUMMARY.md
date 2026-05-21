---
id: T03
parent: S04
milestone: M002
key_files:
  - src/ui/combat_panel/mod.rs
  - src/ui/combat_panel/labels.rs
  - src/ui/combat_panel/render.rs
  - src/windowed/mod.rs
  - tests/windowed_preview_cache.rs
key_decisions:
  - Project Agumon detonate observability from generic `CombatEventKind::OnKernelTransition` payloads into a feature-gated `BabyBurnerFlashState` with a fixed frame lifetime instead of introducing presentation-driven combat logic.
  - Render the detonate proof through the existing combat-panel chip/tooltip path so windowed diagnostics expose source/cast/signal/targets while headless combat state remains immutable.
duration: 
verification_result: passed
completed_at: 2026-05-21T06:35:26.694Z
blocker_discovered: false
---

# T03: Verified and recorded the existing windowed-only Baby Burner detonate flash projection that mirrors generic Agumon detonate transitions into deterministic combat-panel chip state without mutating combat.

**Verified and recorded the existing windowed-only Baby Burner detonate flash projection that mirrors generic Agumon detonate transitions into deterministic combat-panel chip state without mutating combat.**

## What Happened

I audited the planned T03 touchpoints and confirmed the feature was already implemented in the working tree. `src/ui/combat_panel/mod.rs` defines `BabyBurnerFlashState`, a fixed-frame `BabyBurnerFlashDisplay`, and the `latest_baby_burner_flash_trigger`/`observe_baby_burner_flash` helpers that fold `OnKernelTransition::Blueprint(owner="agumon", name="baby_burner_detonate", payload=UnitTarget)` events into deterministic windowed-only presentation state. `src/ui/combat_panel/labels.rs` formats the flash chip text and tooltip with source, cast, signal, targets, and frame counters, while `src/ui/combat_panel/render.rs` threads that chip through the combat action bar without writing back into combat state. `src/windowed/mod.rs` already registers the flash resource and chains `advance_baby_burner_flash_state`, `refresh_preview_damage_cache`, and `observe_baby_burner_flash` in the windowed schedule. `tests/windowed_preview_cache.rs` already injects synthetic detonate transitions, asserts show/decrement/hide behavior across the fixed frame budget, checks tooltip contents, and verifies HP plus `CombatState` remain unchanged. Because the local code already satisfied the task contract, no additional source edits were required; I completed the task by validating the behavior and recording the existing implementation.

## Verification

Ran the task-specific feature-gated test plus the full S04 verification set from the slice plan. `cargo test --features windowed --test windowed_preview_cache` passed, proving the flash appears from the detonate transition, persists for a deterministic number of frames, hides on expiry, and leaves HP/combat state unchanged. The broader detonate, payload, timeline, animation/atlas, library, and headless/windowed build checks also passed: `cargo test --test agumon_baby_burner_reactive`, `cargo test --test unit_died_payload`, `cargo test --test timeline_cue_barrier_pipeline`, `cargo test --test timeline_two_clock_parity`, `cargo test --test anim_player_fsm --test anim_graph_asset --test anim_gameplay_command_forbidden --test clip_atlas_parity`, `cargo test --lib`, `cargo build --no-default-features`, and `cargo build --features windowed`.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --features windowed --test windowed_preview_cache` | 0 | ✅ pass | 292ms |
| 2 | `cargo test --test agumon_baby_burner_reactive` | 0 | ✅ pass | 150ms |
| 3 | `cargo test --test unit_died_payload` | 0 | ✅ pass | 143ms |
| 4 | `cargo test --test timeline_cue_barrier_pipeline` | 0 | ✅ pass | 148ms |
| 5 | `cargo test --test timeline_two_clock_parity` | 0 | ✅ pass | 162ms |
| 6 | `cargo test --test anim_player_fsm --test anim_graph_asset --test anim_gameplay_command_forbidden --test clip_atlas_parity` | 0 | ✅ pass | 248ms |
| 7 | `cargo test --lib` | 0 | ✅ pass | 167ms |
| 8 | `cargo build --no-default-features` | 0 | ✅ pass | 139ms |
| 9 | `cargo build --features windowed` | 0 | ✅ pass | 195ms |

## Deviations

None. The implementation was already present in the local tree, so execution focused on auditing, verification, and canonical task recording rather than additional code changes.

## Known Issues

None.

## Files Created/Modified

- `src/ui/combat_panel/mod.rs`
- `src/ui/combat_panel/labels.rs`
- `src/ui/combat_panel/render.rs`
- `src/windowed/mod.rs`
- `tests/windowed_preview_cache.rs`
