# M011/S08 — Research

**Date:** 2026-04-28

## Summary

S08 introduces **Form Identity** — once-per-round conditional bonuses for the 6 MVP Adults — and removes the engine-level re-entrancy guard for follow-ups (D046, supersedes D026). Both changes are mandated by the M011 plan and explicitly named in D045/D046. The slice owns one Active requirement: **R080** (Form Identity for the 6 Adult units).

The infrastructure already exists in skeleton form: `RoundFlags` was added in S07 and is spawned on every unit, the follow-up listener (`src/combat/follow_up.rs`) is event-driven and reads `UnitSkills.follow_up`, and the 6 Adult units already have basic/skill/ult/follow-up wiring in `assets/data/units.ron`. What's missing is (a) a **second listener slot** (Form Identity is not the same kind of trigger as the existing OnEnemyBreak/OnAllyLowHp/OnEnemyKill follow-ups), (b) **new round-scoped FollowUpTrigger variants**, (c) **new modifier Effect variants** (GrantEnergy, SelfAdvance, BonusToughnessDamage, BonusDamageVsAttribute), (d) a once-per-round flag on `RoundFlags`, (e) per-Adult form identity skill assets in `skills.ron`, and (f) the removal of the `if event.follow_up_depth >= 1 { return Err(OneHopSuppressed) }` guard in `follow_up.rs:161-162`.

The Adult kits are the data the slice exists to deliver. Three of the six (Greymon/Garurumon/Kabuterimon) need **OnFirstHit/SkillOfTagThisRound + GrantEnergy** — the cheapest path. Kyubimon needs **OnStatusApplied(Freeze) + SelfAdvance**. DORUgamon needs an attack-time modifier (BonusToughnessDamage vs Cracked) and Angemon needs another (BonusDamage vs Virus or charging-attack target). The DORUgamon "Cracked" status does not yet exist and the "charging attack" Angemon prong is M012 scope (Charged Attacks deferred). These two should be **simplified to forms expressible with the already-implemented triggers** so all six Adults ship with R080 satisfied.

## Recommendation

**Build Form Identity as a parallel listener slot, not a re-use of the existing `follow_up` field.** Add a sibling `form_identity: Option<FormIdentityConfig>` field on `UnitDef` (with `#[serde(default)]` for backward compatibility) where `FormIdentityConfig = { trigger: FormIdentityTrigger, action: SkillId }`. This avoids breaking the existing `follow_up: Option<FollowUpConfig>` field, lets an Adult carry both an offensive follow-up (e.g. Greymon OnEnemyBreak → greymon_follow_up) AND a Form Identity (e.g. OnFirstHitVsTag(Fire) → greymon_form_identity), and matches the spirit of D045 ("riusa l'infrastruttura") by sharing the listener pipeline, the SkillId/Effect schema, and the CombatEvent bus — without overloading a single `Option` slot with two distinct semantics.

For trigger evaluation, add a **second listener system** (`form_identity_listener_system`) that runs in the same Bevy schedule slot as `follow_up_listener_system`, evaluates against an extended `FormIdentityTrigger` enum, consults `RoundFlags.form_identity_used` (new bool, reset alongside `break_sealed` in `advance_turn_system`), and emits a `FollowUpIntent` reusing the same scheduling pipeline downstream. Form Identity actions resolve via the existing `resolve_follow_up_action_system` — same depth=1 carrying, same lifecycle events.

For effect application, extend `Effect` with **four new modifier variants** (`GrantEnergy(i32)`, `SelfAdvance(i32)`, `BonusToughnessDamage(i32)`, `BonusDamageVsAttribute { attribute: Attribute, bonus_pct: i32 }`) and route them through `apply_effects` with a small extractor pattern matching the existing `skill_base_damage` / `skill_apply_status` pattern. For S08 demo scope, **GrantEnergy and SelfAdvance are the only two that must work end-to-end** — the bonus-damage variants can be parsed and stored but applied as a no-op stub if scope pressures emerge, with a follow-up note for S09 rebalance.

