# Standards — Global (cross-character style rules)

Cross-character rules. Char-specific specs in `standards/<char>.md`.

## Naming convention

Configs: `<digimon>[-<source>][-<variant>].json`

Examples:
- `agumon.json` (default Cyber Sleuth source)
- `agumon-rearise.json` (ReArise source)
- `agumon-newcentury.json` (New Century source)
- `agumon-black.json` (variant of default source)
- `agumon-rearise-snow.json` (variant of ReArise source)
- `gabumon.json`, `patamon.json`, etc.

Output dirs mirror config name: `output/<full_name>/runs/...`.

Palettes: `palettes/<full_name>.gpl` (auto-detected by pipeline_run.py if exists). Falls back to `palettes/<digimon>.gpl` if variant-specific not present.

References per char: `references/<digimon>/canonical.<ext>` (signature ground-truth). Optional siblings (cel-shading.*, etc).

References global: `references/global/octopath2dhighdef.jpeg` + `celshadingzolo.jpg` (style targets).

## Dual-track outputs

Every variant produces BOTH tracks. Each variant assigned ONE primary track for scoring.

### TRACK A — Anime Cel-Shading (hi-res smooth)

**Output**: 512×512+ PNG native render, NO pixelify, NO downscale, NO palette quantize.

**Variants assigned**: `mihoyo_style`, `anime_eevee`, `threelevel`, `toon_bsdf`, `painterly`

**Mandatory:**
- ✅ EXACTLY 3 hard cel tones (shadow/mid/lit) via ColorRamp CONSTANT
- ✅ Smooth curved surfaces (Cycles direct, no pixel grid)
- ✅ Hard black outline 3-6px proportional to render res
- ✅ Saturated colors per `standards/<char>.md` palette spec
- ✅ No AA blur on outline
- ✅ Cel boundaries hard

**Forbidden:**
- ❌ Pixel grid visible
- ❌ Smooth gradient ramps (LINEAR/B_SPLINE)
- ❌ Pale white silhouette (rim/specular > 15% body interior)
- ❌ Outline jaggies
- ❌ Color drift away from char canonical hue

**Score anchors:**
- 90+: matches `references/<char>/canonical.*`: vibrant char color, 3 hard zones, clean outline
- 75-89: 3 zones visible, outline OK, color saturation good
- 60-74: 2 zones distinct, soft transitions, outline visible
- 40-59: gradient soft, weak outline OR desaturation
- <40: smooth blur, broken render, OR pixel-art mismatch

### TRACK B — Pixel Art HD-2D

**Output**: 96-256px PNG, NEAREST downscale + palette `<char>.gpl` strict quantize.

**Variants assigned**: `basic_cel`, `dithered`, `freestyle_outline`, `lospec_toolkit`, `astropulse`, `flat_emission`

**Mandatory:**
- ✅ ≥4 shading levels discrete
- ✅ Crisp pixel grid (no AA blur)
- ✅ Hard 1-2px outline silhouette
- ✅ Internal detail lines (signature features per char — Freestyle helps)
- ✅ Palette ~30-50 col enforced (`palettes/<char>.gpl`)
- ✅ Saturated but earthy (Octopath terra-cotta range)
- ✅ NEAREST downscale

**Forbidden:**
- ❌ Smooth surfaces (anime contamination)
- ❌ <4 shading zones
- ❌ AA-blurred edges
- ❌ Outline missing
- ❌ Auto median-cut palette
- ❌ Char silhouette <30% bbox area

**Score anchors:**
- 90+: matches `references/global/octopath2dhighdef.jpeg` density: 4-6 zones, internal lines, palette rich
- 75-89: ≥4 zones, outline+lines visible, palette match
- 60-74: 3 zones (less than HD-2D ideal), outline OK
- 40-59: 2 zones only, weak outline OR no internal lines
- <40: flat 1-zone, broken, OR anime mismatch

## Track contamination penalty

Variant locked to ONE track (assignment fixed). Mismatched output (anime variant produces pixel look or vice versa) → ×0.5 penalty.

