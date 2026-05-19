---
estimated_steps: 1
estimated_files: 3
skills_used: []
---

# T03: Route Agumon Basic through Sharp Claws timeline data

Expected executor skills: rust-best-practices, rust-testing. Why: the Basic button must resolve a real Sharp Claws skill instead of reusing Baby Flame, and the kernel payload must stay in skills.ron rather than animation graph commands. Do: add a `sharp_claws` SkillDef to `assets/data/digimon/agumon/skills.ron` with implemented single-target targeting, zero SP cost, legacy_ops matching the timeline payload, and a compiled timeline with cast/windup, impact damage, break/aftermath, and recovery beats. Put the presentation barrier on the impact beat carrying `core/deal_damage`, with `cue_id` and `anim` values that T05 can map to the Sharp Claws strike node; keep concrete i32 payload amounts because the current kernel `BeatPayload` schema has no skill param table. Update `assets/data/digimon/agumon/unit.ron` so Agumon's `basic_skill` is `SkillId("sharp_claws")`, while Baby Flame remains available as a skill entry for later slices. Add `tests/agumon_sharp_claws_asset.rs` to parse the Agumon unit and skills data, compile timelines with builtins, assert Basic points to Sharp Claws, assert the impact beat carries the damage payload and presentation cue, and assert no animation graph gameplay command is required. Failure Modes (Q5): unknown hook/selector ids should fail compile with the existing compile error path; missing `sharp_claws` should fail the new asset test before runtime. Load Profile (Q6): no runtime shared-resource change; one additional skill timeline in the book. Negative Tests (Q7): assert Baby Flame still parses, Sharp Claws timeline compile fails if builtins are not registered or references are wrong, and animation graph D001 remains separate. Done when Agumon Basic has a data-backed Sharp Claws timeline ready for the cue barrier.

## Inputs

- `assets/data/digimon/agumon/skills.ron`
- `assets/data/digimon/agumon/unit.ron`
- `src/data/skills_ron/types.rs`
- `src/data/skill_timeline.rs`
- `src/combat/runtime/timeline.rs`
- `tests/anim_gameplay_command_forbidden.rs`

## Expected Output

- `assets/data/digimon/agumon/skills.ron`
- `assets/data/digimon/agumon/unit.ron`
- `tests/agumon_sharp_claws_asset.rs`

## Verification

cargo test --test agumon_sharp_claws_asset --test anim_gameplay_command_forbidden

## Observability Impact

New asset test localizes failures to data routing, timeline compilation, or D001 boundary violations before windowed runtime is involved.
