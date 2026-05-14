# S03: TargetShape::Bounce(N) path-dependent chain con tie-break — Research

**Date:** 2026-05-13
**Depth:** Targeted (extends the S02 resolver foundation; key open question is the hop-≥2 selector + KO/repeat semantics, otherwise mechanically straightforward)

## Summary

S03 adds `TargetShape::Bounce(u8)` to the resolver landscape established by S02. Bounce is path-dependent: hop 0 hits the user-selected primary (alive enemy); each subsequent hop selects a fresh target from the *currently alive* set, recomputed live. The hop-≥2 selector is the only real design question — every other piece of plumbing (validation gates, `apply_damage_only` per-target loop, JSONL events, `combat_cli --scenario` branch) is mechanical reuse of the S02 pattern.

S02 designed `resolve_targets()` as a pure fn over `TargetableSnapshot` returning a flat `Vec<UnitId>`, but Bounce **cannot reuse that signature unchanged**: hop N depends on world state *after* hop N-1 applies damage (and possibly causes a KO). The natural shape is a new pure helper, `resolve_bounce_chain(primary, n, snapshot_fn) -> Vec<UnitId>` where the snapshot is rebuilt between hops — OR — keeping `resolve_targets()` total but stepping hops one at a time inside the pipeline, rebuilding the snapshot after every `apply_damage_only` call. The second form fits the pipeline mut/imm borrow ordering already established by S02 (snapshot read pass → actor_pairs collect → per-target loop with `get_many_mut`).

Two sub-questions need resolution before fixture JSONL is written (carried forward from M018-CONTEXT Q2): **(a)** Can the same target be hit twice in a chain? **(b)** When the current hop's target dies mid-chain, does the next hop's selector see the now-dead unit as eligible (no — alive-only is implied) or skip with absorbed hop (treat as miss)? Recommendation: **no repeats within a chain** (tracking a `HashSet<UnitId>` of already-hit ids), **alive-only filter** for next-hop candidates (KO mid-chain → just remove from eligibility, the chain naturally truncates if no eligible targets remain). This gives the cleanest deterministic JSONL: hop N either fires on a fresh alive target, or the chain ends.

## Recommendation

**Approach: extend the resolver and pipeline, do not invent a new system.** Add `Bounce(u8)` to `TargetShape` enum and to the three validation gates (allowlist `Single|Blast|AllEnemies|Bounce`). In `pipeline.rs:step_app`, change the multi-target match arm to also accept `Bounce(_)` and replace the single `resolve_targets()` call with a hop loop that:

1. Builds the initial snapshot.
2. Selects hop 0 = primary (user-chosen target).
3. For hop k in 1..N: rebuild snapshot from current ECS state (alive only, excluding `already_hit` set), apply hop-≥2 selector. If no candidate remains, break.
4. After each hop, call `apply_damage_only()` exactly as the Blast/AllEnemies loop does, then re-snapshot before the next hop.

**Hop-≥2 selector recommendation: `LowestHpPctAlive` with `slot_index` ascending as tie-break.** Rationale: matches Tentomon Bounce identity (chains seek out the weakest), is fully deterministic (no RNG seed plumbing needed for S03), and reuses the `slot_index` tie-break invariant already established. S04 will *separately* add seeded random as a generally-available selector, but Bounce itself should ship with a deterministic policy so its fixture JSONL diff-stabilizes cleanly.

