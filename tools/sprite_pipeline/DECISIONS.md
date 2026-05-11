# Project Decisions Log

Authoritative record of design/scope decisions. Future agents and dev sessions should READ this before defaulting to old behavior. Append-only — when reversing a prior decision, add a NEW dated entry referencing the old one.

---

## 2026-04-30 — Anime-only direction, drop pixel-art track

**Decision:** Project commits to anime cel-shaded output. Pixel-art track (palette-quantize 128/192/256) deprecated for primary deliverables.

**Rationale:**
- Game type: roguelite card-battler à la Slay the Spire / Inscryption.
- Card battler = static char display, no movement/tile, no procgen overworld → polish-per-sprite > sprite count.
- Pixel-art outline at 128-192 px showed wobbly diagonal artifacts on chibi T-rex curves. Acceptable for proper pixel-art games (manual cleanup expected) but not for our 3D-rendered → quantize pipeline.
- Anime cel-shaded at 1024 native render → LANCZOS downscale to 256/384/512/768 = clean, polished, modern HD-2D feel.

**Implementation:**
- `pipeline_run.py`: `TRACK_CONFIG["pixel"]` removed; only `"anime"` remains.
- `DEFAULT_SHADERS = ["anime_eevee", "mihoyo_style"]`. Other shader files retained, accessible via `--shaders` flag for experimentation.
- `DEFAULT_CAMERAS = [("iso_45", "iso45")]` only. Side/iso30/etc commented but pickable via `--cameras`.

**To revert:** uncomment deprecated entries in pipeline_run.py + restore `pixel` track block from git history.

---

## 2026-04-30 — Camera = iso45 only for primary output

**Decision:** Default camera angle is iso45 (¾ JRPG view).

**Rationale:**
- Slay the Spire / Hearthstone / Inscryption / FFVII menu / Octopath all use ¾ angle for char illustrations.
- `side_right` (profilo flat 2D) loses depth — looks like fighter sprite, not card-battler.
- `iso_30` (top-down ~50° pitch) too vertical for chibi T-rex shape (Agumon/Gabumon) — testa grande, corpo piccolo collassano in blob.
- iso45 = preserves silhouette + adds depth + matches genre conventions.

---

## 2026-04-30 — Render size = 1024 native

**Decision:** `RENDER_SIZE = 1024` in pipeline_run.py. Was previously 768.

**Rationale:**
- 768 → 512 LANCZOS lost fine detail (eye highlights, claw rims).
- 1024 native gives oversampling headroom for downscale; output at 256/384/512 looks anime-quality, not 3D-render-residue.
- Render time impact minimal (~30% slower, GPU-cap aware AUTO threshold in setup_render).

---

## 2026-04-30 — Animation set = 8 actions (Slay-the-Spire-like)

**Decision:** Per char render these 8 anims:

| Name | FBX action suffix | Use |
|------|-------------------|-----|
| idle | bn01 | Default battle stance |
| attack | ba01 | Basic attack card |
| heavy_attack | ba02 | Skill/heavy attack card |
| hurt | bd01 | Damage taken |
| death | bd03 | KO |
| block | bg01 | Defend card |
| skill | bs01 | Special move |
| victory | bv01 | Win pose |

**Rationale:** Card battler combat states. CS Agumon and CS Gabumon both share same FBX schema (`chr<id>_armature|chr<id>_armature|chr<id>_<action>`), so adding new char = drop FBX + 1 config copy + s/chr050/chr<NEW>/g.

---

## 2026-04-30 — Distribution formats = WEBP primary, GIF fallback

**Decision:** Output animated assets as `.webp` (primary, alpha-perfect) + `.gif` (fallback, palette-256, magenta-key transparency hard-thresholded).

**Rationale:**
- WEBP: native Bevy/Godot support, RGBA alpha, quality 92, smaller file size.
- GIF: universal preview (Discord, sketchbooks, browsers without webp). Fixed prior background-flicker bug via global palette + hard alpha threshold + disposal=2.

**Generation:** post-process script in `_anime_picks_v2/<char>/<shader>/<anim>/` after blender render.

---

## 2026-04-30 — Sprite size tiers per asset class

**Decision:** Output ladder 256/384/512/768/1024 generated for every anim. Engine picks size based on asset role:

