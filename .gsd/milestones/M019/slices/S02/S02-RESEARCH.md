# S02 RESEARCH — Effect::Heal { amount_pct_max_hp } primitive

**Calibration:** Targeted research. The slice introduces a new `Effect` variant, a new `TargetShape` (`AllAllies`), one new event (`CombatEventKind::OnHealed`), and a multi-target ally fan-out path that does not yet exist. All seams already have analogues in the codebase (Damage fan-out, Revive, SelfOnly). No new libraries, no novel architecture.

## Summary

Add `Effect::Heal { amount_pct_max_hp: u32, target: TargetShape }` to the skill DSL. Wire it through `resolve_action` → `apply_effects` (single target) and the multi-target pipeline path in `turn_system/pipeline.rs` (for `AllAllies`). Cap to `hp_max`. Skip KO targets (no event on KO). Emit a new `CombatEventKind::OnHealed { amount, hp_after }`. Integration test in `tests/heal_effect.rs` exercising Single/AllAllies/KO-skip/cap.

## Recommendation

1. **Effect variant** — `Effect::Heal { amount_pct_max_hp: u32, target: TargetShape }` in `src/data/skills_ron.rs:204`. Mirror the `target` field shape used by `Effect::Damage` for consistency, even though Heal target shapes are restricted to `Single | SelfOnly | AllAllies`. Validation in `validate_skill_def` should reject enemy-side shapes (`Bounce`, `AllEnemies`, `Blast` on enemy side).

2. **New target shape** — Add `TargetShape::AllAllies` to `src/data/skills_ron.rs:30`. Extend `resolve_targets` in `src/combat/resolution.rs:84` with an `AllAllies` arm: filter `team == primary.team` AND `alive`, sort by `slot_index`. Add to the executable-now whitelist in `target_shape_is_executable_now` (resolution.rs:402). Note: `SelfOnly` already exists for self-heal.

3. **ResolvedAction wiring** — Add fields `heal_pct: u32` and a `heal_target_shape: Option<TargetShape>` (or reuse `target_shape` since Heal is exclusive per skill in scope) to `ResolvedAction` (`src/combat/state.rs`). Add an extractor `skill_heal_pct(&[Effect])` next to `skill_revive_pct` (resolution.rs:334).

4. **Single-target heal in `apply_effects`** — In `src/combat/resolution.rs:533`, insert a heal branch alongside the existing `revive_pct > 0` branch. KO policy: **no-op + no event** when defender is KO'd on `Single`/`SelfOnly`. **Note:** this diverges from the milestone-CONTEXT line "Single KO → CombatEvent::IllegalTarget". The slice roadmap line is authoritative: *"no-op su KO"*. Planner must record this in DECISIONS.md and skip the IllegalTarget emission. Confirmed today: `OnActionFailed { reason: "Target is KO" }` (resolution.rs:574) already handles damage on a KO target; the new heal path should branch *before* that guard so it can quietly no-op without consuming SP. Decision needed: should Heal on KO single target still consume SP? Recommend: **no** (consistent with damage-on-KO which also short-circuits at the validation gate).

5. **Multi-target heal (AllAllies)** — Mirror the `Blast | AllEnemies` block in `src/combat/turn_system/pipeline.rs:175`. Two viable options:
   - **(A)** Extend the existing block to `Blast | AllEnemies | AllAllies` and dispatch on heal-vs-damage at the per-target call site (introduce `apply_heal_only` alongside `apply_damage_only`).
   - **(B)** New parallel block exclusively for ally-fan-out healing.
   Prefer **(A)** because resource consumption (SP/Ult/streak) hoisting is identical and already centralised; the per-target dispatch is the only seam that diverges.

6. **`apply_heal_only` helper** — In `src/combat/resolution.rs` next to `apply_damage_only` (line 420). Signature mirror: takes `(&ResolvedAction, &mut Unit defender, hp_pct: u32)` plus the ally-team flag. KO check first → if KO, return `(default outcome, vec![])` (no event). Otherwise: `let healed = ((unit.hp_max as i64 * pct as i64) / 100).min((unit.hp_max - unit.hp_current) as i64) as i32;` then `unit.hp_current += healed; emit OnHealed { amount: healed, hp_after: unit.hp_current }`.

