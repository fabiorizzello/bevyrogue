# S03 Research: Effect::Cleanse { count: Option<u8> } primitive

**Slice:** S03 of M019 — adds a kernel-level Cleanse primitive that removes N (or all) non-immune debuffs from a target's `StatusBag`, with deterministic ordering and a `CombatEvent::OnCleansed` event. No franchise-specific or skill-specific logic in the kernel — defers selective cleanse / immunity hooks to M021.

## Summary

The cleanse infrastructure is already 80% present:

- `StatusBag::cleanse_debuffs()` (src/combat/status_effect.rs:79) removes **all** debuffs and returns the kinds removed.
- `classify_buff_kind` (status_effect.rs:38) already encodes "Blessed = Buff" → cleanse-immune; every other variant returns Debuff.
- The S02 plumbing (Effect → ResolvedAction extractor → apply_*_only helper → apply_effects branch → pipeline fan-out) is the exact template to copy.

**Note on the roadmap wording.** M019-ROADMAP.md and M019-CONTEXT.md reference "flag immune already present on the StatusEntry." There is **no `immune` field on `StatusInstance`**. The "immune flag" in practice is `classify_buff_kind(kind) == Buff`. Today only `Blessed` returns `Buff`. The kernel must continue to derive immunity from `classify_buff_kind` — no hardcoded list of cleanse-immune status IDs and no new immunity flag. M021 will add per-skill immunity overrides via `trait Skill`.

The only net-new work for S03 is:
1. A new `cleanse_n(count: Option<u8>) -> Vec<StatusEffectKind>` method on `StatusBag` that respects ordering (duration_remaining DESC, insertion-index ASC tiebreak).
2. `Effect::Cleanse { count: Option<u8>, target: TargetShape }` + ally-side-only validator (Heal validator pattern).
3. `CombatEventKind::OnCleansed { kinds: Vec<StatusEffectKind> }`.
4. `apply_cleanse_only` helper + `ResolvedAction.cleanse_count: Option<u8>` field + pipeline wiring (Single/SelfOnly cleanse hook beside `status_to_apply` at pipeline.rs:1722 vs. AllAllies extension of the existing fan-out branch at pipeline.rs:340).
5. `tests/cleanse_effect.rs` direct-call integration tests (apply_effects pattern from `tests/dr_pipeline.rs` / `tests/heal_effect.rs`).

## Implementation Landscape

### Data surface (src/data/skills_ron.rs)

Add the variant alongside `Effect::Heal` at line 234:

```rust
/// Remove up to `count` non-immune debuffs from the target's StatusBag, ordered by
/// duration_remaining DESC with insertion-index ASC as deterministic tiebreak.
/// `count: None` removes all non-immune debuffs. Buff-classified entries (today only
/// Blessed) are never removed. `target` must be an ally-side shape.
Cleanse {
    count: Option<u8>,
    target: TargetShape,
}
```

Validator: clone the `Effect::Heal` block at skills_ron.rs:507-526. Reject `Bounce`, `AllEnemies`, `Blast` with `LegalityReasonCode::WrongSide`.

### Event surface (src/combat/events.rs)

Add at the end of `CombatEventKind`:

```rust
/// Emitted once per cleanse application on a target. `kinds` is the ordered list of
/// statuses removed (matches the cleanse selection order). Empty kinds = no-op cleanse
/// (e.g. count=Some(0) or no debuffs present) — still emitted for telemetry parity with
/// OnHealed { amount: 0 }.
OnCleansed { kinds: Vec<StatusEffectKind> },
```

### StatusBag API (src/combat/status_effect.rs)

Add a new method beside `cleanse_debuffs()` — leave the existing method untouched (additive only):

```rust
/// Remove up to `count` non-immune (Debuff-classified) instances. Selection order:
///   1. duration_remaining DESC (longest-remaining first)
///   2. tiebreak: insertion index ASC (lower Vec idx first)
/// `count = None` ⇒ remove all non-immune debuffs (equivalent to cleanse_debuffs).
/// Returns the kinds removed in selection order.
pub fn cleanse_n(&mut self, count: Option<u8>) -> Vec<StatusEffectKind>
```

