# S03: TargetShape: Bounce(N) path-dependent chain con tie-break

**Goal:** Add TargetShape::Bounce(u8) to the resolver landscape with path-dependent hop chain, hop-≥2 selector = LowestHpPctAlive with slot_index ascending tie-break, no-repeat within a chain (already_hit HashSet), alive-only candidate filter, and a `combat_cli --scenario bounce-chain` proof that produces byte-for-byte deterministic JSONL across N=3 hops including a mid-chain KO event.
**Demo:** CLI scenario con N=3 hops, enemy che muore al hop 2: chain ricalcola hop 3 sui survivors mantenendo tie-break slot_index asc. JSONL log mostra sequenza hop completa con stato vivo/morto a ogni step.

## Must-Haves

- `TargetShape::Bounce(u8)` added to schema and accepted at all three validation gate sites (skills_ron, resolution, action_query) using identical allowlist `Single|Blast|AllEnemies|Bounce`.
- Pure helper `next_bounce_hop(snapshot, &already_hit) -> Option<UnitId>` implementing LowestHpPctAlive (per-mille integer math) + slot_index asc tie-break, covered by table-driven unit tests in resolution.rs.
- Pipeline `step_app` multi-target arm widens to include Bounce(_); for Bounce, runs a hop loop that rebuilds TargetableSnapshot between hops, applies `apply_damage_only` per hop, terminates early when candidate pool empties.
- `already_hit` HashSet is seeded with the primary (hop 0) before the loop; no enemy hit twice in a single chain.
- Fixture skill `chain_bolt` (Implemented, Bounce(3), Damage{target:Bounce(3)}, ToughnessHit) round-trips through RON and passes `validate_skill_def`.
- New integration test `tests/target_shape_bounce_chain.rs` covers: full chain no-KO, mid-chain KO truncation, only-one-alive truncates to 1 hop, primary not revisited when lowest HP%.
- `combat_cli --scenario bounce-chain` × 2 byte-for-byte identical JSONL (DETERMINISM PASS).
- `cargo test` full suite green (0 failures, ~40 binaries), `cargo check --features windowed` clean, M017 regression suites (status_slowed_delay, tempo_resistance, turn_advance_split) and S02 suites (target_shape_blast_spillover, target_shape_aoe_all_order, slot_index_tiebreak) all pass.

## Proof Level

- This slice proves: integration — real headless runtime via combat_cli scenario + integration test under tests/, byte-for-byte JSONL determinism gate.

## Integration Closure

Upstream: TargetableSnapshot/resolve_targets (S02 D02 pure-resolver pattern), apply_damage_only (S02 D03 damage-only-per-target fn), SlotIndex (S02 T01), three-gate allowlist trinity (S02 T03), pipeline multi-target arm (S02 T04). New wiring: hop loop inside pipeline.rs multi-target arm (Bounce branch), `chain_bolt` fixture in skills.ron, `--scenario bounce-chain` dispatcher arm in combat_cli.rs. Remaining before M018 usable end-to-end: S04 (selectors AdjLowest / LowestHpPctAlive as standalone / RandomEnemyAlive{seed} / SingleAlly) — S03 ships LowestHpPctAlive embedded in next_bounce_hop; S04 will lift it into a reusable selector enum.

## Verification

- No new CombatEvent variant. Per-hop visibility comes from existing `OnDamageDealt` events emitted by `apply_damage_only` (sequential, slot-stable ordering). The combat_cli scenario wraps these with a hop counter at print-time only — engine event schema unchanged. JSONL lines remain diff-stable across runs.

## Tasks

