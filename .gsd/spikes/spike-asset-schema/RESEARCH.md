---
spike: SP4
name: clip.ron + animation_fsm.ron schema validation
status: done
created: 2026-05-12
completed: 2026-05-12
parallel_with: any
inputs:
  - docs/future_design_draft/02-02_animation_manifest.md (§2.2 clip.ron paper schema)
  - docs/future_design_draft/02-02b_animation_fsm.md (§2.2b animation FSM paper schema)
  - tools/sprite_pipeline/output/bevy_atlases/agumon_atlas.json (only real atlas output)
  - docs/future_design_draft/digimon/agumon/02_skill_baby_flame.md
  - docs/future_design_draft/digimon/agumon/03_ult_baby_burner.md
  - assets/data/skills.ron (current data shape reference)
outputs:
  - sample_clip.ron               (Agumon, derived from real atlas — 8 clips, 4 invented fields)
  - sample_animation_fsm.ron      (baby_flame 3-node simple + baby_burner 4-node reactive)
  - validation_notes.md           (atlas coverage, schema gaps, cross-spike asks)
verdict: §2.2/§2.2b schema usable for S04 with 3 pipeline gaps mitigated by loader convention
---

# SP4 — Asset schema (clip.ron + animation_fsm.ron) validation

## Goal

Prove the paper schema from §2.2 + §2.2b is **lossless on real data** before
S04 (clip.ron + animation_fsm.ron parser slice) commits. Surface unknowns,
gaps, and cross-spike asks before M017 planning closes.

## Method (executed)

1. ✅ Audited `tools/sprite_pipeline/output/bevy_atlases/agumon_atlas.json`
   — 8 named clips, geometry-only (no fps/loop/texture_path/hitbox).
2. ✅ Hand-authored `sample_clip.ron` — direct field mapping from atlas
   meta+animations, 4 fields explicitly `// INVENTED:` annotated.
3. ✅ Hand-authored `sample_animation_fsm.ron` — two FSMs:
   - `agumon_baby_flame` 3-node (Windup → Strike → Recovery) — simple linear,
     single `EmitDamage + EmitStatus(Heated, stacks=2) + tough_break`.
   - `agumon_baby_burner` 4-node (Windup → Strike → ReactiveDetonate → Recovery)
     — reactive edge `KernelEvent(UnitDied)` priority 10 vs `TimeInNode`
     priority 0. Cross-unit Twin Core listener referenced via the
     `StatusApplied(Heated)` emit from Strike (not direct FSM coupling).
4. ✅ Cross-checked command vocabulary v0 + C2 against canon. All gaps
   already raised in canon (G2 tough_break / G3 stacks / G5 EventPayload /
   G6 multi-target via blueprint). **No new vocabulary verb needed.**
5. ✅ Documented in `validation_notes.md`: atlas coverage matrix, 4 schema
   refinements, hot-reload contract clarification, cross-spike asks.

## Key findings (compact)

### Gap table headline

| Bucket | Count | Notes |
|---|---|---|
| Lossless atlas-to-RON fields | 8 | character, version, frame_size, columns, rows, total_frames + per-clip start/end |
| Lossy / invented fields | 4 | texture_path, default_fps, per-clip fps, per-clip loop_ |
| Out-of-scope-for-atlas (correctly elsewhere) | 3 | hitboxes (don't apply — entity combat), event hooks (in FSM), anchor/pivot (§2.2d killed) |

### Top 3 schema refinements needed
1. Pipeline emits geometry only — needs `texture_path` + per-clip `fps` + `loop`
   flag. Workaround: loader-side convention (`<name>_atlas.png` sibling +
   default_fps=12 + loop_=false unless overridden). Real fix: extend pipeline
   (SP5 or pre-S04 chore).
2. Validator §L gap: no check for "clip name reference exists in clip.ron".
   Add to S03f validator contract.
3. `KernelEvent` predicate filter (`target == strike_target`) needs explicit
   binding to FSM-snapshot target list. S03f must define `EventFilter` variants
   that close over commit-time context.

### Atlas pipeline blocker?
**No hard blocker.** S04 parser can ship using loader-side conventions for the
4 invented fields. Pipeline extension is additive and can land in parallel
without gating M017.

### Cross-spike escalations
- **SP1:** confirmed need for `tough_break_param`, `stacks_param`,
  `multiplier_chain`, `ParamRef::EventPayload`. All canon-aligned.
- **SP2:** Twin Core flows through the kernel event bus
  (`StatusApplied{Heated}`) — pure §2.7 C2 listener pattern, no special FSM
  coupling required.
- **SP3:** `CombatEvent::UnitDied` payload must expose `heated_remaining` (or
  more general `status_remaining: Vec<(StatusId, u8)>`) to support
  `EventPayload` reads on reactive nodes.

## Open questions — RESOLVED (full detail in validation_notes §D)

| Q | Answer |
|---|---|
| `_atlas.json` includes hitbox metadata? | No — and hitboxes don't apply (entity-slot combat, not pixel-region) |
| Frame timing ms vs ticks? | **Frames** are source of truth; ms is UI-metadata only (cosmetic Commands + QTE window) |
| FSM → clip reference by name/path/index? | **By name** (clip map key). Validator must check existence on load |
| Hot-reload: does FSM survive clip reload? | Yes IF revalidated. Frame indices are clip-local; new range OK if covers max node end-frame. Document in §H |

## Out of scope (confirmed)

- Writing the S04 parser (slice S04).
- Generating `clip.ron` for the other 5 Digimon (atlases not regenerated; SP5).
- Modifying `tools/sprite_pipeline/` (pipeline extension is SP5 / pre-S04 chore).
- QTE in `baby_burner` FSM (canon has it; collapsed here to keep spike focused
  on reactive-edge stress, not QTE-suspend).
- 4-node Inhale/Wind/Spit/Recovery canon for `baby_flame` (collapsed to 3-node
  Windup/Strike/Recovery per brief; canon shape is strict superset).

## Files in this spike

| File | Purpose |
|---|---|
| `RESEARCH.md` (this file) | Findings summary + frontmatter index |
| `sample_clip.ron` | Lossless transform of `agumon_atlas.json` to paper schema |
| `sample_animation_fsm.ron` | Two FSMs (simple + reactive) covering Agumon kit |
| `validation_notes.md` | Atlas coverage, schema gaps, open-questions resolution, cross-spike asks |
