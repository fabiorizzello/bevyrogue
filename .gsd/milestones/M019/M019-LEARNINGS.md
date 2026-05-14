---
phase: M019
phase_name: DR pipeline + Heal/Cleanse primitives + PerHop guard
project: bevyrogue
generated: 2026-05-14T09:30:00Z
counts:
  decisions: 6
  lessons: 5
  patterns: 5
  surprises: 3
missing_artifacts: []
---

### Decisions

- **DR sum unclamped at DrBag level; clamped at calculate_damage via (1.0-sum).max(0.0).**
  Values >1.0 are legal in the bag — the formula provides a natural floor. This avoids per-entry caps that would complicate stacking semantics.
  `Source: S01-SUMMARY.md/Key decisions`

- **DR applied as final multiplicative step after break amplification, not inside calculate_damage formula.**
  Keeps qualitative (triangle/tag/break) and quantitative (DR) axes testable in isolation. Observability is free: CombatEvent::Damage carries both pre_dr and final amounts.
  `Source: M019-CONTEXT.md/Architectural Decisions`

- **Floor division for Heal: (hp_max * pct) / 100, not ceiling arithmetic.**
  Consistent with the existing revive formula; no ceiling complexity introduced. Caps at hp_max.
  `Source: S02-SUMMARY.md/Key decisions`

- **cleanse_n ordering: duration_remaining DESC, insertion-index ASC tiebreak.**
  Deterministic without extra data structures. Removes the longest-lived debuffs first, which is the most intuitive "best cleanse" UX policy.
  `Source: S03-SUMMARY.md/Key decisions`

- **ResolvedAction.cleanse_count: Option<Option<u8>> — outer None = not a cleanse skill, inner None = cleanse all.**
  The two-layer Option is the only unambiguous representation of the three-state distinction: not-a-cleanse / cleanse-all / cleanse-N.
  `Source: S03-SUMMARY.md/Key decisions`

- **D001: PerHop runtime guard truncates to available coefficients and emits OnActionFailed diagnostic; never panics.**
  Silent clamp would mask bugs in future blueprint emitters. Reusing OnActionFailed keeps the event taxonomy stable and makes the anomaly observable in JSONL traces.
  `Source: .gsd/DECISIONS.md/D001`

---

### Lessons

- **follow_up.rs has its own local ResolveActorsQuery independent of resolution.rs.**
  When a new component (DrBag, or any future component) is added to the main resolution query tuple, it must also be added to follow_up.rs's local query or the project will fail to compile with a tuple-arity error. (MEM001 already captured — skip persist.)
  `Source: S01-SUMMARY.md/Key decisions`

- **SelfOnly was missing from shape_is_executable and target_shape_is_executable_now.**
  Discovered mid-S02 when AllAllies wiring was added. Both shapes must be added together whenever a new ally-side target shape is introduced. The plan only mentioned AllAllies; SelfOnly was silently broken since its addition.
  `Source: S02-SUMMARY.md/Deviations`

- **Mixed Heal+Cleanse on a single skill is rejected by the legality validator (LegalityReasonCode::MixedEffectKinds).**
  The kernel DSL enforces single-effect-kind per skill until M021 (trait Skill + SkillCtx). Blueprint authors attempting multi-effect skills will see a validation error at skill load time, not a runtime failure.
  `Source: S03-SUMMARY.md/Key decisions`

- **Adding a field to ResolvedAction requires touching all existing construction sites.**
  S03's cleanse_count field required adding `cleanse_count: None` to 9 existing integration test fixtures that construct ResolvedAction directly. The fix is purely additive but requires a codebase-wide search.
  `Source: S03-SUMMARY.md/Deviations`

- **AllAllies fan-out uses actors.get_mut(def_entity) instead of get_many_mut to avoid entity-collision when caster is in the target list.**
  Attempting get_many_mut with a list that may contain the caster entity twice (caster is both attacker and an AllAllies target) will panic in Bevy. The single-entity get_mut loop avoids this.
  `Source: S02-SUMMARY.md/Key decisions`

---

### Patterns

- **apply_effects direct-call pattern (no Bevy world spin-up) for deterministic integration tests.** (MEM003 already captured — skip persist.)
  `Source: S01-SUMMARY.md/Patterns established`

- **apply_heal_only / apply_cleanse_only mirror each other's internal structure: KO guard → compute → mutate → emit event.**
  New single-target effect handlers should follow this same layout for consistency. The KO guard always comes first; effects on KO targets are a silent no-op, sp_ok=true, no event.
  `Source: S02-SUMMARY.md/Patterns established`

- **Either-or dispatch in the AllAllies branch (heal XOR cleanse), enforced by the legality validator at DSL load time.**
  The AllAllies branch in pipeline.rs checks which effect kind is present and routes exclusively. Validator ensures only one kind reaches the branch. This keeps the branch simple and guarantees the validator is the single gate for mixed-effect prevention.
  `Source: S03-SUMMARY.md/Patterns established`

- **Pre-loop guard pattern for PerHop: check curve length before entering the hop loop, emit diagnostic event once, clamp bound to available coefficients.**
  Diagnostic is emitted exactly once regardless of how many hops are skipped. Matches the existing pool-exhaustion truncate pattern in pipeline.rs.
  `Source: S04-SUMMARY.md/Patterns established`

- **AllAllies fan-out reuses the Blast/AllEnemies resource-hoist-then-per-target-dispatch pattern from pipeline.rs.**
  Resource consumption (SP, ult charge) is hoisted before the per-target loop and consumed once per cast, regardless of fan-out width. New fan-out shapes should follow the same hoist-then-dispatch layout.
  `Source: S02-SUMMARY.md/Patterns established`

---

### Surprises

- **DR was applied as a post-calculate_damage subtraction in T03 (S01) before T02 had landed its damage.rs changes.**
  An intermediate commit state had inconsistent DR application logic. T02's multiplicative approach superseded T03's additive subtraction. The final code is correct; the divergence only existed in intermediate commits and was caught before slice completion.
  `Source: S01-SUMMARY.md/Deviations`

- **apply_cleanse_only was raised from pub(crate) to pub to allow import from tests/.**
  The integration test pattern (apply_effects direct call from tests/) requires public visibility for the targeted helper. This mirrors apply_heal_only, which was already pub. Not anticipated in the plan.
  `Source: S03-SUMMARY.md/Deviations`

- **SelfOnly shape was silently non-executable — broken since it was first added.**
  The omission from shape_is_executable and target_shape_is_executable_now was only discovered when AllAllies was wired in S02. Any skill using SelfOnly targeting would have been rejected at legality check without this fix, making the gap behaviorally invisible in existing tests.
  `Source: S02-SUMMARY.md/Deviations`
