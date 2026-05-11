# S08: Form Identity framework + 6 Adult wired + rimozione D026 cap (D046)

**Goal:** Introduce Form Identity — once-per-round conditional bonuses configured 100% in RON — for the 6 MVP Adults (Greymon, Garurumon, Kabuterimon, Kyubimon, DORUgamon, Angemon), reusing the follow-up listener infrastructure (D045), and remove the engine-level re-entrancy guard so chain bounding lives in the data (D046). Validate R080 end-to-end via tests/form_identity.rs.
**Demo:** scenario CLI Greymon: primo hit fire del round genera +5 Energy via Form Identity; scenario re-entrancy chain bounded da stack/cooldown nei dati, niente cap engine

## Must-Haves

- After this: scenario CLI Greymon: primo hit Fire del round genera +5 Energy via Form Identity (visible as EnergyGained event in JSONL); 6 form_identity tests green (one per Adult); chain re-entrancy bounded by data (no infinite loops on the current MVP roster) verified by tests/follow_up_chains.rs; full integration suite green; no `OneHopSuppressed` references remain in src/combat/follow_up.rs or in tests; the legitimate `event.follow_up_depth >= 1` check in ultimate.rs:77 (OnAllyFollowUp gating) is preserved.

## Proof Level

- This slice proves: Integration tests. Six Adult Form Identity behaviors verified end-to-end through the real ECS pipeline (events → listener → resolver → effect application → state mutation + observable event). Chain semantics verified by asserting depth-2+ events DO emit and terminate naturally on the current MVP roster.

## Integration Closure

Form Identity routes through the existing FollowUpIntent → resolve_follow_up_action_system pipeline (no parallel scheduler). The new listener system runs in the same Bevy schedule slot as follow_up_listener_system. RoundFlags.form_identity_used resets in advance_turn_system Part 1 alongside break_sealed (single reset hook, S07 pattern). The local ResolveActorsQuery alias in follow_up.rs and the canonical one in turn_system/mod.rs must remain structurally identical (MEM053 gotcha) — any field addition touches both.

## Verification

- New CombatEventKind::EnergyGained { unit_id, amount } emitted whenever Form Identity grants energy — observable in JSONL logs (BEVYROGUE_JSONL=1) and surfaced in combat_cli's event stream. RoundFlags.form_identity_used is queryable in tests/debug builds for once-per-round verification. The form_identity_listener_system emits info!/debug! traces under target "combat.form_identity" mirroring the follow-up listener pattern.

## Tasks

- [x] **T01: Remove D026 engine re-entrancy guard (D046) and rewrite chain semantics test** `est:S`
  Delete the `if event.follow_up_depth >= 1 { return Err(OneHopSuppressed) }` guard in `src/combat/follow_up.rs:161-163` and the `OneHopSuppressed` variant from `FollowUpSkipReason` (line ~66). This is the smallest surgical change in the slice and unblocks the test rewrite. Leave `event.follow_up_depth >= 1` in `src/combat/ultimate.rs:77` intact — that line is a DIFFERENT semantic (OnAllyFollowUp ult-charge gating: 'is this skill cast a follow-up?') and must be preserved. Update the doc comment on `CombatEvent.follow_up_depth` in `src/combat/events.rs:76-78` to reflect D046 (chains bounded by data, not engine). Remove the `OneHopSuppressed` test case from `src/combat/follow_up_tests.rs` (around lines 341-367 — search for the variant). Rewrite `tests/follow_up_reentrancy.rs` as `tests/follow_up_chains.rs` (delete old, create new) asserting the OPPOSITE of the old behavior: a follow-up emitted at depth 1 CAN trigger another follow-up at depth 2, and chains terminate naturally when no follower's preconditions re-trigger. Use the existing MVP roster — Greymon's follow-up does not re-emit OnEnemyBreak, so a chain Greymon→Greymon does not loop. Assert depth-2 events can appear and that the chain terminates after a finite number of steps. Trust MEM029: drain the event cursor between app.update() calls to avoid Messages ring-buffer pruning.

**Failure modes:** The grep target for the guard removal must be precise — mass-deleting all `follow_up_depth` references will break ultimate charging for Renamon (UltAccumulationTrigger::OnAllyFollowUp). Verify by re-running tests/follow_up_triggers.rs and tests/encounter_e2e.rs which exercise ultimate charging.

**Negative tests:** Add an assertion that a follow-up emitting OnBreak does NOT cause infinite recursion within a single update (chain terminates because the second follower's preconditions don't re-trigger).
  - Files: `src/combat/follow_up.rs`, `src/combat/events.rs`, `src/combat/follow_up_tests.rs`, `tests/follow_up_reentrancy.rs`, `tests/follow_up_chains.rs`
  - Verify: 1) `cargo check` clean. 2) `! grep -n 'OneHopSuppressed' src/ tests/` returns zero matches. 3) `grep -n 'follow_up_depth >= 1' src/combat/ultimate.rs` still returns line 77 (preserved). 4) `cargo test --test follow_up_chains` passes — asserts chain progression at depth 2+ and natural termination. 5) `cargo test` (full suite) — all binaries green; counts: 28 → 28 (rename, no net add).

