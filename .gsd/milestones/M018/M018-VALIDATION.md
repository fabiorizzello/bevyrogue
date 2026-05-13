---
verdict: pass
remediation_round: 0
---

# Milestone Validation: M018

## Success Criteria Checklist
- [x] **Advance/Delay CLI scenario with cap & floor** | S01: CLI `advance-delay-cap` scenario runs deterministically; JSONL shows cap enforcement (80→50) and floor clamp (delta=0 at AV=0); exit 0
- [x] **Turn order recalculation with AdvanceTurn/DelayTurn** | S01: `TurnAdvance(i32)` fully replaced by `AdvanceTurn(u32)` + `DelayTurn(u32)` with cap ±50% at emission; 554 tests green; rg legacy sweep clean
- [x] **M017 Slowed regression maintained** | S01: `status_slowed_delay.rs` and `tempo_resistance.rs` both green; AV outcome (5000→2000) preserved; event variant migrated to `DelayTurn{30}`
- [x] **Blast targeting with spillover to adjacents** | S02: CLI `aoe-blast` scenario resolves Blast (primary + slot_index ±1); JSONL deterministic across 10 runs (byte-for-byte identical); edge slots and KO'd adjacents handled
- [x] **AoE(All) with slot_index tie-break ordering** | S02: AllEnemies targets resolved in slot_index ascending order; fixture test `target_shape_aoe_all_order` passes; per-target damage applied in order
- [x] **Bounce(N) multi-hop with mid-chain KO** | S03: Bounce hop loop rebuilds TargetableSnapshot each hop; KO'd units excluded from candidate pool in subsequent hops; integration test `bounce_next_slot_no_repeat_falloff_ko_mid_chain` passes
- [x] **Extended selectors (NextSlot, LowestHp)** | S03: BounceSelector enum added; pure `select_bounce_hop()` dispatcher; both selectors exercised in integration tests (4/4 pass)
- [x] **Zero regressions on ~40 existing test binaries** | All slices: `cargo test` full suite shows 201 total tests, 0 failures; M017 tests (status_slowed_delay, tempo_resistance, turn_advance_split) remain green

## Slice Delivery Audit
| Slice | SUMMARY.md | Assessment Verdict | Notes |
|-------|-----------|-------------------|-------|
| S01 (AdvanceTurn/DelayTurn split) | ✓ Present | PASS | 554 tests green; CLI scenario advance-delay-cap; TempoResistance curve applied; cap ±50% at emission; floor clamp enforced |
| S02 (Blast/AoE resolver + SlotIndex) | ✓ Present | PASS | SlotIndex(u8) Component; pure resolve_targets(); Blast + AllEnemies(AoE alias) execution; 6 integration tests; JSONL 10x deterministic |
| S03 (BounceSelector + RepeatPolicy + hop loop) | ✓ Present | PASS | BounceSelector/RepeatPolicy enums; select_bounce_hop() dispatcher; generic hop loop in pipeline.rs; DamageCurve scaling; 4 integration tests; 201 total tests green |

**Known Limitations (deferred, not blocking):**
- Per-hop CombatEvent emission for UI/log observability deferred to later slice
- Pool exhaustion does not emit OnActionFailed — silent truncation only
- DamageCurve::PerHop runtime length guard deferred to later slice

## Cross-Slice Integration
| Boundary | Producer Summary | Consumer Summary | Status |
|----------|-----------------|-----------------|--------|
| S01 → S02: AdvanceTurn/DelayTurn primitives | S01 provides unified AdvanceTurn(u32)/DelayTurn(u32) with cap/clamp; all callers migrated | S02 builds on S01's event bus and pipeline.rs infrastructure; M017 regression tests (status_slowed_delay, tempo_resistance, turn_advance_split) green | PASS |
| S01 → S03: AV applicator foundation | S01 provides unified AV applicators; Slowed migrated to DelayTurn{30} | S03 hop loop applies per-hop damage with same cap/clamp framework; parallel hoisting-before-loop pattern preserved | PASS |
| S02 → S03: resolve_targets + SlotIndex | S02 provides SlotIndex(u8) Component at spawn; pure resolve_targets() helper; pipeline fan-out over resolved target list | S03 T02 extends TargetShape enum (Bounce struct variant); T03 consumes apply_damage_only() and TargetableSnapshot; select_bounce_hop uses SlotIndex ordering | PASS |
| Cross-slice regression gate | S01: M017 regression tests green; S02: M017 tests confirmed green (S02 does not touch advance/delay paths) | S03: Full cargo test suite 201 tests, 0 failures across all integration and lib targets | PASS |
| Full-suite determinism | S01 CLI advance-delay-cap (exit 0, JSONL with cap visible); S02 CLI aoe-blast (10x identical stdout) | S03: 4 integration tests target_shape_bounce_chain; all verification_result: passed | PASS |

