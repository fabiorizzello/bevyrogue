---
estimated_steps: 24
estimated_files: 7
skills_used: []
---

# T04: Wire DORUgamon, Angemon (BonusToughnessDamage + BonusDamageVsAttribute) — final 2 Adults + slice closure

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

1) `cargo check` clean. 2) `cargo test --test form_identity dorugamon_first_dark_skill_grants_bonus_toughness` passes. 3) `cargo test --test form_identity angemon_attack_vs_virus_grants_bonus` passes. 4) `cargo test --lib data::skills_ron::tests::parse_canonical_skills_ron` passes (65 skills, all 6 form_identity ids present). 5) `cargo test` (full suite, no-fail-fast) — all 29 binaries green. 6) `grep -c 'form_identity:' assets/data/units.ron` returns 6 (one per Adult). 7) Manual: `echo '' | BEVYROGUE_JSONL=1 cargo run --bin combat_cli 2>&1 | grep -c EnergyGained` is non-zero on a Greymon-Fire-basic round (sanity check — exact count may vary).

## Observability Impact

Two more form_identity-tagged events surface in JSONL: DORUgamon emits a second ToughnessHit-derived OnBreak/toughness event, Angemon emits an extra OnDamageDealt scoped by attribute match. FollowUpTrace.origin_kind=FormIdentity remains the disambiguator. Document the 'separate skill' simplification in T04-SUMMARY for S09 UAT review.
