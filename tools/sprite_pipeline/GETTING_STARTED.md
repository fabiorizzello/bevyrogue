# Getting Started — Sprite Pipeline (other PC)

What you need after `git clone` to render sprites end-to-end.

## 1. System dependencies

```bash
# Blender 5.x (5.1 tested)
sudo apt install blender   # OR snap install blender

# Python 3.10+ with Pillow (used by pixelify + palette extract)
pip install Pillow
```

Optional for image conversion:
```bash
sudo apt install imagemagick
```

## 2. Repo layout (after clone)

Tracked in git:
```
tools/sprite_pipeline/
  scripts/                    # all pipeline code
  configs/                    # per-source render configs (agumon.json, agumon-rearise.json, ...)
  palettes/                   # color palettes (agumon.gpl, ...)
  references/global/          # style targets (octopath, zolo)
  references/<char>/          # char ground-truth images
  standards/                  # scoring rules (global.md + per-char .md)
  raw_models/                 # source FBX + textures (~55MB Agumon, more as we add Digimon)
  plugins/
    BlenderToPixels.blend     # tracked, 2.5MB
    lospec-blender-toolkit/
      Lospec_Blender_Toolkit.blend  # tracked, 1.4MB
      README.md               # tracked
```

NOT in git (regenerable / re-downloadable):
```
output/                  # all renders (3.5GB+ when populated)
raw_renders/             # intermediate Blender frames
plugins/libresprite.AppImage         # 73MB, optional GUI editor
plugins/lospec-blender-toolkit/Examples/   # 28MB sample blends
```

## 3. Optional: libresprite (manual sprite editing)

Only needed if you want to manually clean up sprites with a GUI.

```bash
cd tools/sprite_pipeline/plugins/
wget https://github.com/LibreSprite/LibreSprite/releases/.../libresprite.AppImage
chmod +x libresprite.AppImage
```

Or use Aseprite ($20). Or skip — pipeline works headless without it.

## 4. First render (verify setup)

```bash
# From repo root:
python3 tools/sprite_pipeline/scripts/pipeline_run.py --char agumon --parallel 4 --skip-deps-check
```

Expected output:
```
output/agumon/runs/<timestamp>_<id>/
output/agumon/latest/                  # symlink to latest
  sprites_anime/<variant>_<camera>/idle.png
  sprites_pixel/<variant>_<camera>/idle.png
  comparison_anime.png
  comparison_pixel.png
  manifest.json
```

Single render takes ~2 min × 22 variants×cameras = ~10-15 min on 4-core CPU.

## 5. Add another Digimon

Naming convention: `<digimon>[-<source>][-<variant>]`

Example for Gabumon:
```
1. raw_models/gabumon.fbx + texture
2. configs/gabumon.json   (set model_path, texture_path, action_name, hide_meshes, etc)
3. palettes/gabumon.gpl   (extract via scripts/extract_palette.py OR hand-tune)
4. references/gabumon/canonical.png   (canonical char art)
5. standards/gabumon.md   (copy from agumon.md, replace identity + signature features)
6. python3 scripts/pipeline_run.py --char gabumon --parallel 4 --skip-deps-check
```

Pipeline auto-detects palette `gabumon.gpl`.

## 6. Multi-source same Digimon

Each source = own config:
- `agumon.json` → Cyber Sleuth
- `agumon-rearise.json` → ReArise mobile
- `agumon-newcentury.json` → New Century mobile

Each renders independently to `output/<full-name>/`.

## 7. Variants (palette swaps, alt forms)

`agumon-black.json`, `agumon-rearise-snow.json`, etc. Pipeline falls back from `palettes/<full-name>.gpl` to `palettes/<digimon-base>.gpl` if variant-specific palette doesn't exist.

## 8. Auto-iteration (closed loop)

Inside Claude Code:
```
iterate sprite pipeline for <char>, max 3 iter, target 80
```

Or invoke `/sprite-iterate` slash command. Subagent runs render → review → edit loop until target score reached. See `.claude/skills/sprite-iterate/SKILL.md`.

## 9. Standards / scoring

- `standards/global.md` — cross-char scoring rubric, per-shader specs, camera multipliers
- `standards/<char>.md` — char-specific identity (color, signature features, fix recipes)

Read these before manually scoring or before invoking auto-iteration. Source of truth for "what good looks like" per variant.

## 10. Troubleshooting

| Problem | Fix |
|---------|-----|
| All-black render | Check UV map name. Pipeline shaders use `uv.uv_map = ""` (active layer). For matcap-textured sources use `shader_variant: matcap`. |
| Char facing wrong direction | `cam_axis: "x"` + `iso_presets` use `yaw=270` for char-faces-right (configured in `blender_render.py`). |
| Char lying on side | Adjust `model_rotation_deg`. CS Agumon needs `[90,0,0]`; ReArise/NC need `[0,0,0]`. |
| Outline missing (single-mesh source) | Add `auto_solidify` + `freestyle_outline` to config (see `agumon-rearise.json`). |
| Outline covers thin features (claws) | Hide outline mesh + use `freestyle_outline` config block (see `agumon.json`). |
| Pipeline crashes on shader load | Some shaders may have agent-iteration WIP edits with broken refs. Check `git log --oneline scripts/shaders/` for known-good revision. |

## See also

- `README.md` — pipeline overview + config schema
- `EXTERNAL_TOOLS.md` — alternative pixel-art tools catalogue
- `AUTO_ITERATION.md` — closed-loop iteration internals
- `standards/global.md` — scoring rules
