---
estimated_steps: 6
estimated_files: 4
skills_used: []
---

# T01: Data surface: Effect::Cleanse variant + OnCleansed event + ally-side & mixed-effect validators

Add the Effect::Cleanse { count: Option<u8>, target: TargetShape } variant to the skill DSL, the CombatEventKind::OnCleansed { kinds } event variant, and the validator logic. Validator rejects enemy-side target shapes (Bounce/AllEnemies/Blast) using LegalityReasonCode::WrongSide (clone the Effect::Heal validator block). Validator also rejects skills that carry BOTH Effect::Heal and Effect::Cleanse (mixed Heal+Cleanse forbidden in v0 — deferred to M021). No behavioural wiring yet — only data-model surface plus exhaustiveness fallout in match arms across resolution.rs and follow_up.rs (and any other site rustc flags). cargo check must be green.

Locked decisions for executor:
- Variant fields: `count: Option<u8>` and `target: TargetShape` (order matters — keep `count` first to match research).
- Event shape: `OnCleansed { kinds: Vec<StatusEffectKind> }` — single atomic event per target (mirrors OnHealed).
- Validator file location: same module/pattern as Heal validator at skills_ron.rs:507-526.
- Mixed Heal+Cleanse rejection uses a new LegalityReasonCode variant if one does not already fit; pick the most semantically appropriate existing code (e.g. MixedEffectKinds) or add a new variant with a one-line doc comment.

## Inputs

- `.gsd/milestones/M019/slices/S03/S03-RESEARCH.md`
- `.gsd/milestones/M019/M019-CONTEXT.md`
- `.gsd/milestones/M019/M019-ROADMAP.md`
- `.gsd/milestones/M019/slices/S02/S02-SUMMARY.md`
- `src/data/skills_ron.rs`
- `src/combat/events.rs`
- `src/combat/resolution.rs`
- `src/combat/follow_up.rs`

## Expected Output

- `src/data/skills_ron.rs`
- `src/combat/events.rs`
- `src/combat/resolution.rs`
- `src/combat/follow_up.rs`

## Verification

cargo check --tests must be clean (0 errors). cargo test --test validation_snapshot must still pass. No Effect::Cleanse handling wired yet, so existing tests cannot regress.

## Observability Impact

Introduces CombatEventKind::OnCleansed variant in the event bus — automatically flows through JSONL logger via existing serde derive. No new wiring.
