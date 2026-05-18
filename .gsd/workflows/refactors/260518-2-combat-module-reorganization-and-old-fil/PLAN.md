# Plan — Combat module reorganization and old files cleanup

Date: 2026-05-18
Branch: `gsd/refactor/combat-module-reorganization-and-old-fil`

## Completed Waves

| Wave | Description | Commit |
|------|-------------|--------|
| 6 | Move 22 flat files into mechanics/, encounter/, observability/, turn_system/ | `3e16435` |
| 7 | Remove shared demo skills, update consumers, update 3 captures | `6bd8c42` |

## Remaining Waves

### Wave 8 — Rename `api/` → `runtime/`

The `api/` name is misleading — it's the internal execution engine (intent routing, registry, signal bus, timeline FSM), not an external API surface.

**Steps:**
1. `git mv src/combat/api/ src/combat/runtime/`
2. Update `src/combat/mod.rs`: `pub mod api` → `pub mod runtime`
3. Add re-export alias: `pub use runtime as api;` in `combat/mod.rs` (backward compat for tests)
4. Update all `use crate::combat::api::` → `use crate::combat::runtime::` in `src/` (22 files)
5. Update all `use bevyrogue::combat::api::` → `use bevyrogue::combat::runtime::` in `tests/` (30 files)
6. Update the doc comment in `runtime/mod.rs` to remove the "future rename" note
7. Verify: `cargo check && cargo test`

**Risk:** High file count (52 files touched) but purely mechanical find-replace. No behavioral change.

### Wave 9 — Split large files (tier 1: >900 LOC)

| File | LOC | Split strategy |
|------|-----|----------------|
| `turn_system/pipeline.rs` (2429) | Extract into `pipeline/` submodule | `pipeline/mod.rs` (orchestration/wiring), `pipeline/declaration.rs` (step_declaration), `pipeline/application.rs` (step_app), `pipeline/timeline_exec.rs` (run_timeline_backed_action + helpers) |
| `resolution.rs` (2263) | Extract into `resolution/` submodule | `resolution/mod.rs` (re-exports), `resolution/types.rs` (ResolutionOutcome, TargetEntry, TargetableSnapshot, resolve_targets, select_* helpers ~230 LOC), `resolution/skill_extract.rs` (skill_base_damage, skill_damage_curve, etc. — the ~15 `skill_*` helper fns ~200 LOC), `resolution/apply.rs` (apply_damage_only, apply_heal_only, apply_cleanse_only, apply_legacy_ops ~1600 LOC) |
| `turn_system/mod.rs` (916) | Extract types into `turn_system/types.rs` | Move `ActionIntent`, `EnemyTurnRequestQueue`, `ResolveActorsQuery` type alias (~80 LOC) + helper fns `set_phase`, `emit_combat_event`, `emit_kernel_transition`, `emit_combat_beat` (~90 LOC) into `turn_system/helpers.rs`. mod.rs keeps systems only. |
| `action_query.rs` (904) | Extract into `action_query/` submodule | `action_query/mod.rs` (re-exports), `action_query/types.rs` (snapshot structs + enums: CombatQuerySnapshot, UnitQuerySnapshot, ActionQueryKind, ImplementationStatus, ToughnessAffordance, TargetAffordance, ResourceKind, ResourceAffordanceDetail, ActionAffordance ~300 LOC), `action_query/legality.rs` (query_intent_legality, target_status_for_unit, kit_has_skill, resolve_action_skill, resource_detail_status, build_resource_details, aggregate_target_status, action_and_resource_status_for_snapshot, query_action_affordance ~550 LOC) |

**Re-export strategy:** Each split module re-exports its public items from `mod.rs`. The parent `combat/mod.rs` existing re-exports continue to work. Zero external import changes needed.

**Verify:** `cargo check && cargo test` after each file split.

### Wave 10 — Split large files (tier 2: 650-900 LOC)

| File | LOC | Split strategy |
|------|-----|----------------|
| `runtime/applier.rs` (870) | Extract into `runtime/applier/` submodule | `applier/mod.rs` (intent_applier fn + IntentQueue/IntentExecutionMeta structs + emit_event), `applier/effects.rs` (apply_deal_damage, apply_break_toughness, apply_status, apply_buff, apply_damage_modifier, apply_delay_turn, apply_advance_turn, apply_add_energy, apply_revive, apply_grant_free_skill, apply_blueprint_signal, apply_set_blueprint_state) |
| `runtime/runner.rs` (756) | Keep as-is — it's a single FSM with tightly coupled methods. Splitting would scatter the state machine across files. |
| `mechanics/follow_up.rs` (1072) | Extract into `mechanics/follow_up/` submodule | `follow_up/mod.rs` (re-exports), `follow_up/types.rs` (FollowUpOriginKind, FollowUpIntent, FollowUpTrace, FollowUpDecision, FollowUpSkipReason ~80 LOC), `follow_up/triggers.rs` (follow_up_listener_system + evaluation helpers ~350 LOC), `follow_up/form_identity.rs` (form_identity_listener_system + FormIdentitySnapshot + helpers ~250 LOC), `follow_up/resolve.rs` (resolve_follow_up_action_system ~300 LOC) |
| `mechanics/damage.rs` (723) | Keep as-is — it's types (AttackContext, TriangleMods, DamageBreakdown) + pure functions (triangle_modifiers, calculate_damage). Cohesive single-responsibility, splitting would separate formula from its types. |

**Verify:** `cargo check && cargo test` after each file split.

### Wave 11 — Update captures + verify

1. Update captures to reflect final module structure
2. Full verification: `cargo check && cargo test`
3. Grep for remnants of old patterns (`combat::api::` without re-export alias)
4. Write SUMMARY.md

## Risk notes

- Wave 8: 52 files changed, but purely mechanical rename. `pub use runtime as api;` alias catches any missed imports.
- Wave 9-10: File splits are internal restructuring. Re-exports at each level ensure no external breakage.
- runner.rs and damage.rs are kept as-is: runner is a cohesive FSM, damage is types+formulas.
