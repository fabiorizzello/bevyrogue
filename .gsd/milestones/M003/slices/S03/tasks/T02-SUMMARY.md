---
id: T02
parent: S03
milestone: M003
key_files:
  - src/windowed/render.rs
key_decisions:
  - Shared a per-frame PendingAnimationTicks resource so sprite playback and presentation-only VFX consume the same deterministic animation budget.
  - Kept the windowed particle entities asset-free and gameplay-free by spawning colored Sprite quads from VfxSpawnDescriptor plus tiny target/source helper components.
duration: 
verification_result: passed
completed_at: 2026-05-22T13:27:13.033Z
blocker_discovered: false
---

# T02: Wired Agumon node-entry SpawnParticle commands into deterministic windowed Sprite-quad VFX with caster-only spawning, tick-driven motion/TTL, and unit coverage.

**Wired Agumon node-entry SpawnParticle commands into deterministic windowed Sprite-quad VFX with caster-only spawning, tick-driven motion/TTL, and unit coverage.**

## What Happened

Extended the windowed Agumon renderer to consume node on_enter SpawnParticle commands instead of treating them as inert data. In src/windowed/render.rs I added VfxParticle plus small target/source helper components, constants for particle TTL/size, pure helper seams (entered_node, TTL decrement, resolve_vfx_spawn_xy), and a spawn_vfx_particle path that turns a VfxSpawnDescriptor into a short-lived colored Sprite quad with no physics or gameplay payload. I changed the presentation loop to sample animation time once into PendingAnimationTicks, then use that shared deterministic tick budget for both sprite playback and VFX advancement. advance_agumon_presentation now snapshots the pre-sync node, detects node entry exactly once, resolves the nearest non-caster Agumon sprite as target, gates spawning to the caster-presenting sprite, iterates entered-node on_enter commands, and traces/debugs spawn behavior. I also added advance_vfx_particles to apply tick-driven TTL decay plus presentation-only motion toward the resolved target for FollowTarget/ArcToTarget, with trace logging on despawn. Finally, I extended render.rs unit tests to cover entered_node behavior, TTL countdown saturation to zero, and locus resolution across all supported variants.

## Verification

Verified the windowed build and binary test target after the final visibility cleanup. cargo build --features windowed completed successfully with no warning regression, and cargo test --features windowed --bin bevyrogue passed all 17 tests including the new render.rs helper coverage for node-entry detection, TTL countdown, and VFX locus resolution.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features windowed` | 0 | ✅ pass | 1138ms |
| 2 | `cargo test --features windowed --bin bevyrogue` | 0 | ✅ pass | 1033ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/windowed/render.rs`
