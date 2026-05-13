# S03: TargetShape: Bounce(N) path-dependent chain con tie-break

**Goal:** Refactor TargetShape::Bounce so kernel only provides the chain-iteration primitive; selector, repeat policy, and per-hop damage curve are all declared per-skill in the RON DSL. Kernel exposes a dispatcher that reads these knobs from the skill definition and walks N hops generically. Two distinct fixtures (different selector + curve + repeat combos) prove the kernel carries no per-skill bias. Determinism preserved via slot_index ascending tie-break baked into every selector variant.
**Demo:** CLI scenario con N=3 hops, enemy che muore al hop 2: chain ricalcola hop 3 sui survivors mantenendo tie-break slot_index asc. JSONL log mostra sequenza hop completa con stato vivo/morto a ogni step.

## Must-Haves

- TargetShape::Bounce in RON carries { hops: u8, selector: BounceSelector, repeat: RepeatPolicy } — kernel reads, never hardcodes
- BounceSelector enum in DSL: LowestHpPctAlive, NextSlotAlive, AdjLowest (others can be added without kernel changes); each variant resolves with slot_index asc tie-break
- RepeatPolicy enum in DSL: NoRepeat (default), AllowRepeat
- Per-hop damage curve declared in DSL (constant base_damage + optional falloff_pct OR explicit per-hop array)
- Pipeline hop loop in turn_system/pipeline.rs is generic: dispatches on (selector, repeat, damage_curve) read from the inflight action
- At least 2 fixture skills with distinct (selector, repeat, curve) tuples both round-trip RON and execute end-to-end via combat_cli
- combat_cli scenarios produce byte-for-byte deterministic JSONL across 2 runs each
- Zero regressions in the ~40 test binaries; M017 + S01 + S02 suites green

## Proof Level

- This slice proves: integration — real headless runtime via combat_cli + integration test under tests/, byte-for-byte JSONL determinism gate over two fixture skills.

## Integration Closure

Builds on S02 primitives (TargetableSnapshot, resolve_targets, apply_damage_only, SlotIndex, three-gate allowlist trinity). T01 selector helper from prior plan iteration (already landed in commit af09d40 as a hardcoded LowestHpPct fn) is repurposed as one entry in the BounceSelector dispatch table. Three-gate widening for the old `Bounce(u8)` shape (commits d4dc202, 9bf931d) is preserved but extended to accept the new struct form. After S03: S04 (selectors AdjLowest / LowestHpPctAlive standalone / RandomEnemyAlive{seed} / SingleAlly) consumes the same BounceSelector catalog applied outside Bounce, e.g. for Single targeting variants. Damage curve and repeat policy stay in DSL forever — no future milestone reverts these to kernel hardcoding.

## Verification

- No new CombatEvent variant. Per-hop visibility still comes from existing OnDamageDealt events; combat_cli wraps them with {hop_index, selector, repeat_policy} at print time for human-readable JSONL. JSONL schema documents the selector and repeat policy in use so each scenario log is self-describing.

## Tasks

- [ ] **T01: BounceSelector + RepeatPolicy DSL enums + selector dispatcher (refactor of prior next_bounce_hop)** `est:1h 30m`
  Introduce two enums in src/data/skills_ron.rs: `BounceSelector` (variants: LowestHpPctAlive, NextSlotAlive, AdjLowest — extensible; serde-derived) and `RepeatPolicy` (NoRepeat, AllowRepeat). In src/combat/resolution.rs, replace the previously-landed hardcoded `next_bounce_hop` with a dispatcher `select_bounce_hop(selector: BounceSelector, snapshot: &TargetableSnapshot, already_hit: &HashSet<UnitId>, enemy_team: Team, last_target_slot: Option<u8>) -> Option<UnitId>` that pattern-matches the selector and calls a small pure fn per variant. LowestHpPctAlive logic moves into `select_lowest_hp_pct_alive` (existing integer per-mille math + slot_index asc tie-break preserved). Add `select_next_slot_alive` (lowest slot_index > last_target_slot among alive enemies not in already_hit) and `select_adj_lowest` (alive enemy with |slot - last_target_slot| <= 1 by lowest HP%, slot tie-break). All variants must honor already_hit when the in-effect RepeatPolicy is NoRepeat; the dispatcher receives the policy from the caller and skips the already_hit filter when AllowRepeat. Keep helpers total — no panics. Migrate existing table-driven tests for next_bounce_hop to the new dispatcher; add per-variant tests including AllowRepeat case (same target picked twice when policy allows). Do not yet change TargetShape::Bounce schema — that's T02.
  - Files: `src/data/skills_ron.rs`, `src/combat/resolution.rs`
  - Verify: cargo test --lib resolution::tests && cargo check

- [x] **T02: Extend TargetShape::Bounce schema to struct form { hops, selector, repeat } + damage curve in Effect::Damage** `est:1h 30m`
  Migrate `TargetShape::Bounce(u8)` to struct variant `TargetShape::Bounce { hops: u8, selector: BounceSelector, repeat: RepeatPolicy }` in src/data/skills_ron.rs. Update the three validation gates (validate_skill_def, resolution::target_shape_is_executable_now, action_query::target_status_for_unit) to match the new struct form — keep N>=1 enforcement; reject hops==0 with UnimplementedTargetShape. In `Effect::Damage`, add an optional `per_hop` field: enum `DamageCurve { Constant, Falloff { pct: u16 }, PerHop(Vec<i32>) }` defaulting to Constant via serde default. validate_skill_def enforces: if curve is PerHop(v), v.len() == hops; if Falloff, pct <= 100. The base_damage field is unchanged and feeds the curve (Falloff applies to base; PerHop overrides base per index). Update the existing chain_bolt fixture and all tests asserting on old Bounce(u8) literal. Add a RON round-trip unit test for the struct form mirroring effect_roundtrip_damage_struct_variant, plus a positive validator test for PerHop and Falloff variants and a negative test for PerHop length mismatch.
  - Files: `src/data/skills_ron.rs`, `src/combat/resolution.rs`, `src/combat/action_query.rs`, `assets/data/skills.ron`
  - Verify: cargo test --lib skills_ron::tests && cargo test --lib resolution::tests && cargo check

