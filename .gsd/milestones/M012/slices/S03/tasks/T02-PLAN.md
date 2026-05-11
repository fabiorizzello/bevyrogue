---
estimated_steps: 42
estimated_files: 3
skills_used: []
---

# T02: Migrate canonical skills.ron and add loud validation

---
estimated_steps: 5
estimated_files: 2
skills_used:
  - tdd
  - verify-before-complete
---

Migrate the canonical skill catalog to the new metadata schema and add semantic validation tests that make false or unsupported affordances fail loudly before S04 consumes the data.

Steps:
1. Update all 72 entries in `assets/data/skills.ron` with `targeting` and `implementation` fields. Use consistent RON formatting near existing fields so future content authors can copy patterns.
2. Classify canonical skills without skill-ID-specific runtime rules: normal single-target damage/follow-up/ultimate skills are `Implemented` enemy/alive/single/forbid-self; revive skills are `Implemented` ally/KO/single; Row skills are `Deferred(reason: UnimplementedTargetShape)`; mixed-effect target semantics such as `angemon_ult` must not be marked as a normal implemented action while semantics are unresolved.
3. In `src/data/skills_ron.rs`, implement `validate_skill_book(&SkillBook) -> Result<(), SkillBookValidationError>` or equivalent structured error API. It must reject contradictory metadata with the offending skill id and stable reason/category.
4. Extend in-module tests to cover metadata round-trip, canonical parse+validate, missing required metadata fails parse, unknown metadata field fails parse, `Damage(target: Row)` with `targeting.shape: Single` fails validation, `Revive` with non-KO targeting fails validation, and `Implemented` plus non-single shape fails while current execution supports only Single.
5. Ensure tests assert stable machine reason codes such as `UnimplementedTargetShape`, not localized display copy.

Must-haves:
- Canonical `assets/data/skills.ron` parses and validates cleanly.
- Invalid sample skills fail loudly at either deserialization or semantic validation, depending on whether they are structurally malformed or contradictory.
- Row and mixed-effect unsupported semantics are explicit in data as deferred/disabled metadata, not silently inferred from effects.
- `angemon_ult` remains visible to future consumers only according to truthful implementation metadata; do not normalize it with a skill-id exception in code.

Failure Modes:
- **Malformed RON**: serde parse error should point to the data entry; keep changes small and run the focused parser test often.
- **Contradictory but well-typed data**: semantic validation must return an error rather than letting future UI infer false affordances.
- **Reason mismatch**: reason enum names must stay aligned with `docs/skill_legality_contract.md` vocabulary.

Load Profile:
- Shared resources: one in-memory skill catalog.
- Per-operation cost: validation is O(number of skills × effects per skill), acceptable for a 72-skill catalog.
- 10x breakpoint: catalog validation remains linear; if content grows 10x, error reporting should still name the first offending skill clearly.

Negative Tests:
- **Malformed inputs**: missing `targeting` and unknown targeting field RON snippets must fail deserialization.
- **Error paths**: well-typed contradictory snippets must return validation errors with skill id and reason/category.
- **Boundary conditions**: non-damage revive and form-identity skills must validate from explicit metadata rather than damage-effect defaults.

Verification:
- `cargo test-dev skills_ron`
- `grep -c "targeting:" assets/data/skills.ron` returns 72.
- `grep -c "implementation:" assets/data/skills.ron` returns 72.

Inputs:
- `src/data/skills_ron.rs` — schema from T01 and existing canonical tests.
- `assets/data/skills.ron` — canonical skill catalog to migrate.
- `docs/skill_legality_contract.md` — stable reason/status vocabulary to mirror in code tests.

Expected Output:
- `assets/data/skills.ron` — all canonical skills include explicit targeting and implementation metadata.
- `src/data/skills_ron.rs` — validation API and focused parser/semantic validation tests.

## Inputs

- `src/data/skills_ron.rs`
- `assets/data/skills.ron`
- `docs/skill_legality_contract.md`

## Expected Output

- `assets/data/skills.ron`
- `src/data/skills_ron.rs`

## Verification

cargo test-dev skills_ron && test "$(grep -c 'targeting:' assets/data/skills.ron)" -eq 72 && test "$(grep -c 'implementation:' assets/data/skills.ron)" -eq 72

## Observability Impact

Validation errors become the diagnostic surface for bad skill data. They should include enough detail for a future agent to identify whether the failure is structural RON, metadata/effect mismatch, unsupported target shape, or mixed/deferred semantics.
