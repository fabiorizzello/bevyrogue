---
id: T03
parent: S02
milestone: M003
key_files:
  - src/windowed/render.rs
  - src/windowed/mod.rs
  - assets/digimon/agumon/anim_graph.ron
key_decisions:
  - Auto-release narrowed to genuinely unbridged skills via should_auto_release_unbridged() = skill_start_node().is_none(); sharp_claws/baby_flame/agumon_ult release on their authored ReleaseKernel frame.
  - Bounce is gameplay/VFX only and must NOT enter the animation FSM (user directive). Removed the baby_flame_impact KernelCue priority-10 self-loop edge from anim_graph.ron so the node plays exactly once; baby_flame_impact now has a single TimeInNode edge to baby_flame_recover.
  - KernelCue signal de-overloaded: now means only 'advance to next node'. fire_kernel_cue() restored to unconditional (the interim kernel_cue_advances_node() workaround was deleted) since no self-loop edge remains to arm spuriously.
  - Dropped hop_index from ReleaseFrameKey and removed the agumon::bouncing_fire talent grant from the windowed bootstrap — both existed solely to serve bounce-hop animation re-entry, which no longer occurs.
duration: 
verification_result: passed
completed_at: 2026-05-22T12:08:28.730Z
blocker_discovered: false
---

# T03: Wired Baby Flame and Baby Burner to release on their rendered ReleaseKernel frame (auto-release now fallback-only); a manual-K001 iteration then removed the bounce-to-FSM coupling, making Baby Flame strictly linear.

**Wired Baby Flame and Baby Burner to release on their rendered ReleaseKernel frame (auto-release now fallback-only); a manual-K001 iteration then removed the bounce-to-FSM coupling, making Baby Flame strictly linear.**

## What Happened

Built on T02's seams. The core T03 change in `src/windowed/render.rs` was two-fold: (1) the pre-tick guard that previously auto-released any non-SharpClaws barrier now uses `should_auto_release_unbridged()` (= `skill_start_node().is_none()`), so only genuinely unbridged skills take the fallback while sharp_claws/baby_flame/agumon_ult release on their authored ReleaseKernel cue; (2) the pending-release path drives the multi-barrier walk via `fire_kernel_cue()` (baby_burner charge->launch->recovery; sharp_claws strike->recover) and records a ReleaseFrameKey for dedup.

POST-COMPLETION USER ITERATION (visual K001 feedback — Baby Flame still showed extra windup): root cause was the `baby_flame_impact` node carrying a priority-10 `KernelCue` self-loop edge in `assets/digimon/agumon/anim_graph.ron`, authored to drive bounce-hop re-entry. Unconditionally firing the cue made the impact node replay once even on a single linear hop. The user clarified that bounce is purely gameplay/VFX and must NOT touch the animation FSM. Final resolution:
- Removed the `baby_flame_impact --KernelCue(prio 10)--> baby_flame_impact` self-loop from anim_graph.ron. `baby_flame_impact` now has a single outgoing edge `--TimeInNode--> baby_flame_recover`, so the path `cast -> impact -> recover` is linear by construction and the impact node plays exactly once.
- Removed the bounce-only scaffolding from render.rs: deleted the interim `kernel_cue_advances_node()` workaround (`fire_kernel_cue()` is unconditional again — safe because no self-loop edge remains to spuriously arm), and dropped `hop_index` from `ReleaseFrameKey` (the `(cue_id, node, local_frame)` key suffices since Baby Burner's three barriers carry distinct cue_ids). Also dropped the `agumon::bouncing_fire` talent grant from the windowed demo bootstrap in `src/windowed/mod.rs`.
- Updated render.rs unit tests: renamed one, removed the two bounce-specific dedup tests.

Net effect: the `KernelCue` signal is no longer overloaded (it now means only "advance to the next node"); the combat-side bounce path (pipeline/paths/bounce, target_shape, skills.ron) is untouched — only the bounce->animation-FSM coupling was removed. Combat resolution (barrier release timing, damage-on-impact) is unchanged for all three bridged skills.

## Verification

Ran the full T03 verification gate plus the post-iteration reverification after removing the self-loop and bounce scaffolding. All exit 0: cargo test --test animation (61/61, Baby Flame/Baby Burner atlas-parity drives hold against the authored node union [60,77]); cargo build --features windowed clean; cargo test --features windowed plus --bin bevyrogue render (25 + 10 windowed/render unit tests green, including the narrowed auto-release-fallback and release-bridge tests); cargo test (all headless harnesses, 0 failed, no windowed-gated dep leak per R002/R005). Visual smoothness + damage-on-impact for the linear Baby Flame / Baby Burner playback is deferred to manual K001 sign-off — cargo winx was not launched from auto-mode.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test animation` | 0 | pass | 0ms |
| 2 | `cargo build --features windowed` | 0 | pass | 0ms |
| 3 | `cargo test --features windowed (+ --bin bevyrogue render)` | 0 | pass | 0ms |
| 4 | `cargo test` | 0 | pass | 0ms |

## Deviations

The committed T03 plan delivered the bridge with bounce-hop support (hop-aware ReleaseFrameKey + a KernelCue self-loop for impact re-entry). Post-completion manual K001 review revealed this produced visible extra windup on linear Baby Flame, and the user clarified bounce is VFX-only. The bounce-to-FSM coupling was therefore removed rather than kept — a directed reversal of part of the original plan, not an implementation bug.

## Known Issues

Multi-hop bounce presentation (animating each VFX bounce as a distinct sprite re-entry) is no longer wired in the windowed view; it was only ever active under the now-removed agumon::bouncing_fire demo toggle. If it returns to scope it should be driven by the arriving next-hop barrier, not a self-loop edge. No hurt/flinch animation plays on damaged targets (pre-existing: anim_graph.ron has no hurt node and the bridge only animates the caster).

## Files Created/Modified

- `src/windowed/render.rs`
- `src/windowed/mod.rs`
- `assets/digimon/agumon/anim_graph.ron`