- [x] **T02: Form Identity infrastructure + Greymon canonical demo (GrantEnergy + EnergyGained event)** `est:M`
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
  - Files: `src/combat/kit.rs`, `src/combat/round_flags.rs`, `src/combat/events.rs`, `src/combat/state.rs`, `src/combat/resolution.rs`, `src/combat/follow_up.rs`, `src/combat/turn_system/mod.rs`, `src/combat/mod.rs`, `src/data/units_ron.rs`, `src/data/skills_ron.rs`, `assets/data/units.ron`, `assets/data/skills.ron`, `tests/form_identity.rs`
  - Verify: 1) `cargo check` clean (after every layer: types, listener, resolution, data, test). 2) `cargo test --test form_identity greymon_first_fire_hit_grants_energy` passes. 3) `cargo test --test form_identity greymon_second_fire_hit_blocked` passes. 4) `cargo test --test form_identity greymon_resets_next_turn` passes. 5) `cargo test --lib data::units_ron::tests::round_trip_unit_def` passes (round-trip with new field). 6) `cargo test --lib data::skills_ron::tests::parse_canonical_skills_ron` passes (60 skills). 7) `cargo test` (full suite) — all binaries green including pipeline_dispatch, follow_up_triggers, encounter_e2e, follow_up_chains.

- [x] **T03: Wire Garurumon, Kabuterimon, Kyubimon (GrantEnergy reuse + new SelfAdvance Effect)** `est:M`
  Three more Adults — two reuse T02's plumbing, one needs a small new Effect variant.

**Garurumon (UnitId 13):** form_identity: `(trigger: OnFirstHitVsTagThisRound(Ice), action: SkillId("garurumon_form_identity"))`. Skill: damage_tag: Ice, sp_cost: 0, effects: [GrantEnergy(5)]. (Battery archetype — Slow-sustain support; consistent with the Greymon pattern but on the Ice axis.)

**Kabuterimon (UnitId 14):** form_identity: `(trigger: OnFirstHitVsTagThisRound(Electric), action: SkillId("kabuterimon_form_identity"))`. Skill: damage_tag: Electric, sp_cost: 0, effects: [GrantEnergy(5)].

**Kyubimon (UnitId 15) — NEW Effect::SelfAdvance(i32):**
1) `src/data/skills_ron.rs` — Add `Effect::SelfAdvance(i32)` variant. Add round-trip test. (Effect represents AV self-advance percent; reuses existing CombatEventKind::TurnAdvance retargeted to attacker.)
2) `src/combat/state.rs` — extend `ResolvedAction` with `pub self_advance_pct: i32`.
3) `src/combat/resolution.rs` — Add `fn skill_self_advance` extractor; wire into `resolve_action`. In `apply_effects`, when `resolved.self_advance_pct > 0`, emit `CombatEventKind::TurnAdvance { target: attacker.id, amount_pct: resolved.self_advance_pct }` and call into the existing AV system to advance the attacker's slot in TurnOrder. (Reuse the Renamon/Tempo pattern from S04/S06 — TurnAdvance event is already consumed downstream.) **Risk note:** the existing `Effect::TurnAdvance(i32)` targets the defender by convention; SelfAdvance must NOT collide. Distinct effect variant and distinct ResolvedAction field is the cleanest separation.
4) `assets/data/units.ron` — Kyubimon: `form_identity: Some((trigger: OnStatusApplied(Freeze(speed_reduction:0)), action: SkillId("kyubimon_form_identity")))`. **Note:** the OnStatusApplied trigger needs a *kind* match, not a value match — implement `trigger_matches` for OnStatusApplied(_) to compare only the StatusEffectKind discriminant via `std::mem::discriminant`, ignoring inner fields. Add a freeze unit literal helper if needed.
5) `assets/data/skills.ron` — `kyubimon_form_identity`: damage_tag: Ice, sp_cost: 0, effects: [SelfAdvance(20)]. Update parse_canonical_skills_ron count: 60 → 63 (+3 skills this task) and MVP id list.

**Listener extension (`src/combat/follow_up.rs`):** extend the form_identity listener's `trigger_matches` (or its form-identity equivalent) to handle OnFirstHitVsTagThisRound(DamageTag) AND OnStatusApplied(StatusEffectKind). The OnFirstHitVsTagThisRound match needs the event's damage_tag — read from `OnDamageDealt` event kind. The OnStatusApplied match reads from `OnStatusApplied` event kind (which already exists per S02). Important: only match the trigger if the unit's `form_identity_used == false` AT EVALUATION TIME (snapshot read) — the resolver flips the flag after scheduling.

