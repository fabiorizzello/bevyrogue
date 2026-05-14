---
sliceId: S01
uatType: artifact-driven
verdict: PASS
date: 2026-05-13T08:45:00.000Z
---

# UAT Result — S01: Enum rewrite + RON migration + tests cascade

## Checks

| Check | Mode | Result | Notes |
|-------|------|--------|-------|
| Grep guard — src/ and tests/ clean (no Burn/Freeze/Shock/DeepFreeze) | artifact | PASS | Only matches in `src/combat/status_effect.rs` (enum declarations for reserved Burn/Shock + inline unit tests), `src/data/skills_ron.rs` (validator guard), and `src/combat/turn_system/mod.rs` (reserved no-op match arms). Zero matches in `tests/`. |
| Grep guard — canon variants present (Heated/Chilled/Paralyzed/Slowed/Blessed) | artifact | PASS | 150+ matches across src/combat/status_effect.rs, src/data/skills_ron.rs, assets/data/skills.ron, assets/data/units.ron, src/combat/turn_system/tests.rs, and integration test files. |
| cargo check headless | runtime | PASS | Exit 0, no errors. |
| Full integration test suite (cargo test) | runtime | PASS | All targets green: 0 failed, 0 ignored across all test targets. Two test fixture bugs found and fixed during UAT: (1) `tests/status_multi_kind_coexist.rs` — attacker `UnitSkills.skills` was `vec![]`, so "chilled" and "blessed" skills were rejected by `kit_has_skill` legality check; (2) `tests/status_refresh_max_dur.rs` — same issue for "h1" and "h5" skill ids. Both tests now pass after adding the skill ids to the `UnitSkills.skills` vec. |
| RON load validation — canon ids accepted (cargo run --bin combat_cli) | runtime | PASS | Exit 0, no loader errors, combat events flow normally (OnKernelTransition, OnDamageDealt, OnHitTaken, UltGain). |
| RON validator rejects legacy ids (kind: BurnV0 injected) | runtime | PASS | RON parser emits: `Unexpected variant named 'BurnV0' in enum 'StatusEffectKind', expected one of 'Heated', 'Chilled', 'Paralyzed', 'Slowed', 'Blessed', 'Burn', or 'Shock'`. Note: lists all 7 declared enum variants (5 canon + 2 reserved), not strictly 5 canon ids — this is the RON deserializer's native behavior. |
| Edge case: reserved Burn/Shock rejected at load time (not silently no-op) | runtime | PASS | Gap discovered and fixed during UAT: `validate_skill_book()` existed but was never called during asset loading. Wired into `DataPlugin` via new `validate_skill_book_on_load` system (`src/data/mod.rs`). After fix, injecting `kind: Burn` causes a panic: `SkillBook validation failed: skill_id=flame_bite category=Semantic reason=UnimplementedEffect detail=ApplyStatus uses reserved status kind Burn; valid ids are: heated, chilled, paralyzed, slowed, blessed`. |

## Overall Verdict

PASS — All 6 UAT checks and both edge cases pass after fixing two test fixture bugs (empty `UnitSkills.skills` vecs) and wiring `validate_skill_book` into the asset loading pipeline.

## Notes

### Fixes applied during UAT

**1. Test fixture bugs (2 files)**

`tests/status_multi_kind_coexist.rs` and `tests/status_refresh_max_dur.rs` had `UnitSkills.skills: vec![]`. The legality check `kit_has_skill` (in `src/combat/action_query.rs:609`) requires `ActionIntent::Skill { skill_id }` to match `kit.basic`, `kit.ultimate`, or one of `kit.skills`. Skills not in the kit's list are rejected with `LegalityReasonCode::MissingSkill` before the action resolves — so the status effects were never applied.

Fix: added the test skill ids to `UnitSkills.skills` in each test fixture.

**2. validate_skill_book not wired into loading pipeline**

`validate_skill_book()` in `src/data/skills_ron.rs` correctly rejects reserved `Burn`/`Shock` variants, but was only called in unit tests — never during actual asset loading. As a result, `kind: Burn` in skills.ron was silently accepted and would have been a no-op in combat.

Fix: added `validate_skill_book_on_load` system to `DataPlugin` in `src/data/mod.rs`. System subscribes to `AssetEvent<SkillBook>` via `MessageReader` and panics with a descriptive error if validation fails, listing the 5 valid canon ids.

### Deviation from UAT spec (minor)

Check 6 expected the error to list "the 5 valid canon ids". The RON native enum parser lists all 7 declared variants (including reserved Burn/Shock). The custom validator (for reserved variants via check 6 edge case) correctly lists only the 5 canon ids in its error message.
