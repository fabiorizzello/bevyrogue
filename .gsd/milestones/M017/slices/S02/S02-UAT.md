# S02: §H.1 StatusBag multi-instance + refresh_max_dur + BuffKind cleanse — UAT

**Milestone:** M017
**Written:** 2026-05-13T08:52:33.246Z

# S02 UAT — §H.1 StatusBag policy

## Scenario 1: refresh_max_dur
- Apply Heated(dur=2), re-apply Heated(dur=1) → remaining dur stays 2
- Test: `status_refresh_max_dur` ✅

## Scenario 2: multi-kind coexistence
- Apply Heated and Chilled to same unit → both present independently
- Test: `status_multi_kind_coexist` ✅

## Scenario 3: cleanse policy
- Apply Weakened (Debuff) + Blessed (Buff) → cleanse removes Weakened, Blessed survives
- Test: `status_cleanse_policy` ✅

## Scenario 4: fresh apply accuracy
- Apply status with correct initial duration
- Test: `status_accuracy` ✅

## Scenario 5: combat coherence
- Full combat run with status effects — no panics, correct tick/expiration
- Test: `combat_coherence` ✅

## Smoke
- `cargo run` headless exit 0 ✅

## Grep guard
- No `Vec<StatusEffect>` in src/ or tests/ ✅
