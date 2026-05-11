---
estimated_steps: 16
estimated_files: 6
skills_used: []
---

# T03: Numerical rebalance pass — turn T02 fixtures green and finalize combat_design sez. 9

Iterate on `assets/data/units.ron` and `assets/data/skills.ron` (HP, toughness, base damage, SP costs, ultimate triggers, status durations) until all three T02 scenario fixtures pass within the R083 bands. Decide upfront whether to wire `BonusToughnessDamage` and `BonusDamageVsAttribute` into `apply_effects`, or remove the unused variants — document the call in S09-ASSESSMENT.md and (if removing) record a superseding decision in `.gsd/DECISIONS.md`.

**Recommended decision (default plan):** keep the S08 `fire-a-separate-skill` workaround for DORUgamon and Angemon. Strip `BonusToughnessDamage(i32)` and `BonusDamageVsAttribute { attribute, bonus_pct }` variants from `src/data/skills_ron.rs::Effect` and the matching round-trip test. Rationale: minimum-change path, S08 already proves the workaround feels correct, and dead variants are MVP debt. If during rebalance the workaround visibly under-delivers (e.g. DORUgamon ramp is too shallow), switch to wiring — this requires extending `ResolvedAction` and `apply_effects` in `src/combat/resolution.rs`, plus tests covering both effect paths. **Document the chosen path early in T03 execution** so the rest of the task scope is bounded.

**Rebalance levers (apply in order, lowest-blast-radius first):**
1. Skill base damage / SP cost in `skills.ron` (preferred — local, testable, no cross-unit ripple).
2. Unit HP / toughness / weakness lists in `units.ron` (per-tier; minion HP for minion-tier TTK, mini-boss for mini-boss-tier, etc.).
3. Ultimate trigger / cap / charge per event for ult-pacing.
4. Form Identity grant amounts (e.g. GrantEnergy(5) → 7 if boss tier needs more energy ramp).

**Iteration loop:** rerun the three scenario tests, read the actual turn count from the assertion message, adjust the smallest lever, retest. Avoid mass tuning across multiple files in a single edit — single-lever iterations make the cause/effect legible. Each commit-worthy intermediate state should leave `cargo test` green for non-scenario tests; only the three scenario tests are allowed to be red mid-rebalance.

**Side hazards to verify don't regress:**
- `cargo test --test triangle_matchup` — triangle multipliers (D043 ratios).
- `cargo test --test ultimate_meter` — ult charge accumulation.
- `cargo test --test form_identity` — all 6 Adults still trigger correctly.
- `cargo test --test toughness_categories` — break category requirements.
- `cargo test --test sp_economy` — SP cap and child discount.

**Final step — combat_design.md sez. 9 finalization:** Update `docs/combat_design.md` sez. 9 (Form Identity) to annotate that the framework is wired in S08 (cite the 6 wired Adults: Greymon Fire/GrantEnergy, Garurumon Ice/GrantEnergy, Kabuterimon Electric/GrantEnergy, Kyubimon Freeze-status/SelfAdvance, DORUgamon Dark-skill/separate-toughness-skill, Angemon Virus-attack/separate-light-skill). If `BonusToughnessDamage` / `BonusDamageVsAttribute` were stripped, add a short note explaining the `fire-a-separate-skill` choice and pointing at the relevant decision id.

**Definition of done:** all three scenario tests green; full `cargo test` green (29+ binaries); `docs/combat_design.md` sez. 9 has a 'wired in M011' annotation; if any decision was made (wire vs strip), `.gsd/DECISIONS.md` has a new D04X or D050+ entry recording it.

## Inputs

- `assets/data/units.ron`
- `assets/data/skills.ron`
- `src/data/skills_ron.rs`
- `src/combat/resolution.rs`
- `docs/combat_design.md`
- `tests/scenario_minion_ttk.rs`
- `tests/scenario_miniboss_ttk.rs`
- `tests/scenario_boss_ttk.rs`

## Expected Output

- `assets/data/units.ron`
- `assets/data/skills.ron`
- `docs/combat_design.md`

## Verification

cargo test --test scenario_minion_ttk && cargo test --test scenario_miniboss_ttk && cargo test --test scenario_boss_ttk && cargo test && grep -q -i 'wired in M011\|wired in S08\|S08 implementation' docs/combat_design.md && echo 'OK'

## Observability Impact

- Signals added/changed: none new; rebalance is data-only unless Bonus* variants are wired (in which case ResolvedAction extension and a new branch in apply_effects).
- How a future agent inspects this: scenario test failures point at numerical mismatch vs. R083 band; jsonl dump under BEVYROGUE_JSONL=1 shows per-action damage breakdown with damage_tag for triangle attribution.
- Failure state exposed: any non-scenario test regression during rebalance is caught by the full `cargo test` gate before sign-off.