| Asset role | Recommended size |
|-----------|------------------|
| Card portrait icon | 192-256 |
| Enemy basic mob | 256-384 |
| Player + mid enemy | 512 ★ default |
| Elite/mini-boss | 512-768 |
| Boss finale | 768-1024 (native) |
| Cinematic/key art | 1024 |

---

## 2026-04-30 — Bake picks: 512 only, flat layout

**Decision:** `_anime_picks_v2/<char>/<shader>/` contains animation files DIRECTLY (no per-anim subfolder). Single output size = 512. Both `anime_eevee` and `mihoyo_style` baked.

**Layout:**
```
_anime_picks_v2/<char>/<shader>/
    idle.webp   idle.gif
    attack.webp attack.gif
    ... (8 anims)
```

**Rationale:**
- Multi-size ladder (256/384/512/768/1024) was speculative — engine still picking sizes. Bake on demand later.
- Per-anim subfolder added directory noise without value (1 webp + 1 gif each).
- 512 = card-battler default per size-tiers table above.

**Generation:** `python3 sprite_pipeline/scripts/bake_picks.py --all` (or `--char <name>`). Reads `output/<char>/latest/sprites_anime/<shader>_iso45/frames_<anim>/` + writes 512px WEBP+GIF. WIPES per-shader output dir before writing — no stale files.

**Playback fps:** default 15fps. Matches source 60fps × `frame_step=4` sampling ratio. Earlier 12fps caused idle anims (tentomon, patamon) to look sluggish/choppy.

**Trailing-frame dedup (FIXED upstream):** Blender exports frames inclusive of `frame_end`; cyclic anims (idle, victory, block) often have frame_start ≈ frame_end → loop holds duplicate at seam = stutter. Fixed in `blender_render.py:render_animation`: post-render compares frame_00 vs frame_last via numpy on `bpy.data.images.pixels` (RGB only — alpha edges flip), drops last when mean diff ≤ 1.0/255. Bake script no longer dedups (assumed clean source). Manual override at bake time: `--drop-last N`.

**Bake parallelism:** `bake_picks.py` uses `ProcessPoolExecutor` over individual anims (char × shader × anim flat job list). Default workers = `cpu_count - 1`. Override via `--workers N`.

**Per-anim playback fps:** source FBX = 60fps. Each config anim has its own `frame_step` (idle=4, attack=4, victory=8, block=2 etc — varies per char). Bake derives `fps = SOURCE_FPS / frame_step` per anim so motion plays at source-true speed. Earlier fixed fps=15 made step=8 anims play 2× too slow and step=2 play 2× too fast. Override (force same fps everywhere): `--fps N`.

**Single-anim re-bake:** `bake_picks.py --char <name> --anim idle` overwrites only that anim's webp+gif. Other anims/dirs untouched (no wipe). Useful for quick playback-tuning iterations.

**To revert to multi-size ladder:** restore prior bake script from git or extend `bake_picks.py` to loop sizes + reintroduce per-anim subfolder.

---

## 2026-04-30 — NewCentury: textures recovered, basic_cel works

**Decision:** Keep agumon-newcentury config active. NOT skipped despite earlier brief skip.

**History:**
1. Initial diagnosis: matcap LUT only, diffuse texture missing → render produced gray body with no signature features → temporarily renamed config to `.skip`.
2. User dropped `Mobile - Digimon New Century - Rookie Digimon - Agumon _ BlackAgumon _ SnowAgumon.zip` in raw_models/.
3. `yagushou_base.png` (diffuse UV-mapped) extracted along with mask_g, sss, snow + black variants.
4. Config restored: `texture_path` → `yagushou_base.png`, `shader_variant` → `basic_cel`. Char now renders correctly.

**Variants available** (texture swap only, same FBX):
- yagushou (default Agumon) — wired
- xueyagushou (SnowAgumon) — texture extracted, not wired in config
- yagushouhei (BlackAgumon) — texture extracted, not wired in config

---

## 2026-04-30 — ReArise scrapped

**Decision:** Drop `agumon-rearise` entirely. Removed `raw_models/agumon-rearise/`, `configs/agumon-rearise.json`, `output/agumon-rearise/`.

**Rationale:**
- ReArise rig produced authentic battle-stance pose (one paw forward) but visually less "card portrait" friendly than CS upright pose.
- CS Agumon achieved 85+ scores reliably; ReArise required separate shader tuning for marginal benefit.
- Reduces maintenance: 1 source per Digimon name preferred unless strong reason for 2.

