# M019/S01 — Research: BuffKind::DR primitive + damage formula integration

**Date:** 2026-05-14

## Summary

S01 adds a generic **damage-reduction (DR)** primitive to the combat kernel and wires it into `calculate_damage` as a multiplicative mitigation step. M019 success criteria simplify the design relative to the M017 spike sketch (`.gsd/spikes/spike-kernel-primitives/sketches/damage_reduction.rs`) and decision D005: **no clamp, no intra-unit max-replace, no per-source tracking** — DR instances on a target are summed freely, and the resulting damage is clamped at 0. All franchise / per-skill rules (Gabumon `fur_cloak`, Patamon `holy_aegis` aura, intra-unit collapse semantics) are explicitly deferred to M021 (`trait Skill` + `SkillCtx`); the kernel only owns the primitive.

The current damage formula in `src/combat/damage.rs:86-116` is `base × tag_mod × tri.dmg_modifier × break_mod × status_amp_mod × attacker_dmg_mult`, rounded. There is no defender-side mitigation layer beyond `tag_mod` (resist=0.75). The slice inserts a new `(1.0 − dr_sum).max(0.0)` factor and clamps `final_damage` at 0.

Storage: introduce a new lightweight per-unit component `DrBag` (Vec of `{value: f32, duration: u32}`), kept out of `StatusBag` so it does not interact with status taxonomy, cleanse, or accuracy rolls. `BuffKind` (existing in `status_effect.rs:32`) stays as a binary classifier for cleanse; the roadmap label "BuffKind::DR" is satisfied by the dedicated DR component rather than by extending the enum — this keeps `BuffKind` semantics intact (Buff vs Debuff for cleanse) and avoids polluting `StatusBag` with value-carrying entries. A planner-level alternative (extend `BuffKind` enum to `{Buff, Debuff, DR}` and rework `StatusInstance`) is captured below.

## Recommendation

**Add a new `DrBag` component + sum helper, integrated into `calculate_damage` as an additional multiplicative step. No clamp on `dr_sum`; clamp `final_damage.max(0)`.**

Concretely:

1. Define `DrInstance { value: f32, duration: u32 }` and `DrBag(Vec<DrInstance>)` in a new module `src/combat/buffs.rs` (mirrors `status_effect.rs` layout). Insert on units at bootstrap as `DrBag::default()`.
2. Pure helper `sum_dr(bag: Option<&DrBag>) -> f32` returning the unclamped sum.
3. Extend `calculate_damage` signature with `defender_dr: Option<&DrBag>`. Formula becomes:
   ```
   dr_sum   = sum_dr(defender_dr)              // unclamped, can exceed 1.0
   dr_mod   = (1.0 - dr_sum).max(0.0)          // never negative
   raw      = base × tag_mod × tri × break × amp × atk_mult × dr_mod
   final    = round(raw).max(0)                // clamp at 0
   ```
4. Add `dr_pct: i32` to `DamageBreakdown` (integer percentage, for log visibility). `OnDamageDealt` event payload stays unchanged for now (existing tests do not assert breakdown).
5. Update both call sites in `src/combat/resolution.rs` (lines ~459-485 and ~619-643) to fetch the defender's `DrBag` from the world and pass it.
6. Decrement durations: extend the per-unit turn-tick path that already calls `StatusBag::tick_all` so it also ticks `DrBag` (drop instances at duration 0).
7. Skill DSL: no new `Effect` variant in S01 — M019 success criteria for S01 are about the **primitive + formula integration**, not about applying DR via skills. A future slice (or M021) can add `Effect::ApplyDR { value, duration }`; integration tests in S01 manipulate `DrBag` directly via `world.insert_one` to keep scope honest.

This matches P001 (kernel-generic, no Digimon names) and the spike research (RESEARCH.md §149-151), while honouring the M019 simplifications stated in the roadmap (no cap, summed freely).

## Implementation Landscape

### Key Files

- `src/combat/damage.rs` — `calculate_damage` (lines 86-116) is the single source of truth for the damage formula. Add `defender_dr` parameter and the `dr_mod` step. Extend `DamageBreakdown` with `dr_pct`. The existing 18-test multiplicative matrix (lines 180-388) is the regression harness; new tests should mirror that style.
- `src/combat/resolution.rs` — two call sites of `calculate_damage`:
  - line ~478: `apply_damage_only`-style path (revive-less follow-up branch).
  - line ~636: `apply_effects` main path.
  Both build `AttackContext`, compute `attacker_dmg_mult` from `attacker_statuses`, then call `calculate_damage`. Both must be updated to also pull defender's `DrBag` from the world (same query pattern as `defender_status`/`StatusBag`).