- [ ] **T01: Add Bounce(u8) variant + pure next_bounce_hop() selector with table-driven tests** `est:1h`
  Add `Bounce(u8)` as a tuple variant to `TargetShape` in `src/data/skills_ron.rs`. Add a pure helper `next_bounce_hop(snapshot: &TargetableSnapshot, already_hit: &HashSet<UnitId>, primary_team: Team) -> Option<UnitId>` in `src/combat/resolution.rs`. Selector semantics: filter snapshot to entries where (team == primary_team.opposite() OR same-side enemies of caster — pass enemy team explicitly), alive == true, id not in already_hit; rank by HP%-per-mille ascending (`(hp_current * 1000) / hp_max`) with `slot_index` ascending as deterministic tie-break; return first or None. Use integer math only (no f32) — per-mille mirrors the existing HP-fraction convention. Add `#[cfg(test)] mod tests` cases inside resolution.rs alongside existing `resolve_targets_*` tests: (a) two enemies same HP% → lower slot_index wins; (b) one enemy at lower HP% but already_hit → next-lowest selected; (c) all candidates KO → None; (d) empty pool → None; (e) primary in already_hit → not selected. Do NOT yet wire Bounce into validation gates or pipeline — those are T02/T03. Keep `resolve_targets()` total: for `TargetShape::Bounce(_)`, return `vec![primary]` (hop 0 only).
  - Files: `src/data/skills_ron.rs`, `src/combat/resolution.rs`
  - Verify: cargo test --lib resolution::tests::next_bounce_hop && cargo check

- [ ] **T02: Widen three validation gates to accept Bounce(_)** `est:45m`
  Update the three allowlist sites that previously gated non-Single shapes behind UnimplementedTargetShape to accept Bounce(_):
  1. `src/data/skills_ron.rs:~282` in `validate_skill_def` — change the match `TargetShape::Single | TargetShape::Blast | TargetShape::AllEnemies` to `... | TargetShape::Bounce(_)`. Also enforce `N >= 1` in the same block: if shape is Bounce(0), return `UnimplementedTargetShape` ("Bounce(0) has no hops"). Row and SelfOnly remain deferred.
  2. `src/combat/resolution.rs:241-243` in `target_shape_is_executable_now` — extend the allowlist with `TargetShape::Bounce(_)`.
  3. `src/combat/action_query.rs:485-492` in `target_status_for_unit` — extend the same allowlist.
  Update any test asserting on the rejected-shape error message (Row remains the canonical reject case in `validate_rejects_implemented_non_single_shape`). Add a positive unit test confirming Bounce(3) validates and Bounce(0) is rejected.
  - Files: `src/data/skills_ron.rs`, `src/combat/resolution.rs`, `src/combat/action_query.rs`
  - Verify: cargo test --lib skills_ron::tests && cargo test --lib resolution::tests && cargo check

- [ ] **T03: Extend pipeline multi-target arm with Bounce hop loop + add integration test** `est:2h`
  In `src/combat/turn_system/pipeline.rs` around the multi-target arm (currently line 182: `if matches!(inflight.action.target_shape, TargetShape::Blast | TargetShape::AllEnemies)`), widen the match to also cover `TargetShape::Bounce(_)`. Inside the arm, branch on shape:
  - For `Blast | AllEnemies`: existing `resolve_targets()` + per-target loop unchanged.
  - For `Bounce(n)`: implement hop loop. Hoist SP/ult/streak in Phase 1 exactly as the existing path does (resources spent once-per-cast even if chain truncates — see S02 D04). Seed `let mut already_hit: HashSet<UnitId> = HashSet::from([inflight.action.target])`. Hop 0 = primary; call `apply_damage_only` against the primary entity. For hop k in 1..n: rebuild `TargetableSnapshot` from `actors.iter()` (alive-only filter handled inside `next_bounce_hop`), call `next_bounce_hop(&snapshot, &already_hit, attacker_team.opposite())`. If `None`, break. Otherwise, insert id into `already_hit`, look up entity via the stable `actor_pairs` Vec, call `apply_damage_only` on it. Phase 3 attacker-side effects remain after the loop, unchanged. Use the existing `get_many_mut` / borrow-release pattern (rebuild snapshot after each mut borrow drops).
  Add new integration test `tests/target_shape_bounce_chain.rs` (functional naming per CLAUDE.md) following the structure of `tests/target_shape_blast_spillover.rs`. Cases: (1) N=3 full chain no KO, asserts 3 distinct OnDamageDealt events with slot_index tie-break order on equal HP%; (2) N=3 with primary HP-engineered so hop-2 KOs target, chain still hits hop 3 on remaining alive enemy or truncates if none; (3) only-one-alive-enemy → 1 hop only; (4) N=3 primary is lowest-HP%, confirm hop 1 does NOT revisit primary.
  - Files: `src/combat/turn_system/pipeline.rs`, `tests/target_shape_bounce_chain.rs`
  - Verify: cargo test --test target_shape_bounce_chain --test target_shape_blast_spillover --test target_shape_aoe_all_order --test slot_index_tiebreak

