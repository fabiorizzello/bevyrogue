# Standards — Renamon

Char-specific spec. Used together with `standards/global.md` (cross-char rules).

## Identity

- **Type**: Beast Man (fox)
- **Body color**: Golden yellow (`#E6B800` canonical)
- **Hue range**: 40-60° HSL (yellow/gold). Out-of-range = global hard fail.
- **Saturation**: high. Pale yellow = automatic ≤40 color score.
- **Gloves**: Purple (`#800080`)
- **Belly/Tail tip**: White/Cream (`#FDF5E6`)
- **Eye**: Light Blue

## Sources available

| Source | Config | Quality |
|--------|--------|---------|
| Digimon Links | `renamon.json` | chr391.glb, good rig, full animations |

## Signature features (HARD CHECK)

These features MUST be visibly distinguishable in output. If absent / muddy / occluded → cap variant total at 70 regardless other criteria.

1. **Purple sleeves/gloves** — MUST be saturated purple, clearly separate from yellow arms.
2. **Bushy tail (white-tipped)** — large tail must be visible, with a clear color transition to white at the tip.
3. **Yin-yang symbols** — white/black circular symbols on the purple gloves.
4. **Pointy fox ears** — tall, upright ears.
5. **Slender proportions** — should look tall and lean compared to Rookie peers like Agumon.

## Color targets

**Canonical hex anchors** (palette must include):
- `#E6B800` — Golden yellow (Body)
- `#800080` — Purple (Gloves)
- `#FDF5E6` — Old Lace / Cream (Belly/Tail tip)
- `#000000` — Outline black
- `#FFFFFF` — Yin-yang white
- `#5DADE2` — Eye blue

Forbidden in output:
- Desaturated "dirty" yellow dominating body.
- Pure white silhouette > 15% (rim/specular failure).

## Reference paths

- `references/renamon/renamon.png` — Canonical reference.
- `references/global/octopath2dhighdef.jpeg` — HD-2D pixel art density target.
- `references/global/celshadingzolo.jpg` — cel-shading 3-tone principle.

## Fix recipes (Renamon-specific)

### Pale body / Desaturated Purple

1. Verify `renamon.gpl` palette usage.
2. Hand-tune `palettes/renamon.gpl`: boost saturation of yellows and purples.
3. Reduce lit-tint white in ColorRamp.

### Muddy Yin-yang symbols

1. Increase texture resolution or use a sharper shader variant.
2. Ensure `freestyle_outline` isn't so thick it obscures hand details.