For D046, the change is small and surgical: delete the depth guard in `evaluate_follow_up`, delete the `OneHopSuppressed` variant (and its references in `ultimate.rs:77` which uses `follow_up_depth >= 1` for a different semantic — that line is a legitimate "is this a follow-up event?" check and **must stay**), update the comment on `CombatEvent.follow_up_depth` (events.rs:76-77), and rewrite `tests/follow_up_reentrancy.rs` to assert that chains DO progress (bounded by resources) instead of asserting suppression. The data side has no current Adult kit with infinite re-entrancy potential — Greymon's follow-up costs no SP but produces no further OnEnemyBreak, so chains terminate naturally. Confirm in the new test that a Greymon→Greymon→... loop does not occur because the chain doesn't re-trigger its own preconditions.

## Implementation Landscape

### Key Files

**Type / schema layer**
- `src/combat/kit.rs` — Add `FormIdentityTrigger` enum (or extend `FollowUpTrigger` with new round-scoped variants per D045 wording — see Decision Point #1 below). Add `FormIdentityConfig { trigger, action }` mirroring `FollowUpConfig`. Extend `UnitSkills` with `pub form_identity: Option<FormIdentityConfig>`.
- `src/combat/round_flags.rs` — Add `pub form_identity_used: bool` (one flag is sufficient for "1 trigger per round per unit" — finer-grained per-trigger flags only needed if a single Adult ever has two Form Identities, which R080 says is not the MVP case).
- `src/data/units_ron.rs` — Add `#[serde(default)] pub form_identity: Option<FormIdentityConfig>` to `UnitDef`. Update `round_trip_unit_def` test fixture to include it.
- `src/data/skills_ron.rs` — Extend `Effect` enum with `GrantEnergy(i32)`, `SelfAdvance(i32)`, `BonusToughnessDamage(i32)`, `BonusDamageVsAttribute { attribute: Attribute, bonus_pct: i32 }`. Add round-trip tests for each.

**Listener / pipeline layer**
- `src/combat/follow_up.rs` — (a) Add `form_identity_listener_system` paralleling `follow_up_listener_system`. It iterates the same roster snapshot but checks `form_identity` instead of `follow_up`, and only fires when `RoundFlags.form_identity_used == false`. On schedule, it must set `form_identity_used = true` (write-back via Commands or a new component-mutation pass — simpler: emit a marker event and do the flag set in the resolver). (b) Add a new `FollowerSnapshot` field `form_identity: Option<FormIdentityConfig>`. (c) **Delete** lines 161-162 (the `if event.follow_up_depth >= 1` guard) and the `OneHopSuppressed` variant. (d) Update the `FollowUpRosterQuery` to include `Option<&'static RoundFlags>` so the listener can read the flag (currently it only has Unit/Team/UnitSkills/Ko/Stunned).
- `src/combat/turn_system/mod.rs:222-224` — Reset `form_identity_used = false` next to `break_sealed = false` in the TurnAdvanced loop.
- `src/combat/resolution.rs:90-146` — Add four extractor functions (`skill_grant_energy`, `skill_self_advance`, `skill_bonus_toughness_dmg`, `skill_bonus_damage_vs_attribute`) following the `skill_base_damage` pattern. Extend `ResolvedAction` (in `src/combat/state.rs:29-44`) with the corresponding fields. Apply them in `apply_effects:148-292`:
  - `GrantEnergy` → mutate the attacker's `Energy` component; emit a new `CombatEventKind::EnergyGained { unit_id, amount }` for JSONL observability.
  - `SelfAdvance` → emit `CombatEventKind::TurnAdvance { target: attacker, amount_pct }` (already exists, just retarget self).
  - Bonus damage modifiers → multiply `base_damage` / `toughness_damage` before the formula call when conditions match.
- `src/combat/events.rs` — Add `EnergyGained { unit_id: UnitId, amount: i32 }` variant. Update the comment on `follow_up_depth` (D046: chains bounded by data, not engine).

**Data layer**
- `assets/data/units.ron` — Add `form_identity: Some((trigger: ..., action: SkillId(...)))` block on each of the 6 Adults (UnitId 12-17). Existing `follow_up` blocks stay.
- `assets/data/skills.ron` — Add 6 new skill assets `<adult>_form_identity` with the appropriate modifier-only effects. Update the `parse_canonical_skills_ron` count assertion (currently `assert_eq!(book.0.len(), 59, ...)` → 65) and add the 6 new ids to the MVP assertion list.

**Test layer**
- `tests/follow_up_reentrancy.rs` — Rewrite the OneHopSuppressed assertions: instead, assert that chained follow-ups DO emit at depth 2+, and that a kit lacking re-trigger preconditions terminates naturally after N hops where N matches kit-defined bounds. Drop the `s10_` prefix in line with the functional-naming convention (CLAUDE.md). Rename to e.g. `tests/follow_up_chains.rs`.
- `tests/form_identity.rs` (NEW) — Six end-to-end tests, one per Adult, asserting Form Identity fires once per round, does not fire twice, resets next turn, and emits the appropriate observable event. Greymon test is the canonical demo from the roadmap: "primo hit fire del round genera +5 Energy via Form Identity."
- `src/combat/follow_up_tests.rs:341-367` — Remove the `OneHopSuppressed` unit test path. The `evaluate_follow_up` matrix test can stay if the depth case is removed.

### Build Order

1. **D046 first (small, surgical, unblocks the test rewrite).** Delete the guard, delete the variant, update the events.rs comment, rewrite `follow_up_reentrancy.rs` (or split it) to assert chain progression. **Verify:** `cargo test follow_up` plus the full suite — currently 28 binaries green. The S07 toughness_categories tests do not depend on the guard.
2. **Round-scoped trigger + GrantEnergy + Greymon Form Identity (the demo).** Smallest end-to-end vertical: extend `FormIdentityTrigger`, `Effect::GrantEnergy`, `RoundFlags.form_identity_used`, `UnitDef.form_identity`, `UnitSkills.form_identity`, the new listener system, `EnergyGained` event, the resolution.rs path, and the Greymon kit (units.ron + skills.ron). **Verify:** new `tests/form_identity.rs::greymon_first_fire_hit_grants_energy` plus the full suite.
3. **The other 5 Adults (parallel application of the same plumbing).** Garurumon/Kabuterimon use the same OnFirstHitOfTagThisRound + GrantEnergy combo with different DamageTags. Kyubimon needs `OnStatusApplied(StatusEffectKind)` trigger + `SelfAdvance` effect. DORUgamon and Angemon — see Open Risks for the simplified MVP framing. **Verify:** the 6 form_identity tests all pass.
4. **JSONL observability sweep.** Confirm the new `EnergyGained` event appears in the JSONL log via an extension to `tests/damage_breakdown_log.rs` or a new fixture.

### Verification Approach

Per-step:
- `cargo check` after each schema change.
- `cargo test --no-fail-fast` for the full suite (currently 28 binaries; expect 29 with the new `tests/form_identity.rs`).
- `cargo test --test follow_up_reentrancy` (or its rename) to verify D046 chain semantics.
- `cargo test --test form_identity` for the 6 new fixtures.
- `cargo run --bin combat_cli` for a manual sanity check that Greymon's first Fire hit visibly emits the EnergyGained event in the in-CLI event log (per S04's combat_cli scaffold).
- `! grep -rn 'OneHopSuppressed\|follow_up_depth >= 1' src/combat/follow_up.rs` — must return zero matches (the `ultimate.rs:77` use is **kept** because it's a different semantic). Actually be careful: the grep should target only the suppression site, not blanket-delete depth checks.

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| Per-unit per-round flag store | `RoundFlags` component (S07) | Already spawned on every unit, already reset in `advance_turn_system`, already wired through `ResolveActorsQuery`. Just add a field. |
| Trigger → action dispatch via event bus | `follow_up_listener_system` + `resolve_follow_up_action_system` (`follow_up.rs`) | The Form Identity flow is structurally identical: snapshot roster → match trigger → emit `FollowUpIntent` → resolver runs the action through the standard pipeline. Reuse the resolver path verbatim. |
| Self-AV modification | `ActionValue::self_advance(amount)` in `src/combat/av.rs:38` plus the existing `CombatEventKind::TurnAdvance { target, amount_pct }` | Don't introduce a parallel "self-advance" event; just emit TurnAdvance with `target: attacker`. |
| Once-per-round semantics | The S07 break-seal pattern: `RoundFlags` field, set on use, cleared in `advance_turn_system` Part 1 | Identical mechanic. Mirror it for `form_identity_used`. |
| RON schema additions with backward compat | `#[serde(default)]` (already used for `tempo_resistant` and `toughness_category` in `units_ron.rs:38-42`) | Unblocks add-without-migrate for `form_identity`. |

## Constraints

- **Headless-first (D015):** all new systems must register without `windowed`. The Form Identity listener has no UI; trivially compliant.
- **No per-Digimon code (D020):** every Form Identity must be expressible in RON via the new `(trigger, action)` pair. **No** `if unit.id == "greymon"` branches in `src/combat/`.
- **Determinism (R019):** Form Identity itself is deterministic (no RNG), but if a future variant uses an accuracy roll it must go through `CombatRng` (the S02 resource). For S08 no RNG is needed.
- **Event-bus single source of truth (D022):** Form Identity outputs (energy gained, self-advance) must be emitted as `CombatEvent` variants, not as silent component mutations. JSONL must observe them.
- **`Energy` component is currently spawned but never mutated by combat systems.** Bootstrap inserts `Energy::default()` (`bootstrap.rs:133`) and `RoundEnergyTracker::default()` but no apply_effects path touches `Energy`. S08 adds the first such mutation. The existing `RoundEnergyTracker` with its 10-secondary / 30-external caps is **not** the right surface for Form Identity — those caps are designed for the per-action gain economy (basic-attack drips, follow-up rewards). Form Identity gains should bypass `RoundEnergyTracker` (they are inherently round-capped to one trigger) and write directly to `Energy.gain(amount)`.
- **Bevy 0.18 query mutability:** the form_identity listener needs read access to `RoundFlags`, but the resolver needs write access. The existing `ResolveActorsQuery` already has `Option<&'static mut RoundFlags>` (mod.rs:66, follow_up.rs:97), so the resolver can flip the flag in `resolve_follow_up_action_system` after `step_app` returns. The listener-side check must use a fresh snapshot read (not the mut query).

## Common Pitfalls

- **`follow_up_depth >= 1` is used in TWO places with TWO meanings.** `follow_up.rs:161` is the D026 suppression guard (DELETE). `ultimate.rs:77` is the `OnAllyFollowUp` ult-charge trigger (KEEP — it's asking "is this skill cast a follow-up?"). Mass-deleting the depth check will break ultimate charging for Renamon (UltAccumulationTrigger::OnAllyFollowUp). The grep target must be precise.
- **`follow_up_reentrancy.rs:285-409` makes hard assertions on `OneHopSuppressed`.** This whole test must be rewritten, not patched. Rename to a chain-semantics test that asserts the chain DOES proceed and is bounded by kit resources (not by the engine).
- **The 28-binary count assertion lives in S07's summary, not in code.** No test asserts "28 binaries"; it's prose. Adding `tests/form_identity.rs` raises the count to 29.
- **`parse_canonical_skills_ron` (skills_ron.rs:204) hard-codes `assert_eq!(book.0.len(), 59, ...)`.** Adding 6 form-identity skills must update this to 65 (or whatever final count) and extend the MVP id assertion list.
- **Greymon's Form Identity per design says "First hit vs Burning each round"** — that's a *conditional* trigger ("vs Burning"), not just "first Fire hit". The MVP-cheapest reading is `OnFirstHitVsTagThisRound(Fire)` — fires on any first Fire hit per round, no Burning check. The design also says "5 Energy O +10% Toughness dmg" (either/or). Pick one for S08; "5 Energy" is simpler and matches the roadmap demo string. Note in code that the Burning predicate is deferred to S09 if needed.
- **DORUgamon's "Cracked" status doesn't exist in `StatusEffectKind`** (Burn/Freeze/Shock only). Don't add Cracked in S08 — simplify DORUgamon's Form Identity to **`OnFirstSkillCastWithTag(Dark)` → `BonusToughnessDamage`** (modeled as a one-shot bonus that the next toughness hit consumes — or, simpler, a free `dorugamon_form_identity` skill with high ToughnessHit). Document the deviation; full Cracked is M012.
- **Angemon's Form Identity per design has two prongs** ("vs Virus OR vs charging attack"). Charged Attacks are M012. Simplify to **`OnAttackVsAttribute(Virus)` → `BonusDamageVsAttribute`** for S08. The trigger fires when the attacker (Angemon) hits a Virus defender; once-per-round.
- **The `RoundEnergyTracker` 10-secondary / 30-external caps will silently swallow Form Identity GrantEnergy if you route through `try_gain`.** Bypass it; write directly to `Energy::gain`.
- **The `form_identity` UnitDef field default must be `None`** so Childs and Devimon parse cleanly without modification.

## Open Risks

- **D046 risk surfaced explicitly in the M011 risk register:** "D046 senza cap engine espone loop patologici non vincolati dai dati." Mitigation already named: UAT in S09 validates that finite kit resources bound chains. For S08, the verification bar is "no chain in the current MVP roster reaches infinite depth on a fixed encounter" — verify with a chain-progression test using the existing roster (no cap, but no infinite loop either because no Adult re-triggers its own preconditions on its own follow-up).
- **DORUgamon and Angemon Form Identity simplifications may not satisfy the spirit of R080.** R080 says "Ogni Adult ha 1 trigger condizionale once-per-round + effect modifier configurato 100% in RON." Both simplifications meet the letter (one trigger, one effect, RON-configured) but lose design intent (Cracked, charging attack). Surface this as a Deviation in S08-SUMMARY for product owner sign-off in S09 UAT. R080 stays Active until UAT.
- **Form Identity actions reuse `resolve_follow_up_action_system`,** which expects a damage-dealing skill (it runs the full step_app pipeline). Modifier-only skills (Damage(0), no toughness hit, just GrantEnergy) will run through `apply_effects` and emit `OnDamageDealt { amount: 0, ... }` — semantically noisy. Either (a) make `apply_effects` skip the `OnDamageDealt` emission when `base_damage == 0` and there's no toughness hit, or (b) accept the zero-damage event as a side-effect and document it for S09. (a) is cleaner; ~10 lines.
- **`form_identity_used` is a single bool per unit.** Fine for R080's "1 trigger per Adult" but wrong if S09 rebalance asks for "Adult X has trigger A AND trigger B." If that surfaces, escalate to per-trigger flag bitset on `RoundFlags` — defer the design decision until S09 actually requests it.

## Sources

- D045 (DECISIONS.md): Form Identity reuses follow-up infrastructure; extends FollowUpTrigger with round-scoped variants and Effect with modifier variants; adds RoundFlags.
- D046 (DECISIONS.md): Re-entrancy bounded by data, not engine. Removes the guard. UAT in S09 validates.
- combat_design.md sez. 9 + sez. 12 (lines 165-289): Form Identity definitions per Adult.
- M011-CONTEXT.md (Risks table): "D046 senza cap engine espone loop patologici non vincolati dai dati — UAT in S09 valida."
- S07-SUMMARY.md "Patterns Established for S08+": confirms `RoundFlags` is the canonical per-round flag store and that the per-turn reset hook is in place.
- M011-ROADMAP.md S08 demo string: "scenario CLI Greymon: primo hit fire del round genera +5 Energy via Form Identity."
