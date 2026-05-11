# Standards — Tentomon

Char-specific spec. Used together with `standards/global.md` (cross-char rules).

## Identity

- **Type**: Insect (Beetle-like)
- **Body color**: Crimson Red (`#C83232` canonical)
- **Hue range**: 0-20° HSL (Red). Out-of-range = global hard fail.
- **Saturation**: High.
- **Eye**: Grey/Greenish-grey with "pixel" or cross-hatch pattern.
- **Claws**: Large, grey/metallic.
- **Legs**: 6 total (Fore-legs, Middle-legs, Hind-legs).
- **Wings**: Green (when open).

## Sources available

| Source | Config | Quality |
|--------|--------|---------|
| Cyber Sleuth | `tentomon.json` | 6 legs, good detail, 4 arms + 2 legs structure |

## Signature features (HARD CHECK)

These features MUST be visibly distinguishable in output. If absent / muddy / occluded → cap variant total at 70 regardless other criteria.

1. **6 Legs / 4 Arms** — Tentomon has 6 limbs total. At least 4 should be visible in three-quarter view.
2. **Large Grey Claws** — Claws on all limbs must be visible and distinct from the red body.
   - Hand claws: Large, single/multiple points.
   - Foot claws: Large, grounding the character.
3. **Patterned Eyes** — Grey eyes should have a subtle green tint or visible cross-hatch pattern (not just solid black/white).
4. **Antennae** — Long antennae extending from the head.
5. **Red Shell** — Segmented beetle-like shell on the back.

## Color targets

**Canonical hex anchors** (palette must include):
- `#C83232` — Crimson (Body)
- `#8B0000` — Dark Red (Shadow)
- `#FF4B4B` — Light Red (Highlight)
- `#808080` — Grey (Claws/Antennae)
- `#647864` — Greenish Grey (Eyes)
- `#00FF00` — Green (Wings)
- `#000000` — Outline black

Forbidden in output:
- Pinkish or Orange-ish red dominating body.
- Pure white silhouette > 15% (rim/specular failure).

## Reference paths

- `references/tentomon/tentomon.webp` — Reference image. Red shell, grey claws, patterned eyes. **GROUND TRUTH**.
- `references/global/octopath2dhighdef.jpeg` — HD-2D pixel art density target.
- `references/global/celshadingzolo.jpg` — cel-shading 3-tone principle.

## Fix recipes (Tentomon-specific)

### Pale / Pinkish body

1. Verify `--palette tentomon.gpl` flag passed.
2. If body looks pink, shift Red hue towards 0° and increase saturation.
3. Reduce lit-tint white in ColorRamp to avoid washing out the red.

### Missing Claws / Muddy limbs

1. Use `freestyle_outline` to define limb boundaries.
2. Ensure grey color for claws is distinct enough from red body shadow.
3. If claws are too small, check if `auto_solidify` is covering them.

### All-black eyes

1. Increase emission or change eye material to ignore some shadows.
2. Ensure the eye texture (cross-hatch) is properly mapped.
