# Standards — Dorumon

Char-specific spec. Used together with `standards/global.md` (cross-char rules).

## Identity

- **Type**: Furry dragon beast
- **Body color**: Light purple/lavender body with darker purple stripes and tufts
- **Hue range**: 260-290° HSL (purple). Out-of-range = global hard fail.
- **Saturation**: moderate. Pale grey-lavender = automatic ≤40 color score.
- **Eye**: black/dark with white highlight
- **Forehead**: red diamond interface
- **Tail**: large, bushy, purple

## Sources available

| Source | Config | Quality |
|--------|--------|---------|
| Cyber Sleuth | `dorumon.json` | 4 meshes (chr112_0-3), chr112_3 is outline-hull, good rig, multiple actions |

## Variants

- `dorumon-black` (Death-X-DORUmon style) — TODO

## Signature features (HARD CHECK)

These features MUST be visibly distinguishable in output. If absent / muddy / occluded → cap variant total at 70 regardless other criteria.

1. **Brow Interface** — red diamond/triangle on forehead, signature
   - Must be clearly red and visible as a distinct shape on the brow.
2. **Fur Tufts** — large darker purple tufts on ears and cheeks
   - Must show distinct volume and darker purple color.
3. **Claws (hands AND feet)** — white/cream, oversized
   - Hand claws: ≥2-3 white pixel groupings per hand visible
   - Foot claws: ≥2-3 white pixel groupings per foot visible
4. **Bushy Tail** — large purple tail behind body
5. **Stripes** — darker purple stripes on limbs/back

## Color targets

**Canonical hex anchors** (palette must include):
- `#A890C8` — mid-lavender (body)
- `#705090` — dark purple (tufts/stripes)
- `#E02020` — interface red
- `#FFFFFF` — claw pure white
- `#000000` — eye/outline black

Forbidden in output:
- Pale grey `#D0D0D0` dominating body (desaturation failure)
- Pure white silhouette > 15% (rim/specular failure)

## Reference paths

- `references/dorumon/dorumon.png` — Canonical reference.
- `references/global/octopath2dhighdef.jpeg` — HD-2D pixel art density target.
- `references/global/celshadingzolo.jpg` — cel-shading 3-tone principle.

## Fix recipes (Dorumon-specific)

### Pale / desaturated body

1. Verify `--palette dorumon.gpl` flag passed (or auto-detected).
2. Hand-tune `palettes/dorumon.gpl`: ensure lavender `#A890C8` and dark purple `#705090` are present and saturated.
3. Reduce lit-tint white in ColorRamp.

### Missing Interface Red

1. Ensure red `#E02020` is in the palette.
2. If interface too small, ensure `freestyle_outline` isn't covering it (reduce thickness).
