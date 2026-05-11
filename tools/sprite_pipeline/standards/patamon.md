# Standards — Patamon

Char-specific spec. Used together with `standards/global.md` (cross-char rules).

## Identity

- **Type**: Mammal / Holy-adjacent
- **Body color**: Vibrant orange-gold top, cream/white belly
- **Hue range**: 25-45° HSL (orange/gold). Out-of-range = global hard fail.
- **Saturation**: High. Pale washed-out brown = automatic ≤40 color score.
- **Eye**: Large, blue (usually circular)
- **Ears**: Oversized, wing-like, used for flight
- **Wings**: Small vestigial bird-like wings on the back (white/cream)

## Sources available

| Source | Config | Quality |
|--------|--------|---------|
| Cyber Sleuth | `patamon.json` | 2 meshes (chr096_0, chr096_1), good rig, full animations |

## Signature features (HARD CHECK)

These features MUST be visibly distinguishable in output. If absent / muddy / occluded → cap variant total at 70 regardless other criteria.

1. **Large Ears (Wings)** — The signature "wings" on the head. Must be clearly defined and separate from the body silhouette.
2. **Horizontal Color Split** — Clear boundary between the orange top head/back and the cream/white muzzle/belly.
3. **Small Back Wings** — Tiny white wings on the back should be visible in side/iso views.
4. **Blue Eyes** — Circular blue eyes must be visible and not just black dots.
5. **Chibi Proportions** — Round, ball-like body.

## Color targets

**Canonical hex anchors** (palette must include):
- `#D48D3F` — mid-orange (top body)
- `#B66F2A` — shadow orange
- `#FFF3E0` — cream/white belly
- `#E0D0C0` — shadow cream
- `#4A90E2` — eye blue
- `#000000` — outline black

Forbidden in output:
- Muddy brown or greyish orange.
- Pure white silhouette > 15% (rim/specular failure).

## Reference paths

- `references/patamon/patamon.webp` — Official artwork. Vibrant gold-orange, cream belly, blue eyes. **GROUND TRUTH**.
- `references/global/octopath2dhighdef.jpeg` — HD-2D pixel art density target.
- `references/global/celshadingzolo.jpg` — cel-shading 3-tone principle.

## Fix recipes (Patamon-specific)

### Pale / desaturated body

1. Verify `--palette patamon.gpl` flag passed (or auto-detected).
2. Hand-tune `palettes/patamon.gpl` to ensure saturated oranges are present.
3. Reduce lit-tint white bias in shader.

### Missing Horizontal Split

1. Ensure the texture `chr096a01.png` is correctly loaded.
2. If shading washes out the contrast, harden the ColorRamp to `CONSTANT` to preserve the texture's own color boundaries.

### Muddy Blue Eyes

1. If eyes look black, increase `Emission` or `Value` for the blue eye UV region.
2. Ensure `freestyle_outline` isn't drawing too many internal lines over the eyes.