**Tests in tests/form_identity.rs:**
- `garurumon_first_ice_hit_grants_energy` — same skeleton as Greymon test, Ice axis.
- `kabuterimon_first_electric_hit_grants_energy` — Electric axis.
- `kyubimon_freeze_application_self_advances` — Kyubimon casts a Freeze-applying skill (needs a freeze-applying skill in her kit OR ally's kit; simplest: use existing kyubimon_basic if it applies Freeze, otherwise inline a Freeze skill in test fixture). Assert TurnAdvance{target:Kyubimon} event emitted, RoundFlags.form_identity_used flips to true. Assert second freeze in same round does NOT re-trigger.

**Failure modes:** OnStatusApplied is also fired for status applications by enemies — guard the listener so the form_identity owner is the *applier*, not the *target*. Use the event's `source` field. **Negative tests:** verify Greymon's form_identity does not trigger on Garurumon's Ice basic (different damage_tag scope per Adult).
  - Files: `src/combat/follow_up.rs`, `src/combat/state.rs`, `src/combat/resolution.rs`, `src/data/skills_ron.rs`, `assets/data/units.ron`, `assets/data/skills.ron`, `tests/form_identity.rs`
  - Verify: 1) `cargo check` clean. 2) `cargo test --test form_identity garurumon_first_ice_hit_grants_energy` passes. 3) `cargo test --test form_identity kabuterimon_first_electric_hit_grants_energy` passes. 4) `cargo test --test form_identity kyubimon_freeze_application_self_advances` passes. 5) `cargo test --lib data::skills_ron::tests::parse_canonical_skills_ron` passes (63 skills). 6) `cargo test` (full suite) — all binaries green; greymon_first_fire_hit_grants_energy still passes (no regression).

- [x] **T04: Wire DORUgamon, Angemon (BonusToughnessDamage + BonusDamageVsAttribute) — final 2 Adults + slice closure** `est:M`
  Two Adults requiring two new Effect variants. Both are simplifications of the design intent (DORUgamon's full 'Cracked' state and Angemon's 'charging-attack' prong are M012 scope per S08 research). Document the simplifications as deviations in the task summary for product owner sign-off in S09 UAT.

**DORUgamon (UnitId 16) — NEW Effect::BonusToughnessDamage(i32):**
1) `src/data/skills_ron.rs` — Add `Effect::BonusToughnessDamage(i32)` variant + round-trip test.
2) `src/combat/state.rs` — extend `ResolvedAction` with `pub bonus_toughness_dmg: i32`.
3) `src/combat/resolution.rs` — Extractor + wire-in. In `apply_effects`, when this skill IS being applied AND the target is in toughness phase, ADD `bonus_toughness_dmg` to the `toughness_damage` value before calling `Toughness::apply_hit`. Behavior: dorugamon_form_identity skill itself has only [BonusToughnessDamage(10)] — but the trigger fires on a regular dark skill cast and the bonus must apply to *that* triggering skill, not to a separate fired skill. **This is a meaningful design twist:** the cleanest approach is for the form_identity listener to NOT fire a separate FollowUpIntent and instead emit a one-shot 'bonus pending' marker that the next ToughnessHit consumes. ALTERNATIVE simpler approach: fire `dorugamon_form_identity` as a separate skill that has ToughnessHit(10) (no Damage), targeting the same defender. This double-resolves slightly but matches T02's pattern. Pick the simpler approach for S08 — note the deviation in summary.
4) `assets/data/units.ron` — DORUgamon: `form_identity: Some((trigger: OnFirstSkillCastWithTag(Dark), action: SkillId("dorugamon_form_identity")))`. 
5) `assets/data/skills.ron` — `dorugamon_form_identity`: damage_tag: Dark, sp_cost: 0, effects: [ToughnessHit(10)]. (Per the simpler approach. If Effect::BonusToughnessDamage is added in step 1 but unused in this skill, leave it parsed but no-op for S08; add a code comment that it's wired but reserved for S09 rebalance.)

**Angemon (UnitId 17) — NEW Effect::BonusDamageVsAttribute { attribute, bonus_pct }:**
1) `src/data/skills_ron.rs` — Add the variant + round-trip test.
2) `src/combat/state.rs` — extend `ResolvedAction` with `pub bonus_dmg_vs_attribute: Option<(Attribute, i32)>`.
3) `src/combat/resolution.rs` — Extractor + wire. Same dilemma as DORUgamon: applying a *modifier* to a triggering skill is conceptually different from firing a *new* skill. For S08 use the same simpler approach: `angemon_form_identity` is a separate Damage skill that fires ONLY when the target is Virus, dealing modest bonus damage. The trigger `OnAttackVsAttribute(Virus)` evaluates against the defender's attribute on the OnDamageDealt event.
4) `assets/data/units.ron` — Angemon: `form_identity: Some((trigger: OnAttackVsAttribute(Virus), action: SkillId("angemon_form_identity")))`.
5) `assets/data/skills.ron` — `angemon_form_identity`: damage_tag: Light, sp_cost: 0, effects: [Damage(amount: 15, target: Single)]. Update parse_canonical_skills_ron count: 63 → 65 (+2 skills) and MVP id list.

