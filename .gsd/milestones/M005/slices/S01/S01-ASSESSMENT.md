---
sliceId: S01
uatType: artifact-driven
verdict: PASS
date: 2026-05-26T00:00:00.000Z
---

# UAT Result — S01: Hurt-on-hit reaction

## Checks

| Check | Mode | Result | Notes |
|-------|------|--------|-------|
| Headless animation test suite (119 tests) | runtime | PASS | `cargo test --test animation` → 119 passed, 0 failed |
| All 4 stance_reaction_mapping tests present and passing | runtime | PASS | `hit_maps_to_hurt_node`, `death_maps_to_death_node`, `death_takes_precedence_over_hurt_in_batch`, `non_reaction_kinds_and_empty_batch_map_to_none` — all at tests/animation/stance_reaction_mapping.rs:12,22,35,47 |
| Full windowed test suite (33 tests) | runtime | PASS | `cargo test --features windowed` → 33 passed, 0 failed |
| `drive_hurt_reactions` function exists and is wired | artifact | PASS | render.rs:907; registered `.after(spawn_unit_sprites).after(resolve_action_system).before(advance_agumon_presentation)` at lines 312-315 |
| Dedup guard: HashSet<UnitId> deduplications strikes | artifact | PASS | `let struck: HashSet<UnitId>` at render.rs:916 — unit struck twice in one window flinches once |
| Death events filtered out (deferred to S02) | artifact | PASS | `filter(|event| stance_reaction_for(&event.kind) == Some(StanceReaction::Hurt))` at render.rs:918 — Death events never reach the hurt seeding path |
| Mid-cast protection: hurt skipped if sprite not Idle | artifact | PASS | `if !matches!(sprite.mode, AgumonPlaybackMode::Idle)` at render.rs:937 — non-Idle sprites skipped with trace log at line 942 |
| Hurt node seeded via `drive_stance_reaction` | artifact | PASS | `sprite.drive_stance_reaction(hurt_node.clone(), stance_graph.clone())` at render.rs:948 |
| hurt→idle TimeInNode transition in stance.ron | artifact | PASS | `assets/digimon/agumon/stance.ron:22`: `(from: "hurt", to: Node("idle"), when: TimeInNode)` — authored return to idle after hurt frames |
| Hurt frames are 46–52 per clip.ron and stance.ron | artifact | PASS | `assets/digimon/agumon/clip.ron`: `"hurt": (start: 46, end: 52)`; `assets/digimon/agumon/stance.ron`: `"hurt": (frames: (46, 52))` |
| `reaction.rs` has no windowed symbol leaks (R002/R005) | artifact | PASS | Grep for wgpu/winit/egui/bevy_render/cfg(feature in `src/animation/reaction.rs` returned no matches — dep boundary clean |
| `stance_reaction_for` explicit match (no catch-all) | artifact | PASS | reaction.rs:40-43: `OnHitTaken → Hurt`, `UnitDied → Death`, all other variants enumerated — future variants force compile error |
| `resolve_stance_reaction` death-precedence logic | artifact | PASS | reaction.rs:83-94: returns Death on first Death, else Hurt if any Hurt, else None |
| `StanceReaction::stance_node()` returns correct node IDs | artifact | PASS | reaction.rs:29-30: `Hurt → NodeId("hurt")`, `Death → NodeId("death")` — matches authored stance.ron node names |
| Trace logging on both reaction-driven and skip paths | artifact | PASS | render.rs:942 (mid-cast skip trace) and render.rs:953 (hurt-driven trace) — target: "windowed.agumon_playback" |
| Visible flinch in `cargo winx` (frames 46–52 → idle) — K001 | human-follow-up | NEEDS-HUMAN | Auto-mode cannot launch the windowed binary. Human reviewer must: (1) run `cargo winx`, (2) trigger Sharp Claws, (3) confirm opponent sprite transitions into hurt frames 46–52 then returns to idle; (4) confirm player also flinches on counter-attack |

## Overall Verdict

PASS — all 15 automatable artifact and runtime checks passed; one visual sign-off (K001) remains as NEEDS-HUMAN per the stated UAT type (K001 is explicitly a manual `cargo winx` pass/fail, outside auto-mode reach).

## Notes

**Automatable checks summary:**
- Headless suite: 119/119 green. Windowed suite: 33/33 green. No regressions.
- Code structure confirms all three S01 safety properties are enforced structurally, not by convention: (1) Death is filtered by type equality before the seeding loop; (2) the dedup HashSet prevents double-flinch in the same event window; (3) the Idle-mode guard prevents hurt from interrupting an in-flight skill animation.
- The hurt→idle return path is fully authored in stance.ron — `drive_hurt_reactions` seeds only the "hurt" entry node; the TimeInNode transition handles the return with no new code needed.
- No windowed symbols leaked into reaction.rs — the headless/windowed boundary remains clean per R002/R005.

**K001 manual sign-off instructions:**
1. Run `cargo winx` (alias for `cargo run --features windowed`)
2. Trigger Sharp Claws from the player Agumon
3. Observe opponent sprite enters hurt frames (46–52) on the impact frame
4. Observe opponent sprite returns to idle ~0.5s after impact (TimeInNode transition fires)
5. Optionally: let opponent counter-attack to confirm player also flinches
6. Optionally: trigger two rapid strikes to confirm the dedup guard (single flinch, not double)
