---
spike: SP4
artifact: validation_notes
date: 2026-05-12
---

# SP4 — Validation Notes

Findings from hand-authoring `sample_clip.ron` + `sample_animation_fsm.ron`
against the real `tools/sprite_pipeline/output/bevy_atlases/agumon_atlas.json`
and the canon design docs (§2.2, §2.2b).

---

## §A — What `_atlas.json` covers (lossless surface)

The bevy-v1 atlas json contains **only frame geometry**. Concretely:

| Field | Type | Maps to `clip.ron` |
|---|---|---|
| `meta.character` | string | `Clip.meta.character` (direct) |
| `meta.version` | string | `Clip.meta.version` (direct) |
| `meta.frame_size.w/h` | u32×2 | `Clip.meta.frame_size` (direct) |
| `meta.columns` / `meta.rows` | u32 | `Clip.meta.columns/rows` (direct) |
| `meta.total_frames` | u32 | `Clip.meta.total_frames` (direct) |
| `animations.<name>.start_index` | u32 | `Clip.clips[name].start` (direct) |
| `animations.<name>.end_index` | u32 | `Clip.clips[name].end` (direct) |
| `animations.<name>.count` | u32 | redundant (= `end - start + 1`, verified) |

8 named clips in agumon atlas: `attack, block, death, heavy_attack, hurt,
idle, skill, victory`. All 8 map cleanly.

### §A1 — Lossless field count
- **8 fields directly mapped** (atlas → clip.ron geometry).
- **0 fields lost** (every atlas field has a `clip.ron` home).

---

## §B — What `_atlas.json` MISSES (gaps the schema demands)

| Schema field (§2.2) | Atlas has? | Resolution in spike |
|---|---|---|
| `clip.ron::meta.texture_path` | ❌ | INVENTED `digimon/agumon/agumon_atlas.png` from pipeline naming convention. Pipeline must emit this or loader must derive by sibling-path convention. |
| `clip.ron::meta.default_fps` | ❌ | INVENTED `12` from design doc references (`agumon/02 §3` ms-at-12fps). |
| `clip.ron::clips[name].fps` | ❌ | INVENTED per-clip (12 for action, 6 for idle, 14 for skill). Atlas gives no per-anim timing. |
| `clip.ron::clips[name].loop_` | ❌ | INVENTED heuristically (idle=true, all else=false). |
| Hitbox metadata (per-frame or per-clip) | ❌ | NOT in spike (out of §2.2 scope — see §D below). |
| Frame-event hooks (impact frame, particle slot) | ❌ | Lives in `clipmontage.ron` / `animation_fsm.ron`, NOT in atlas. Correctly separated by §2.2 2-asset model. |
| Anchor / pivot points (mouth, weapon, claw_tip) | ❌ | Explicitly killed by §2.2d (no rig data exists). Collapsed to `VfxLocus = SelfCenter | EntityCenter(_) | ...`. Correctly not in atlas. |

### §B1 — Gap table headline

| Category | Count |
|---|---|
| Lossless atlas-to-RON fields | 8 |
| Lossy/invented fields (must be added by pipeline or convention) | 4 |
| Out-of-scope-for-atlas fields (correctly live elsewhere) | 3 |

---

## §C — Schema refinements needed before S04 parser commits

### §C1 — `clip.ron` pipeline extensions (BLOCKER for S04 parser)
1. **Add `texture_path` to pipeline output.** Either emit it in `_atlas.json::meta` OR codify a sibling-path convention (`<name>_atlas.png` next to `<name>_atlas.json`) into the loader contract. SP5 (pipeline-determinism) must align on this.
2. **Add `fps` to pipeline output**, either per-clip in `animations` or as a single `meta.default_fps`. Without this, every `clip.ron` requires a manual edit post-pipeline-gen → defeats "lossless from json" goal of §2.2 migration §1.
3. **Add `loop` flag to pipeline output** for `animations.<name>`. Currently zero hint; heuristics scale badly past Agumon.

### §C2 — `animation_fsm.ron` schema gaps surfaced in practice

While hand-writing the two FSMs, the following clarifications emerged:

