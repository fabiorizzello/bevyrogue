# S04: Selectors estesi: AdjLowest, LowestHpPctAlive, RandomEnemyAlive{seed}, SingleAlly — Research

**Date:** 2026-05-13
**Depth:** Targeted (mostly mechanical extension of S02's pure-resolver + snapshot pattern; one real question — RON seed plumbing for the random selector)

## Summary

S04 productizes four target selectors as composable building blocks that skills can declare in `skills.ron`. Three of the four (`AdjLowest`, `LowestHpPctAlive`, `SingleAlly`) are deterministic and require no RNG plumbing; the fourth (`RandomEnemyAlive{seed}`) needs a per-skill seed surfaced through the RON schema and combined with the existing `CombatRng` resource (rng.rs:9 — seeded `StdRng` with `Default::from_seed(42)`).

The S02 foundation makes this slice mostly mechanical. `TargetableSnapshot` + `TargetEntry` (resolution.rs:56-70) already carry the four fields needed for every selector (`id`, `team`, `slot_index`, `alive`); adding `hp_current` and `hp_max` per entry — or a pre-computed `hp_pct_milli` — covers `LowestHpPctAlive`. Pure fns alongside `resolve_targets()` (one per selector, each returning `Option<UnitId>`) keep testability identical to S02: snapshot-in, id-out, no ECS, table-driven tests in the `#[cfg(test)] mod tests` block of `resolution.rs`.

The harder design question is **how skills declare which selector they use**. Today `SkillTargeting` (skills_ron.rs:46-67) carries shape/side/life/self-rule/hp-rule. Selectors are orthogonal to shape (e.g. a Single skill with `LowestHpPctAlive` is "auto-aim weakest enemy"; a Blast skill with `AdjLowest` picks the primary as the lowest-HP-pct enemy then spills). Recommendation: add an `Option<TargetSelector>` field to `SkillTargeting`, defaulting to None (= caller-supplied target, current behavior). When set, the pipeline's *primary* target is overridden by the selector before resolve_targets() is called. Selector composability with shape stays clean: Bounce(N) S03 hop-0 selector is still "caller-supplied target", hops ≥1 use the existing LowestHpPctAlive (which S04 lifts to a standalone selector).

## Recommendation

**Approach: extend the schema, add a pure `select_primary()` fn, override in the pipeline before `resolve_targets()` is called.**

Schema (`src/data/skills_ron.rs`):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TargetSelector {
    AdjLowest,             // among adj-to-current-active alive enemies, pick lowest HP%
    LowestHpPctAlive,      // among alive enemies, pick lowest HP%
    RandomEnemyAlive { seed: u64 },
    SingleAlly,            // legality only — pairs with TargetSide::Ally
}

pub struct SkillTargeting {
    pub shape: TargetShape,
    pub side: TargetSide,
    pub life: TargetLife,
    pub self_rule: SelfTargetRule,
    #[serde(default)]
    pub target_hp_rule: TargetHpRule,
    #[serde(default)]
    pub selector: Option<TargetSelector>,  // NEW
}
```

Resolver (`src/combat/resolution.rs`):

```rust
pub fn select_primary(
    selector: &TargetSelector,
    actor_id: UnitId,
    snapshot: &TargetableSnapshot,
    rng: &mut CombatRng,
) -> Option<UnitId> { ... }
```

Tie-break for `LowestHpPctAlive` and `AdjLowest`: lowest `(hp_current * 1000) / hp_max`, ties broken by `slot_index` ascending (matches S02 invariant). For `RandomEnemyAlive`, the per-skill seed is **combined** with `CombatRng` state via `StdRng::seed_from_u64(skill_seed ^ rng.next_u64())` — this gives per-skill determinism while preserving combat-wide RNG re-seedability for replay. Alternative considered: ignore CombatRng, use only the skill seed (simpler, but every cast of the same skill picks the same target every time — bad for variety). The XOR-mix preserves both properties.

`SingleAlly` is **not** a selector in the pure-fn sense — it's a legality marker that should validate `targeting.side == Ally` at `validate_skill_def` time. The selector value, if set to `SingleAlly`, means "the caller-supplied target must be an ally and alive"; the pipeline uses the user-chosen target unchanged. Treat this as a `TargetSide::Ally` validation strengthener with a friendly name.

Pipeline integration (`src/combat/turn_system/pipeline.rs:step_declaration`, around line 70-80 after `resolve_action`): if the resolved action has `targeting.selector = Some(sel)`, call `select_primary(sel, ...)` and override `action.target` before the inflight is constructed. This keeps the existing multi-target arm (line 182+) unchanged — selectors decide *primary*, shape decides *spread*.

**Fixture skills + CLI scenario**: add four `Implemented` skills to `skills.ron` — one per selector — each using `TargetShape::Single` so the spread logic is decoupled from selector logic. `combat_cli --scenario selectors` casts each in sequence, prints the resolved primary id+slot per cast, emits JSONL with `{skill_id, selector, resolved_target_id, resolved_target_slot, source_id}`. Determinism gate: 2× run byte-diff (random selector with seed=42 must pick the same target both runs).

## Implementation Landscape

### Key Files

**Schema:**
- `src/data/skills_ron.rs` — add `TargetSelector` enum (with `#[serde(deny_unknown_fields)]` on struct variants like `RandomEnemyAlive`). Add `selector: Option<TargetSelector>` to `SkillTargeting` with `#[serde(default)]`. Add validation in `validate_skill_def`: `SingleAlly` selector requires `side == Ally`. Add unit tests for RON round-trip (mirror skills_ron.rs:461 `effect_roundtrip_damage_struct_variant` pattern for each variant — especially `RandomEnemyAlive { seed: u64 }`).

**Resolver:**
- `src/combat/resolution.rs` — extend `TargetEntry` (currently `id, team, slot_index, alive` at line 58-63) with `hp_current: i32, hp_max: i32` (needed for `LowestHpPctAlive`/`AdjLowest`). Or alternative: pre-compute `hp_pct_milli: i32` in the snapshot builder to keep `select_primary` integer-arithmetic-only. Either works; pre-computed is slightly safer (one less place to do the same math). Add `select_primary()` pure fn + 4 selector helper fns. Add table-driven tests alongside existing `resolve_targets_*` tests.

**Pipeline:**
- `src/combat/turn_system/pipeline.rs:step_declaration` (around line 72-95) — after `resolve_action()` returns `action`, check `action.target_selector` (need to thread `targeting.selector` through `ResolvedAction` — see state.rs `ResolvedAction` struct — add `target_selector: Option<TargetSelector>` field). If present, build a `TargetableSnapshot` from `actors` (same pattern as line 187-201 in `step_app`), call `select_primary`, override `action.target`. RNG access: `step_declaration` doesn't currently take `CombatRng` — needs to be added to the system signature. Check `turn_system::resolve_action_system` to wire it through.
- `src/combat/state.rs::ResolvedAction` — add `target_selector: Option<TargetSelector>` (carried through from `SkillTargeting`).
- `src/combat/resolution.rs::resolve_action` — populate the new field from `skill.targeting.selector`.

**Snapshot builder needs HP per entry:**
- `src/combat/turn_system/pipeline.rs` — where `TargetableSnapshot` is built (line 191-201), include `hp_current` and `hp_max` from `unit` in each `TargetEntry`. Trivial extension.

**Action query:**
- `src/combat/action_query.rs` — if `selector` is set, the *displayed* targets in the affordance UI should ideally collapse to the auto-selected one. For M018 (headless first, UI deferred), simplest is: leave `query_all_target_affordances` returning the full list, and let the pipeline override at `step_declaration` time. The user sees all affordances but the cast auto-aims. Document this as a known limitation; full UI integration is post-M018.

**Fixtures:**
- `assets/data/skills.ron` — add 4 fixture skills:
  - `auto_strike` (Single + LowestHpPctAlive) — Implemented
  - `adj_focus` (Single + AdjLowest) — Implemented
  - `chaos_bolt` (Single + RandomEnemyAlive{seed: 42}) — Implemented
  - `holy_aid` (Single + SingleAlly + side: Ally + Revive) or simpler (e.g. heal-shaped damage with side: Ally) — Implemented
  - Round-trip RON parse asserted in the existing `parse_canonical_skills_ron` test (skills_ron.rs:776) — bump asset count from 74.

**CLI scenario:**
- `src/bin/combat_cli.rs` — new `run_selectors_scenario()` function. Build 3-enemy mock encounter with distinct HP fractions (e.g. 60/40, 80/40, 40/40 → enemy 3 has lowest HP%), cast each of the 4 fixture skills, print resolved primary for each. Emit JSONL per cast with `{skill_id, selector_kind, resolved_target_id, resolved_target_slot}`. Add `Some("selectors")` arm to dispatcher (line 1050).

**Tests:**
- `tests/target_selector_resolution.rs` (new, functional naming) — integration test covering all 4 selectors end-to-end (cast → primary chosen → damage lands on expected unit). Mirror `target_shape_blast_spillover.rs` structure.
- Pure-resolver unit tests in `src/combat/resolution.rs` `#[cfg(test)] mod tests` — table-driven, one mod per selector, covering tie-breaks, empty pools, KO'd candidates, single-candidate.
- RNG determinism test: same skill seed + same CombatRng default seed → same target id across two App runs.

### Build Order

**First proof: `select_primary()` + `TargetSelector` enum + RON round-trip.** This is the schema + pure-fn foundation. Once round-trip is green and all 4 selectors have table-driven unit tests, downstream wiring is mechanical.

**Second: thread `selector` through `ResolvedAction` and override `action.target` in `step_declaration`.** Integration test `tests/target_selector_resolution.rs` covers all four selectors casting on a 3-enemy app.

**Third: fixture skills + CLI scenario + determinism gate.** Determinism for `RandomEnemyAlive` is the load-bearing check — 2× run byte-diff on `--scenario selectors` must pass.

### Verification Approach

- `cargo test --test target_selector_resolution --test target_shape_blast_spillover --test target_shape_aoe_all_order --test slot_index_tiebreak` — must all pass.
- `cargo test` full suite — 0 failures across all binaries.
- `cargo check --features windowed` — 0 errors.
- `combat_cli --scenario selectors` × 2, byte-for-byte JSONL diff — DETERMINISM PASS (this is the key gate for the random selector).
- Greppable invariant: each selector variant appears in `match` arms in `select_primary` (4 arms), and in `validate_skill_def` selector-side validation.
- RON catalog count test (`parse_canonical_skills_ron` at skills_ron.rs:776): bump from 74 → 78.

## Constraints

- **Determinism is hard constraint:** `RandomEnemyAlive{seed}` MUST be reproducible across runs. Per-skill seed + CombatRng-state XOR mix gives both replay-determinism and per-skill variety. No wall-clock anywhere.
- **CombatRng is a single shared resource** (rng.rs:9). The pipeline already accesses it via `Option<ResMut<CombatRng>>` in `step_app` (used for status-accuracy rolls at line 1201). `step_declaration` needs the same access added.
- **Selector is orthogonal to shape:** a Single-shape skill with LowestHpPctAlive is the canonical "auto-aim weakest" pattern. The selector overrides the *primary*; shape resolves spread off that primary. Don't conflate the two.
- **Headless first:** all four selectors must work without `windowed`. They do — pure fns over a snapshot.
- **`SingleAlly` is legality, not selection:** validation strengthens `side: Ally`, but the user-chosen target is still used unchanged. Don't write a "find an ally" pure fn; instead validate the user's choice is a living ally.

## Common Pitfalls

- **Per-skill seed without combat-state mix → every cast same target.** If `chaos_bolt` always targets the same enemy because its seed is fixed, players notice. Mix with `CombatRng` state.
- **HP percent integer math drift** — use `(hp_current * 1000) / hp_max` (per-mille), not `(hp_current * 100) / hp_max` (per-cent). Per-cent has too few resolution levels and creates spurious ties on small HP differences.
- **`#[serde(default)]` on `selector` field** — without this, every existing fixture in `skills.ron` (~70+ skills) becomes invalid (missing field). With `default`, `selector: None` is implicit; existing fixtures stay untouched.
- **`#[serde(deny_unknown_fields)]` on `TargetSelector::RandomEnemyAlive`** — RON struct variants need this to reject typos. Match the pattern used by other struct variants in `skills_ron.rs` (e.g. `SkillTargeting` at line 47, `Effect` at line 158).
- **`AdjLowest` semantics: adjacent to whom?** Two reasonable readings: (a) adjacent to the actor (proximity to attacker — only matters if actor is on the same team grid, but here actor is on the Ally side and targets are on the Enemy side, so the slot_index ±1 reference for adjacency must be *the lowest-HP-pct enemy*, then "adj-lowest" = "the enemies adjacent to the lowest-HP-pct enemy on the same team"). (b) adjacent to the user-chosen target. Recommendation: **interpretation (a) — adjacent to the lowest-HP-pct enemy**. This makes AdjLowest a "smart pivot" selector and is the natural counterpart to Blast (which spreads from a chosen primary).
- **ResolvedAction.target_shape vs target_selector cohabitation** — `resolve_action` populates `target_shape` from `targeting.shape`. The new `target_selector` field follows the same pattern. Make sure `resolve_action_uses_targeting_shape_over_damage_effect_shape` test (resolution.rs:731) is unaffected.

## Open Risks

- **AdjLowest semantic ambiguity** (see pitfalls above) — needs a one-line decision in the slice plan before implementation. The user (or planning agent) should confirm interpretation (a) vs (b).
- **Selector visibility in action_query UI** — if a windowed UI is wired post-M018, the user expects the affordance to indicate "auto-targeted" instead of listing 4 enemies. For M018 headless, this is a known limitation; no breaking change.
- **`SingleAlly` + side: Ally validation interplay** — the `validate_skill_def` already enforces side: Ally for revive skills (line ~317). The new `SingleAlly` selector should reuse the same enforcement path, not duplicate it.
- **Schema migration risk for skills.ron** — adding `selector: Option<...>` with `#[serde(default)]` is non-breaking for the 70+ existing fixtures. The `parse_canonical_skills_ron` test at skills_ron.rs:776 catches any breakage. Bump expected count from 74 to 78 to cover the four new fixtures.

## Skills Discovered

| Technology | Skill | Status |
|------------|-------|--------|
| Bevy 0.18 ECS | `bevy` (installed) | available |
| Rust testing | `rust-testing` (installed) | available |
| Rust idioms | `rust-best-practices` (installed) | available |
| Serde (RON) | none directly | not needed — well-established pattern via `#[serde(default)]` + `#[serde(deny_unknown_fields)]` already used throughout `skills_ron.rs` |

## Sources

None. All design constraints derived from M018-CONTEXT, S02-SUMMARY, and direct read of `src/combat/resolution.rs`, `pipeline.rs`, `skills_ron.rs`, `rng.rs`, `events.rs`, `unit.rs`, `follow_up.rs` (for prior-art on lowest-id and team-aware selection patterns).