## Per-variant primary track + max scores

| Variant | Track | Max score |
|---------|-------|-----------|
| basic_cel | B Pixel | 85 |
| dithered | B Pixel | 95 |
| freestyle_outline | B Pixel | 95 |
| lospec_toolkit | B Pixel | 95 |
| flat_emission | B Pixel | 65 |
| astropulse | B Pixel | 95 |
| toon_bsdf | A Anime | 95 |
| anime_eevee | A Anime | 95 |
| threelevel | A Anime | 95 |
| mihoyo_style | A Anime | 95 |
| painterly | A Anime | 95 |
| matcap | (source-specific) | 90 |

## Visual references

### `references/global/octopath2dhighdef.jpeg` — Pixel Art HD-2D

Octopath Traveler character. Style target for **pixel-art-leaning variants**:
- HD-2D pixel art high detail density (NOT 8-bit minimal)
- Realistic anime proportions
- 4-6 level shading visible (shadow_deep/shadow/mid/lit/highlight)
- Black outline silhouette + internal detail lines
- Palette ~30-50 col per char
- Pixel grid crisp ma "paintery" feel
- ~80-100px tall typical

**Variants aligned**: `basic_cel`, `dithered`, `freestyle_outline`, `astropulse`, `lospec_toolkit`, `flat_emission`

### `references/global/celshadingzolo.jpg` — Anime Cel-Shading principles

Zoro (One Piece) illustration. Style cel-shading **principles**:
- EXACTLY 3 hard tones (highlight/mid/shadow)
- Hard boundaries (NO gradient)
- Saturated anime colors
- Outline nero netto silhouette + internal cleanup
- Smooth curved hi-res surfaces (NOT pixel grid)

**Variants aligned**: `toon_bsdf`, `anime_eevee`, `threelevel`, `mihoyo_style`, `painterly`

### `references/<char>/canonical.<ext>` — Char-specific ground truth

Per-char visual ground truth. Defines:
- Body color hue / palette
- Signature features (claws, ears, fins, etc — see char file)
- Proportions (chibi / realistic / etc)
- Mood (cute / aggressive / etc)

**Subagent compares against this** for char-specific scoring.

## Per-shader specs

### 1. `basic_cel` — Posterize emission + outline

Pixel art crisp con palette posterizzata. Char body color in bande discrete (luminanza step). Outline nero 1-2px su silhouette. NO smooth gradient. NO rim white.

Mandatory: outline visible, color zones discrete, hue match canonical, pixel grid crisp.
Forbidden: white-dominated body, rim light prominente, gradient continuo, outline rotto.
Score: 90+: 4 bands + outline perfect + saturo. 75-89: 3 bands. 60-74: 2 bands thin.

### 2. `toon_bsdf` — Cycles Lambert + ColorRamp 3-band emission

3 livelli cel-shading hard-stepped. Lambert dot + ColorRamp CONSTANT. Body cool-shadow + warm-lit.

Mandatory: 3 zones distinct, hard boundaries, outline, body NOT black, char facing right.
Forbidden: black body (lighting broken), smooth gradient, color drift to mustard.

### 3. `anime_eevee` — Eevee Diffuse + 2-3 ColorRamp + thicker outline

Anime classic. Eevee. Outline mesh scaled 1.03x. Manga-like.

Mandatory: outline thicker than basic_cel, cel bands hard, Eevee compat, anime warmth.
Forbidden: linear ramp, outline thin, mustard desat.

### 4. `threelevel` — Cycles 3-band cel + Fresnel rim + outline 1.04x

3 livelli + rim light Fresnel sui bordi (subtle). Più polished.

Mandatory: 3 zones, rim subtle (≤10% body white), outline thicker, saturation.
Forbidden: rim overpowering, body desat, rim assente.

### 5. `flat_emission` — Pure texture emission (max 65)

Solo texture color piatto, NO shading. Outline. Baseline.

