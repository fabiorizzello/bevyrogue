# Sprite Pipeline — 3D model → Pixel Art / Anime Sprites

Toolchain to convert 3D models (FBX/GLB/OBJ/DAE) into dual-track sprite outputs (anime cel-shading hi-res + HD-2D pixel art) for bevyrogue.

## Quick start

See [`GETTING_STARTED.md`](GETTING_STARTED.md) for clone-to-render walkthrough on a fresh PC.

```bash
# After clone + Blender 5.x + pip install Pillow:
python3 tools/sprite_pipeline/scripts/pipeline_run.py --char agumon --parallel 4 --skip-deps-check
```

Output: `tools/sprite_pipeline/output/agumon/latest/sprites_anime/<variant>_<camera>/idle.png` + same under `sprites_pixel/`.

## Folder layout

```
sprite_pipeline/
  scripts/
    pipeline_run.py         # Main orchestrator (run isolation, manifests, quality gates)
    blender_render.py       # Headless render: 3D → frame PNG (Cycles, freestyle, solidify helpers)
    pixelify.py             # Downscale + palette quantize (called by pipeline_run)
    quality_gate.py         # Validates renders (alpha, color variance, bbox)
    extract_palette.py      # K-means palette from texture
    inspect_model.py        # Debug: list actions/armature/UV maps/bbox
    multi_shader.sh         # Legacy: render N shader variants (superseded by pipeline_run)
    shaders/
      basic_cel.py          # Posterize emission + outline
      toon_bsdf.py          # Cycles 3-band Lambert
      anime_eevee.py        # Eevee 2-3 band ColorRamp
      threelevel.py         # 3-band cel + Fresnel rim
      mihoyo_style.py       # HSR-inspired 2-band + rim
      painterly.py          # Soft B-spline ramp
      freestyle_outline.py  # Freestyle vector lines + 4-band posterize
      flat_emission.py      # Pure texture (baseline)
      dithered.py           # Bayer pattern + 2-band
      astropulse.py         # BlenderToPixels compositor
      lospec_toolkit.py     # Lospec compositor
      matcap.py             # Sphere-mapped (camera-space normal lookup)
  configs/
    <char>[-<source>][-<variant>].json   # Per-source render config
  palettes/
    <char>.gpl              # Auto-detected by pipeline_run (with base-name fallback)
  references/
    global/                 # Cross-char style targets (octopath, zolo)
    <char>/canonical.*      # Char-specific ground truth
  standards/
    global.md               # Cross-char scoring rules + per-shader specs
    <char>.md               # Char-specific identity + signature features
  plugins/                  # Tracked: BlenderToPixels.blend, Lospec_Blender_Toolkit.blend
  raw_models/               # Source FBX + textures (tracked)
  raw_renders/              # Hi-res Blender output (gitignored)
  output/                   # Final dual-track sprites + manifests (gitignored)
```

## Dependencies

- **Blender 5.x** (5.1 tested) — `apt install blender` or snap
- **Python 3.10+** + Pillow — `pip install Pillow`
- Optional: ImageMagick, libresprite/Aseprite (manual cleanup)

## Naming convention

`<digimon>[-<source>][-<variant>]`. Examples: `agumon`, `agumon-rearise`, `agumon-newcentury`, `agumon-black`. See `GETTING_STARTED.md` §5–7.

## Auto-iteration

Closed-loop render → review → edit cycle via Claude Code skill:

```
/sprite-iterate
# or in chat: "iterate sprite pipeline for <char>, max 3 iter, target 80"
```

See `.claude/skills/sprite-iterate/SKILL.md` for full workflow.

## Pipeline stages

1. **Render** (`blender_render.py`): Import model → apply rotation pivot → bind shader (UV-active or matcap) → setup freestyle/solidify outline → render N×M (variants×cameras) per animation frame at hi-res.
2. **Pixelify track** (`pixelify.py`): NEAREST downscale 768→256 + palette quantize via `palettes/<char>.gpl` strict.
3. **Anime track**: raw render PNG passes through unchanged (smooth Cycles surfaces).
4. **Sheet assembly** + **comparison grids**: per-track `comparison_anime.png` / `comparison_pixel.png` for visual selection.

## Standards / scoring

Read `standards/global.md` (cross-char rules) + `standards/<char>.md` (char identity) before manually scoring or invoking auto-iteration. Source of truth for "what good looks like" per variant + camera.

## See also

- [`GETTING_STARTED.md`](GETTING_STARTED.md) — clone-to-render quickstart
- [`EXTERNAL_TOOLS.md`](EXTERNAL_TOOLS.md) — alternative pixel-art tool catalogue
- [`AUTO_ITERATION.md`](AUTO_ITERATION.md) — closed-loop iteration internals
- [`docs/sprite_pipeline_evaluation.md`](../../docs/sprite_pipeline_evaluation.md) — workflow comparison + decisions
