---
id: T03
parent: S04
milestone: M002
key_files:
  - src/ui/combat_panel/mod.rs
  - src/ui/combat_panel/labels.rs
  - src/ui/combat_panel/render.rs
  - src/ui/combat_panel/widgets.rs
  - src/windowed/mod.rs
  - tests/windowed_preview_cache.rs
key_decisions:
  - Projected Baby Burner detonate `OnKernelTransition::Blueprint` messages into a windowed-only `BabyBurnerFlashState` with a fixed six-frame lifetime so presentation remains deterministic and combat-state-free.
  - Kept flash projection as separate observe/tick systems and render-only label helpers, then collapsed the preview/flash resources into a tuple system param to stay within Bevy's system-parameter arity limit for `combat_panel`.
duration: 
verification_result: passed
completed_at: 2026-05-21T06:05:09.720Z
blocker_discovered: false
---

# T03: Added a windowed-only Baby Burner detonate flash chip that projects generic Agumon transitions into deterministic combat-panel state without mutating combat.

**Added a windowed-only Baby Burner detonate flash chip that projects generic Agumon transitions into deterministic combat-panel state without mutating combat.**

## What Happened

Added a windowed-only Baby Burner flash projection seam to the combat panel. In `src/ui/combat_panel/mod.rs` I introduced `BabyBurnerFlashState`, a pure `latest_baby_burner_flash_trigger` extractor over `CombatEvent` messages, and deterministic `advance_baby_burner_flash_state` / `observe_baby_burner_flash` systems that only mutate presentation-owned state. In `src/ui/combat_panel/labels.rs` I added reusable label/tooltip helpers that surface signal owner/name, cast id, and target ids without depending on egui widgets. In `src/ui/combat_panel/render.rs` and `src/ui/combat_panel/widgets.rs` I threaded that state into the existing action-bar chip row so the flash renders alongside the existing cue-barrier telegraph without touching combat mutation. In `src/windowed/mod.rs` I registered the resource plus the fixed-order `advance -> refresh preview cache -> observe flash` update chain. In `tests/windowed_preview_cache.rs` I extended the feature-gated headless suite to inject synthetic Baby Burner detonate transitions, assert the flash appears immediately, persists for the configured frame budget, hides on expiry, aggregates multiple deterministic targets from the same cast, and leaves HP/combat state unchanged. During verification, the first compile surfaced a Bevy system-arity issue after adding one more `Res` to `combat_panel`; I resolved that by grouping the preview and flash resources into a single tuple system parameter before rerunning the full slice verification set.

## Verification

Verified the new windowed flash behavior directly with `cargo test --features windowed --test windowed_preview_cache`, including show/decrement/hide semantics and unchanged HP/combat state. Then ran the full S04 slice verification suite: Agumon reactive detonate, UnitDied payload, timeline cue barrier pipeline, two-clock parity, animation/atlas regressions, the windowed preview/flash suite, `cargo test --lib`, `cargo build --no-default-features`, and `cargo build --features windowed`; all passed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test agumon_baby_burner_reactive` | 0 | ✅ pass | 4590ms |
| 2 | `cargo test --test unit_died_payload` | 0 | ✅ pass | 4878ms |
| 3 | `cargo test --test timeline_cue_barrier_pipeline` | 0 | ✅ pass | 756ms |
| 4 | `cargo test --test timeline_two_clock_parity` | 0 | ✅ pass | 1057ms |
| 5 | `cargo test --test anim_player_fsm --test anim_graph_asset --test anim_gameplay_command_forbidden --test clip_atlas_parity` | 0 | ✅ pass | 988ms |
| 6 | `cargo test --features windowed --test windowed_preview_cache` | 0 | ✅ pass | 3306ms |
| 7 | `cargo test --lib` | 0 | ✅ pass | 8240ms |
| 8 | `cargo build --no-default-features` | 0 | ✅ pass | 6411ms |
| 9 | `cargo build --features windowed` | 0 | ✅ pass | 18138ms |

## Deviations

Also updated `src/ui/combat_panel/widgets.rs` and `src/windowed/mod.rs` to render the new chip and register the windowed-only flash resource/systems in the existing UI plugin schedule.

## Known Issues

None.

## Files Created/Modified

- `src/ui/combat_panel/mod.rs`
- `src/ui/combat_panel/labels.rs`
- `src/ui/combat_panel/render.rs`
- `src/ui/combat_panel/widgets.rs`
- `src/windowed/mod.rs`
- `tests/windowed_preview_cache.rs`
