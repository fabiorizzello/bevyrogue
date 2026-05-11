# Standards — Gabumon

Char-specific spec. Used together with `standards/global.md` (cross-char rules).

## Identity

- **Type**: reptilian with beast pelt
- **Body color**: pale yellow/tan (under pelt)
- **Pelt color**: white with blue stripes
- **Hue range**: 40-60° HSL (yellow/tan) for body; 190-230° HSL (blue) for stripes.
- **Horn**: single yellow horn with markings
- **Belly**: pink with purple/blue pattern

## Sources available

| Source | Config | Quality |
|--------|--------|---------|
| Cyber Sleuth | `gabumon.json` | Good rig, multiple meshes |
| Links | `gabumon-links.json` | Single mesh (chr151), good rig |

## Signature features (HARD CHECK)

These features MUST be visibly distinguishable in output. If absent / muddy / occluded → cap variant total at 70 regardless other criteria.

1. **Horn** — single yellow horn on head, signature
   - Test: zoom 200%, horn shape must be clear and distinct from pelt.
2. **Garurumon Pelt** — white fur covering back and head
   - Must be white/light grey, distinct from yellow body.
3. **Pelt Stripes** — blue stripes on the white pelt
   - At least 2-3 blue stripes should be visible on the back/sides.
4. **Belly Pattern** — pink/purple pattern on abdomen
   - Must show a distinct color break from the yellow body.
5. **Claws** — three claws on hands and feet
   - Should be visible as white/cream points.

## Color targets

**Canonical hex anchors** (palette must include):
- `#FFF3B0` — body yellow (light)
- `#F7D57B` — body yellow (mid)
- `#FFFFFF` — pelt white
- `#4080FF` — pelt stripe blue
- `#FF80C0` — belly pink
- `#8040FF` — belly pattern purple
- `#FFE0B0` — claw highlight (cream/white)
- `#000000` — outline black

## Reference paths

- `references/gabumon/gabumon.png` — Canonical reference.
- `references/global/octopath2dhighdef.jpeg` — HD-2D pixel art density target.
- `references/global/celshadingzolo.jpg` — cel-shading 3-tone principle.

## Fix recipes (Gabumon-specific)

### Muddy stripes

1. Increase stripe color saturation in palette.
2. Ensure `posterize` level isn't merging stripes into white pelt (use 4 levels).
3. If stripes are too thin, they might disappear in pixel track; ensure `ortho_scale` is tight.

### Missing Belly Pattern

1. If the belly is occluded by pose, try different camera angles.
2. Ensure pink/purple are present in the palette.

### Horn blending with Pelt

1. Horn yellow must be distinct from Pelt white.
2. Outline should separate horn from head pelt.
