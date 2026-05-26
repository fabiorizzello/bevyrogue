---
id: T01
parent: S03
milestone: M006
key_files:
  - src/windowed/render.rs
key_decisions:
  - DigimonSprite carries stance_graph_id/skill_graph_id as String data fields; all stance/skill resolve_snapshot calls read these off the queried sprite instead of module consts
  - Only idle_for needs the id params in its signature — start_skill/return_to_idle/drive_stance_reaction preserve the ids implicitly since they never reset those fields
  - Per-sprite resolve moved inside the loops; missing-graph early-return became per-sprite continue (behaviorally identical while only Agumon exists)
  - Spawn site retains AGUMON_*_GRAPH_ID consts as the data source; const removal deferred to S04 per slice plan
duration: 
verification_result: passed
completed_at: 2026-05-26T11:24:13.723Z
blocker_discovered: false
---

# T01: Renamed AgumonSprite/AgumonPlaybackMode to data-carrying DigimonSprite/DigimonPlaybackMode and routed all stance/skill graph resolution through carried ids instead of consts

**Renamed AgumonSprite/AgumonPlaybackMode to data-carrying DigimonSprite/DigimonPlaybackMode and routed all stance/skill graph resolution through carried ids instead of consts**

## What Happened

Generalized the windowed presentation component in src/windowed/render.rs (the rename was fully contained there — confirmed no AgumonSprite/AgumonPlaybackMode references outside render.rs).

(1) Renamed `struct AgumonSprite -> DigimonSprite` and `enum AgumonPlaybackMode -> DigimonPlaybackMode` (Idle/Skill variants already generic — name change only). All field/type references, the `classify_same_skill_sync` signature, `mode_trace_fields`, `sync_agumon_mode`, and the two in-module unit tests were updated via global replace.

(2) Added two data fields to DigimonSprite: `stance_graph_id: String` and `skill_graph_id: String`, documented as the per-Digimon data source so S04/S05 can add Digimon by spawning with different ids rather than editing render.rs.

(3) Threaded the ids through the FSM methods: `idle_for` gained the two id params, populated at the spawn site (spawn_unit_sprites) which in S03 still passes the AGUMON_STANCE_GRAPH_ID / AGUMON_SKILL_GRAPH_ID consts as the data source (const removal deferred to S04). `start_skill` / `return_to_idle` / `drive_stance_reaction` are `&mut self` and never touch the two new fields, so the ids are preserved across every mode transition automatically — the cleanest threading.

(4) Switched all const-reading resolve_snapshot call sites to read off the queried sprite: advance_agumon_presentation now resolves the stance graph inside the per-sprite scope at the idle-restore branch using `sprite.stance_graph_id` (removed the pre-loop binding); drive_hurt_reactions and drive_death_reactions resolve per-sprite inside their loops (early `return` on missing graph became per-sprite `continue` — behaviorally identical for the single-graph case); sync_agumon_mode resolves the skill graph from `sprite.skill_graph_id` and the post-fallback stance trace from `sprite.stance_graph_id`. The two warn!/trace! `graph_id` fields now render the carried id. The spawn site keeps the const stance resolve since it is the data source.

Behavior is identical — only Agumon exists this slice. The const removal and camera-shake cue wiring are explicitly downstream (S04 / S03 later tasks).

## Verification

cargo build --features windowed exits 0 with zero warnings (grep -c warning == 0). cargo test --features windowed --test windowed_only green: 54 passed, 0 failed. grep confirms DigimonSprite/DigimonPlaybackMode present (33 hits) and AgumonSprite/AgumonPlaybackMode absent in render.rs. AGUMON_STANCE_GRAPH_ID / AGUMON_SKILL_GRAPH_ID now appear only at the import and the spawn-site data source, as required.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo build --features windowed` | 0 | pass | 2430ms |
| 2 | `cargo test --features windowed --test windowed_only` | 0 | pass (54 passed; 0 failed) | 20000ms |
| 3 | `cargo build --features windowed 2>&1 | grep -c warning` | 0 | pass (0 warnings) | 500ms |
| 4 | `grep -n AgumonSprite\|AgumonPlaybackMode src/windowed/render.rs` | 1 | pass (absent) | 50ms |

## Deviations

none

## Known Issues

none

## Files Created/Modified

- `src/windowed/render.rs`
