---
estimated_steps: 3
estimated_files: 1
skills_used: []
---

# T01: Generalize AgumonSprite to data-carrying DigimonSprite

Why: extension-first presentation requires the windowed component to carry stance/skill graph ids as DATA so S04/S05 add Digimon without editing it, and the milestone requires zero AgumonSprite/AgumonPlaybackMode identifiers to remain. This is the highest-blast-radius seam (S03-RESEARCH seam 1) — land it first and confirm green before wiring cues.

Do: In src/windowed/render.rs (the rename is fully contained here — verified no references outside render.rs): (1) rename struct AgumonSprite -> DigimonSprite and enum AgumonPlaybackMode -> DigimonPlaybackMode (variants Idle/Skill are already generic — name change only). (2) Add two fields to DigimonSprite carrying the graph ids as data: stance_graph_id: String and skill_graph_id: String. (3) Thread the ids through idle_for / start_skill / return_to_idle / drive_stance_reaction so they are preserved across mode transitions; idle_for's signature gains the two ids, populated at the spawn site (L702 spawn_unit_sprites) which in S03 still passes AGUMON_STANCE_GRAPH_ID / AGUMON_SKILL_GRAPH_ID consts (const removal is S04 — do NOT pull it forward). (4) Switch the resolve_snapshot call sites that currently read AGUMON_STANCE_GRAPH_ID / AGUMON_SKILL_GRAPH_ID consts (advance_agumon_presentation ~L778, drive_hurt_reactions ~L1176, drive_death_reactions ~L1252, sync_agumon_mode ~L1712/1747) to read sprite.stance_graph_id / sprite.skill_graph_id off the queried DigimonSprite instead — moving the resolve inside the per-sprite scope where it currently sits before the loop. The spawn site (~L711) keeps the const since it is the data source. Update all field/type references, the classify_same_skill_sync signature, and the in-module unit tests (~L2026/2049) to the new names. Keep behaviour identical — only Agumon exists this slice.

Done when: cargo build --features windowed exits 0 with zero warnings; cargo test --features windowed --test windowed_only stays green; grep confirms DigimonSprite/DigimonPlaybackMode present and AgumonSprite/AgumonPlaybackMode absent in render.rs.

## Inputs

- `src/windowed/render.rs`
- `src/windowed/mod.rs`

## Expected Output

- `src/windowed/render.rs`

## Verification

cargo build --features windowed
