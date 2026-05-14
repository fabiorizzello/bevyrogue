# S03: Effect::Cleanse { count: Option<u8> } primitive

**Goal:** Add Effect::Cleanse { count: Option<u8>, target: TargetShape } as a kernel primitive that removes up to N non-immune debuffs from an ally's StatusBag, ordered duration-DESC with insertion-index ASC tiebreak. Emit CombatEventKind::OnCleansed { kinds } per target. Buff-classified entries (today only Blessed) are never removed. Mixed Heal+Cleanse in the same skill is rejected by the validator. Kernel stays franchise-agnostic — no skill_id branching, no hardcoded immunity list.
**Demo:** Test integration tests/cleanse_effect.rs: cleanse count=2 rimuove 2 debuff non-immuni; Blessed (immune) non rimosso; count=None svuota tutti i debuff non-immuni; CombatEvent::Cleansed nel JSONL.

## Must-Haves

- 1) Effect::Cleanse variant present in skills_ron.rs with ally-side validator (rejects Bounce/AllEnemies/Blast via LegalityReasonCode::WrongSide); validator also rejects skills carrying both Effect::Heal and Effect::Cleanse. 2) StatusBag::cleanse_n(count) removes non-immune debuffs in deterministic order (duration_remaining DESC, idx ASC tiebreak); returns the removed kinds in selection order; count=None removes all non-immune debuffs; count=Some(0) is a no-op; Blessed (Buff classification) never removed. 3) apply_cleanse_only helper: KO target is silent no-op (no event, sp_ok=true), otherwise calls cleanse_n and emits OnCleansed { kinds } even when kinds is empty (telemetry parity with OnHealed amount=0). 4) Pipeline wiring: Single/SelfOnly cleanse dispatched at the status_to_apply site in pipeline.rs; AllAllies cleanse fan-out extends the existing AllAllies branch and dispatches per-target via apply_cleanse_only. 5) tests/cleanse_effect.rs: 8 deterministic apply_effects-pattern tests cover count=Some(N) ordering, tiebreak by insertion index, count=None empties all non-immune debuffs (Blessed survives), count=Some(0) no-op with empty event, Blessed-only bag no-op, count exceeds debuff count, KO no-op no event, empty bag empty event. 6) cargo test green across the full suite (no regression in heal_effect.rs, dr_pipeline.rs, follow_up_triggers.rs, status_blessed_offensive.rs); cargo check clean. 7) Kernel remains franchise-agnostic: no Digimon names, no skill_id branches, no hardcoded cleanse-immune list — immunity derives solely from classify_buff_kind.

## Proof Level

- This slice proves: Integration tests via direct apply_effects calls (no Bevy world). 8 cases in tests/cleanse_effect.rs deterministically exercise ordering, tiebreak, immunity, KO policy, and edge counts. Inline #[cfg(test)] unit tests for StatusBag::cleanse_n ordering inside status_effect.rs to land the highest-risk primitive before pipeline wiring.

## Integration Closure

Cleanse is wired at the same pipeline mutation site as status_to_apply (Single/SelfOnly) and inside the AllAllies fan-out branch added by S02. OnCleansed events flow through the existing CombatEvent bus and JSONL logger via the serde::Serialize derive — no extra wiring. No existing test fixture uses Effect::Cleanse, so JSONL traces stay byte-identical to pre-S03 (non-regression invariant honoured).

## Verification

- New CombatEventKind::OnCleansed { kinds: Vec<StatusEffectKind> } emitted per target on every cleanse application (empty kinds vector for no-op cleanses, mirroring OnHealed amount=0). Flows through the existing CombatEvent bus and JSONL logger automatically.

## Tasks

- [x] **T01: Data surface: Effect::Cleanse variant + OnCleansed event + ally-side & mixed-effect validators** `est:S`
  Add the Effect::Cleanse { count: Option<u8>, target: TargetShape } variant to the skill DSL, the CombatEventKind::OnCleansed { kinds } event variant, and the validator logic. Validator rejects enemy-side target shapes (Bounce/AllEnemies/Blast) using LegalityReasonCode::WrongSide (clone the Effect::Heal validator block). Validator also rejects skills that carry BOTH Effect::Heal and Effect::Cleanse (mixed Heal+Cleanse forbidden in v0 — deferred to M021). No behavioural wiring yet — only data-model surface plus exhaustiveness fallout in match arms across resolution.rs and follow_up.rs (and any other site rustc flags). cargo check must be green.
  - Files: `src/data/skills_ron.rs`, `src/combat/events.rs`, `src/combat/resolution.rs`, `src/combat/follow_up.rs`
  - Verify: cargo check --tests must be clean (0 errors). cargo test --test validation_snapshot must still pass. No Effect::Cleanse handling wired yet, so existing tests cannot regress.

- [x] **T02: StatusBag::cleanse_n + apply_cleanse_only + ResolvedAction.cleanse_count + extractor** `est:M`
  Implement the cleanse primitive end-to-end except for pipeline dispatch. Steps:
  - Files: `src/combat/status_effect.rs`, `src/combat/state.rs`, `src/combat/resolution.rs`
  - Verify: cargo check --tests clean. cargo test --lib runs the inline #[cfg(test)] mod tests for cleanse_n — all ordering / tiebreak / count edge cases pass. Full integration suite (cargo test) remains green since apply_cleanse_only is not yet reachable from the pipeline.

- [x] **T03: Pipeline wiring (Single/SelfOnly + AllAllies fan-out) + tests/cleanse_effect.rs integration suite** `est:M`
  Wire apply_cleanse_only into the pipeline and add the integration test file. Two pipeline sites and one new test file.
  - Files: `src/combat/turn_system/pipeline.rs`, `src/combat/follow_up.rs`, `tests/cleanse_effect.rs`
  - Verify: cargo test --test cleanse_effect — all 8 cases pass deterministically. cargo test — full integration suite green (heal_effect.rs, dr_pipeline.rs, follow_up_triggers.rs, status_blessed_offensive.rs, validation_snapshot.rs unaffected). cargo check clean.

## Files Likely Touched

- src/data/skills_ron.rs
- src/combat/events.rs
- src/combat/resolution.rs
- src/combat/follow_up.rs
- src/combat/status_effect.rs
- src/combat/state.rs
- src/combat/turn_system/pipeline.rs
- tests/cleanse_effect.rs
