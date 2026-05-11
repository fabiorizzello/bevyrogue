# External Tools — FBX → Pixel Art

Catalogo di tool online/desktop per convertire 3D models (FBX, GLB, OBJ, DAE) in pixel art sprites. Verificati 2026-04. Quality marks soggettivi basati su review + screenshot.

## TL;DR

| Goal | Tool consigliato |
|------|------------------|
| Free + Blender + best-supported | **Astropulse Blender to Pixels** |
| Free + standalone GUI | **PIXELARTOR** |
| Free + simple Blender addon | **Pixelize (LeonardoDocs)** |
| Paid + most polished | **PixelOver** |
| Paid + multi-direction sprite + bones | **PixelOver** |
| Free + Eevee anime shine effects | **Lucas Roedel addon (Tramshy fork)** |

---

## Categoria A — Blender plugins/addons (compositor + shader)

### **Astropulse — Blender to Pixels** ⭐⭐⭐⭐⭐

| | |
|---|---|
| Type | Blender file with compositor nodes |
| Cost | Free (name your own price, 0$ OK) |
| Source | https://astropulse.itch.io/blender-to-pixels |
| Mirror | https://astropulse.gumroad.com/l/BlenderToPixels |
| Platform | Windows, macOS, Linux |
| Size | 327 KB |
| Rating | 5.0/5 (8 ratings su itch.io) |
| Maintainer | Astropulse (creator Retro Diffusion) |

**Features**:
- Compositor-based (post-render, no special model setup)
- Up to 8 color ramps per material zone
- Automatic Bayer dithering
- Customizable outline thickness/color
- Depth-based fog filter
- Special effects filters
- In-editor documentation
- Works with any model (no Python addon required)

**Workflow**:
1. Download `BlenderToPixels.zip` da itch.io.
2. Estrai → file `.blend` con compositor pre-configurato.
3. Import tuo modello nello scene template oppure copia compositor nodes nel tuo file.
4. Render via compositor → output pixel art.

**Pro**: Battle-tested, free, version-agnostic (Blender 3.x/4.x/5.x), no install procedure.

**Contro**: Manual download itch.io (no direct curl, gated dietro purchase form anche gratis).

**Integration con nostra pipeline**: appendere compositor scene via `bpy.data.libraries.load()` o copiare nodes manualmente in `blender_render.py`.

---

### **Lucas Roedel — Pixel Art Rendering** (Tramshy fork) ⭐⭐⭐⭐

| | |
|---|---|
| Type | Python addon Blender installabile |
| Cost | Free |
| Source | https://github.com/Tramshy/pixel-art-addon-mod |
| Original | https://lucasroedel.gumroad.com/l/pixel_art |
| Platform | Blender 4.0-4.5 (5.x untested) |
| Maintainer | Tramshy (fork attivo) |

**Features**:
- Eevee-based pixel render
- Bayer dithering
- Multi-light support
- Anime shine effects (customizable opacity/reflection)
- Particle pixelated smoke effects

**Install**:
```bash
# Download zip → Blender Preferences > Add-ons > Install
# Enable "Pixel Art Rendering"
```

**Pro**: Anime-style cel shading + shine effects integrato.

**Contro**: Recommends Blender 4.1 per single-sample clean output. 5.x potential issues. Eevee-only (no Cycles).

---

### **Pixelize** (LeonardoDocs) ⭐⭐⭐

| | |
|---|---|
| Type | Python addon Blender |
| Cost | Free, open source MIT |
| Source | https://github.com/LeonardoDocs/Pixelize |
| Language | Python (commenti in Italian, EN translation in progress) |

**Features**:
- Resolution downscale + pixel scale control
- Color quantization retro palette
- Pixel-style lighting effects
- Game-ready sprite export

**Pro**: Open source, simple, codebase leggibile.

**Contro**: Doc minimale (no animation/dithering details). Non production-grade rispetto Astropulse.

---

### **Pixel Art Renderer** ⭐⭐⭐

| | |
|---|---|
| Type | Blender addon |
| Cost | Likely free |
| Source | https://blender-addons.org/pixel-art-renderer/ |

**Features**:
- Real-time pixel art preview in viewport
- Stylized controls

**Pro**: Real-time feedback durante design.

**Contro**: Less feature-rich di Astropulse, viewport-focused.

---

### **Sprytile** ⭐⭐⭐

| | |
|---|---|
| Type | Blender addon, tile mapping |
| Cost | Free / Pay what you want itch.io |
| Source | https://jeiel.itch.io/sprytile |