Implementation: enumerate `self.0.iter().enumerate()`, filter `classify_buff_kind == Debuff`, collect `(idx, &inst)`, sort by `(Reverse(duration_remaining), idx)`, truncate to `count`, collect indices to remove, then `swap_remove` in descending idx order (or build new Vec preserving non-removed). Determinism: stable sort by `(Reverse(dur), idx)` is sufficient since indices are unique.

### Resolution (src/combat/resolution.rs)

- Add `skill_cleanse_count(effects: &[Effect]) -> Option<Option<u8>>` extractor — returns `Some(count)` when the skill carries an `Effect::Cleanse`, else `None`. (Wrapping in `Option<_>` distinguishes "not a cleanse skill" from "cleanse with count=None".)
- Extend `ResolvedAction` in src/combat/state.rs with `pub cleanse_count: Option<Option<u8>>` (or a flatter design — see Open Decisions). Populate in `resolve_action` (resolution.rs:283).
- Add `apply_cleanse_only(action: &ResolvedAction, bag: &mut StatusBag) -> (ResolutionOutcome, Vec<CombatEventKind>)` mirroring `apply_heal_only` at line 561. KO check: cleanse on KO is a no-op (no event), same policy as Heal. Body calls `bag.cleanse_n(count)`, emits `OnCleansed { kinds }` (always, even when kinds is empty — telemetry parity).
- **apply_effects signature constraint:** `defender_status: Option<&StatusBag>` is read-only (resolution.rs:597). Two routing options:
  - **(a) Wire cleanse at pipeline-level** (recommended): mirror `status_to_apply` at pipeline.rs:1722. apply_effects stays unchanged; pipeline reads `resolved.cleanse_count`, and after apply_effects succeeds calls `bag.cleanse_n` then emits OnCleansed. Single/SelfOnly path = same site as status_to_apply. AllAllies path = extend the AllAllies branch at pipeline.rs:340-358 to also pull `def_bag` via the row tuple.
  - **(b) Change apply_effects signature** to `Option<&mut StatusBag>` for defender_status. Larger blast radius — touches every apply_effects call site (12+ test sites in resolution.rs alone, plus follow_up.rs's local ResolveActorsQuery — see MEM001 gotcha).
  - **Recommendation: (a).** Status mutation already happens at pipeline-level today (status_to_apply pattern), so cleanse fits naturally there. Keeps apply_effects pure-read on defender_status. Smaller diff.
- Add `target_shape_is_executable_now`: no change needed — Single/SelfOnly/AllAllies are already whitelisted (added in S02).

### Pipeline wiring (src/combat/turn_system/pipeline.rs)

Two sites:

1. **Single/SelfOnly cleanse hook** — add a block beside the `status_to_apply` site at line 1722. If `resolved.cleanse_count.is_some()` and `outcome.succeeded` and `!outcome.ko`, call `bag.cleanse_n(count)` on the defender bag (using `defender_bag` already in scope) and emit `OnCleansed`. Mirror the StatusBag fallback at line 1742 (fresh bag on missing component) — but for cleanse on missing bag, the result is trivially empty; emit `OnCleansed { kinds: [] }`.

2. **AllAllies cleanse fan-out** — extend the existing `AllAllies` branch at pipeline.rs:340-358. Today the branch only mut-borrows `def_unit`. For cleanse it must also mut-borrow `def_bag`. Two sub-options:
  - Detect cleanse vs. heal at the top of the fan-out, then route to either `apply_heal_only` (existing) or a new `apply_cleanse_only` per-target. Either-or per skill is the simpler contract — Heal and Cleanse cannot coexist in the same skill (only one Effect::* extractor wins per resolved action; the planner should add a validator forbidding mixed Heal+Cleanse to keep the kernel simple, or define a fixed precedence and document it).

### Test file (tests/cleanse_effect.rs)

Pattern: direct-call to `apply_cleanse_only`, no Bevy world. Mirrors `tests/heal_effect.rs` (referenced by MEM003).

Cases:
- `cleanse_count_some2_removes_two_longest_debuffs` — bag with 4 debuffs (durations 3,1,2,4), count=Some(2) → removes the ones with duration 4 and 3 in that order; remaining bag has the two with duration 1 and 2; `OnCleansed { kinds: [dur4_kind, dur3_kind] }`.
- `cleanse_count_some2_tie_break_lower_insertion_index_first` — two debuffs same duration: the one inserted first goes first in the removed kinds vec.
- `cleanse_count_none_removes_all_debuffs_keeps_blessed` — bag with Blessed + 3 debuffs, count=None → all 3 debuffs removed, Blessed survives.
- `cleanse_count_some_zero_emits_empty_event_no_state_change` — bag with 2 debuffs, count=Some(0) → no removals, `OnCleansed { kinds: [] }` still emitted.
- `cleanse_blessed_only_no_op` — bag with only Blessed, count=Some(5) → no removals (Blessed is immune); event with empty kinds.
- `cleanse_count_exceeds_debuff_count_removes_all_no_panic` — bag with 2 debuffs, count=Some(10) → removes both, no panic.
- `cleanse_on_ko_target_no_op_no_event` — KO defender → no state change, no event (mirrors heal KO policy).
- `cleanse_on_empty_bag_emits_empty_event` — empty StatusBag, count=Some(3) → no removals, `OnCleansed { kinds: [] }`.

Naming is functional per CLAUDE.md — no `s##_` prefix. Tests deterministic (no RNG, no wall-clock). Reuse `ally()` helper pattern from tests/heal_effect.rs.

## Files & Purpose

| File | Purpose |
|---|---|
| `src/data/skills_ron.rs` | Add `Effect::Cleanse { count, target }`; clone Heal validator for ally-side-only check. |
| `src/combat/events.rs` | Add `CombatEventKind::OnCleansed { kinds }`. |
| `src/combat/status_effect.rs` | Add `StatusBag::cleanse_n(count)` with duration-DESC + idx-ASC ordering. |
| `src/combat/resolution.rs` | Add `skill_cleanse_count` extractor, populate `ResolvedAction.cleanse_count`, add `apply_cleanse_only` helper. |
| `src/combat/state.rs` | Add `cleanse_count: Option<Option<u8>>` to `ResolvedAction`. |
| `src/combat/turn_system/pipeline.rs` | Hook cleanse at status_to_apply site (Single/SelfOnly) and extend AllAllies fan-out (line 340) to mut-borrow def_bag and dispatch cleanse. |
| `src/combat/follow_up.rs` | Likely no-op: `Effect::*` is not matched here (only `Effect::Damage`/`ToughnessHit` in test builders). Verify exhaustiveness in case rustc warns on new variant. **MEM001 gotcha applies if a new component is added to the resolution query — we are NOT adding a component, only writing to existing StatusBag, so no ResolveActorsQuery arity change is expected.** |
| `tests/cleanse_effect.rs` | New integration test file (8 cases above). |

## Natural Seams (suggested task split for planner)

- **T01 — Data surface + event variant + validator.** Add `Effect::Cleanse`, `CombatEventKind::OnCleansed`, validator (ally-side only). Exhaustiveness fallout (resolution.rs/follow_up.rs match arms). `cargo check` green, no behavioural wiring.
- **T02 — `StatusBag::cleanse_n` + `apply_cleanse_only` + `ResolvedAction.cleanse_count` + extractor.** No pipeline wiring yet. Inline `#[cfg(test)] mod tests` for `cleanse_n` ordering (compact unit test allowed per CLAUDE.md). `cargo check` green.
- **T03 — Pipeline wiring (Single/SelfOnly via status_to_apply site; AllAllies via fan-out extension) + `tests/cleanse_effect.rs`.** Full integration suite green; no regression in heal_effect / dr_pipeline / follow_up_triggers.

This mirrors the S02 T01/T02/T03 layout exactly.

## First Proof (highest-risk / biggest unblocker)

`StatusBag::cleanse_n` ordering and tiebreak determinism. If the selection is wrong (e.g. uses iteration order without considering duration), the entire primitive is observably broken in JSONL replays. Land this with inline unit tests in T02 before the pipeline wiring in T03 — same approach taken in M017/S02 for the StatusBag lifecycle skeleton.

## Verification

- `cargo test --test cleanse_effect` — all 8 cases pass deterministically.
- `cargo test` (full suite) — green, no regression in heal_effect.rs, dr_pipeline.rs, follow_up_triggers.rs, status_blessed_offensive.rs.
- `cargo check` — clean.
- JSONL identity neutral check: existing test fixtures contain **no** Cleanse skills, so combat traces stay byte-identical to pre-S03 — same non-regression invariant honoured in S02.

## Open Decisions for Planner

These should be locked down in the slice plan (PLAN.md) before T01 so executor agents have no ambiguity:

1. **`ResolvedAction.cleanse_count` shape.** Recommended: `pub cleanse_count: Option<Option<u8>>` where outer `None` = "not a cleanse skill" and `Some(inner)` = "cleanse with `inner` count". Alternative: flat `pub cleanse: bool` + `pub cleanse_count: Option<u8>`. The first is more typesafe; the second is more readable. Pick one explicitly.
2. **Event shape.** Recommended: single `OnCleansed { kinds: Vec<StatusEffectKind> }` per target (atomic, mirrors Heal). Alternative explored in CONTEXT.md: per-removal `OnStatusExpired { reason: "cleansed" }`. The atomic event is simpler to consume in follow-up listeners and matches the heal precedent.
3. **count=Some(0) policy.** Recommended: emit `OnCleansed { kinds: [] }` for telemetry parity with Heal-at-full-HP (amount=0 still emits). Alternative: skip emission entirely.
4. **Mixed Heal + Cleanse in the same skill.** Recommended: validator forbids it (only one of `Effect::Heal` or `Effect::Cleanse` per skill). Defers any precedence question to M021 trait Skill design. Cheap and safe.
5. **Cleanse on KO target.** Recommended: no-op silently (no event), same policy as Heal on KO. CONTEXT.md is silent on this — locking it down here.
6. **count = Some(0) vs count = None.** Both are valid encodings but mean different things. Some(0) = "remove zero items now" (no-op for this cast). None = "remove all non-immune debuffs". Document both in the Effect doc comment.

## Risks

None significant. The only failure mode is non-deterministic ordering in `cleanse_n` (e.g. relying on HashSet iteration), which the inline unit tests in T02 will catch immediately. No new RNG, no new wall-clock, no API changes to apply_effects. The MEM001 gotcha around `ResolveActorsQuery` in follow_up.rs does **not** apply because S03 does not add a new component — it mutates the existing `StatusBag`.

## Sources

- `.gsd/milestones/M019/M019-ROADMAP.md` (slice S03 demo line, success criterion #3).
- `.gsd/milestones/M019/M019-CONTEXT.md` (architectural decisions: Cleanse ordering durata-DESC + slot_index ASC tiebreak).
- `.gsd/milestones/M019/slices/S02/S02-SUMMARY.md` (template: T01/T02/T03 layout, apply_heal_only pattern).
- `.gsd/milestones/M019/slices/S02/S02-PLAN.md` (template: integration closure, verification structure).
- `src/combat/status_effect.rs` (existing `cleanse_debuffs`, `classify_buff_kind`, `StatusBag` layout).
- `src/combat/resolution.rs` (apply_heal_only at line 561; apply_effects branching at line 619-635; resolve_targets AllAllies at line 95).
- `src/combat/turn_system/pipeline.rs` (status_to_apply mutation site at line 1722; AllAllies fan-out at line 340).
- `tests/heal_effect.rs` (direct-call integration test pattern; MEM003).
- Memory MEM001 (follow_up.rs local ResolveActorsQuery gotcha — does NOT apply here, no new component).

## Skills Discovered

None installed. The slice is pure local Rust/Bevy work, no new external technology. Existing `bevy-ecs-expert` skill is already available via system prompt and applies to the pipeline mut-borrow ergonomics if the planner needs it.
