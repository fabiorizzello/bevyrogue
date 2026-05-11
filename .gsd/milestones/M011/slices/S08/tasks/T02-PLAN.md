---
estimated_steps: 23
estimated_files: 13
skills_used: []
---

# T02: Form Identity infrastructure + Greymon canonical demo (GrantEnergy + EnergyGained event)

Build the Form Identity vertical end-to-end with Greymon as the canonical demo per the roadmap demo string ('primo hit fire del round genera +5 Energy via Form Identity').

**Schema layer:**
1) `src/combat/kit.rs` — Add a new enum `FormIdentityTrigger` with all 4 variants up-front so T03/T04 don't need to re-touch this file: `OnFirstHitVsTagThisRound(DamageTag)`, `OnStatusApplied(StatusEffectKind)`, `OnFirstSkillCastWithTag(DamageTag)`, `OnAttackVsAttribute(Attribute)`. Add `FormIdentityConfig { trigger: FormIdentityTrigger, action: SkillId }` mirroring `FollowUpConfig`. Extend `UnitSkills` with `pub form_identity: Option<FormIdentityConfig>` (initialize to `None` in all existing UnitSkills construction sites — fallback in turn_system/mod.rs around line 274 is one such site).
2) `src/combat/round_flags.rs` — Add `pub form_identity_used: bool` (Default::default = false). Update doc comment.
3) `src/data/units_ron.rs` — Add `#[serde(default)] pub form_identity: Option<FormIdentityConfig>` to `UnitDef`. Update the `round_trip_unit_def` test to include the new field (set to None for Agumon).
4) `src/data/skills_ron.rs` — Add `Effect::GrantEnergy(i32)` to the `Effect` enum. Add a round-trip test for it. Update `parse_canonical_skills_ron` count assertion: `book.0.len()` from 59 → 60 (only Greymon's form_identity skill ships in T02; T03/T04 will increment further).
5) `src/combat/state.rs` — extend `ResolvedAction` with `pub energy_grant: i32`.

**Listener / pipeline:**
6) `src/combat/follow_up.rs` — Add `form_identity_listener_system` paralleling `follow_up_listener_system`. It uses a NEW query `FormIdentityRosterQuery` that adds `Option<&'static RoundFlags>` to FollowUpRosterQuery. The listener iterates events, evaluates against the new FormIdentityTrigger enum (only OnFirstHitVsTagThisRound is wired in T02 — others stub-return TriggerMismatch), checks `RoundFlags.form_identity_used == false`, and emits a `FollowUpIntent` reusing the existing scheduler. ALSO add a flag-flip step: after emitting the FollowUpIntent, write `form_identity_used = true` via Commands or by extending FollowUpIntent with a `mark_form_identity_used: bool` field that the resolver consumes. Simpler approach: extend the existing `ResolveActorsQuery` (already has Option<&mut RoundFlags>) — set the flag inside `resolve_follow_up_action_system` when the intent is a Form Identity intent. To distinguish, add an enum tag on FollowUpIntent: `pub origin_kind: FollowUpOriginKind { FollowUp, FormIdentity }` (default FollowUp). Use this tag inside the resolver. **MEM053 gotcha:** the `ResolveActorsQuery` alias in follow_up.rs:82-99 must stay structurally identical to turn_system/mod.rs — no field additions needed here since RoundFlags is already at index 12 (S07 already extended both).
7) `src/combat/turn_system/mod.rs:222-224` — In the TurnAdvanced loop, set `flags.form_identity_used = false` next to `flags.break_sealed = false`.
8) `src/combat/mod.rs` — Register `form_identity_listener_system` in the App schedule alongside `follow_up_listener_system`. Confirm headless (no `windowed` gate needed).

**Effect application:**
9) `src/combat/events.rs` — Add `EnergyGained { unit_id: UnitId, amount: i32 }` variant to `CombatEventKind`. Update any `match event.kind` exhaustive matchers downstream to handle the new variant (likely `_ => {}` is sufficient for log/jsonl_logger paths).
10) `src/combat/resolution.rs` — Add `fn skill_grant_energy(effects: &[Effect]) -> i32` extractor following the `skill_base_damage` pattern. Wire `energy_grant` field in `resolve_action`. In `apply_effects` (around line 148+), when `resolved.energy_grant > 0`, mutate the attacker's `Energy` component directly via `Energy::gain(amount)` (BYPASS RoundEnergyTracker per the research; this is round-capped by form_identity_used). Push `CombatEventKind::EnergyGained { unit_id: attacker.id, amount }` to the events vec.
11) `apply_effects` MUST receive a `&mut Energy` for the attacker — this is a new query argument. Trace the call chain back to pipeline.rs and turn_system/mod.rs and add `Option<&mut Energy>` to ResolveActorsQuery (extending it from 13 to 14 elements). **Critical: extend BOTH the canonical query in turn_system/mod.rs AND the alias in follow_up.rs:82-99 (MEM053).** If this proves too wide a query, an alternative is a separate `EnergyMutQuery` and an extra Commands.entity().get_mut::<Energy>() pattern — but the unified query is simpler.

