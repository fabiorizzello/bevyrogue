---
phase: M017
phase_name: "Status taxonomy v0 rewrite (canon §H.1)"
project: bevyrogue
generated: "2026-05-13T11:30:00Z"
counts:
  decisions: 9
  lessons: 4
  patterns: 5
  surprises: 3
missing_artifacts:
  - S06-ASSESSMENT.md (verdict captured in S06-SUMMARY front-matter only)
---

### Decisions

- **Reserved Burn/Shock declared in StatusEffectKind enum but rejected at load-time by RON validator (fail-fast, not silent no-op).** Burn and Shock are reserved §H.1 gas-era variants; they exist as enum arms to prevent future name collisions, but the RON allow-list immediately rejects them with a clear error listing the 5 valid canon ids. This is intentionally more aggressive than silent no-op.
  Source: S01-SUMMARY.md/Key decisions

- **Legacy semantic test assertions deleted rather than `#[ignored]`.** Existing status test assertions (DoT, SpeedModifier, action cancel) were removed entirely and replaced fresh by S03–S05. Leaving them `#[ignored]` would have obscured coverage gaps and made the test suite misleading.
  Source: S01-SUMMARY.md/Key decisions

- **StatusBag introduced as a per-unit consolidated component with single-instance-per-(target,kind) enforcement at apply().** Rather than storing multiple components or a Vec, StatusBag enforces the single-instance policy at the call site, eliminating stacking bugs at their source.
  Source: S02-SUMMARY.md/Architecture

- **BuffKind classification (Buff/Debuff) attached to StatusEffect at creation time.** Cleanse operations become a simple drain-by-kind filter: no bespoke per-status cleanse logic needed. Blessed is classified as BuffKind::Buff, making it cleanse-immune without a separate immunity flag.
  Source: S02-SUMMARY.md/Architecture

- **Chilled −20% AV implemented via derived-read at AV-gain site, not via SpeedModifier component mutation.** Mutating a shared SpeedModifier in Bevy ECS introduced ordering hazards across systems. Reading Chilled status presence directly at the AV-gain calculation site is deterministic and system-order-agnostic.
  Source: S03-SUMMARY.md/Key decisions

- **Paralyzed skip-turn implemented in process_turn_advanced_system by gating action dispatch.** When the active unit has Paralyzed in its StatusBag, the action is consumed and a TurnAdvanced event is still emitted (turn advances), but the action itself is suppressed. This keeps the turn pipeline deterministic.
  Source: S04-SUMMARY.md/Architecture

- **Slowed delay-on-apply emits TurnAdvance { amount_pct: -30 } on first apply.** Rather than modifying the turn order table retroactively, Slowed pushes the unit's next turn window forward by 30% immediately when the status is applied. This is observable in the JSONL event stream.
  Source: S04-SUMMARY.md/Architecture

- **Blessed damage multiplier (×1.15) and +1 Ult charge are both applied in apply_effects, using the existing attacker_dmg_mult thread.** S03 extended calculate_damage with status_amp_pct; S05 threaded attacker_dmg_mult using the same extension point, avoiding a separate Blessed-specific pipeline branch.
  Source: S05-SUMMARY.md/Key decisions

- **S06 observability closure verdict captured only in SUMMARY front-matter, no standalone ASSESSMENT.md.** This was a documentation shortcut — functionally correct, but it breaks the pattern of having a separate S0N-ASSESSMENT.md artifact for each slice. Accepted as a minor gap.
  Source: S06-SUMMARY.md/Verification

---

### Lessons

- **Load-time RON validator pattern surfaces actionable errors for content authors.** Validating status ids against an explicit allow-list at `DataPlugin` load time is better than compile-time gating for data files: it catches typos and migration misses before any system runs and prints a clear error listing the 5 valid ids. The reserved Burn/Shock variants in the enum prevent future enum-space collisions without granting runtime applicability.
  Source: S01-SUMMARY.md/Patterns established