- `src/combat/buffs.rs` — **new file**. Owns `DrInstance`, `DrBag`, `sum_dr`. Pure module (no Bevy systems beyond `Component` derive).
- `src/combat/mod.rs` — `pub mod buffs;` re-export.
- `src/combat/bootstrap.rs` — spawn `DrBag::default()` on every unit that today gets `StatusBag::default()`. Grep for `StatusBag::default()` insertion to find the seam.
- `src/combat/status_effect.rs` — **no changes** to `BuffKind` enum; `cleanse_debuffs` must not see DR instances (they live in a separate component, so naturally untouched).
- Per-turn tick path — find the system that calls `StatusBag::tick_all` (turn_system module) and add a sibling tick for `DrBag`. A `DrBag::tick_all() -> Vec<()>` (or just `tick`) that decrements durations and drops zero-entries keeps symmetry.
- `tests/dr_pipeline.rs` — **new** integration test. Mirror existing test style in `tests/status_blessed_offensive.rs`.

### Build Order

1. **First proof — pure formula change.** Add `DrBag`/`sum_dr` and rewire `calculate_damage` signature + tests in `src/combat/damage.rs` (unit tests). This is the highest-risk piece because the existing 18-test matrix must keep passing with `defender_dr=None`. Once the unit tests are green, the kernel formula is locked.
2. **Wire the two `resolution.rs` call sites.** Pass `Option<&DrBag>` from the world; default `None` is structurally equivalent to today's behaviour.
3. **Bootstrap insertion** of `DrBag::default()` on units.
4. **Duration tick** alongside `StatusBag::tick_all`.
5. **Integration tests** (`tests/dr_pipeline.rs`) covering the four cases the roadmap lists: DR singolo, DR×N sommato, DR+resist combinato, DR durante Break, plus an over-100% clamp case (sum 1.5 → damage 0, OnDamageDealt emitted with amount=0).

Steps 1-2 are the only ones that can break existing tests; 3-5 are additive.

### Verification Approach

```bash
cargo test --test dr_pipeline                # new integration tests
cargo test calculate_damage                  # existing 18-row matrix must stay green
cargo test --test status_blessed_offensive   # cross-check the closest existing integration
cargo check                                  # headless default
```

Observable behaviours to assert in `tests/dr_pipeline.rs`:

- **Singolo:** base=100, DR=0.30 → final=70; `OnDamageDealt { amount: 70, .. }`.
- **Sommato:** two `DrInstance{value:0.20}` on the same target → final=60.
- **Combinato con resist:** base=100, tag_mod=0.75 (resist), DR=0.20 → 100×0.75×0.80 = 60.
- **Durante Break:** base=100, break_mod=2.0, DR=0.30 → 100×2.0×0.70 = 140.
- **Clamp a 0:** DR sum = 1.5 → dr_mod=0.0 → final=0, no panic, `OnDamageDealt` emitted with `amount: 0`, defender `hp_current` unchanged.

Determinism: `DrBag` ordering must not affect `sum_dr` (sum is commutative on f32; document the precision tolerance — tests use integer post-round values so float noise is absorbed by `.round()`).

## Don't Hand-Roll

| Problem | Existing Solution | Why Use It |
|---------|------------------|------------|
| Per-unit value/duration store with tick semantics | `StatusBag` shape in `src/combat/status_effect.rs:52-107` | Same shape (Vec + tick_all + apply); copy the layout for `DrBag` to keep the mental model uniform. Do NOT actually reuse the type — keeping them separate preserves cleanse semantics and avoids enum bloat. |
| Multiplicative formula matrix tests | `src/combat/damage.rs:180-388` (18-row matrix) | Mirror this style for DR; one row per combination of (DR present/absent × tag_mod bucket × break) keeps coverage explicit. |
| Test harness with deterministic units | `tests/status_blessed_offensive.rs` | Reuse the same bootstrap pattern (build App headless, insert units, send `ActionIntent`). |

## Constraints

