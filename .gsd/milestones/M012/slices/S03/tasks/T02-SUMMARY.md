---
id: T02
parent: S03
milestone: M012
key_files:
  - src/data/skills_ron.rs
  - assets/data/skills.ron
key_decisions:
  - Use a first-error `SkillBookValidationError` with `skill_id`, category, stable reason code, and detail for diagnostics.
  - Represent row skills as deferred with `UnimplementedTargetShape`, mixed damage+revive skills as deferred with `UnimplementedEffect`, and passive form-identity entries as hidden/non-selectable catalog data.
duration: 
verification_result: passed
completed_at: 2026-04-30T21:58:12.852Z
blocker_discovered: false
---

# T02: Migrated canonical skills.ron to explicit targeting metadata and added semantic skill-book validation.

**Migrated canonical skills.ron to explicit targeting metadata and added semantic skill-book validation.**

## What Happened

I updated the canonical skill catalog so every one of the 72 skill entries now carries explicit `targeting` and `implementation` metadata in proper RON syntax. Single-target offensive skills remain implemented, revive skills are now truthfully KO-ally single-target entries, row-based skills are deferred with `UnimplementedTargetShape`, mixed damage+revive semantics on `angemon_ult` are deferred with `UnimplementedEffect`, and passive form-identity entries are hidden rather than exposed as normal actions.

In `src/data/skills_ron.rs`, I added a structured `SkillBookValidationError` plus `validate_skill_book(&SkillBook) -> Result<(), SkillBookValidationError>` so semantic contradictions fail loudly with the offending skill id, a stable category, a stable legality reason, and a human detail string. I also expanded the in-module tests to cover canonical parse+validate, missing targeting metadata parse failure, unknown-field parse failure, implemented-non-single rejection, revive-with-non-KO rejection, row-damage/single-target mismatch, and mixed-effect deferral.

I verified the final state by rerunning the focused `cargo test-dev skills_ron` target and the canonical catalog-count checks after the last code change. The focused test suite passed cleanly, and the catalog still reports 72 `targeting:` entries and 72 `implementation:` entries.

## Verification

Fresh verification completed after the last code change: `cargo test-dev skills_ron` passed, including the canonical parse+validate test and the new negative semantic cases. I also rechecked the migrated catalog counts and confirmed there are exactly 72 `targeting:` entries and 72 `implementation:` entries in `assets/data/skills.ron`.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test-dev skills_ron` | 0 | ✅ pass | 289ms |
| 2 | `test "$(grep -c 'targeting:' assets/data/skills.ron)" -eq 72 && test "$(grep -c 'implementation:' assets/data/skills.ron)" -eq 72` | 0 | ✅ pass | 2ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/data/skills_ron.rs`
- `assets/data/skills.ron`