7. **Event** — Add `CombatEventKind::OnHealed { amount: i32, hp_after: i32 }` to `src/combat/events.rs:26`. Mirror `OnRevive` shape (which already emits `hp_after`). No follow-up listener changes expected for this slice — Heal is not a trigger today.

8. **JSONL trace** — `jsonl_logger.rs` and `observability.rs` should already serialise `CombatEventKind` via the existing `serde::Serialize` derive — no extra wiring required. Confirm by re-reading the logger after the variant lands.

9. **RON test fixture** — Add 1–2 skills to `assets/data/skills.ron` (e.g. `holy_aegis_heal` single, `holy_rest` AllAllies) or use a fixture-only RON loaded by the test. Prefer the fixture-only path to avoid balance churn.

10. **Integration test** — `tests/heal_effect.rs`, naming functional per CLAUDE.md convention. Match the lightweight pattern used in `tests/dr_pipeline.rs` (S01): call `apply_effects` / `apply_heal_only` directly with handcrafted `Unit` instances, no Bevy world. Cases:
    - Heal Single on damaged ally: amount = floor(`hp_max * pct / 100`), capped to `hp_max - hp_current`, `OnHealed { amount, hp_after }` emitted.
    - Heal Single at full HP: amount = 0, event still emitted with `amount: 0` (or suppressed — decide; recommend emit, mirrors `OnDamageDealt { amount: 0 }`).
    - Heal Single on KO target: no state change, **no event**, no SP consumed.
    - Heal AllAllies with 1 KO + 2 alive damaged: KO untouched and no event; both alive receive heal, event order by `slot_index` ascending (matches `resolve_targets` for Blast/AllEnemies).
    - Cap test: ally at hp_max-3 with 50%-of-max heal → healed exactly 3, hp_after == hp_max.
    - Identity test (optional): combat without any heal skill produces unchanged JSONL trace.

## Implementation Landscape

- `src/data/skills_ron.rs:30` — `TargetShape` enum; add `AllAllies`.
- `src/data/skills_ron.rs:204` — `Effect` enum; add `Heal { amount_pct_max_hp: u32, target: TargetShape }`.
- `src/data/skills_ron.rs:303` — `validate_skill_def`; reject illegal Heal target shapes (Enemy-side / Bounce).
- `src/combat/events.rs:26` — `CombatEventKind`; add `OnHealed`.
- `src/combat/resolution.rs:84` — `resolve_targets`; add `AllAllies` arm (filter same team + alive, sort slot asc).
- `src/combat/resolution.rs:233` — `resolve_action`; extract `heal_pct` into `ResolvedAction`.
- `src/combat/resolution.rs:334` — add `skill_heal_pct(&[Effect])` next to `skill_revive_pct`.
- `src/combat/resolution.rs:402` — `target_shape_is_executable_now`; include `AllAllies`.
- `src/combat/resolution.rs:420` — add `apply_heal_only` mirroring `apply_damage_only`.
- `src/combat/resolution.rs:533` — `apply_effects`; insert heal branch (after KO guards, before damage branch, but *before* the KO guard on Single so it can no-op silently).
- `src/combat/state.rs` — `ResolvedAction`; add `heal_pct: u32` field.
- `src/combat/turn_system/pipeline.rs:175` — multi-target block; extend to include `AllAllies` with heal dispatch.
- `src/combat/follow_up.rs` — local `ResolveActorsQuery` mirror per S01 lesson: keep the tuple shape in sync if any new component is added. **For this slice no new component is required**, but the `Effect` enum match in follow_up.rs:660 must be updated if its match is exhaustive (verify; today it only inspects `Damage`/`ToughnessHit`).
- `assets/data/skills.ron` — fixture skill(s) for the test, or fixture-only RON file under `tests/fixtures/`.
- `tests/heal_effect.rs` — new integration test, functional naming per CLAUDE.md.

## First Proof (Highest-Risk / Biggest Unblocker)

Start with `apply_heal_only` + the `Effect::Heal` variant + `OnHealed` event, then write the **AllAllies-with-KO** test case directly against `apply_heal_only` (no pipeline) — this proves the KO-skip + cap policy in isolation and is the most behaviour-defining part. Pipeline wiring (`AllAllies` fan-out) is mechanical mirroring of the existing `Blast | AllEnemies` block once the per-target helper is correct.

