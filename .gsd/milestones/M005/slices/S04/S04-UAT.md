# S04: S04: bevy_enoki integration spike (one effect) — UAT

**Milestone:** M005
**Written:** 2026-05-26T09:33:18.363Z

# S04 UAT: bevy_enoki integration spike (one effect)

## UAT Type
**Mixed contract + integration.** Mechanical proof is automated (dep-gating test, windowed build, source-contract tests, windowed_only suite). Visual proof requires a human in `cargo winx` — deferred to S05/K001 sign-off.

## Preconditions
- Checkout is on the commit that completed T03.
- `cargo` resolves cleanly (lockfile up to date after T01 online `cargo tree --features windowed`).
- A display is available for `cargo winx` steps (GPU context required for enoki rendering).

## Automated Verification Steps

1. **Dep-gating (R005/R016)**
   - Run: `cargo test --test dependency_gating`
   - Expected: `2 passed` — `bevy_enoki_absent_from_headless_graph` and `bevy_enoki_present_in_windowed_graph` both `ok`.
   - Outcome: PASS (verified at slice close-out).

2. **Windowed build**
   - Run: `cargo build --features windowed`
   - Expected: exits 0 with no errors or warnings about enoki types.
   - Outcome: PASS (verified at slice close-out).

3. **Windowed contract + parse tests**
   - Run: `cargo test --features windowed --test windowed_only`
   - Expected: 46+ passed, 0 failed. Key tests: `impact_effect_parses_into_enoki_schema`, `enoki_plugin_is_registered`, `spawn_effect_by_id_enoki_branch_spawns_correct_components`, `spawn_effect_by_id_quad_loop_unchanged_for_other_effects`.
   - Outcome: PASS (verified at slice close-out).

4. **Full headless suite**
   - Run: `cargo test`
   - Expected: 51+ passed, 0 failed. No new headless failures.
   - Outcome: PASS (verified at slice close-out).

## Manual UAT Steps (require `cargo winx`)

5. **baby_flame.impact renders through enoki**
   - Launch: `cargo run --features windowed`
   - Action: Trigger Agumon's Baby Flame skill so it reaches impact.
   - Expected: a short burst of fast-decelerating ember particles (yellow-white core fading to transparent orange-red) appears at the impact anchor, then self-despawns. No flat quad placeholder visible for this effect.
   - Outcome: **Deferred to S05 / K001 visual sign-off.**

6. **Other effects unaffected**
   - Action: Trigger Sharp Claws and Baby Burner; observe both combatants.
   - Expected: Sharp Claws and Baby Burner VFX still render as before (quad path); no visual regression on hurt flash, death fade, or damage numbers.
   - Outcome: **Deferred to S05 / K001.**

7. **Fail-loud diagnostic on missing asset**
   - Precondition: temporarily rename `baby_flame_impact.particle.ron` to a wrong path.
   - Action: launch `cargo winx` and check console output target `windowed.agumon_playback`.
   - Expected: a `WARN` message names "baby_flame.impact" with LoadState::Failed; no panic or silent drop.
   - Outcome: **Deferred (manual destructive test).**

## Edge Cases
- If `AgumonEnokiVfx` resource is absent (e.g. missing asset at startup), `spawn_effect_by_id` falls through to the quad path for baby_flame.impact — no crash.
- The kernel/FSM cue/barrier control flow (`request_release`, `fire_kernel_cue`) is unchanged; enoki spawn does not block or delay combat progression.

## Not Proven By This UAT
- Visual quality / artistic look of the enoki burst — reserved for S05 K001 sign-off.
- All three Agumon skills migrated to enoki — that is S05's goal.
- Performance / frame-time impact of the enoki particle system — deferred to S05 soak.

