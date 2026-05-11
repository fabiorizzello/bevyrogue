# S01: UI-readiness gap matrix and legality contract — UAT

**Milestone:** M012
**Written:** 2026-04-30T19:22:04.007Z

# S01 UAT — UI-readiness gap matrix and legality contract

## Preconditions
- Rust toolchain installed (`cargo` available)
- Working directory: project root (bevyrogue)
- No `windowed` feature needed — tests are headless

## Test Cases

### TC-01: Gap matrix doc exists and is non-empty
**Steps:**
1. Run `ls -la docs/combat_ui_readiness_gap_matrix.md`
**Expected:** File exists, size > 0 bytes (actual: ~10KB)

### TC-02: Gap matrix test binary passes all 7 checks
**Steps:**
1. Run `cargo test --test ui_readiness_gap_matrix_docs`
**Expected:** `test result: ok. 7 passed; 0 failed; 0 ignored`
Checks include: classification vocabulary present, R085/D053/D054 links present, hard-boundary text present, all required mechanics named, no TBD/TODO placeholder text, each required status family (Implemented/ToFixNow/Deferred/Hidden) used at least once, downstream contract reason examples named.

### TC-03: Legality contract doc exists and is non-empty
**Steps:**
1. Run `ls -la docs/skill_legality_contract.md`
**Expected:** File exists, size > 0 bytes (actual: ~11KB)

### TC-04: Legality contract test binary passes all 10 checks
**Steps:**
1. Run `cargo test --test skill_legality_contract_docs`
**Expected:** `test result: ok. 10 passed; 0 failed; 0 ignored`
Checks include: all four status types (ActionStatus/TargetStatus/ResourceStatus/ImplementationStatus) present, all four variant families (Enabled/Disabled/Deferred/Hidden) present, R084 and D053 links present, engine parity requirement stated, no skill-ID-specific UI rule boundary stated, reason codes separated from display strings, all 24+ required reason codes present (NotActiveUnit, WrongPhase, AttackerKo, AttackerStunned, MissingSkill, SpShortfall, UltimateNotReady, UnimplementedEffect, UnimplementedTargetShape, TargetNotFound, TargetIsSelf, TargetIsCommander, WrongSide, TargetKo, TargetNotKo, TargetFullHp, TargetNotDamaged, NoValidTargets, ToughnessEnemyOnly, TamerGaugeDeferred, TamerCommandDeferred, ChargedTelegraphDeferred, EnemyTraitDeferred, EnergyCapReached).

### TC-05: Both test binaries pass together (integration check)
**Steps:**
1. Run `cargo test --test ui_readiness_gap_matrix_docs --test skill_legality_contract_docs`
**Expected:** Both binaries report `test result: ok`, combined 17 tests, 0 failures.

### TC-06: Hard boundary text is machine-readable
**Steps:**
1. Run `grep -c "skill-ID-specific" docs/skill_legality_contract.md docs/combat_ui_readiness_gap_matrix.md`
**Expected:** Both files contain the hard-boundary phrase (count ≥ 1 each)

### TC-07: No placeholder text in either doc
**Steps:**
1. Run `grep -iE 'TBD|TODO' docs/skill_legality_contract.md docs/combat_ui_readiness_gap_matrix.md`
**Expected:** No output (exit code 1) — placeholder-free

### TC-08: Downstream slices can use these docs as contract baseline
**Precondition:** S02 or later slice references these docs
**Steps:**
1. Confirm that S02-S07 task plans reference `docs/skill_legality_contract.md` or `docs/combat_ui_readiness_gap_matrix.md` as their contract baseline
**Expected:** At least one downstream slice's plan cites these artifacts explicitly