1. **`clip:` reference syntax.** §M sketch uses `clip: "skill"` — a **string name**, not a path or index. Confirmed: reference by clip-name (matches `clip.ron::clips` map key). Hot-reload friendly (clip rename = breaks reference but caught by validator §L).
2. **Node `frames` is clip-local, not atlas-global.** §M sketch shows `frames: (0, 12)` for "windup" — clip-local indices. The validator §L check "Frame range in-bounds" must be against `clip.total_frames` (the **clip's own** frame count, NOT the atlas total). The §L wording is ambiguous; recommend tightening to "`s < e ≤ clip_def.end - clip_def.start + 1`".
3. **Multiple `on_enter` Commands ordering.** §C4 G4 (agumon/02 §8) confirms RON declaration order = emission order. Validator §L should NOT reorder. Worth a contract test.
4. **`KernelEvent(UnitDied { target: StrikeTarget })` predicate filter — what is `StrikeTarget`?** §C4 reactive-signature table refers to `target == strike_target` but doesn't define the binding. **Likely answer:** the `EventFilter` must support a closure over the FSM's snapshot-once target list (resolved at commit). New `EventFilter` variant or implicit context. Action item for S03f.
5. **Recovery padding policy (§G7 frame-budget).** Two competing approaches: (a) two Recovery node variants chosen via edge priority; (b) implicit Hold modifier on the single Recovery node. Picked (a) in canon but spike collapsed to single node. S04 parser needs to support either; validator should enforce "total frames across all reachable paths == clip.total_frames" (loose) or warn on mismatch.

### §C3 — Command vocabulary gaps surfacing in practice

Per the brief, SP1/SP3 already flagged `ApplyBuff`, `EmitHeal`, `EmitCleanse`,
`TargetShape` expansion, status taxonomy. Confirming from agumon-only stress:

| Gap | Already raised by | Confirmed by this spike |
|---|---|---|
| `EmitDamage.tough_break_param` | agumon/02 §8 G2 (canon) | YES — every Strike node uses it |
| `EmitStatus.stacks_param` | agumon/02 §8 G3 (canon) | YES — Heated +2 from skill |
| `ParamRef::EventPayload` (live read from kernel event) | agumon/03 §8 G5 (canon) | YES — required by `reactive_detonate` |
| `EmitDamage.multiplier_chain: Vec<ParamRef>` | agumon/03 §8 G5 | YES — `heated_remaining * detonate_per_stack` |
| `StatusKind` enum vs free-form string `id` | §C2 cross-ref to 02-08 §H.1 | YES — `"Heated"` is the only string used here; validator must enforce enum |
| `BuffKind` enum on `ApplyBuff` | §C2 ApplyBuff | NOT exercised by Agumon (no buffs in baby_flame/baby_burner) — defer to other Digimon spikes |

**No NEW gaps surfaced from Agumon stress that weren't already in canon.**
SP4 confirms the §C/§C2 vocabulary is sufficient for Agumon's two skills.

---

## §D — Open questions from the spike skeleton — RESOLVED

### Q1: Does `_atlas.json` include hitbox metadata?
**No.** The atlas json is geometry-only (frame indices + grid layout). Hitboxes
are not mentioned anywhere in §2.2 or §2.2b — combat targets entities, not pixel
regions (HSR-style slot combat). **Hitbox concept does not apply** to this design.
The "hit" semantics live entirely in `EmitDamage.target_ref` (resolved entity),
not in pixel space. No action needed.

### Q2: Frame timing — ms vs ticks? Source of truth?
**Frames are the source of truth for FSM logic.** Per §G rule 1, `Hold/SpeedMul/
Loop` are expressed in **frame logici** (not ms). ms appears only as:
- `qte_window_ms` (UI metadata, with `headless_default_param` for determinism)
- `Shake.duration_ms`, `SpawnParticle.motion.ms` (cosmetic, no-op in headless)
- Design doc `ms (@12fps ref)` columns (documentation only)

**FSM is frame-counter driven**, headless-safe by construction. Pipeline must
either emit `fps` per clip or a global `default_fps`, or the loader must default
to a known constant (e.g. 12). **Recommendation:** emit `fps` in atlas json (§C1.2).

### Q3: How does `animation_fsm.ron` reference `clip.ron` — name, path, or index?
**By name** (clip map key). Confirmed via §M sketch (`clip: "skill"`).
- **Pros:** stable across renumbering, hot-reload-friendly, validator can check
  reference existence at load.
- **Cons:** renames break references silently unless validator catches them.
- **Decision:** name reference; validator §L MUST add "clip reference exists in
  clip.ron" check (not currently in §L table — recommend adding).

### Q4: Hot-reload contract — does FSM survive a `clip.ron` reload?
Not explicitly answered in §2.2b. Inferred from §G + §H + §L:
- **FSM holds clip-name string + node-local frame indices**, NOT direct frame
  pointers. So a `clip.ron` reload with updated frame ranges → FSM still valid
  IF the new range covers the FSM's max node end-frame.
- **If clip.ron reload changes the range to be too small** (e.g. `skill` shrinks
  from 17 to 10 frames, but FSM node `recovery` uses frames (12, 17)) → validator
  must re-run on reload, FSM rejected, fallback to previous valid FSM (or panic
  in dev mode).
- **Recommendation:** S03f validator runs both at boot AND on each `clip.ron`
  hot-reload. FSM survives reload iff revalidation passes. Document in §H.

---

## §E — Atlas-pipeline blocker assessment for S04

**Is `_atlas.json` (current bevy-v1 format) sufficient to drive S04 parser?**

**No, not as-is.** Three pipeline extensions required first (§C1.1–3):
- `texture_path` (or sibling-path convention)
- `fps` (per-clip or global default)
- `loop` flag

Two paths forward:
1. **Extend pipeline first (block S04).** Adds 3 fields to `_atlas.json` writer in `tools/sprite_pipeline/`. Estimated trivial (write-side only, all data exists). Owner: SP5 spike or a pre-S04 chore.
2. **Loader-side conventions + manual `clip.ron` curation (unblock S04).** Loader derives `texture_path` from atlas path, defaults `fps=12`, treats `loop_=false` unless overridden by adjacent `clip_overrides.ron`. Ship S04 with this; pipeline catches up later.

**Recommendation: Option 2 for S04.** Conventions are cheap, S04 parser only
needs the geometry the atlas already provides. Pipeline extension can land in
parallel without blocking M017.

---

## §F — Cross-spike escalations

**To SP1 (kernel primitives):** confirmed — `tough_break_param`, `stacks_param`,
`multiplier_chain`, `EmitHeal`/`EmitCleanse`/`ApplyBuff`, `ParamRef::EventPayload`
all needed. Agumon alone exercises 4/6 (no heal/cleanse/buff in Agumon kit).

**To SP2 (blueprint API):** `Twin Core` is a **listener-side passive in Gabumon's
blueprint** observing `CombatEvent::StatusApplied{Heated}` emitted by Agumon's
`baby_burner` Strike node. The cross-unit signal flow goes through the kernel
event bus, NOT a direct FSM reference. This matches §2.7 C2 dual-role pattern.
SP2 should confirm the listener registration shape (likely
`SkillBehavior::on_kernel_event`).

**To SP3 (skill DSL):** `ParamRef::EventPayload(name)` requires the kernel event
schema to expose typed payload fields (e.g. `UnitDied.heated_remaining: u8`).
This is **new surface area** vs current `CombatEvent` (which is mostly opaque
discriminants). SP3 must extend `CombatEvent::UnitDied` payload to carry
`status_remaining: Vec<(StatusId, u8)>` or similar.

**No blocker raised — all 3 cross-spike asks are additive and align with
existing canon.**

---

## §G — Summary verdict

- Atlas → clip.ron is **lossless on geometry, but pipeline misses 3 fields** the schema requires (texture_path / fps / loop). Workable via convention; bumpable to pipeline later.
- Animation FSM schema (§2.2b) is **sufficient as-written for Agumon's two skills**. No new Command verb required beyond §C/§C2.
- 4 schema refinements recommended (§C2.1–4) — none block S04, all worth a follow-up sticky in M017 planning.
- 1 new validator check requested: "clip name reference exists" (§D Q3).
- 1 hot-reload contract clarification needed (§D Q4) — recommend documenting in §H of canon.
- **S04 parser unblocked** via convention-based loader. Pipeline extension lands in parallel.