- [ ] **T03: Generic kernel hop loop: dispatch on selector + repeat + damage curve read from inflight action** `est:2h 30m`
  Refactor the Bounce arm in src/combat/turn_system/pipeline.rs so the kernel carries zero per-skill bias. Read (hops, selector, repeat) from the inflight action's TargetShape and the damage curve from the Effect::Damage being applied. Hop 0 = primary; for hop k in 1..hops rebuild TargetableSnapshot from actors, call select_bounce_hop(selector, snapshot, already_hit, enemy_team, last_target_slot), break on None. Track last_target_slot for selectors that need it (NextSlotAlive, AdjLowest). already_hit insertion gated by repeat policy: NoRepeat inserts after each hop; AllowRepeat skips insertion (or always-empty set). Per-hop damage computation: Constant → base_damage; Falloff{pct} → base_damage * (100 - pct*k) / 100 floored at 1; PerHop(v) → v[k]. SP/ult/streak hoist (S02 D04) unchanged — paid once pre-loop regardless of truncation. Add integration test tests/target_shape_bounce_chain.rs with 4 cases: (1) LowestHpPct + NoRepeat + Constant full chain N=3 no KO; (2) NextSlotAlive + NoRepeat + Falloff(20%) with KO mid-chain → chain truncates or skips to next slot; (3) LowestHpPct + AllowRepeat + PerHop[30,15,5] → same target may be hit twice when still lowest HP% after first hop; (4) pool exhaustion truncates silently. Each case asserts on per-hop damage delta from OnDamageDealt to prove curve is honored.
  - Files: `src/combat/turn_system/pipeline.rs`, `tests/target_shape_bounce_chain.rs`
  - Verify: cargo test --test target_shape_bounce_chain --test target_shape_blast_spillover --test target_shape_aoe_all_order --test slot_index_tiebreak

- [ ] **T04: Two distinct Bounce fixture skills exercising different (selector, repeat, curve) tuples** `est:1h`
  Add two Implemented fixture skills to assets/data/skills.ron that exercise the kernel's generic dispatcher: (a) `chain_bolt` — Bounce{hops:3, selector:LowestHpPctAlive, repeat:NoRepeat} + Damage{base:18, curve:Constant} (canonical bounce, no curve); (b) `arc_bolt` — Bounce{hops:3, selector:NextSlotAlive, repeat:NoRepeat} + Damage{base:24, curve:Falloff{pct:25}} (slot-walking with falloff). Optionally add (c) `echo_strike` — Bounce{hops:2, selector:LowestHpPctAlive, repeat:AllowRepeat} + Damage{curve:PerHop[20,12]} to prove AllowRepeat path. Each must round-trip RON and survive SkillBook::load validation. Add focused unit tests in skills_ron.rs `#[cfg(test)] mod tests`: per-fixture deserialize + validate; assert the in-memory representation matches the expected struct (selector variant, repeat variant, curve variant). Cross-link fixture metadata (id, label) to satisfy any existing data_loaders integration test.
  - Files: `assets/data/skills.ron`, `src/data/skills_ron.rs`
  - Verify: cargo test --lib skills_ron::tests && cargo test --test data_loaders

- [ ] **T05: combat_cli scenarios for each fixture + determinism gate × N + final regression sweep** `est:1h 30m`
  Add `run_bounce_chain_scenario()` and `run_arc_bolt_scenario()` to src/bin/combat_cli.rs (mirroring run_aoe_blast_scenario structure). Each spins up a deterministic 3-enemy mock encounter with HP values engineered to drive a meaningful chain through the fixture's selector + curve combo (chain_bolt: HP gradient surfacing LowestHpPct progression; arc_bolt: slot-walking with falloff visible in per-hop damage). Emit one JSONL line per hop with `{event:"BounceHop", hop_index, source_id, target_id, target_slot, target_hp_pre, target_hp_post, damage_dealt, selector, repeat_policy, ko, skill_id}` — wrap existing OnDamageDealt events at print time, no engine schema churn. Add `Some("bounce-chain")` and `Some("arc-bolt")` arms to the dispatcher. Run determinism gate per scenario: invoke twice, capture stdout, byte-diff must be empty. Final sweep: `cargo test` full suite green (S02 + M017 suites included), `cargo check --features windowed` clean. Document determinism diff results in verification evidence.
  - Files: `src/bin/combat_cli.rs`
  - Verify: cargo build --bin combat_cli && bash -c 'for s in bounce-chain arc-bolt; do cargo run --quiet --bin combat_cli -- --scenario $s > /tmp/${s}1.txt 2>&1 && cargo run --quiet --bin combat_cli -- --scenario $s > /tmp/${s}2.txt 2>&1 && diff -q /tmp/${s}1.txt /tmp/${s}2.txt; done' && cargo test && cargo check --features windowed

## Files Likely Touched

- src/data/skills_ron.rs
- src/combat/resolution.rs
- src/combat/action_query.rs
- assets/data/skills.ron
- src/combat/turn_system/pipeline.rs
- tests/target_shape_bounce_chain.rs
- src/bin/combat_cli.rs
