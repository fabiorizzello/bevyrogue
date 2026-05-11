# S05: Resource caps (R073) + Child mechanics — UAT

**Milestone:** M011
**Written:** 2026-04-27T20:25:21.747Z

## S05 UAT: Resource Caps and Child Mechanics

**Preconditions:**
- Cargo build successful: `cargo build --tests`
- All 341 tests passing: `cargo test`

### Test Case 1: Child SP Discount After 2 Consecutive Basics

**Steps:**
1. Spawn a Child unit with SpPool {current: 5, max: 5} and BasicStreak {count: 0}
2. Execute Action 1: Basic attack (sp_cost 0, ult_effect GainFromBasic)
   - Expected: BasicStreak.count = 1, SpPool unchanged (5)
3. Execute Action 2: Basic attack
   - Expected: BasicStreak.count = 2, SpPool unchanged (5)
4. Execute Action 3: Skill attack (sp_cost 3)
   - Expected: Effective cost = 3 - 1 (discount) = 2
   - Expected: SpPool = 5 - 2 = 3
   - Expected: BasicStreak.count = 0 (reset after discount)
5. Execute Action 4: Skill attack (sp_cost 3, no streak)
   - Expected: Effective cost = 3 (no discount)
   - Expected: SpPool = 3 - 3 = 0
   - Expected: BasicStreak.count = 0 (unchanged)

**Verification:** Run `cargo test --test resource_caps -- child_discount_after_two_basics --nocapture`

---

### Test Case 2: SP Non-Basic Cap Enforcement

**Steps:**
1. Create RoundSpTracker {secondary_gained: 0, external_gained: 0}
2. Attempt to gain 1 non-Basic SP
   - Expected: actual gain = 1, tracker.non_basic_gained = 1
3. Attempt to gain 1 non-Basic SP
   - Expected: actual gain = 1, tracker.non_basic_gained = 2
4. Attempt to gain 1 non-Basic SP (cap exhausted)
   - Expected: actual gain = 0, tracker.non_basic_gained = 2 (clamped)
5. Call tracker.reset()
   - Expected: tracker.non_basic_gained = 0
6. Attempt to gain 2 non-Basic SP
   - Expected: actual gain = 2, tracker.non_basic_gained = 2

**Verification:** Run `cargo test --test resource_caps -- sp_non_basic_cap_enforced --nocapture`

---

### Test Case 3: Adult Unit No Discount (Negative Case)

**Steps:**
1. Spawn an Adult unit with same setup as Test Case 1
2. Execute 5 consecutive Basic attacks
   - Expected: BasicStreak.count = 5 (increments normally)
3. Execute Skill (sp_cost 3)
   - Expected: Effective cost = 3 (no -1 discount for Adult)
   - Expected: SpPool reduced by full 3

**Verification:** Covered by resolution_tests.rs unit test `test_adult_no_discount_after_basics`

---

### Test Case 4: Child 1 Basic Insufficient for Discount (Negative Case)

**Steps:**
1. Spawn a Child unit
2. Execute 1 Basic attack
   - Expected: BasicStreak.count = 1
3. Execute Skill (sp_cost 3)
   - Expected: BasicStreak does not qualify (count < 2)
   - Expected: Effective cost = 3 (no discount)
   - Expected: BasicStreak.count = 0 after (still resets, but no discount fired)

**Verification:** Covered by resolution_tests.rs unit test `test_child_one_basic_no_discount`

---

### Test Case 5: Energy Per-Turn Cap (10 Secondary / 30 External)

**Steps:**
1. Spawn a unit with Energy {current: 0, max: 100} and RoundEnergyTracker {secondary_gained: 0, external_gained: 0}
2. Gain 10 secondary Energy
   - Expected: actual gain = 10, tracker.secondary_gained = 10
3. Gain 5 more secondary Energy (cap exhausted)
   - Expected: actual gain = 0, tracker.secondary_gained = 10 (clamped)
4. Gain 30 external Energy
   - Expected: actual gain = 30, tracker.external_gained = 30
5. Gain 10 more external Energy (cap exhausted)
   - Expected: actual gain = 0, tracker.external_gained = 30 (clamped)
6. Call tracker.reset()
   - Expected: secondary_gained = 0, external_gained = 0, full budgets available

**Verification:** Covered by energy.rs unit tests: `test_secondary_cap`, `test_external_cap`, `test_caps_independent`, `test_reset_restores_budget`

---

**Summary:**
All UAT cases exercise the core contract of S05: SpPool max=5 with +2 non-Basic cap, Energy component with 10/30 caps, and Child -1 SP discount after 2+ Basics. Integration tests (Test Cases 1 & 2) prove end-to-end functionality. Unit tests (Test Cases 3, 4, 5) verify boundary cases and cap math.
