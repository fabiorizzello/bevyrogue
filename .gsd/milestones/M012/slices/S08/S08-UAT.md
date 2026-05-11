# S08: Enemy counterplay declaration surface — UAT

**Milestone:** M012
**Written:** 2026-05-01T16:51:10.317Z

# S08 UAT: Enemy Counterplay Declaration Surface

## Preconditions
- Working directory: project root with `cargo test-dev` available
- `assets/data/units.ron` is the canonical roster
- All S08 task summaries show `verification_result: passed`

## Test Cases

### TC-01: Canonical Devimon declarations round-trip from RON through query
**Steps:**
1. Run `cargo test-dev --test enemy_counterplay_affordance`
2. Verify test `canonical_devimon_projection_surfaces_implemented_and_deferred_states` passes

**Expected:** TempoAnchor affordance has `ImplementationStatus::Implemented`; TypeTrap and ReactiveArmor affordances have `ImplementationStatus::Deferred` with non-empty reason codes.

---

### TC-02: Shielded unit produces Implemented BreakSeal; Armored unit does NOT produce Implemented ReactiveArmor
**Steps:**
1. Run `cargo test-dev --test enemy_counterplay_affordance`
2. Verify test `shielded_break_seal_is_implemented_while_armored_reactive_armor_stays_deferred` passes

**Expected:** Shielded fixture → BreakSeal `Implemented`. Armored fixture → ReactiveArmor `Deferred`. No false positive.

---

### TC-03: Empty minion yields zero affordances without panic
**Steps:**
1. Run `cargo test-dev --test enemy_counterplay_affordance`
2. Verify test `empty_minion_declarations_stay_empty_and_hidden_telegraphs_stay_hidden` passes

**Expected:** Empty declaration list → zero trait affordances, no charged telegraph, no panic.

---

### TC-04: Consumer source files contain no hardcoded enemy names or skill IDs for counterplay decisions
**Steps:**
1. Run `cargo test-dev --test action_affordance_consumers`
2. Verify tests `combat_cli_source_does_not_hardcode_counterplay_names` and `combat_windowed_source_does_not_reintroduce_ko_or_skill_id_hardcoding` pass

**Expected:** Source-scan tests confirm neither `combat_cli.rs` nor `combat_panel.rs` contains forbidden strings (`devimon`, `ogremon`, charged skill IDs, or `signature_traits`-based branching).

---

### TC-05: Deferred reason codes survive snapshot round-trip and remain visible
**Steps:**
1. Run `cargo test-dev --test action_affordance_consumers`
2. Verify tests `counterplay_snapshot_exposes_deferred_trait` and `counterplay_deferred_reason_codes_remain_visible_in_resource_status` pass

**Expected:** Deferred declaration with reason code round-trips through UnitQuerySnapshot and the formatted label is non-empty and non-generic.

---

### TC-06: Hidden charged telegraph surfaces correct ResourceStatus
**Steps:**
1. Run `cargo test-dev --test action_affordance_consumers`
2. Verify test `counterplay_snapshot_exposes_hidden_charged_telegraph` passes

**Expected:** Hidden charged-attack declaration → `ResourceStatus::Hidden` with the correct `ResourceKind`.

---

### TC-07: Scenario TTK tests show no behavioral regression
**Steps:**
1. Run `cargo test-dev --test scenario_boss_ttk --test scenario_miniboss_ttk`

**Expected:** Both scenario tests pass; boss and miniboss TTK values match pre-S08 baselines.

---

### TC-08: Windowed feature compiles cleanly
**Steps:**
1. Run `cargo check --features "dev windowed"`

**Expected:** Finished with no errors.

---

### TC-09: Full test suite remains green
**Steps:**
1. Run `cargo test-dev`

**Expected:** All test targets pass, 0 failures across the full suite.

---

## Edge Cases

- **Roster fixture backward compatibility:** Any inline `UnitDef` that omits `enemy_traits` and `charged_attack` must still deserialize (enforced by serde defaults). Verified by T01 backward-compatibility parse test.
- **Armored → ReactiveArmor guard:** A unit with `toughness_category: Armored` but no explicit `ReactiveArmor` declaration must yield `Deferred` for Reactive Armor, not `Implemented`. Covered by TC-02.
- **No declaration reuse across team:** `EnemyCounterplayKit` is spawned per unit from its `UnitDef`; ally units with no declarations correctly return empty affordance lists.