**Features**:
- Tile mapping in Blender 3D
- Powerful per environment design pixel-art-style

**Use case**: Più adatto a level/environment design, NON character sprite render.

---

## Categoria B — Standalone tools (no Blender required)

### **PixelOver** ⭐⭐⭐⭐⭐ (paid, most polished)

| | |
|---|---|
| Type | Standalone desktop app |
| Cost | Paid (~$15-20) |
| Source | https://pixelover.io/ |
| Platform | Windows, macOS, Linux |

**Features**:
- Import 3D models direttamente
- Camera presets + auto-rotate per export multi-direction
- Bones + key animation system
- Pixel-perfect transformations
- Style transfer + outlining + dithering options
- Aseprite-like editing post-import

**Pro**: Most professional, multi-direction nativo, bones rig support.

**Contro**: Paid, separate app (no Blender pipeline integration).

**Quando usare**: Indie con budget piccolo, vuole tool dedicato no Blender.

---

### **PIXELARTOR** ⭐⭐⭐⭐

| | |
|---|---|
| Type | Standalone Electron app |
| Cost | Free, MIT open source |
| Source | https://github.com/Chleba/PIXELARTOR |
| Language | JavaScript (99.3%) |
| Platform | Windows, macOS, Linux (build da source) |

**Features**:
- Import GLTF, FBX
- Camera modes: orthogonal + projective
- Multi-direction angle rendering
- Animation playback dal file 3D
- Export GIF + ZIP sprite frames
- Outline effects + 3 light types (hemisphere/direct/point) + shadows
- Customizable canvas

**Install**:
```bash
git clone https://github.com/Chleba/PIXELARTOR
cd PIXELARTOR
yarn install && yarn dist
# Or download release binary
```

**Pro**: Free, MIT, multi-direction nativo, animation support.

**Contro**: No palette/dithering esplicito (basic conversion). UI Electron meno polished di Blender plugins.

---

### **SpriteStack.io** ⭐⭐⭐

| | |
|---|---|
| Type | Web/desktop hybrid |
| Cost | Paid |
| Source | https://spritestack.io/ |

**Features**:
- Voxel + low-poly + sprites in unified workflow
- Animate 3D objects → 2D spritesheet retro renderer
- Import existing 3D/2D assets

**Pro**: Specialized retro look out-of-box.

**Contro**: Voxel-centric, less suited per FBX rigged Digimon.

---

### **Astropulse — Pixeldetector** ⭐⭐⭐ (utility)

| | |
|---|---|
| Type | Python script standalone |
| Cost | Free open source |
| Source | https://github.com/Astropulse/pixeldetector |
| Stars | 341+ |

**Features**:
- Restore pixel art images degraded da JPEG compression
- Auto-palette reduction
- Dependencies: Pillow, Numpy, Scipy

**Use case**: Post-process CLEANUP, **NON conversion 3D→pixel**. Combina con altri tool.

---

## Categoria C — AI-based services (online)

### **Pixa.com — 3D to Pixel Art** ⭐⭐⭐

| | |
|---|---|
| Type | Web service AI |
| Cost | Freemium |
| Source | https://www.pixa.com/create/3d-model-to-pixel-art-converter |

**Features**:
- Upload render del 3D → AI converte in pixel art
- Specifica bit-depth (8-bit, 16-bit, etc.)
- Custom palette selection
- Outlines, dithering opzionali

**Pro**: Zero setup, pure web tool.

**Contro**: AI-based = identity drift possibile, output meno predictable di tool deterministici.

---

### **Layer.ai Image to Sprite** ⭐⭐⭐⭐

| | |
|---|---|
| Type | Web tool + REST API |
| Cost | Pay-per-use (~$0.10-0.50/sprite) |
| Source | https://www.layer.ai/tools/layer--image-to-sprite |

**Features**:
- Upload qualsiasi immagine (incluso 3D render)
- Preserves silhouette + core features
- API access disponibile
- Batch processing

**Pro**: Reference-based fidelity buona, API per automation.

**Contro**: Paid, AI può drift su batch grandi.

---

### **PixelLab** ⭐⭐⭐⭐

| | |
|---|---|
| Type | Web + MCP server (Claude Code/Cursor compatible) |
| Cost | Subscription (~$10-15/mo) o credit pack |
| Source | https://www.pixellab.ai/ |
| MCP | https://api.pixellab.ai/mcp |