- [ ] **T04: Add chain_bolt fixture skill + RON round-trip test for Bounce(N) inside Effect::Damage** `est:45m`
  Add an Implemented fixture skill `chain_bolt` to `assets/data/skills.ron`. Targeting: `shape: Bounce(3)`, side: Enemy, life: Alive. Effects: `Damage { target: Bounce(3), base_damage: 18, damage_tag: ... }` (target must equal targeting.shape exactly — copy-paste pitfall; see resolution.rs:301). Include a `ToughnessHit`-style toughness damage value matching project conventions (cross-reference existing `nova_burst` Blast fixture). Add a focused RON round-trip unit test in `src/data/skills_ron.rs` `#[cfg(test)] mod tests` mirroring `effect_roundtrip_damage_struct_variant` (line ~461) but for `Effect::Damage { target: Bounce(3), ... }` — proves tuple-inside-struct-variant deserialization is correct. Also add a load-and-validate test confirming `chain_bolt` survives the full `SkillBook::load` pipeline.
  - Files: `assets/data/skills.ron`, `src/data/skills_ron.rs`
  - Verify: cargo test --lib skills_ron::tests::chain_bolt_roundtrip && cargo test --test data_loaders

- [ ] **T05: combat_cli bounce-chain scenario + determinism gate + final regression sweep** `est:1h 30m`
  Add `run_bounce_chain_scenario()` to `src/bin/combat_cli.rs` mirroring `run_aoe_blast_scenario` (line 921). Build a deterministic 3-enemy mock encounter where HP values force a meaningful chain (e.g. primary slot-1 at HP 60, two others at 50 and 40 — first hop lands on user-chosen primary, then next_bounce_hop visits lowest HP% remaining). At hop 2, deal enough damage to KO the current target so the chain naturally recomputes on the remaining survivor. Emit one JSONL line per hop with fields `{event:"BounceHop", hop_index, source_id, target_id, target_slot, target_hp_pre, target_hp_post, ko, skill_id:"chain_bolt"}` — wrap the existing per-hop damage events with the hop counter at print-time only (no new CombatEvent variant). Add `Some("bounce-chain")` arm to the dispatcher around line 1050. Run determinism gate: invoke `cargo run --bin combat_cli -- --scenario bounce-chain` twice, capture stdout to two files, byte-diff must be empty. Then run final regression sweep: `cargo test` full suite (must be all green, S02 + M017 suites included), `cargo check --features windowed` clean. Document the determinism result in T05 verification evidence.
  - Files: `src/bin/combat_cli.rs`
  - Verify: cargo build --bin combat_cli && bash -c 'cargo run --quiet --bin combat_cli -- --scenario bounce-chain > /tmp/bounce1.txt 2>&1 && cargo run --quiet --bin combat_cli -- --scenario bounce-chain > /tmp/bounce2.txt 2>&1 && diff -q /tmp/bounce1.txt /tmp/bounce2.txt' && cargo test && cargo check --features windowed

## Files Likely Touched

- src/data/skills_ron.rs
- src/combat/resolution.rs
- src/combat/action_query.rs
- src/combat/turn_system/pipeline.rs
- tests/target_shape_bounce_chain.rs
- assets/data/skills.ron
- src/bin/combat_cli.rs
