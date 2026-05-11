---
estimated_steps: 15
estimated_files: 7
skills_used: []
---

# T03: Wire Garurumon, Kabuterimon, Kyubimon (GrantEnergy reuse + new SelfAdvance Effect)

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

## Inputs

- `src/combat/follow_up.rs`
- `src/combat/state.rs`
- `src/combat/resolution.rs`
- `src/data/skills_ron.rs`
- `src/data/units_ron.rs`
- `assets/data/units.ron`
- `assets/data/skills.ron`
- `tests/form_identity.rs`

## Expected Output

- `src/combat/follow_up.rs`
- `src/combat/state.rs`
- `src/combat/resolution.rs`
- `src/data/skills_ron.rs`
- `assets/data/units.ron`
- `assets/data/skills.ron`
- `tests/form_identity.rs`

## Verification

1) `cargo check` clean. 2) `cargo test --test form_identity garurumon_first_ice_hit_grants_energy` passes. 3) `cargo test --test form_identity kabuterimon_first_electric_hit_grants_energy` passes. 4) `cargo test --test form_identity kyubimon_freeze_application_self_advances` passes. 5) `cargo test --lib data::skills_ron::tests::parse_canonical_skills_ron` passes (63 skills). 6) `cargo test` (full suite) — all binaries green; greymon_first_fire_hit_grants_energy still passes (no regression).

## Observability Impact

Three new EnergyGained events fire-able (Greymon/Garurumon/Kabuterimon), one new SelfAdvance-targeted TurnAdvance event (Kyubimon). FollowUpTrace.origin_kind = FormIdentity for all four, distinguishing Form Identity scheduling from regular follow-ups in JSONL output.
