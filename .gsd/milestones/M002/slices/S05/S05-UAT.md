# S05: Full kit: Agumon vs Agumon dummy — UAT

**Milestone:** M002
**Written:** 2026-05-21T11:12:23.410Z

# S05 UAT: Full kit — Agumon vs Agumon dummy

## UAT Type
Manual windowed session (environment-limited: requires a Linux display session; cannot be executed in headless CI or gsd_exec).

## Preconditions
1. A Linux desktop session with `DISPLAY` or `WAYLAND_DISPLAY` set.
2. The repo is on the `master` branch with all S05 changes present.
3. `cargo build --features windowed --bin bevyrogue` completes successfully (already verified).

## Steps and Expected Outcomes

### 1. Launch the windowed session
```
cargo run --features windowed --bin bevyrogue
```
**Expected:** Window opens. Two Agumon sprites appear on screen — ally (left, x≈−200) and enemy/dummy (right, x≈+200, horizontally flipped). Both sprites show an HP bar above them. No panic in the console. Egui combat panel renders with at least Basic/Skill/Ultimate buttons.

### 2. Verify HP bars display correct initial state
**Expected:** Both HP bars are full. The roster panel (or HUD) shows Agumon HP and Agumon Dummy HP at max values.

### 3. Basic action — Sharp Claws
Click the **Basic** button in the egui combat panel.
**Expected:**
- Sharp Claws animation plays on the ally sprite (windup → strike → recovery, ~S02 path).
- Enemy dummy HP bar drops by ~5–6 HP (exact value depends on stats).
- Enemy sprite briefly tints/blinks (TargetHurtState countdown active for HURT_FRAMES=12 frames).
- No panic. Phase strip updates in the combat panel.

### 4. Skill action — Baby Flame (multi-hop)
Click the **Skill** button.
**Expected:**
- Baby Flame animation plays: cast → per-hop impact (N visible impact beats matching the kernel `hop_index` loop count; default bouncing_fire rank ≥ 1 means at least 2 hops).
- Each hop produces a visible iteration of the impact animation (`baby_flame_impact` self-transition fires per KernelCue).
- Enemy dummy HP drops by the multi-hop damage total.
- Enemy sprite blinks/tints each time a hop lands (`OnHitTaken` event per hop).
- No panic. No non-determinism.

### 5. Ultimate action — Baby Burner (with reactive detonate)
First, ensure the enemy dummy is Heated (attack with Skill once or twice; Baby Flame applies thermal_spark which stacks Heated). Then click **Ultimate**.
**Expected:**
- Baby Burner animation plays: windup (`baby_burner_charge`) → ReleaseKernel → impact (`baby_burner_launch` frames) → recovery (`baby_burner_recovery`).
- Impact beat emits damage, break, and thermal_spark BlueprintSignal.
- If enemy HP reaches 0 while Heated, the reactive detonate flash (S04 chip) triggers on adjacent alive enemies.
- **Twin Core badge** appears in the egui panel after the Ultimate resolves (TwinCoreBadgeState primed for 60 frames by any twin_core blueprint signal; visible as the chip label).
- Enemy dummy HP reaches 0 and is marked defeated.
- No panic. Console shows no unexpected errors.

### 6. Twin Core badge countdown
After the badge appears, wait ~2 seconds (60 frames at 30fps).
**Expected:** Badge disappears after the countdown elapses (TwinCoreBadgeState cleared at zero).

### 7. Stability soak (optional, recommended)
Leave the session running for ~30 seconds with repeated Basic/Skill presses.
**Expected:** No panic, stable FPS, no memory growth visible in top/htop.

## Edge Cases
- If enemy dummy HP reaches 0 before a Heated stack: reactive detonate should not trigger (no adjacent alive enemy with Heated). Verify graceful completion without crash.
- Clicking Ultimate with insufficient SP should produce a graceful failure message, not a panic.
- Clicking buttons while animation is mid-flight (windowed suspension) should be a no-op or queue, not crash.

## Not Proven By This UAT
- Render-side sprite tint (TargetHurtState → wgpu tint color): TargetHurtState resource is fully wired and tested headlessly, but the visual tint system drawing from it in `render.rs` was deferred. A user may observe no visible color change on hit even though the resource is correct.
- Render-side Twin Core chip drawing in `render.rs`: TwinCoreBadgeState resource and chip helpers are wired and tested headlessly, but the actual egui rendering call was deferred. Badge may not appear visually in the panel even though the resource counts down correctly.
- Hot-reload mid-skill (S06 scope).
- Multi-unit party (M003+ scope).
- Repomix architectural review (S06 scope).
