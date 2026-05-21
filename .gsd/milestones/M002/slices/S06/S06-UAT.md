# S06: Windowed smoke end-to-end + repomix review gate — UAT

**Milestone:** M002
**Written:** 2026-05-21T12:06:44.922Z

# S06 UAT: Windowed Smoke End-to-End + Repomix Review Gate

## UAT Type
Operational — live windowed session (manual, user-executed per K001) + automated regression matrix.

## Preconditions
1. Repository is at a clean state (no uncommitted source changes that would affect the build).
2. `cargo` toolchain present and `windowed` feature builds successfully (`cargo build --features windowed` exits 0).
3. `assets/data/digimon/agumon/skills.ron` is the canonical version from repo (no local edits).
4. Terminal capable of capturing stdout/stderr (the capture helper handles this).

---

## Part A: Automated Regression Matrix (already executed in T03)

| Check | Command | Result |
|-------|---------|--------|
| M001 headless suite | `cargo test` | PASS |
| Windowed integration suite | `cargo test --features windowed --test windowed_only` | PASS |
| Headless-only build | `cargo build --no-default-features` | PASS |
| Windowed build | `cargo build --features windowed` | PASS |
| R005 dep-gating | grep winit/wgpu/egui outside windowed | PASS (doc-comment only) |
| R006 repo hygiene | find . -maxdepth 1 -name '*.md' | PASS (none) |
| I3 two-clock parity | timeline parity tests in timeline harness | PASS |

**This part is complete and evidence is in regression-matrix.md.**

---

## Part B: Windowed Smoke UAT (user-executed, K001)

### Steps

1. **Launch via capture helper:**
   ```bash
   bash scripts/capture-windowed-smoke.sh
   ```
   This starts `cargo run --features windowed` and tees output into `.gsd/milestones/M002/slices/S06/uat-evidence/windowed-smoke-<timestamp>.log`.

2. **Observe startup:** Window opens; Agumon idle animation cycles. Phase strip shows `Intro` or `PlayerTurn`. No panic in console.

3. **Execute Sharp Claws (Player turn):**
   - Select Sharp Claws. Observe: windup → strike → recovery frames on screen; damage number floats on impact frame; dummy HP bar decrements; §9 phase strip advances.
   - Expected: `ReleaseKernelCue` hop_index = 0; no VFX entity accumulation visible across subsequent turns.

4. **Execute Bouncing Fire (Player turn, requires charges):**
   - Select Bouncing Fire. Observe: multi-hit loop visible; HP bar decrements for each hit; floating damage per hop; kernel hop count matches visible hit count.
   - Expected: hop_index increments for each bounce.

5. **Pre-heat dummy for Baby Burner reactive detonate:**
   - Allow dummy to receive enough hits that the reactive condition is satisfied (per runbook step — dummy must accumulate 3+ hits under Baby Burner Ultimate status).

6. **Execute Baby Burner Ultimate (Player turn):**
   - Select Baby Burner. Observe: charged skill activation; reactive detonate triggers flash VFX on the impact frame; dummy HP drops significantly; no duplicate VFX entities accumulating.
   - Expected: flash VFX fires once per reactive hit; deterministic outcome.

7. **Hot-reload mid-skill:**
   - While a skill animation is in progress, touch `assets/data/digimon/agumon/skills.ron` (e.g., `touch assets/data/digimon/agumon/skills.ron`).
   - Expected: asset reload event fires; world state (HP, turn, phase) is not corrupted; no panic; skill completes normally on its pre-reload data.

8. **Observe TwinCore badge and HUD:**
   - Verify TwinCoreBadge is displayed; HP bars reflect current values; no stale floaters accumulate.

9. **Session end / close window:**
   - Close window or Ctrl+C. Verify capture script exits non-zero only on panic (should be 0 for clean close).

### Expected Pass Signals
- No panic stacktrace in console or log.
- Stable FPS throughout (no perceptible stutter; no unbounded VFX entity growth visible).
- §9 phase strip updates correctly across all skill phases.
- Floating damage numbers appear on impact frames and fade cleanly.
- Hot-reload leaves world state intact (HP, phase, turn counts unchanged by asset reload).
- Log file written to `uat-evidence/` with non-empty content.

### Expected Failure Signals
- `thread 'main' panicked` in log → BUILD FAIL.
- VFX entities accumulating across turns without cleanup → VFX memory leak.
- Phase strip stuck or repeating states → event-routing regression.
- HP does not decrement after damage skill → kernel cue regression.
- Crash or corruption after hot-reload → asset reload guard regression.

### Edge Cases
- Baby Burner reactive detonate fires when dummy accumulation condition not met → verify detonate guard in Baby Burner passive blueprint.
- Multi-hit Bouncing Fire with 0 remaining charges → verify charge-guard does not allow execution.

---

## Not Proven By This UAT
- Performance under sustained load (> 60 seconds of combat).
- Correctness of exact damage numbers (covered by headless kernel tests, not visual inspection).
- Network / multiplayer correctness (out of M002 scope).
- RON editor round-trip (M003+ scope).
- Digimon other than Agumon (M003+ scope).
