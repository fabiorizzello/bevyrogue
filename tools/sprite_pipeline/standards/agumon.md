# Standards — Agumon

Char-specific spec. Used together with `standards/global.md` (cross-char rules).

## Identity

- **Type**: chibi T-rex bipede
- **Body color**: vibrant Adventure orange (`#F39A2E` canonical)
- **Hue range**: 20-40° HSL (orange). Out-of-range = global hard fail.
- **Saturation**: high. Pale yellow-tan = automatic ≤40 color score.
- **Eye**: green
- **Mouth**: small, single tooth visible from side
- **Tail**: short stubby

## Sources available

| Source | Config | Quality |
|--------|--------|---------|
| Cyber Sleuth | `agumon.json` | 3 meshes (chr050_0/_1/_2), good rig, single anim |
| ReArise | `agumon-rearise.json` | single mesh (chr050event), 33 bones, 12 actions |
| New Century | `agumon-newcentury.json` | 108 bones rig, 36+ actions, matcap texture |

## Variants

- `agumon-black` (Adventure dark variant) — TODO
- `agumon-newcentury-snow` (snow palette swap) — TODO

## Signature features (HARD CHECK)

These features MUST be visibly distinguishable in output. If absent / muddy / occluded → cap variant total at 70 regardless other criteria.

1. **Claws (hands AND feet)** — white/cream, oversized, signature
   - Hand claws: ≥2-3 white pixel groupings per hand visible
   - Foot claws: ≥2-3 white pixel groupings per foot visible
   - Test: zoom 200%, count individual claw points
   - Source caveat: Cyber Sleuth source texture has tan claws (NOT white) — claw whitening must come from shader bright_ramp + Z-position mask, OR texture paint
2. **Snout extended** — readable as T-rex muzzle, not flat-faced
3. **Green eye** — visible single eye in side view, NOT blacked out
4. **Tooth** — small visible from open mouth (side view)
5. **Stubby tail** — short visible behind body

## Color targets

**Canonical hex anchors** (palette must include):
- `#F39A2E` — mid-orange (Adventure canonical body)
- `#E08020` — terra deep
- `#C06010` — shadow orange
- `#FFE0B0` — claw highlight (cream/white)
- `#FFFFFF` — claw pure white
- `#000000` — outline black
- `#80B860` — eye green (mid)
- `#50783C` — eye green dark

Forbidden in output:
- Pale yellow `#F5E090` or similar washed-tan dominating body
- Pure white silhouette > 15% (rim/specular failure)

## Reference paths

- `references/agumon/canonical.png` — Digimon New Century 3D viewer screenshot. Vibrant yellow-orange, 2-3 zone hard cel, hard outline, white claws+teeth, chibi cute. **GROUND TRUTH**.
- `references/global/octopath2dhighdef.jpeg` — HD-2D pixel art density target.
- `references/global/celshadingzolo.jpg` — cel-shading 3-tone principle.

## Fix recipes (Agumon-specific)

### Pale / desaturated body (white/cream dominant)

1. Verify `--palette agumon.gpl` flag passed (or auto-detected by pipeline_run.py).
2. If palette enforced and still pale → palette source itself muted (e.g. K-means from Cyber Sleuth texture is yellow-tan). Hand-tune `palettes/agumon.gpl`: replace pale yellows with saturated oranges (`#F39A2E`, `#E08020`, `#C06010`).
3. Reduce rim-light additive in shader (mihoyo_style: `add_rim.Fac` 0.5→0.2).
4. Reduce lit-tint white in ColorRamp ((1.0,1.0,1.0)→(1.05, 1.0, 0.95) warm bias).
5. Bump posterize 4→3 for harder color zones.

### Black claws (CS source)

1. Hide outline mesh `chr050_2` completely (covers thin claw geometry as silhouette).
2. Use Freestyle silhouette+border (drawn only on perimeter, not over claws).
3. Add Z-position mask in shader: world Z < 0.15 → white-mix (foot claws are at low Z).
4. Source-asset fix (highest impact): paint claw UV regions WHITE in `chr050a01.png` directly. CS texture has tan claws, Adventure-style has white.

### Missing outline

1. CS: outline_meshes config must include `chr050_2` (legacy, now we hide and use Freestyle).
2. ReArise/NewCentury: no separate outline mesh → use `auto_solidify` + freestyle.

### CS-specific

- Mesh roles: `chr050_0` = body, `chr050_1` = specular/detail (mat `chr050spec_01`), `chr050_2` = outline mesh (mat `MTR_line01`).
- Hide `chr050_2` (and optionally `chr050_1`) when using Freestyle for outline.