- **Deferring a "nice to have" integration test (Chilled turn-order shift) in S03 created a coverage gap that persisted through S04 and S06.** The S03 assessment marked the end-to-end turn-order assertion for Chilled as "optional/deferred." Neither S04 nor S06 picked it up. Result: SC-3 (Chilled) closes as PARTIAL in the validation. Future slices should not defer integration tests without a named follow-up slice that owns the closure.
  Source: M017-VALIDATION.md/Requirement Coverage

- **StatusBag's single-instance policy makes the refresh_max_dur contract trivially verifiable.** Because apply() enforces at most one active entry per kind, the refresh test only needs to assert the post-refresh duration value — no scan for duplicates required. This simplicity was only achievable because the invariant was enforced at the structural level, not the caller level.
  Source: S02-SUMMARY.md/Patterns established

- **S05 Blessed cleanse-immune test required zero src/ changes beyond what S02 already wired.** S02's BuffKind::Buff classification of Blessed was sufficient; S05 only needed to write the integration test. This confirms that the BuffKind-classified cleanse pattern is correctly closed at the data model level.
  Source: S05-SUMMARY.md/Key decisions

---

### Patterns

- **Status taxonomy vocabulary split: 5 active canon variants vs 2 reserved gas-era variants.** Declare reserved variants in the enum to prevent future naming collisions, but reject them at the RON validator allow-list to prevent runtime applicability. The allow-list error message lists the 5 valid ids explicitly.
  Source: S01-SUMMARY.md/Patterns established

- **Load-time RON id allow-list wired into DataPlugin.** Validate all `Effect::ApplyStatus` kind fields against a known-good list at data load time (`validate_skill_book_on_load`). Fail loudly with a message listing valid options. This is the canonical pattern for data-driven status application gating.
  Source: S01-SUMMARY.md/Patterns established

- **BuffKind-classified cleanse: attach buff/debuff polarity to StatusEffect at creation, drain by kind on cleanse.** This makes cleanse logic a one-liner and avoids per-status special-casing. Buff-classified entries are immune by default; no bespoke immunity flags needed.
  Source: S02-SUMMARY.md/Patterns established

- **Derived-read speed modifier for Chilled: query StatusBag at AV-gain site instead of maintaining a separate SpeedModifier component.** Avoids system ordering hazards in Bevy ECS. Prefer derived reads over component mutation when the modifier is conditional on a status that could be applied or removed by another system in the same frame.
  Source: S03-SUMMARY.md/Key decisions

- **StatusBag per-unit component with single-instance-per-kind enforcement.** Consolidate all active status effects for an entity into one component. Enforce the "at most one entry per StatusEffectKind" invariant inside apply(), not at caller sites. This eliminates an entire class of stacking bugs at the structural level.
  Source: S02-SUMMARY.md/Architecture

---

### Surprises

- **Chilled −20% AV could not be wired as a SpeedModifier component mutation without introducing system ordering hazards.** The derived-read approach (query bag at AV-gain site) was discovered mid-S03 as the correct solution. The initial plan assumed a separate SpeedModifier component, but Bevy ECS ordering made that fragile.
  Source: S03-SUMMARY.md/Key decisions

- **Blessed cleanse-immunity was fully covered by S02's BuffKind::Buff classification — S05 needed zero src/ changes.** The delegation from S05 to S06 for observability was similarly lightweight: S06 consumed foundation work from S01–S05 without needing new src/ changes, only new test assertions.
  Source: S05-SUMMARY.md/Key decisions; S06-SUMMARY.md/What Happened

- **157 files changed across the milestone branch with 6858 insertions and 2939 deletions — larger than anticipated for a "vocabulary + apply/tick" scope.** The breadth came from the cascade of test migrations (S01) and the per-status pipeline integrations requiring coordinated changes across status_effect.rs, damage.rs, turn_system, sp.rs, and ultimate.rs in each slice.
  Source: git diff --stat master..HEAD