**Data:**
12) `assets/data/units.ron` — Greymon (UnitId(12)) gets `form_identity: Some((trigger: OnFirstHitVsTagThisRound(Fire), action: SkillId("greymon_form_identity")))`. Existing `follow_up: ...` block stays. (Other Adults will be wired in T03/T04 — leave their form_identity unset for now; `#[serde(default)]` makes that legal.)
13) `assets/data/skills.ron` — Add one new skill `greymon_form_identity`: damage_tag: Fire, sp_cost: 0, effects: [GrantEnergy(5)]. Update the MVP id assertion list in `parse_canonical_skills_ron` to include `"greymon_form_identity"`.

**Test:**
14) `tests/form_identity.rs` (NEW) — Add test `greymon_first_fire_hit_grants_energy`: spawn Greymon vs a Fire-vulnerable target, drive a Fire basic, assert exactly one EnergyGained{amount:5} event emitted, attacker's Energy.current is 5, RoundFlags.form_identity_used == true. Add test `greymon_second_fire_hit_blocked`: drive two Fire basics in same round, assert only one EnergyGained event total. Add test `greymon_resets_next_turn`: drive one Fire basic, advance turn, drive a second Fire basic, assert two EnergyGained events (one per round).

**Failure modes:** Energy gain through RoundEnergyTracker.try_gain will silently swallow the 5 (10-cap secondary, but ALSO secondary_gained accumulates from basic-attack drips elsewhere). Bypass by writing directly to `Energy::gain` — the once-per-round flag IS the cap. Modifier-only skills (base_damage = 0, no toughness hit) emitting OnDamageDealt{amount:0} would be noisy — short-circuit OnDamageDealt emission when base_damage == 0 AND toughness_damage == 0 in apply_effects. Plan ~5 lines for this guard.

**Load profile:** Listener iterates the full unit roster (≤4 ally + 2-3 enemies in MVP). Evaluation per-event is O(roster). No new allocations on the hot path beyond what follow_up_listener_system already does — Vec<FollowerSnapshot> is reused.

**Negative tests:** form_identity field absent from RON parses to None (Childs and Devimon: covered by the existing parse_canonical_units_ron test, which must still pass without modification because of `#[serde(default)]`).

## Inputs

- `src/combat/kit.rs`
- `src/combat/round_flags.rs`
- `src/combat/events.rs`
- `src/combat/state.rs`
- `src/combat/resolution.rs`
- `src/combat/follow_up.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/mod.rs`
- `src/combat/energy.rs`
- `src/data/units_ron.rs`
- `src/data/skills_ron.rs`
- `assets/data/units.ron`
- `assets/data/skills.ron`

## Expected Output

- `src/combat/kit.rs`
- `src/combat/round_flags.rs`
- `src/combat/events.rs`
- `src/combat/state.rs`
- `src/combat/resolution.rs`
- `src/combat/follow_up.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/mod.rs`
- `src/data/units_ron.rs`
- `src/data/skills_ron.rs`
- `assets/data/units.ron`
- `assets/data/skills.ron`
- `tests/form_identity.rs`

## Verification

1) `cargo check` clean (after every layer: types, listener, resolution, data, test). 2) `cargo test --test form_identity greymon_first_fire_hit_grants_energy` passes. 3) `cargo test --test form_identity greymon_second_fire_hit_blocked` passes. 4) `cargo test --test form_identity greymon_resets_next_turn` passes. 5) `cargo test --lib data::units_ron::tests::round_trip_unit_def` passes (round-trip with new field). 6) `cargo test --lib data::skills_ron::tests::parse_canonical_skills_ron` passes (60 skills). 7) `cargo test` (full suite) — all binaries green including pipeline_dispatch, follow_up_triggers, encounter_e2e, follow_up_chains.

## Observability Impact

New CombatEventKind::EnergyGained variant emitted on every Form Identity grant. New `target: "combat.form_identity"` log target with info!/debug! parity to combat.follow_up. RoundFlags.form_identity_used queryable in tests (component snapshot). Listener flag-set is observable via FollowUpTrace (extend with `origin_kind: FollowUpOriginKind` so JSONL distinguishes follow-ups from form identity).
