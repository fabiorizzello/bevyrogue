# S03: Mode parity (DryRun ≡ Execute ≡ Preview) + Two-clock invariant — UAT

**Milestone:** M021
**Written:** 2026-05-15T09:05:57.939Z

# UAT — S03: Mode parity (DryRun ≡ Execute ≡ Preview) + Two-clock invariant

**UAT Type:** Automated integration test suite (deterministic, headless, no wall-clock, no unseeded RNG)

## Preconditions

- Branch: `milestone/M021`
- `cargo build` succeeds
- All four S03 tasks complete (T01–T04)

## Test Steps

1. **Execute≡DryRun≡Preview on finisher branch**
   ```
   cargo test --test timeline_mode_parity mode_parity_execute_dryrun_preview_match_on_finisher_branch
   ```
   **Expected:** 1 passed. The normalize() stream from Execute, DryRun, and Preview are equal, and the stream contains a DealDamage(50) + DealDamage(200) finisher sequence.

2. **Execute≡DryRun≡Preview on normal branch**
   ```
   cargo test --test timeline_mode_parity mode_parity_execute_dryrun_preview_match_on_normal_branch
   ```
   **Expected:** 1 passed. Stream from all three modes contains DealDamage(50) + DealDamage(100). Proves the predicate routes both ways (live, not dead).

3. **HeadlessAuto≡Windowed end-of-cast parity**
   ```
   cargo test --test timeline_two_clock_parity
   ```
   **Expected:** 1 passed. `awaiting_cue_count >= 1` (Windowed stall is real), stall located at the "cast" beat (presentation beat), and normalize(headless_pending) == normalize(windowed_pending).

4. **Circuit-breaker Halt at MAX_HOPS=256**
   ```
   cargo test --test timeline_circuit_breaker
   ```
   **Expected:** 1 passed. `outcome == StepOutcome::Halted`; pending contains exactly 256 DealDamage intents; draining via intent_applier yields 256 OnDamageDealt events without panic.

5. **Full regression suite**
   ```
   cargo test
   ```
   **Expected:** exit 0, all tests pass (no S01/S02 regressions, no inline unit test regressions).

6. **Headless compile check**
   ```
   cargo check
   ```
   **Expected:** `Finished` with 0 new warnings introduced by S03.

7. **Windowed compile check**
   ```
   cargo check --features windowed
   ```
   **Expected:** `Finished` with 0 new warnings introduced by S03.

8. **P001 kernel purity guard**
   ```
   rg "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/api/
   ```
   **Expected:** 0 matches.

## Edge Cases Covered

- **Both predicate branches live:** T02 spawns world in two distinct HP states so edge-A (finisher) and edge-B (normal/fallback) are each exercised in separate test functions.
- **Windowed stall uniqueness:** T03 asserts the stall occurred exactly at the Presentation-bearing beat, not bypassed by HeadlessAuto short-circuit.
- **Circuit-breaker count boundary:** T04 documents and asserts that exactly 256 (not 257) DealDamage intents accumulate — body fires for hop_index 0..=255, breaker trips at hop_index==256 before executing the 257th iteration.

## Not Proven By This UAT

- Live-pipeline wiring of Clock into turn_system/pipeline (S04 scope).
- `Intent::Reject` on Halt (deferred per D006 — keeps stream clock/mode-independent).
- `bevy::log::warn!` log capture (T01 adds the signal; capturing log output in tests is out of scope — StepOutcome::Halted is the integration contract).
- Multi-level nested Loop circuit-breaker behavior (single-level loop only in S03).
- Windowed UI rendering / animation playback (full Windowed runtime is S04).
