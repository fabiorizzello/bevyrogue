---
id: T02
parent: S05
milestone: M002
key_files:
  - src/ui/combat_panel/mod.rs
  - src/windowed/mod.rs
  - tests/windowed_only/windowed_hud_hp_bar.rs
  - tests/windowed_only.rs
key_decisions:
  - HpBarView and FloatingDamageView placed in src/ui/combat_panel/mod.rs (lib crate) so they are importable by tests via bevyrogue:: — matching the existing pattern for BabyBurnerFlashState and PhaseStripDisplay
  - Tests placed in tests/windowed_only/windowed_hud_hp_bar.rs and included in windowed_only harness per R003 — the task plan's --test windowed_hud_hp_bar target would violate R003's single-harness-per-scope rule
  - FloatingDamage spawn_time lifetime left as time.elapsed_secs() per task plan guidance; test uses spawn_time=-(FLOATING_LIFETIME_SECS+1) to simulate expired entries at elapsed=0
duration: 
verification_result: passed
completed_at: 2026-05-21T10:29:48.178Z
blocker_discovered: false
---

# T02: Added HpBarView + FloatingDamageView computed resources with windowed-only systems; 6 new windowed_only harness tests assert HP pct and damage-number text/anchor without display or CombatState mutation.

**Added HpBarView + FloatingDamageView computed resources with windowed-only systems; 6 new windowed_only harness tests assert HP pct and damage-number text/anchor without display or CombatState mutation.**

## What Happened

T02 introduced two feature-gated display-state resources in src/ui/combat_panel/mod.rs: HpBarView (per-unit HP bar entries with pct clamped to [0,1]) driven by compute_hp_bar_view reading Unit components read-only, and FloatingDamageView (per-entry text + alpha + anchor unit_id) driven by compute_floating_damage_view reading FloatingDamage components via time.elapsed_secs() lifetime — no frame-counter change needed since the existing spawn_time approach is used only for visibility reads. Both systems are pure projections: no CombatState mutations. Both resources and systems were registered in the windowed UiPlugin in src/windowed/mod.rs. Six tests were added to tests/windowed_only/windowed_hud_hp_bar.rs and included in the windowed_only harness (R003 compliance — not a standalone binary). The previous auto-fix attempt had tried to run --test encounter_bootstrap_windowed which doesn't exist as a target; that was a T01 artifact. T02's correct verification target is --test windowed_only.

## Verification

cargo build --features windowed: exit 0. cargo test --features windowed --test windowed_only: 13/13 passed (6 new windowed_hud_hp_bar tests + 7 prior). cargo test --lib: 0 tests (lib has no unit tests), exit 0. cargo build --no-default-features: exit 0.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features windowed` | 0 | pass | 5000ms |
| 2 | `cargo test --features windowed --test windowed_only` | 0 | pass — 13/13 tests | 780ms |
| 3 | `cargo test --lib` | 0 | pass | 33330ms |
| 4 | `cargo build --no-default-features` | 0 | pass | 23420ms |

## Deviations

Verification command changed from cargo test --features windowed --test windowed_preview_cache --test windowed_hud_hp_bar (invalid targets per R003) to cargo test --features windowed --test windowed_only. Test file placed at tests/windowed_only/windowed_hud_hp_bar.rs (harness case) rather than tests/windowed_hud_hp_bar.rs (standalone binary) per R003.

## Known Issues

None.

## Files Created/Modified

- `src/ui/combat_panel/mod.rs`
- `src/windowed/mod.rs`
- `tests/windowed_only/windowed_hud_hp_bar.rs`
- `tests/windowed_only.rs`