**Features**:
- Standard mode: skeleton template humanoid/quadruped (cheap)
- Pro mode: AI reference-based con upload (caro)
- Animation templates (idle/walk/attack/etc) skeleton-rigged
- Multi-direction (4/8 dir auto)

**Pro**: MCP integration nativa Claude Code, animation auto.

**Contro**: Standard mode skeleton humanoid forza pose bipede (Agumon T-rex stance non ideale).

---

## Categoria D — Workflow generators (text-to-asset)

### **PixelLab AI**, **AutoSprite**, **Ludo.ai**, **God Mode AI**, **SEELE AI**

Tutti AI-based text/image-to-pixel-sprite. Coperti in dettaglio in [`docs/sprite_pipeline_evaluation.md`](../../docs/sprite_pipeline_evaluation.md). NON specifici per FBX→pixel quindi marginal per use case 3D model conversion.

---

## Comparison matrix

| Tool | Free? | Blender? | Multi-dir | Animation | Outline | Dither | Setup |
|------|-------|----------|-----------|-----------|---------|--------|-------|
| Astropulse Blender to Pixels | ✅ | ✅ | manual | ✅ via Blender | ✅ | ✅ | medium |
| Lucas Roedel addon | ✅ | ✅ | manual | ✅ via Blender | partial | ✅ | easy |
| Pixelize | ✅ | ✅ | manual | partial | partial | partial | easy |
| Sprytile | ✅ | ✅ | tile-only | ❌ | ❌ | ❌ | easy |
| PixelOver | ❌ | no | ✅ auto | ✅ | ✅ | ✅ | easy |
| PIXELARTOR | ✅ | no | ✅ auto | ✅ | ✅ | ❌ | medium |
| SpriteStack | ❌ | no | ✅ | ✅ | ✅ | ✅ | easy |
| Pixa.com | freemium | no | ❌ | ❌ | ✅ | ✅ | easy |
| Layer.ai | ❌ | no | ❌ | ❌ | ✅ | ✅ | easy |
| PixelLab | ❌ | no | ✅ | ✅ skeleton | ✅ | ❌ | easy |
| **Custom pipeline** (this repo) | ✅ | ✅ | configurable | ✅ | ✅ cel-shade | ✅ | done |

---

## Decision tree

```
Hai modello 3D (FBX/GLB) e vuoi pixel art?
├─ Vuoi Blender + free + best polish?
│    → Astropulse Blender to Pixels
├─ Vuoi standalone GUI + free + multi-direction?
│    → PIXELARTOR
├─ Vuoi paid + bones/animation polish?
│    → PixelOver
├─ Vuoi AI + zero setup?
│    → Pixa.com / Layer.ai
└─ Vuoi pipeline custom CI-friendly headless?
     → Pipeline custom (questo repo) — già implementato
```

---

## Note workflow custom (questo repo)

La nostra pipeline (`scripts/`) implementa già:
- ✅ FBX/GLB/OBJ/DAE import
- ✅ Auto-camera bbox-based
- ✅ Cel-shaded textured material + inverted-hull outline (replica Cyber Sleuth toon shading)
- ✅ AA disabled + samples=1 per crisp pixel
- ✅ Per-frame animation bbox sampling
- ✅ 8 stili pre-set tramite `multi_style.sh`
- ✅ Custom palette `.gpl` enforcement
- ✅ JSON atlas Bevy-ready

Vantaggio vs tool esterni: **headless + scriptable + git-friendly** (configs JSON tracciabili). CI/CD integration possibile.

Limitazione: cel-shading look base. Per refinement avanzato (Bayer dither, depth fog, advanced color ramps) integrare Astropulse compositor o Lucas Roedel addon.

---

## Plugin integration TODO

Drop addon in `plugins/` e integrare via:

```python
# In blender_render.py — load Astropulse compositor template
bpy.ops.wm.append(
    filepath=f"{plugins_dir}/BlenderToPixels.blend/Scene/PixelArtScene",
    directory=f"{plugins_dir}/BlenderToPixels.blend/Scene/",
    filename="PixelArtScene"
)
```

Oppure copiare manualmente compositor nodes da `.blend` template a tuo file via Python API.

---

## References

- [docs/sprite_pipeline_evaluation.md](../../docs/sprite_pipeline_evaluation.md) — Full workflow analysis
- [README.md](README.md) — Pipeline custom usage
- [Astropulse on X](https://x.com/RealAstropulse) — Latest pixel art tools updates
- [Cody Claus (Astropulse)](https://astropulse.co/) — Tool collection homepage