- **P001 kernel-generic:** no Digimon names, no `if skill_id ==` in damage path. DR is plumbed as a generic component; who/why grants DR is blueprint/M021 territory.
- **No new RON Effect variant in S01.** Roadmap success criteria for S01 are formula-side only. Adding `Effect::ApplyDR` belongs in a follow-up (and ideally waits for `trait Skill`/`SkillCtx` in M021 so per-skill scaling is expressible).
- **Headless first:** no winit/wgpu/egui in any of the new code; `DrBag` is a plain `Component` like `StatusBag`.
- **Determinism:** `sum_dr` order-independence holds because addition on f32 is commutative; the post-round `i32` damage is fully deterministic across iteration orders.
- **Existing 18-test damage matrix is the regression gate.** Changing `calculate_damage`'s signature breaks every call site — both `resolution.rs` sites and the unit tests need an explicit `None` for `defender_dr`.

## Common Pitfalls

- **Negative damage from over-100% DR.** `(1.0 - 1.2)` is `-0.2`, which would *increase* damage by sign flip after `.round()`. Clamp `dr_mod` at 0 (`.max(0.0)`) BEFORE multiplying, not afterwards.
- **Forgetting to clamp `final_damage.max(0)`.** Even with `dr_mod >= 0`, rounding of a near-zero positive raw can land at 0 fine; the real risk is future negative modifiers. Add the clamp defensively — the roadmap explicitly requires "damage clampato a 0 senza panic".
- **Mutating `StatusBag` for DR.** Tempting because `BuffKind` already exists there, but it kills cleanse semantics (DR would either be wiped by every `cleanse_debuffs` call, or you'd need to introduce a third "neither buff nor debuff" classification). Keep DR in its own component.
- **Tick coupling.** If `DrBag::tick` is not called alongside `StatusBag::tick_all`, DR becomes permanent. Find the call site (turn_system per-turn end-of-turn pass) and add the sibling call in the same PR.
- **Bootstrap omission.** Units without `DrBag` will be passed `None` and behave identically — so an omitted bootstrap insert won't fail any test, just makes future `apply_dr` calls awkward (would require `.entry().or_insert_with`). Add the default insertion proactively.
- **`OnDamageDealt { amount: 0 }` already happens.** The branch `if resolved.base_damage > 0 || resolved.toughness_damage > 0` (resolution.rs:454, :618) gates entry; once entered, the event is always emitted. Tests can rely on this — but assert it explicitly.

## Open Risks

- **`BuffKind` naming dissonance.** The roadmap title says "BuffKind::DR primitive" but the cleanest implementation keeps DR out of `BuffKind`. Flag in the slice summary so future readers don't expect `BuffKind::DR` to literally exist as an enum variant. Alternative (deeper refactor) — extending `BuffKind` to `{Buff, Debuff, DR}` and changing `StatusInstance` to carry an optional `value: f32` — is rejected for S01 scope but worth recording in DECISIONS.md.
- **No skill-side trigger in S01.** Tests insert `DrBag` directly. If the planner wants a smoke test where a real skill applies DR, the simplest mid-ground is a test-only helper, not a new `Effect` variant. Keep this in the test file, not in `skills_ron.rs`.
- **Toughness/Break interaction.** Roadmap says "DR durante Break — damage clampato a 0 senza panic". Today `break_mod = 2.0` multiplies before DR; current ordering (`base × tag × tri × break × amp × atk × dr`) is what the existing matrix expects to remain stable. Confirm by re-running the 18 tests after the change.
- **Future `trait Skill` migration.** M021 will likely move resource/effect application to `Intent`s; adding DR via `Intent::ApplyDR { value, duration }` is the natural next step. Keep `DrBag::apply(value, duration)` as a public method so M021 only needs to wire the `Intent` handler.

## Sources

- `src/combat/damage.rs:86-116` — current `calculate_damage` and `DamageBreakdown` shape.
- `src/combat/status_effect.rs:32-107` — `BuffKind`, `StatusBag` layout to mirror.
- `src/combat/resolution.rs:459-485, 619-643` — the two call sites to update.
- `.gsd/spikes/spike-kernel-primitives/sketches/damage_reduction.rs` — prior design sketch (clamped/source-tracked); M019 explicitly simplifies away the clamp and source tracking.
- `.gsd/spikes/spike-kernel-primitives/RESEARCH.md` §149-151 — D-M017-DR-MODULE proposed module layout.
- `.gsd/DECISIONS.md` D005 — original DR taxonomy; superseded for S01 scope by M019 roadmap simplification (record this in DECISIONS when slice completes).
- `.gsd/KNOWLEDGE.md` P001 — kernel-generic constraint that forbids per-Digimon DR logic in the kernel.