**Listener extension (`src/combat/follow_up.rs`):** extend `trigger_matches` (form-identity branch) to handle OnFirstSkillCastWithTag(DamageTag) and OnAttackVsAttribute(Attribute). For OnFirstSkillCastWithTag: match against OnSkillCast event kind (existing) by reading the skill's damage_tag from the SkillBook, OR match against OnDamageDealt and check that the attacker is the form-identity owner AND it was a skill cast (not a basic). For OnAttackVsAttribute: match against OnDamageDealt and look up the defender's Attribute from the roster snapshot — extend FollowerSnapshot with `attribute: Attribute` if not already present.

**Tests in tests/form_identity.rs (final 2):**
- `dorugamon_first_dark_skill_grants_bonus_toughness` — DORUgamon casts a Dark skill at Devimon (Armored), assert two ToughnessHit-derived events: the original from the skill, plus the form_identity-fired one (or, if the modifier approach was chosen, assert toughness_damage_dealt is original+10).
- `angemon_attack_vs_virus_grants_bonus` — Angemon attacks Devimon (Virus), assert a follow-up Damage event from angemon_form_identity. Negative: Angemon attacks a non-Virus target, assert NO bonus event.

**Slice closure:**
- `cargo test` full-suite green (29 binaries: 28 prior + tests/form_identity.rs new; tests/follow_up_chains.rs replaced tests/follow_up_reentrancy.rs in T01, net 0).
- Verify `parse_canonical_skills_ron` counts the final 65 skills and MVP id list contains all 6 form_identity ids: greymon_form_identity, garurumon_form_identity, kabuterimon_form_identity, kyubimon_form_identity, dorugamon_form_identity, angemon_form_identity.
- Verify all 6 Adult units in units.ron have the form_identity field set; Childs and Devimon do NOT (parse via `#[serde(default)]`).
- `BEVYROGUE_JSONL=1 cargo run --bin combat_cli` smoke check (manual): pick a party with Greymon, drive a Fire basic, observe EnergyGained{amount:5} in the JSONL stream — confirms end-to-end observability per the slice demo string.
- The 6 form_identity tests in tests/form_identity.rs all pass.

**Failure modes:** The 'fire a separate skill' approach for DORUgamon/Angemon may double-resolve and emit duplicate OnDamageDealt events. If this breaks downstream invariants (e.g. damage_breakdown_log.rs assertions on event count), document the deviation and either (a) add a 'follow-up suppression' on the duplicate skill or (b) accept the noise and let S09 numerical rebalance settle the design. **Negative tests:** Angemon's form_identity does NOT fire when attacking a Data target; DORUgamon's form_identity does NOT fire on a Light skill cast.
  - Files: `src/combat/follow_up.rs`, `src/combat/state.rs`, `src/combat/resolution.rs`, `src/data/skills_ron.rs`, `assets/data/units.ron`, `assets/data/skills.ron`, `tests/form_identity.rs`
  - Verify: 1) `cargo check` clean. 2) `cargo test --test form_identity dorugamon_first_dark_skill_grants_bonus_toughness` passes. 3) `cargo test --test form_identity angemon_attack_vs_virus_grants_bonus` passes. 4) `cargo test --lib data::skills_ron::tests::parse_canonical_skills_ron` passes (65 skills, all 6 form_identity ids present). 5) `cargo test` (full suite, no-fail-fast) — all 29 binaries green. 6) `grep -c 'form_identity:' assets/data/units.ron` returns 6 (one per Adult). 7) Manual: `echo '' | BEVYROGUE_JSONL=1 cargo run --bin combat_cli 2>&1 | grep -c EnergyGained` is non-zero on a Greymon-Fire-basic round (sanity check — exact count may vary).

## Files Likely Touched

- src/combat/follow_up.rs
- src/combat/events.rs
- src/combat/follow_up_tests.rs
- tests/follow_up_reentrancy.rs
- tests/follow_up_chains.rs
- src/combat/kit.rs
- src/combat/round_flags.rs
- src/combat/state.rs
- src/combat/resolution.rs
- src/combat/turn_system/mod.rs
- src/combat/mod.rs
- src/data/units_ron.rs
- src/data/skills_ron.rs
- assets/data/units.ron
- assets/data/skills.ron
- tests/form_identity.rs