All boundary contracts honored. S01 → S02 → S03 compose cleanly end-to-end: the advance/delay primitives from S01 underpin the multi-target resolver in S02, which in turn provides the SlotIndex and resolve_targets infrastructure consumed by S03's Bounce hop loop.

## Requirement Coverage
No formal per-requirement IDs were advanced or invalidated in M018 (the milestone context notes requirements were to be populated in planning from `.gsd/REQUIREMENTS.md`). Coverage is assessed against the two core primitives and acceptance criteria stated in M018-CONTEXT.md.

| Requirement | Status | Evidence |
|-------------|--------|---------|
| Primitive 1: Time-manipulation split (AdvanceTurn/DelayTurn) | COVERED | S01: TurnAdvance(i32) replaced; AdvanceTurn(u32)/DelayTurn(u32) emitted with ±50% cap; TempoResistance curve applied; 554 tests green; CLI scenario logs per-application JSONL |
| Primitive 2: TargetShape resolver expansion (Blast, AoE, Bounce) | COVERED | S02: Blast + AllEnemies; S03: Bounce with BounceSelector/RepeatPolicy/DamageCurve; all via pure resolve_targets() / select_bounce_hop() |
| Acceptance: Advance/Delay JSONL logging step-by-step | COVERED | S01: CLI scenario logs kind, target, amount_pct_requested, amount_pct_capped, av_pre, av_delta, av_post |
| Acceptance: Bounce chain with KO mid-chain | COVERED | S03: TargetableSnapshot rebuilt each hop; KOs shrink candidate pool; integration test bounce_next_slot_no_repeat_falloff_ko_mid_chain passes |
| Acceptance: Deterministic slot_index tie-break | COVERED | S02: SlotIndex(u8) Component; slot_index_tiebreak test confirms per-team ranges {0,1,2}; JSONL 10x identical |
| Acceptance: Zero regressions on ~40 test binaries | COVERED | S01: 554 tests; S02/S03: 201 tests total; all M017 regression tests green |
| Acceptance: Lifecycle operations | N/A | M018 is pure-logic; no lifecycle ops by design |

All materially testable requirements COVERED. No requirements invalidated or re-scoped.

## Verification Class Compliance
| Class | Planned Check | Evidence | Verdict |
|-------|--------------|----------|---------|
| Contract | Integration tests in `tests/` cover advance/delay split, Blast/AoE/Bounce shapes, selectors, boundary cases with deterministic seeds | S01: `turn_advance_split.rs` (6/6 pass); S02: `target_shape_blast_spillover.rs`, `target_shape_aoe_all_order.rs`, `slot_index_tiebreak.rs` (6 tests pass); S03: `target_shape_bounce_chain.rs` (4/4 pass) | PASS |
| Integration | CLI scenario binary executes example skills per primitive; JSONL log shows target list, AV step-by-step, turn order stable across runs | S01: CLI `advance-delay-cap` (cap visible: 80→50; floor clamp visible: delta=0); S02: CLI `aoe-blast` deterministic across 10 invocations (25 lines identical); S03: Bounce hop loop in pipeline with per-hop damage | PASS |
| Operational | N/A — M018 is pure-logic, no lifecycle operations | Not planned; context explicitly excludes operational scope | N/A |
| UAT | Headless artifact-driven UAT; shell-scriptable checks via cargo test + cargo run CLI commands | S01 UAT: 6 numbered checks (legacy sweep, boundary tests, M017 regression, CLI output, full suite, windowed) all specified; S02 UAT: 7 numbered checks (new test binaries, no regressions, M017 tests, determinism gate, JSONL content, feature gate, fixture load) all specified; S03 UAT: smoke test + 4 test cases + edge cases specified | PASS |


## Verdict Rationale
All three independent reviewers returned PASS. M018 delivers its two foundational primitives — AdvanceTurn/DelayTurn split (S01) and TargetShape resolver expansion with Blast, AoE(All), and Bounce (S02/S03) — with comprehensive integration test coverage (201 tests, 0 failures), byte-for-byte deterministic CLI scenario output, and zero regressions on the M017 baseline. Cross-slice boundaries are cleanly honored: S01 primitives consumed by S02 and S03; S02 SlotIndex/resolve_targets foundation consumed by S03's Bounce hop loop. Known limitations (per-hop CombatEvent emission, OnActionFailed on pool exhaustion) are explicitly deferred and not blocking the milestone contract.
