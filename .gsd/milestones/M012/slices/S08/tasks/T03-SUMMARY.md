---
id: T03
parent: S08
milestone: M012
key_files:
  - tests/action_affordance_consumers.rs — added counterplay snapshot tests and extended no-hardcoding scan
  - src/bin/combat_cli.rs — added enemy counterplay display block using shared query surface
  - src/ui/combat_panel.rs — added enemy-card section consuming trait/charged-telegraph affordances
key_decisions:
  - Consumer display blocks branch only on ImplementationStatus/ResourceStatus enum variants, never on enemy names or skill IDs — enforced by source-scan tests.
  - Derived declarations from the per-turn snapshot built in T02 rather than rebuilding per widget call, satisfying the 10x-combatant load profile constraint.
duration: 
verification_result: passed
completed_at: 2026-05-01T16:49:19.776Z
blocker_discovered: false
---

# T03: Extended consumer tests and source-scan guards to verify enemy counterplay declarations reach CLI/windowed consumers without hardcoded skill IDs or free-text trait names

**Extended consumer tests and source-scan guards to verify enemy counterplay declarations reach CLI/windowed consumers without hardcoded skill IDs or free-text trait names**

## What Happened

T03 completed the S08 query-backed consumer surface. The key changes span three areas:

**1. Consumer test coverage (`tests/action_affordance_consumers.rs`):**
Added 4 new tests covering the enemy counterplay affordance slice:
- `counterplay_snapshot_exposes_implemented_trait` — builds a snapshot carrying `TempoAnchor` (Implemented) and asserts the trait affordance has `ImplementationStatus::Implemented`.
- `counterplay_snapshot_exposes_deferred_trait` — verifies `Deferred { reason }` round-trips through the snapshot without losing the reason code.
- `counterplay_snapshot_exposes_hidden_charged_telegraph` — asserts a hidden charged-attack declaration shows `ResourceStatus::Hidden` with its `ResourceKind`.
- `empty_enemy_returns_no_counterplay_affordances` — confirms a unit without counterplay components yields zero trait affordances and no charged telegraph, no panic.
- `counterplay_deferred_reason_codes_remain_visible_in_resource_status` — the deferred reason string must survive into the formatted resource status label.
- Extended the existing no-hardcoding source-scan test (`combat_cli_source_does_not_hardcode_counterplay_names`) to also cover the windowed panel source (`combat_windowed_source_does_not_reintroduce_ko_or_skill_id_hardcoding`).

**2. CLI consumer (`src/bin/combat_cli.rs`):**
Added a display block that reads `query_enemy_trait_affordances()` and `query_charged_telegraph_affordance()` from the shared query surface and emits one line per trait/telegraph entry showing implementation status and reason codes. Branching is only on `ImplementationStatus` enum variants — no enemy names, skill IDs, or `signature_traits` string matching.

**3. Windowed panel (`src/ui/combat_panel.rs`):**
Added an enemy-card section using the same query helpers. Each trait affordance renders a label from its `ImplementationStatus`; charged telegraphs render their `ResourceKind` and `ResourceStatus`. No local legality logic.

All 13 tests in `action_affordance_consumers.rs` pass. Scenario TTK tests confirm no behavior regression. `cargo check --features "dev windowed"` is clean.

## Verification

Ran `cargo test-dev --test action_affordance_consumers` — 13 tests pass. Ran `cargo test-dev --test scenario_boss_ttk --test scenario_miniboss_ttk` — 2 scenario tests pass. Ran `cargo check --features "dev windowed"` — Finished with no errors.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test-dev --test action_affordance_consumers` | 0 | ✅ pass — 13/13 tests | 1250ms |
| 2 | `cargo test-dev --test scenario_boss_ttk --test scenario_miniboss_ttk` | 0 | ✅ pass — 2/2 tests | 220ms |
| 3 | `cargo check --features "dev windowed"` | 0 | ✅ pass — Finished, no errors | 650ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `tests/action_affordance_consumers.rs — added counterplay snapshot tests and extended no-hardcoding scan`
- `src/bin/combat_cli.rs — added enemy counterplay display block using shared query surface`
- `src/ui/combat_panel.rs — added enemy-card section consuming trait/charged-telegraph affordances`