---

## 2026-04-30 — Char roster strategy

**Current roster:**
- agumon (CS chr050) ★ playable / boss-tier
- gabumon (CS chr151) ★ playable / boss-tier
- ~~agumon-newcentury~~ DROPPED 2026-04-30 (`configs/agumon-newcentury.json.skip`): NewCentury texture intrinsically paler/yellower than CS variant; max-iter loop stuck at ~83 vs ≥85 target. CS agumon already covers the slot. Re-enable by renaming `.skip` → `.json`.

**Adding a new Digimon:**
1. Drop `chr<id>.fbx` + `chr<id>a01.png` in `raw_models/<name>/`
2. Copy `configs/agumon.json` → `configs/<name>.json`, replace `chr050` → `chr<id>` in paths + 8 action_name fields
3. Set `hide_meshes`: typically `["chr<id>_3"]` (outline-hull) or test render to identify. Smaller meshes (chr<id>_0 with low vert count) often = horns/details, KEEP visible.
4. Run: `python3 sprite_pipeline/scripts/pipeline_run.py --char <name>`
5. Post-process: bake WEBP+GIF set via `_anime_picks_v2` generation script.

---

## Infrastructure fixes (permanent, do not revert)

- **blender_render.py:apply_action** → `action_slot` binding for Blender 4.4+ layered actions. Without this, mesh stays in rest pose for some FBX sources.
- **blender_render.py:import_model** → FBX import args `automatic_bone_orientation=True, ignore_leaf_bones=True, use_anim=True`. Game-exported FBX (mobile/Unity) require these or rig deforms incorrectly.
- **blender_render.py:setup_render** → AUTO Cycles GPU detection (HIP/CUDA/OPTIX/ONEAPI/METAL) with sample threshold (default 16). At samples=1 (our default) GPU init overhead beats render gain → CPU faster, AUTO falls back correctly.
- **pipeline_run.py:stdout** → reconfigured to UTF-8 with `errors="replace"` to prevent Windows cp1252 crash on `→` glyph.
- **pipeline_run.py:build_comparison** → font candidates extended with Windows fonts (Arial Bold, Segoe UI Bold) so comparison sheets have readable labels on Windows.

---

## 2026-04-30 — Renamon attack workaround (CS source)

**Decision:** Renamon `attack` slot uses `bs01` (skill) action first half (frames 1-30, step 3) instead of `ba01`.

**Rationale:** CS chr391.fbx packs all evolution-form actions (chr458 Kyubimon, chr569 Taomon, chr705/706/707/750) and `ba01` action keyframes auxiliary bones intended for evolution forms. When applied to Rookie Renamon armature, mid-late frames deform mesh badly (head detaches, sleeves stretch). frame_00 clean → frame_07+ broken. bs01 is full-body and renders clean.

**Failed alternative:** Digimon Links Renamon FBX (Mobile) has clean 12-action set but Mobile-FBX bone-axis convention conflicts with Blender FBX importer's `automatic_bone_orientation` option. With `True`: action keyframes don't drive mesh (T-pose). With `False`: action drives bones but mesh weights misaligned (limbs contort). Fix would require `better_fbx` addon or external FBX→glTF conversion. Parked.

**Infrastructure side-effect (KEEP):** `blender_render.py:import_model` now accepts per-config `fbx_import_opts` override (used Try/diagnose Links). Default opts unchanged for other chars.

---

## 2026-05-01 — Default render device = CPU

**Decision:** New renders use `--cycles-device CPU`. GPU forced only on demand (e.g. when bumping `cycles_samples` ≥16 for quality runs).

**Rationale:**
- Project default `cycles_samples=1`. At that sample count, GPU init overhead (HIP/CUDA backend warmup ~3-5s) beats GPU render gain. CPU finishes ~15-25s/anim, GPU ~15-50s/anim with bigger variance.
- Reduces thermal load on multi-char loops (7 chars × 8 anims = 56 renders).
- AUTO heuristic in `blender_render.py:setup_render` already picks CPU at samples<16; this just makes the decision explicit at orchestrator level.

**To override:** `pipeline_run.py --cycles-device GPU` for one-off quality runs at high samples.

---

