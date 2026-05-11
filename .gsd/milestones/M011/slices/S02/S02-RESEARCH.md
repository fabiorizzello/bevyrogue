# M011/S02 — Research

**Date:** 2026-04-27
**Slice:** Damage Tag rename + matchup ±25% + Attribute Triangle v5.3 in-line
**Owns:** R075 (Damage Tags +25%/-25%), R076 (Triangle multipliers in-line)
**Depends on:** S01 (action pipeline lifecycle, complete)

## Summary

S02 is a schema-and-formula slice with three intertwined deliverables: (1) atomic rename `Element → DamageTag` across types/fields/RON keys, (2) replace the existing additive ±50% per-element Resistances vector with per-enemy Vec<DamageTag> weak/resist lists producing ×1.25/×0.75 multiplicative modifiers in the damage formula, (3) replace the existing additive ±0.5 attribute-triangle (`tipo_table` in `damage.rs`) with the v5.3 multiplicative quartet (`dmg_in ×0.87`, `dmg_out ×1.11`, `tough_in ×0.87`, `status_accuracy_out ×0.90`) calculated in-line as a pure function. No new state, no per-Digimon code (D020), no aliases (D044).

The work is mechanical at the rename layer (~18 source files + 16 test files + 2 RON assets touched) and surgical at the formula layer (`src/combat/damage.rs` is the single home for both axes). The key non-mechanical decisions are: where status_accuracy gets applied (single insertion site at `src/combat/turn_system/pipeline.rs:192`), what to do about `Resistances([i8;6])` whose ordinality is tied to Element (drop entirely — roster has all zeros — and replace with `resists: Vec<DamageTag>` on `UnitDef`), and how to keep `pipeline_dispatch.rs` lifecycle contract intact while adding the new status-resist failure path.

The slice is a one-shot atomic commit per D044. After it lands, scenario test Greymon (Vaccine, Fire) vs Devimon (Virus, weak Light, resist Fire) should produce log JSONL with explicit `tag_mod`, `triangle_mod` and `final_dmg` reflecting the v5.3 numbers.

## Recommendation

Land the slice as **one mechanical rename pass** followed by **two formula edits**, in this order:

1. **Rename** `Element → DamageTag` and reshape variants (`Water→Ice`, `Electro→Electric`, drop `Plant`) atomically in `src/combat/types.rs`, then sweep the compile errors. Same commit: rename `basic_element → basic_damage_tag` (UnitDef) and `element → damage_tag` (SkillDef + ResolvedAction + AttackContext + CombatEventKind::OnBreak + LogEntry::Break + ValidationLogEntry::Break). RON files updated in pari passu. The single test using `Element::Plant` (`tests/bootstrap_spawn_composition.rs:107`) is rewritten to use a non-Plant tag.
2. **Replace `Resistances` with per-enemy weak/resist lists.** Drop the `Resistances([i8;6])` struct (zero meaningful values in the roster). Replace `UnitDef.resistances` with `resists: Vec<DamageTag>`. Update `Unit` component to drop `resistances` and gain `resists: Vec<DamageTag>`. `Toughness.weaknesses: Vec<DamageTag>` already exists and stays the source for both break detection AND damage modifier (single source of truth: defender weak list lives on `Toughness`, defender resist list lives on `Unit`). The `classify` helper (`toughness.rs:49`) and `calculate_damage` both read from the same lists.
3. **Rewrite `calculate_damage`** to use multiplicative modifiers per v5.3:
   - `tag_mod = 1.25 if defender weak | 0.75 if defender resists | 1.0 otherwise`
   - `(dmg_in, dmg_out, tough_in, status_acc_out) = triangle_modifiers(attacker.attribute, defender.attribute)` where dmg_in is applied when defender wins the triangle (defender takes less), dmg_out is applied when attacker loses (attacker deals less but the v5.3 framing flips it: ×1.11 = damage SUFFERED by defender goes UP, so it's also applied to outgoing damage in `calculate_damage`. Re-read MEM022 carefully and clarify in code).
   - Final: `damage = base × tag_mod × triangle_dmg_modifier × (2.0 if break else 1.0)` rounded.
   - Drop the existing `clamp(0.25, 2.5)` floor/cap (artifact of the additive model). The multiplicative model is naturally bounded.
4. **Add `status_accuracy_out` guard** in `pipeline.rs:192`. When attacker loses triangle, roll seeded RNG against `100 × status_accuracy_out` and if it misses, emit a new `OnStatusResisted { kind }` event (do not fire `OnStatusApplied`, do not consume status). Lifecycle contract per S01 is preserved (resist is a "core event" between PreApp and Applied — same shape as OnDamageDealt).

The status-accuracy roll requires a deterministic RNG. There is no existing combat RNG resource — Shock cancel-roll at `src/combat/turn_system/mod.rs:267` uses `rand::thread_rng()` (CLAUDE.md determinism violation, pre-existing tech debt). S02 should add a `CombatRng(SmallRng)` resource seeded at bootstrap and use it for both new accuracy rolls AND retrofit the Shock roll. Alternative: defer RNG-bearing accuracy to a follow-up if the rebalance owner prefers a deterministic-only first cut (then status_accuracy is just a flat threshold pre-computed at apply time, no roll).

## Implementation Landscape

### Key Files

**Schema/types (rename + reshape):**
- `src/combat/types.rs` — `enum Element` → `enum DamageTag` (Physical, Fire, Ice, Electric, Light, Dark). Drop `Resistances([i8;6])` struct.
- `src/combat/unit.rs` — drop `Unit.resistances`, add `Unit.resists: Vec<DamageTag>`.
- `src/combat/toughness.rs` — `weaknesses: Vec<Element>` → `Vec<DamageTag>`. `classify` signature/body updated. Drop `target_resists: &Resistances` parameter — switch to `Vec<DamageTag>` resist list.
- `src/combat/state.rs` — `ResolvedAction.element` → `damage_tag`.
- `src/combat/events.rs` — `CombatEventKind::OnBreak { element }` → `{ damage_tag }`.
- `src/combat/log.rs` — `LogEntry::Break.element` → `damage_tag`.
- `src/combat/observability.rs` — `ValidationLogEntry::Break.element` → `damage_tag`. Update `format_weaknesses`. Update snapshot string format (test fixture in `tests/validation_snapshot.rs:124` will need its expected string updated).
- `src/data/skills_ron.rs` — `SkillDef.element` → `damage_tag`.
- `src/data/units_ron.rs` — `UnitDef.basic_element` → `basic_damage_tag`. Drop `resistances`. Add `resists: Vec<DamageTag>`. Update inline test fixtures.

**Formula:**
- `src/combat/damage.rs` — full rewrite of `calculate_damage` and `tipo_table` per v5.3 multiplicative model. `AttackContext.element` → `damage_tag`. Add new pure helper `triangle_modifiers(att: Attribute, def: Attribute) -> TriangleMods { dmg_modifier, tough_modifier, status_acc_modifier }`.
- `src/combat/resolution.rs` — `apply_effects` calls update (line 217, 223, 225, 240). The `Toughness::apply_hit` and `classify` now consume `resolved.damage_tag`.
- `src/combat/turn_system/pipeline.rs:192-198` — wrap status insertion in accuracy check. Emit `OnStatusResisted` on miss. Update `OnBreak` field destructure (line 154).

**RON assets (keys + values):**
- `assets/data/skills.ron` — every `element: Foo` → `damage_tag: Foo`. Value remap: `Water→Ice`, `Electro→Electric`. Lines 12, 47, 89, 116-117, 189-203, 247-268 + every other skill.
- `assets/data/units.ron` — every `basic_element:` → `basic_damage_tag:`. Value remap. `weaknesses: [Water]` → `[Ice]`. `Resistances((0,0,0,0,0,0))` line removed; new `resists: []` per unit (all empty for current roster).

**Tests (16 files + 4 in src/combat tests):**
- `tests/bootstrap_spawn_composition.rs` — only Plant usage (line 107); rewrite Hackmon to use `Physical` or `Electric`.
- `tests/combat_coherence.rs`, `tests/boundary_contract.rs`, `tests/encounter_e2e.rs`, `tests/sp_economy.rs`, `tests/status_effect_apply.rs`, `tests/status_effect_integration.rs`, `tests/revive_semantics.rs`, `tests/patamon_revive.rs`, `tests/event_stream.rs`, `tests/validation_snapshot.rs`, `tests/follow_up_triggers.rs`, `tests/follow_up_reentrancy.rs`, `tests/ultimate_meter.rs`, `tests/commander_flow.rs` — mechanical `Element::X → DamageTag::Y` and `element: → damage_tag:` rewrites.
- `src/combat/damage_tests.rs` — needs full rewrite. Existing 24-cell matrix is built around the additive ±0.5 / clamp(0.25, 2.5) model; the v5.3 multiplicative model has different expected values. New matrix: 3 tag buckets (weak/neutral/resist) × 3 triangle buckets (win/neutral/lose) × 2 break states = 18 cells, plus Free-attacker / Free-defender neutrality + `dmg_out` symmetry checks.
- `src/combat/resolution_tests.rs`, `src/combat/follow_up_tests.rs`, `src/combat/turn_system/tests.rs` — rename sweep.

**Out of touch:**
- `src/combat/follow_up.rs`, `src/combat/sp.rs`, `src/combat/ultimate.rs`, `src/combat/turn_order.rs` — no Element references.

### Build Order

1. **First commit (compile-only)**: types.rs rename + Toughness/Unit shape change. Build will be red until the sweep is done — that's expected and is the value of an atomic rename per D044. Do NOT introduce a temporary alias.
2. **Sweep src/**: walk every red-error file in dependency order (types → unit/toughness → state/events/log → resolution/observability → data/* → turn_system/pipeline → ui/combat_panel → bootstrap). Do not touch tests yet.
3. **Sweep tests/ + src/**`*_tests.rs**`. Mechanical text edits only.
4. **RON assets**: skills.ron + units.ron field renames + value remap + drop Resistances + add `resists: []`.
5. **Now formula**: rewrite `calculate_damage` per v5.3. Update `damage_tests.rs` with the new expected-value matrix. Validate with `cargo test --test damage_tests` — these are the cheapest, highest-confidence tests.
6. **Status accuracy guard + OnStatusResisted event**: add the new `CombatEventKind` variant (lifecycle contract — extend `event_stream.rs` exhaustive matcher and the three test-local matchers per S01-SUMMARY pattern). Wire the accuracy roll in `pipeline.rs:192`. Add `CombatRng` resource. Add a focused test (`tests/status_accuracy.rs`) that exercises Vaccine attacker → Data defender (attacker loses) status application with seed-controlled outcomes.
7. **JSONL logger sanity**: re-run a Greymon-vs-Devimon-style scenario and verify `tag_mod`/`triangle_mod`/`final_dmg` are observable in JSONL output. The current `CombatEvent` schema doesn't expose intermediate modifiers — consider extending `OnDamageDealt` with optional `tag_mod_pct: i32` and `triangle_mod_pct: i32` fields, OR (cheaper) emit a new `OnDamageBreakdown` event before `OnDamageDealt`. Roadmap explicitly requires "log JSONL mostra tag_mod, triangle_mod e final_dmg coerenti" — do not skip this.

### Verification Approach

- `cargo test --no-fail-fast` → all 24 binaries green (S01 baseline). Naming sweep regressions surface as compile errors first; behavioral regressions surface in `damage_tests`, `combat_coherence`, `encounter_e2e`.
- `cargo test --test damage_tests` after step 5 → new v5.3 matrix passes.
- `cargo test --test pipeline_dispatch` → S01 lifecycle contract holds (the new `OnStatusResisted` slots between PreApp and Applied like other core events).
- New test `tests/triangle_matchup.rs`: parametric verification of all 16 (attacker, defender) attribute pairs producing the expected `(dmg_modifier, tough_modifier, status_acc_modifier)` triple from `triangle_modifiers`.
- Manual: BEVYROGUE_JSONL=1 cargo run on a Greymon-vs-Devimon scenario (will be properly hooked in S04 combat_cli; for now construct via integration test fixture). Inspect JSONL for one round, verify field presence and arithmetic consistency.

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| Deterministic RNG | `rand::SeedableRng` + `SmallRng` already in dependency tree (used by `enemy_ai.rs`) | Resource-based seeded RNG is the established pattern for headless determinism (R019). |
| JSONL serialization of new event variants | `serde::Serialize` derive on `CombatEventKind` (already in events.rs:21) | Auto-derives covers OnStatusResisted with no extra work. |

## Constraints

- **Headless first** (CLAUDE.md, D015): the formula and accuracy roll must execute identically with and without the `windowed` feature. Formula is pure — no constraint risk. Accuracy RNG must seed from a Resource, not wall-clock or `thread_rng`.
- **No per-Digimon code** (D020): triangle / tag-mod / accuracy live in `damage.rs` as pure functions, never in unit-specific branches.
- **Determinism** (R019): `rand::thread_rng()` at `src/combat/turn_system/mod.rs:267` is pre-existing tech debt; if S02 introduces accuracy rolls it should establish the `CombatRng` resource pattern and the Shock roll can be retrofitted in a follow-up (or in this slice if cheap).
- **Lifecycle contract** (S01): `OnStatusResisted` must be a core event between PreApp and Applied. `OnActionResolved` must close every action that opened with `OnActionDeclared`, including those where status missed.
- **No save format to preserve** (D044 rationale): MVP single-player, no migration debt — atomic rename is correct.
- **`Toughness::weaknesses` is the SOLE current home of weak/resist matchup** (no parallel `WeaknessChart` resource exists). Keep it the SOLE home post-rename: defender weak list → `Toughness.weaknesses: Vec<DamageTag>`, defender resist list → `Unit.resists: Vec<DamageTag>` (or merge both onto Toughness if we accept the "Toughness exists only on enemies" implication that allies have no inherent resistances either — currently true).

## Common Pitfalls

- **Triangle direction confusion** — the v5.3 multipliers describe the EFFECT (defender takes less / attacker deals less), not the SIDE that wins/loses syntactically. MEM022 uses both framings ("dmg_in ×0.87 if defender wins" vs "dmg_out ×1.11 on damage suffered by attacker losing"). Pick ONE framing in code (recommend: "dmg_modifier applied to outgoing damage" — single number per attack, value 0.87 / 1.0 / 1.11 depending on triangle outcome) and document the convention in a one-line comment above `triangle_modifiers`.
- **`Resistances` index ordinality** — current `resistances.0[element as usize]` depends on `Element` discriminant order. After dropping the struct, every test that did `resistances.0[X] = 50` must be rewritten in `resists: vec![DamageTag::X]` form. Don't try to preserve the array.
- **`validation_snapshot.rs:124`** — has a hardcoded expected snapshot string (`"...weaknesses=[Water]..."`). Will fail loudly on the rename; just update the expected string. Easy fix, easy to miss.
- **`pipeline_dispatch.rs` exhaustive matchers** — three test-local helpers (S01-SUMMARY mentions follow_up_reentrancy.rs, follow_up_triggers.rs, combat_coherence.rs) use `_ => "Other"` wildcards so OnStatusResisted will compile-pass there. The strict matcher in `tests/event_stream.rs` MUST get an explicit arm.
- **JSONL field exposure for tag_mod/triangle_mod** — roadmap explicitly demands these in the log. The current `OnDamageDealt { amount, kind }` doesn't carry them. Decide upfront: extend the variant or add a sibling event. Don't ship S02 without this — S04 combat_cli depends on it for the rebalance feedback loop.
- **Plant variant** — only `tests/bootstrap_spawn_composition.rs:107` references `Element::Plant`. After rename, Plant has no DamageTag equivalent (absorbed into Physical/Electric per design); rewrite Hackmon's basic_damage_tag to `Physical` (Hackmon is a Vaccine reptile — Physical is fine for a placeholder).
- **Break event field name** — `OnBreak { element }` flows through the ENTIRE event log, the JSONL serialization, observability snapshots, AND the `LogEntry::Break` enum AND `ValidationLogEntry::Break`. Three layers, all must rename together.

## Open Risks

- **Resists model**: roster currently has zero non-default Resistances. If the rebalance owner (S09) intends to express "resist X" for any enemy, they need the new `resists: Vec<DamageTag>` field functional from S02 onward. If there is hesitation about which axis — defender Vec list vs per-skill explicit — clarify before S09 starts; don't rebuild this in S09.
- **`status_accuracy` rollout**: introducing RNG into the status-application path breaks the historical "status always applies" assumption (visible in tests like `status_effect_apply.rs`). Existing tests will need to either pin the RNG to "always hit" (seed = some value where roll ≤ threshold) or assert resist scenarios. Budget time for this.
- **`tag_mod`/`triangle_mod` log fields**: if the chosen exposure is "extend `OnDamageDealt`", that's a serialization breaking change for any downstream JSONL consumer (currently none, but pipeline_dispatch tests assert event shape).

## Skills Discovered

| Technology | Skill | Status |
|------------|-------|--------|
| Bevy 0.18 ECS | (none directly applicable) | none found |
| Rust serde / RON | (none) | none found |
| Digimon canon (Attribute Triangle, Damage Tag mapping) | bundled `digimon` skill | available (use for canon checks if Hackmon basic_damage_tag is contested) |

## Sources

- `.gsd/DECISIONS.md` D043 (Triangle v5.3 in-line), D044 (Element→DamageTag atomic rename), D046 (re-entrancy in data, not engine).
- `.gsd/REQUIREMENTS.md` R075 (Damage Tags ±25%), R076 (Triangle multipliers).
- `docs/combat_design.md` v5.3 §1 (Attribute Triangle), §2 (Damage Tags).
- `.gsd/milestones/M011/slices/S01/S01-SUMMARY.md` (lifecycle event contract, exhaustive matcher pattern, follow_up_depth flow).
- GSD memories MEM022 (Triangle v5.3), MEM023 (rename plan), MEM024 (DamageTag/Attribute on Form Identity triggers), MEM014 (toughness enemy-side only).
