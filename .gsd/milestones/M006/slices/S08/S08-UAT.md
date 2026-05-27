# S08: Register Renamon diamond_storm_leaf cue, Agumon cast proof, spawn-miss diagnostics — UAT

**Milestone:** M006
**Written:** 2026-05-27T11:23:24.623Z

# S08 UAT

## UAT Type
Headless structural + K001 manual windowed sign-off

## Preconditions
- `cargo build --features windowed` succeeds
- `cargo test` and `cargo test --features windowed` both exit 0
- `assets/digimon/renamon/diamond_storm_leaf.particle.ron` exists
- Renamon is registered via the per-species `register(app)` pattern (no core file edits)

## Steps and Expected Outcomes

### Headless (automated)
1. Run `cargo test --features windowed --test windowed_only`
   - **Expected:** 75 passed, 0 failed; includes `renamon_cast_cue_maps_to_registered_effect`, `agumon_cast_cue_maps_to_registered_effect`, `renamon_reactions_use_shared_engine_defaults`
2. Run `cargo test`
   - **Expected:** all suites pass, 0 failures

### Manual windowed sign-off (K001)
3. Launch `cargo run --features windowed` (winx alias)
   - **Expected:** Renamon idles with sprite present (S06 proof)
4. Trigger a Renamon cast (diamond_storm skill)
   - **Expected:** diamond_storm_leaf particle effect emits at cast site; no warn-once log line appears
5. Trigger an Agumon cast (Baby Flame or Baby Burner)
   - **Expected:** Agumon's registered enoki effect emits normally; no regression
6. (Diagnostic check) Temporarily remove a cue registration and trigger cast
   - **Expected:** exactly one `[WARN] cast cue spawned no particle — cue id unregistered…` line per cue id in stdout; no repeat on subsequent frames

## Edge Cases
- A `SpawnParticle` cue whose effect id is registered in `OnEnterEffectRegistry` but whose `.particle.ron` path is absent from `EnokiVfxRegistry` should also trigger the warn-once path (covered by structural test setup)
- Renamon hurt/death reactions use engine defaults (`hurt`, `death` nodes); no species-specific override needed — `renamon_reactions_use_shared_engine_defaults` asserts stance.ron conformance

## Not Proven By This UAT
- Visual quality of the diamond_storm_leaf particle effect (frame budget, density, color) — requires K001 human sign-off
- Camera-shake cue behavior during Renamon cast (not authored for Renamon in S08)
- Performance under many simultaneous cue-driven effects