## Constraints and Risks

- **KO policy divergence:** Milestone CONTEXT says "Single KO → IllegalTarget". S02 ROADMAP says "no-op su KO". The slice goal wins; the planner must save a `gsd_decision_save` entry recording this resolution so M021 design downstream doesn't get blamed.
- **Cap semantics:** `amount_pct_max_hp` is `% of hp_max` — cap is `hp_current + healed <= hp_max`, not "cap amount itself to hp_max". Integer arithmetic with i64 widening to avoid `i32 * 100` overflow when `hp_max` is large (today caps are small, but no reason to be sloppy).
- **AllAllies includes self?** Yes — by convention in HSR-like designs and consistent with `AllEnemies` including all enemies regardless of slot. Skip the attacker only if KO (the KO filter naturally handles it). Verify against game design intent during planning.
- **DR identity-neutrality (S01):** Heal must not regress the DR-neutrality JSONL golden. Since Heal is opt-in via skill RON, no Heal skill in baseline fixtures → no trace change. But adding fixture skills to `skills.ron` will change loader behaviour. Prefer the **fixture-only RON** path for tests to keep `assets/data/skills.ron` free of test-only entries.
- **`apply_effects` ordering:** Existing function has tight invariants (validation → SP consumption → effect application → ult/streak bookkeeping). Inserting heal between validation and damage is the safe spot. The Single-KO no-op for Heal must run *before* the generic `if defender_unit.is_ko()` early-return so it can bail without `OnActionFailed`. Add a dedicated guard: `if resolved.heal_pct > 0 && defender_unit.is_ko() { return early with sp_ok=true, no events }` — meaning SP is **not** consumed when heal target is KO. Confirm with planner.
- **Follow-up coupling (per S01 forward intelligence):** S01 SUMMARY warns "When adding new components to the main ResolveActorsQuery in resolution.rs, remember to update follow_up.rs's local query in the same change". This slice does **not** add components, only fields to `ResolvedAction` and new enum variants — should be safe. Still: planner T01 should grep `ResolveActorsQuery` to confirm no drift.

## Verification

- `cargo check` — confirms enum + match exhaustiveness across all sites.
- `cargo test --test heal_effect` — 5 cases listed above.
- `cargo test` — full suite green; confirms no regression on `dr_pipeline.rs` and existing skill RON loader/validator tests.
- Manual: `cargo run --bin combat_cli` with a fixture encounter using a Heal skill, inspect JSONL for `OnHealed` entries.

## Skills Discovered

None installed during this research. Local Rust+Bevy patterns are already documented in CLAUDE.md and `docs/combat_current.md`. No new technology surface for this slice.

## Don't Hand-Roll

- Use the existing `resolve_targets` + `TargetableSnapshot` + `TargetEntry` plumbing; do not introduce a parallel ally-targeting snapshot type.
- Use the existing `apply_damage_only` resource-hoisting pattern in `pipeline.rs` for the AllAllies fan-out; do not invent a second SP-consumption path.
- Use `serde::Serialize` on the new event variant (derived on `CombatEventKind` already) — JSONL serialisation is automatic.

## Sources

- `src/data/skills_ron.rs:30,204,303` — TargetShape enum, Effect enum, validator.
- `src/combat/resolution.rs:84,233,402,420,533` — target resolver, action resolver, executable filter, single-target damage helper, full apply_effects path.
- `src/combat/turn_system/pipeline.rs:175–390` — multi-target damage fan-out reference.
- `src/combat/events.rs:1–146` — CombatEventKind variants (OnRevive at line 42 is the closest analogue for OnHealed).
- `src/combat/unit.rs:35–44` — Unit::is_ko, Unit::revive (revive_pct math reference).
- `.gsd/milestones/M019/slices/S01/S01-SUMMARY.md` — DR pipeline lessons (follow_up.rs query drift, apply_effects direct-call test pattern).
- `tests/dr_pipeline.rs` — integration test pattern to mirror.
- `.gsd/milestones/M019/M019-CONTEXT.md` — milestone-level Heal policy (Single-KO IllegalTarget) — **superseded by S02 ROADMAP no-op policy**, flagged above.