Mandatory: uniform color, outline, texture preserved.
Forbidden: cel bands, color shift heavy.
NOTA: max 65 perché manca cel-shading.

### 6. `astropulse` — BlenderToPixels compositor template

Pixel art dithered (Bayer pattern). Compositor multi-color ramp. Output DIVERSO da basic_cel.

Mandatory: dithering visible, output ≠ flat_emission, compositor engaging.
Forbidden: identical to flat_emission (compositor failed), empty frame.

### 7. `mihoyo_style` — HSR-inspired 2-band Lambert + Fresnel rim + outline 1.05x

HSR look: 2 bande, rim prominente ma controllato, outline thick.

Mandatory: 2 zones, rim VISIBLE, outline thick, saturation high.
Forbidden: rim dominante (white >15%), desat, outline thin, 3 livelli.
NOTA CRITICA: facile finire white silhouette → cap 60 if body white >15%.

### 8. `dithered` — Bayer pattern via Magic texture + 2-band

Pattern dithering visibile. Magic Blender screen-space. Effetto retrò.

Mandatory: dithering pattern, 2-band underlying, outline, body color visible under.
Forbidden: pattern absent (looks flat/basic), troppo dense, color drift.

### 9. `freestyle_outline` — Blender Freestyle line render + 4-level posterize

Freestyle vector lines invece inverted-hull. Linee silhouette/crease/border. Body 4-band posterize.

Mandatory: Freestyle lines NOT empty, crease+silhouette visible, 4-band, NO inverted-hull mesh.
Forbidden: render empty, lines absent, lines >3px.

### 10. `painterly` — Soft B-spline ramp + dark-navy outline

Look painterly NON hard-cel. B_SPLINE (soft). Outline dark-navy. Watercolor feel.

Mandatory: soft transitions (NOT hard), outline navy, distinct from hard-cel.
Forbidden: hard CONSTANT bands, outline pure black, identical to anime_eevee.

### 11. `lospec_toolkit` — Lospec compositor template

Retro pixel via Lospec compositor (ditherer + shadeless + Lospec palette). Output DIVERSO da flat_emission.

Mandatory: output ≠ flat_emission, Lospec dither/palette, outline, retro feel.
Forbidden: identical to flat_emission (compositor failed), empty, full-saturated.

### 12. `matcap` — Sphere-mapped texture (source-specific)

Used when source texture is matcap (sphere-mapped, NOT UV). Texture lookup via camera-space normal projection.

Mandatory: body color visible (texture loaded correctly), no UV-black render.
Forbidden: all-black render (Reflection coord broken), texture absent.

## Per-camera modifiers

Score multiplied by camera fit:

| Camera | Multiplier | Reason |
|--------|-----------|--------|
| `iso_threequarter` | ×1.0 | sweet spot HSR combat |
| `side_right`, `side_left` | ×1.0 | profilo classico |
| `iso_45` | ×0.95 | iso classic JRPG |
| `iso_30` | ×0.85 | top-down troppo schiaccia chibi |
| `front`, `back` | ×0.7 | char proporzioni difficili da front/back |

## Global hard fails

Independent of variant, AUTOMATIC max 30 score if:

1. Render all-black or all-white
2. Char silhouette <5% non-transparent pixels
3. Char facing wrong direction (snout sinistra in side_right)
4. Char sotto-piedi (camera below)
5. Char schiacciato (iso pitch troppo estremo)
6. Color hue out of `standards/<char>.md` declared range

## Decision rubric

```
score = base_per_variant_spec × camera_multiplier × global_modifier

base_per_variant_spec: apply spec section above per variant
camera_multiplier: per-camera table
global_modifier: 1.0 normal; 0.3 if any global hard fail
```

SUCCESS verdict (best_score ≥ 80) requires:
- ≥3 variants score ≥ 75 (consistent, not lucky one-shot)
- Best variant ≥ 80 + passes all mandatory for its spec
- No regression vs previous iteration

If only 1 variant ≥ 80 but mandatory broken → NEEDS_ITERATION not SUCCESS.
