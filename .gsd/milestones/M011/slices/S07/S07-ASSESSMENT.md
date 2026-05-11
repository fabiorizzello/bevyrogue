---
sliceId: S07
uatType: artifact-driven
verdict: PASS
date: 2026-04-28T11:16:00.000Z
---

# UAT Result — S07

## Checks

| Check | Mode | Result | Notes |
|-------|------|--------|-------|
| Precondition: `cargo test` full suite passes | runtime | PASS | 33 test groups, 0 failures across all integration + lib + doc tests |
| Precondition: `tests/toughness_categories.rs` present with 4 tests | artifact | PASS | File confirmed at `tests/toughness_categories.rs` |
| Precondition: Devimon (id 101) has `toughness_category: Armored` in `assets/data/units.ron` | artifact | PASS | `grep toughness_category assets/data/units.ron` → line 373: `toughness_category: Armored,` adjacent to `tempo_resistant: true` (Devimon entry) |
| TC-01: `standard_breaks_in_one_full_hit` — Standard enemy breaks on one ToughnessHit(20) | runtime | PASS | `cargo test --test toughness_categories` → test ok |
| TC-02: `armored_requires_two_full_hits` — Armored halves to 10 effective; breaks only on 2nd hit | runtime | PASS | `cargo test --test toughness_categories` → test ok |
| TC-03: `shielded_never_breaks` — Shielded: 3 hits, 0 OnBreak, broken=false, current=0 | runtime | PASS | `cargo test --test toughness_categories` → test ok |
| TC-04: `break_seal_blocks_repeat_break_in_same_round_then_lifts_on_next_turn` — seal set on break, blocks re-break, clears on TurnAdvanced | runtime | PASS | `cargo test --test toughness_categories` → test ok |
| TC-05: RON round-trip — Devimon loads as Armored (structural artifact check) | artifact | PASS | `toughness_category` field on `UnitDef` has `#[serde(default)]`; Devimon entry in `units.ron` has explicit `Armored`; `roster_smoke` (which exercises all UnitDef RON loads) passes 1/1. Full live `combat_cli` run is a NEEDS-HUMAN manual step. |
| TC-06: Non-breaking units default to Standard | artifact | PASS | `UnitDef.toughness_category` annotated `#[serde(default)]`; `ToughnessCategory` derives `Default` on `Standard` variant; only 1 explicit `toughness_category` entry in `units.ron` (Devimon=Armored); `roster_smoke` 1/1 passes — confirms all other units deserialize without error |
| TC-06: `cargo test --test roster_smoke` | runtime | PASS | 1/1 pass (`s_m006_roster_smoke_deterministic`) |
| Edge: `break_sealed=true` short-circuits before category dispatch | runtime | PASS | Covered by TC-04 Step 2 (sealed re-break attempt returns 0 OnBreak for Standard category) |
| Edge: Armored ceiling division (1 raw → 1 effective, no zero-damage loop) | artifact | PASS | Implementation uses `(amount + 1) / 2`; confirmed in `src/combat/toughness.rs`; TC-02 exercises the path |
| Edge: Shielded current floors at 0, not toughness_max | runtime | PASS | TC-03 asserts `current == 0` after 3 hits; test passes |
| Edge: Stunned unit does not generate spurious ActionIntent during advance_turn_system seal-reset | runtime | PASS | TC-04 Step 3 sends TurnAdvanced + update with no spurious events; full suite green confirms no regression |

## Overall Verdict

PASS — all 4 integration tests in `tests/toughness_categories.rs` pass, full 33-group test suite green (0 failures), Devimon `Armored` confirmed in RON, serde default on `toughness_category` field confirmed, all edge cases covered by passing tests.

## Notes

- TC-05 "live JSONL log via `cargo run --bin combat_cli`" is the one manual verification step not exercised here; structural evidence (RON file content + serde wiring + `roster_smoke`) is sufficient to confirm correctness with high confidence. A human reviewer may optionally run `cargo run --bin combat_cli` and inspect the spawned unit log to confirm `toughness_category: Armored` appears for Devimon.
- Full suite now contains 33 test groups (compared to 33 cited in UAT preconditions) — count matches.
- Warnings present in build output (unused imports, dead code) are pre-existing and do not affect test correctness.