**No-repeat policy**: maintain `already_hit: HashSet<UnitId>` inside the bounce loop, exclude these from the candidate pool. Once the candidate pool is empty (all alive enemies hit, or all KO'd), the chain terminates early and the JSONL just shows fewer hop entries. Document this as an emergent property — N=3 hops with only 2 alive enemies → 2 hop entries, no "missed hop" event.

**Carry decisions to S04**: the selectors S04 will productize (`AdjLowest`, `LowestHpPctAlive`, `RandomEnemyAlive{seed}`, `SingleAlly`) are a strict superset of what Bounce hop-≥2 needs. Implementing `LowestHpPctAlive` as a small standalone fn now and reusing it in S04 is the right ordering — S04's task list shrinks by one selector.

## Implementation Landscape

### Key Files

**Schema + resolver:**
- `src/data/skills_ron.rs` — extend `TargetShape` enum with `Bounce(u8)` (tuple variant; hop count). Update `validate_skill_def` allowlist (line ~280) to permit `Bounce(_)`. Update the `tests` module test `validate_rejects_implemented_non_single_shape` if a Bounce-shaped probe is added (deferred Row remains the canonical reject case — Bounce becomes accepted).
- `src/combat/resolution.rs` — keep `resolve_targets()` total: for `Bounce(_)`, return `vec![primary]` (hop 0 only) so callers that want the full chain explicitly step. Add a new fn `next_bounce_hop(snapshot, already_hit) -> Option<UnitId>` implementing `LowestHpPctAlive` + `slot_index` tie-break. `target_shape_is_executable_now` allowlist (line 242): add `Bounce(_)`. `target_shape_rejection_reason` follows.
- `src/combat/action_query.rs` — line ~487: add `Bounce(_)` to the executable shape allowlist used in `target_status_for_unit`. Otherwise `target_status_for_unit` continues to validate the user-chosen primary (Bounce's hop 0); per-hop targets are not surfaced through affordances (planner choice — the user only picks the primary).

**Pipeline:**
- `src/combat/turn_system/pipeline.rs` — line 182 `match` on `Blast | AllEnemies`: widen to also cover `Bounce(_)`. Inside the multi-target arm, branch on the shape:
  - For `Blast | AllEnemies`: existing `resolve_targets()` call + per-target loop, **unchanged**.
  - For `Bounce(n)`: new loop body — `already_hit` HashSet seeded with `inflight.action.target`; iterate up to `n` times, rebuilding `TargetableSnapshot` from `actors.iter()` each iteration (alive-only, exclude already_hit), selecting next hop via `next_bounce_hop`, calling `apply_damage_only` exactly as the existing loop does. Resource hoisting (Phase 1) and post-loop attacker effects (Phase 3) need **no change** — they fire once-per-cast, identical to S02's design. The actor_pairs `Vec<(Entity, UnitId)>` lookup remains valid as long as we rebuild the snapshot each hop (no entity moves during the loop).

**Fixtures + CLI:**
- `assets/data/skills.ron` — add `chain_bolt` (Implemented, Bounce(3), Damage{target: Bounce(3)}, ToughnessHit). Note: `Effect::Damage{target}` validates against `targeting.shape` (resolution.rs:301), so `target: Bounce(3)` must match the targeting shape exactly. Confirm `Effect::Damage` serializes Bounce variants correctly — `target: Bounce(3)` is a tuple variant inside a struct variant, may need a brief round-trip test.
- `src/bin/combat_cli.rs` — new `run_bounce_chain_scenario()` function in the same style as `run_aoe_blast_scenario` (line 921). Builds 3-enemy mock encounter, runs N=3 bounce, kills middle enemy at hop 2 to force the chain-recompute path. Emits one `BounceHop` JSONL line per hop with `{hop_index, target_id, target_slot, target_hp_pre, target_hp_post, ko}`. Add `Some("bounce-chain")` arm to the dispatcher (line 1050).

**Tests:**
- `tests/target_shape_bounce_chain.rs` (new, functional naming per CLAUDE.md) — integration test mirroring `target_shape_blast_spillover.rs:1-80` structure. Cases:
  1. N=3, three alive enemies, no KO → 3 distinct hops, slot_index tie-break on equal HP%.
  2. N=3, hop 2 KOs target, only one alive enemy left → chain truncates at hop 3 (or hop 2 if `already_hit` blocks the only remaining target).
  3. N=3, only one alive enemy at cast time → 1 hop only (already_hit blocks repeats).
  4. N=3, primary survives but is below LowestHpPct — confirm primary is **not** revisited at hop 1.
- Pure-resolver unit tests in `src/combat/resolution.rs` `#[cfg(test)] mod tests` (alongside existing `resolve_targets_*` tests) covering `next_bounce_hop()` with table-driven snapshots (lowest HP% tie-break on slot_index asc, empty-pool returns None, all-KO returns None).

### Build Order

**First proof: pure `next_bounce_hop()` + Bounce in the validation gate trinity.** This is the highest-risk piece (selector semantics need to be locked) and unblocks everything downstream. Write the selector + table-driven tests, then add `Bounce(u8)` to the three validation sites, then confirm `cargo check` is green.

**Second: pipeline hop loop.** Once the selector is proven pure, the pipeline change is a localized edit inside the existing multi-target arm — extend the match, add the loop, reuse `apply_damage_only`. Integration test `tests/target_shape_bounce_chain.rs` verifies end-to-end.

**Third: fixture skill + CLI scenario.** Once integration tests pass, add `chain_bolt` to `skills.ron` (be careful with `Effect::Damage{target: Bounce(3)}` round-trip) and wire `--scenario bounce-chain`. Determinism gate: 2× run + byte-diff on JSONL.

### Verification Approach

- `cargo test --test target_shape_bounce_chain --test target_shape_blast_spillover --test target_shape_aoe_all_order --test slot_index_tiebreak` — must all pass (S02 regressions caught).
- `cargo test` full suite — 0 failures across ~40 binaries. M017 regression tests (`status_slowed_delay`, `tempo_resistance`, `turn_advance_split`) must remain green.
- `cargo check --features windowed` — 0 errors.
- `combat_cli --scenario bounce-chain` × 2, byte-for-byte JSONL diff — DETERMINISM PASS.
- Greppable invariant: `Bounce` appears in the allowlist at all three gate sites (`skills_ron.rs:~282`, `resolution.rs:242`, `action_query.rs:~487`).

## Constraints

- **Determinism is hard constraint** (CLAUDE.md): no wall-clock, no unseeded RNG. The `LowestHpPctAlive + slot_index tiebreak` selector avoids RNG entirely for S03. Per-mille HP% computed as `(hp_current * 1000) / hp_max` to avoid float drift — Bevy uses i32 consistently for HP elsewhere.
- **`Effect::Damage { target }` must match `targeting.shape` exactly** (resolution.rs:301-310 in `validate_skill_def`). If we use `targeting.shape: Bounce(3)`, the effect must be `Damage { target: Bounce(3) }` — same hop count. This is a copy-paste pitfall in the fixture.
- **Pure-resolver pattern (S02 D02):** `next_bounce_hop` should consume a `TargetableSnapshot` + `&HashSet<UnitId>` (already-hit) — no ECS access. The pipeline rebuilds the snapshot between hops.
- **Headless first:** all systems must run without `windowed`. Bounce involves no UI surface.
- **JSONL stability:** existing JSONL events use `CombatEvent`/`CombatEventKind` (events.rs). Adding a new `BounceHop` variant is **not** recommended — the per-hop `OnDamageDealt` events already carry attacker/target/amount and are emitted by `apply_damage_only`. The CLI scenario can wrap them with a hop counter at print time without polluting the engine event schema.

## Common Pitfalls

- **Snapshot staleness mid-chain** — if the hop loop reuses the initial snapshot, KO'd-at-hop-2 targets stay "alive" in the snapshot's view for hop 3 candidate selection. **Avoid:** rebuild the snapshot from `actors.iter()` at the top of each hop iteration. The actor_pairs `Vec<(Entity, UnitId)>` is stable (entities don't move), so just the alive/hp fields need refresh.
- **Bounce(0) edge case** — semantically meaningless (0 hops). The validation gate should accept it only as a deferred shape, or the loop should be a no-op (zero hops = zero damage events, action effectively a no-op). Recommendation: validate `N >= 1` in `validate_skill_def` and reject Bounce(0) as `UnimplementedTargetShape`.
- **`already_hit` seeding** — the primary (hop 0) must be in `already_hit` *before* the loop starts; otherwise hop 1's selector may re-pick the primary if it has lowest HP%. This is the no-repeat policy made concrete.
- **Resource consumption ordering** — S02's Phase 1 (SP/ult/streak hoist) runs **before** any damage. Bounce inherits this for free: even if the chain truncates early (fewer hops than N), SP/ult are spent in full. This is intentional (you paid for the skill cast, not for hits landed).
- **JSONL fan-out per hop already deterministic** — `apply_damage_only` emits per-target `OnDamageDealt`. Since hops are sequential and target ids come from `next_bounce_hop` (slot_index tie-break), the event order is naturally stable. No extra sort needed.
- **Effect::Damage{target: Bounce(N)} RON serialization** — Bounce is a tuple variant inside a struct field. Round-trip test (mimic `effect_roundtrip_damage_struct_variant` at skills_ron.rs:461) recommended before adding the fixture.

## Open Risks

- **The hop-≥2 selector choice is a design lock-in.** If post-M018 the user wants a different Bounce semantic (e.g. Tentomon = lowest HP%, but Devimon = random), we need either a `Bounce(N, Selector)` variant or a per-skill selector override. Recommendation: ship S03 with `LowestHpPctAlive` hardcoded for Bounce, expose other selectors as standalone `TargetShape::SingleSelected(Selector)` in a future milestone if needed. Document this in the SUMMARY so the post-M018 roster-identity slice knows.
- **Action affordance UI** — current `action_query.rs::target_status_for_unit` enumerates per-target affordances. For Bounce, the user only picks the primary (hop 0); future hops are not surfaced. This is consistent with how Blast surfaces only the primary (spillover is implicit). No change needed beyond extending the allowlist, but the windowed UI (out of scope M018) may want a "shows next-hop preview" affordance later.
- **Snapshot rebuild cost** — rebuilding `TargetableSnapshot` once per hop is O(units) and the iteration count is bounded by N (small, typically 3). No perf concern, but worth noting if Bounce(N>10) is ever used.

## Skills Discovered

| Technology | Skill | Status |
|------------|-------|--------|
| Bevy 0.18 ECS | `bevy` (installed) | available — already used in this codebase |
| Rust testing | `rust-testing` (installed) | available — integration test in `tests/` follows existing conventions |
| Rust idioms | `rust-best-practices` (installed) | available — selector fn purity, no-clone snapshot pattern |

## Sources

None. All design constraints derived from M018-CONTEXT, S02-SUMMARY, and direct read of `src/combat/resolution.rs`, `pipeline.rs`, `skills_ron.rs`, `action_query.rs`.