## 2026-05-01 — Digimon Links models added (chibi Mobile variants)

**Decision:** Adopted Digimon Links FBX (Mobile) as PRIMARY for renamon. Added Links variants alongside CS for agumon, gabumon, patamon.

**Rationale:**
- CS chr391 Renamon FBX bundles evolution-form actions (chr458 Kyubimon...chr750) that key auxiliary bones; `ba01` deforms Rookie mesh badly mid-anim. Links chr391 has clean 12-action rig with no aux-bone cruft.
- Links Mobile FBX bone-axis convention conflicts with Blender's built-in `automatic_bone_orientation` import. Workaround: external `FBX2glTF` (in `sprite_pipeline/tools/`) → `.glb` round-trip. Imports clean via Blender's gltf path.

**Per-char status:**
- `renamon` (now Links primary; CS skipped to `renamon-cs.json.skip`).
- `agumon-links`, `gabumon-links`, `patamon-links` exist alongside CS variants. Either can serve as primary depending on art direction (Mobile = chibi/playful; CS = upright/serious).

**Pipeline support:**
- `blender_render.py:import_model` accepts cfg `fbx_import_opts` per-config override.
- `blender_render.py` adds cfg `drop_objects` (delete placeholder meshes like FBX2glTF's `Icosphere` stub).
- `blender_render.py` adds cfg `model_scale_factor` (FBX2glTF outputs cm-scale, requires 100×).
- `blender_render.py:apply_action` slot picker now prefers slot with `pose.bones` fcurves. FBX2glTF emits multiple slots per action (`OBJ_root` for root translation, `OBGRP_mesh`/`OBGRP_joint` for bone curves). Old code grabbed first OBJECT slot = often `OBJ_root` → only translation animated, body stayed static. Critical for Mobile FBX → glTF route.

---

## 2026-05-01 — Playback fps standardized to 12 (anime "on 2s")

**Decision:** All char anim configs use `frame_step=5` (60fps source / 5 = 12fps playback).

**Rationale:**
- 12fps = anime industry "on 2s" convention (24fps overall, char drawn every 2nd frame).
- Reference: Hollow Knight (12fps), Slay the Spire/Inscryption (8-12fps card portraits).
- Earlier `frame_step=2` (30fps) too smooth for indie cel aesthetic + 2.5× more renders.
- Bake derives per-anim fps automatically from `frame_step` (`60 / step`).

**Override:** `bake_picks.py --fps N` to force fps everywhere; per-anim `frame_step` in config controls source-side sampling.

---

---

## 2026-05-01 — MiHoYo style as definitive shader

**Decision:** Adopted `mihoyo_style` as the exclusive primary shader for all characters. `anime_eevee` and others removed from active rotation.

**Rationale:**
- **Visual Depth:** `mihoyo_style` consistently achieved 5-8% higher color variance than Eevee.
- **Modern Look:** Fresnel rim lighting provides a "luminous" silhouette that defines large shapes (ears, horns) better than flat cel-shading.
- **Technical Integrity:** The 3-band system (shadow/mid/overdrive) enables better material separation (e.g., metallic reflections on Tentomon, soft fur on Dorumon).

**Implementation:**
- `pipeline_run.py`: `DEFAULT_SHADERS = ["mihoyo_style"]`.
- `anime_eevee.py` and other secondary shaders deleted from `scripts/shaders/`.

---

## 2026-05-01 — Cyber Sleuth (CS) models as primary (Agumon, Gabumon, Patamon)

**Decision:** Reversed prior decision. The **Cyber Sleuth (CS)** variants are now the primary deliverables for Agumon, Gabumon, and Patamon. Links variants moved to `.skip`.

**Rationale:**
- **Superior Quality:** Empirical testing showed CS models produce 2-5% higher color variance and significantly sharper detail in MiHoYo shading.
- **Mesh Density:** Higher poly counts in CS allow for smoother Fresnel rim lighting and better defined small features (teeth, claws).
- **Consensus:** Technical audit confirmed CS renders look "fuller" and more premium than the lighter Links rigs.

**Finalized Roster:**
- `agumon` (CS Primary)
- `gabumon` (CS Primary)
- `patamon` (CS Primary)
- `dorumon` (CS Primary)
- `tentomon` (CS Primary)
- `renamon` (Links Primary — no CS available in workspace)

