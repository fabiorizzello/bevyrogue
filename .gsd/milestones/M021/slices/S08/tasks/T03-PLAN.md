---
estimated_steps: 10
estimated_files: 3
skills_used: []
---

# T03: Add Bouncing Fire Loop branch to baby_flame and register predicate+selector+hook

**Why:** The S08 success criterion requires 'Bouncing Fire OFF≡baseline' — proving that with talent rank 0 the intent stream from baby_flame is identical to the current no-loop timeline. This is the first production use of `BeatKind::Loop` in a real blueprint. It proves the Loop infrastructure from S02/S03 works in a real Digimon context and the gate mechanism cleanly gates off the branch.

**Do:**
1. Introduce a minimal `TalentRanks` Bevy Resource in `src/combat/blueprints/agumon/mod.rs` (or a shared location like `src/combat/api/talent.rs` if appropriate): a `HashMap<String, u8>` mapping talent keys to ranks. Default all to 0.
2. Register predicate `"agumon/has_bouncing_fire"` in ExtRegistries: reads `TalentRanks` from `ctx.world`, returns `ranks.get("agumon::bouncing_fire").copied().unwrap_or(0) >= 1`.
3. Register selector `"agumon/bounce_pick_next"`: picks next alive enemy not in `ctx.cast_hit_set`. Returns empty vec if none available (terminates loop).
4. Register hook `"agumon/on_bounce_hop"`: enqueues `Intent::DealDamage` with half the base amount (9 for baby_flame's 18), adds target to `cast_hit_set`.
5. Extend `baby_flame` timeline in `assets/data/skills.ron`: add a `bounce_loop` beat with `kind: Loop`, body containing a `bounce_hop` beat using hook `"agumon/on_bounce_hop"` and selector `"agumon/bounce_pick_next"`, `exit_when: "agumon/bounce_exit"` (predicate returning true when selector returns empty). Add edge `impact_signal → bounce_loop` with gate `"agumon/has_bouncing_fire"` and fallback edge `impact_signal → cast_end` (a terminal beat, or simply let impact_signal be the last beat with the gate controlling entry to the loop).
6. Register predicate `"agumon/bounce_exit"` that returns true when no more valid targets exist (selector would return empty).
7. Ensure `validate_timeline_refs` covers the new beats/edges.

**Done when:** `cargo test bouncing_fire` passes (test written in T04); `cargo check` passes with the new timeline structure.

## Inputs

- `src/combat/blueprints/agumon/mod.rs`
- `src/combat/api/registry.rs`
- `src/combat/api/runner.rs`
- `src/combat/api/runner_common.rs`
- `assets/data/skills.ron`
- `src/combat/blueprints/twin_core/mod.rs`

## Expected Output

- `assets/data/skills.ron`
- `src/combat/blueprints/agumon/mod.rs`

## Verification

cargo check
