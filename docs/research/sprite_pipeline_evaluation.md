# Sprite Pipeline Evaluation — bevyrogue

> Valutazione alternative AI gen + workflow per asset Digimon fan-game.
> Scope: 12 Digimon (6 rookie + 6 adult), idle + 3 attack ognuno, target pixel art roguelite.

**Data**: 2026-04-28
**Hardware**: AMD RX 6700 XT 12GB, Linux ROCm.
**Constraints**: budget basso, indie solo dev, fan-game (non commercial).

---

## TL;DR

| Goal | Path consigliato |
|------|------------------|
| **100% fidelity Digimon** | Rip sprite ufficiali Re:Digitize/World 3 + Aseprite tween |
| **A-tier indie production** | Hybrid: rip + AnimateDiff locale + PixelLab MCP per skeleton anim |
| **Solo AI gen budget zero** | SD locale + LoRA Digimon CivitAI + Aseprite cleanup |
| **Solo AI gen plug-and-play** | Nano Banana Pro MCP + PixelLab MCP + Aseprite MCP |

---

## 1. Tier qualità achievable

### S Tier — AAA-comparable
**Solo via**: pixel artist umano commissioned (~$2400-3600 per 12 char) o team dedicato.
AI gen 2026 plateau ~80% AAA per pixel art coherenza.

### A Tier — Indie production-grade ⭐ TARGET
**Achievable** con AI + cleanup manuale + LoRA tuo.
Comparabile Aethermancer, Cassette Beasts, Cobalt Core.

### B Tier — Prototyping
**Achievable** con AI plug-and-play, no setup tecnico.
Comparable indie demo, jam game.

---

## 2. AI Generation services

### 2.1 — PixelLab MCP

**Tipo**: Native MCP integration Claude Code.

**Capabilities**:
- `create_character` (text-to-sprite, skeleton template humanoid/quadruped)
- `animate_character` (template anim su char esistente)
- Pro mode (web only): AI reference-based, 8-dir auto

**Quality**:
- Sprite gen standard: ⭐⭐⭐ rough, skeleton humanoid forza anatomia bipede umana
- Sprite gen Pro (web upload): ⭐⭐⭐⭐ ottimo
- Animation skeleton-rigged: ⭐⭐⭐⭐⭐ best in class

**Limitazione critica**: standard mode skeleton humanoid = **NON adatto Agumon-T-rex** (output forzato bipede umano, no T-rex stance).

**Costo**:
- Standard: 1 gen/sprite (~$0.05)
- Pro: 20-40 gen/sprite (~$2-5)
- Animation template: 1 gen/direction
- Subscription mensile o credit pack

**Use case ideale**:
- Animation skeleton-rigged su char esistente (idle, walk, attack templates).
- NON character creation Digimon-specific (skeleton limit).

**Verdict**: ⭐⭐⭐⭐ per anim, ⭐⭐ per char custom Digimon.

---

### 2.2 — Nano Banana Pro (Gemini 3 Pro Image)

**Tipo**: Google AI image gen, accesso via:
- Gemini API native
- Gemini CLI extension `nanobanana`
- MCP servers: `shinpr/mcp-image`, `YCSE/nanobanana-mcp`, `freema/pixelforge-mcp`

**Capabilities**:
- Text-to-image
- Image editing multi-turn
- Reference image upload (up to 14 ref + 5 char identity)
- Best-in-class character consistency

**Quality**:
- Identity preservation: ⭐⭐⭐⭐⭐ leader categoria
- Style coerenza batch: ⭐⭐⭐⭐⭐
- Pixel art native: ⭐⭐ (genera illustration, downscale needed)
- Multi-turn editing: ⭐⭐⭐⭐⭐

**Costo**: ~$0.13/img Pro, $0.04/img Flash (NB2).

**Use case ideale**:
- Char gen alta-fedeltà con reference upload.
- Multi-pose stesso char identity-locked.
- Concept generation.

**Verdict**: ⭐⭐⭐⭐⭐ per gen char illustration, ⭐⭐⭐ per pixel art finale (cleanup needed).

---

### 2.3 — Stable Diffusion locale + LoRA stack

**Tipo**: Self-hosted, ComfyUI/A1111 + ROCm.

**Stack**:
- Base: Pony Diffusion XL o SDXL.
- Char LoRA: Agumon LoRA, Gabumon LoRA, etc (CivitAI free).
- Style LoRA: PixelArtRedmond, PixelWaveSDXL.
- ControlNet: OpenPose (pose lock).
- IPAdapter: identity transfer.
- AnimateDiff: motion module per cycle.

**Quality**:
- Char fidelity con LoRA: ⭐⭐⭐⭐⭐
- Pose control: ⭐⭐⭐⭐⭐ (ControlNet best)
- Style coerenza con LoRA tuo: ⭐⭐⭐⭐⭐
- Animation cycle (AnimateDiff): ⭐⭐⭐⭐ idle/walk, ⭐⭐⭐ attack
- Pixel art native (con PixelArt LoRA): ⭐⭐⭐⭐

**Costo**:
- Setup: 2-4h una-tantum (ROCm + ComfyUI).
- Gen ongoing: $0 (electricity).
- LoRA training tuo: ~$0.50 locale, $1-2 cloud RunPod.

**Performance RX 6700 XT**:
- SD 1.5 512×512: 3-5s/img.
- SDXL 1024×1024: 18-25s/img.
- AnimateDiff 16-frame 512px: 30-90s.
- LoRA training SDXL: 1.5-2h.

**Use case ideale**:
- Batch generation infinite, $0 ongoing.
- Style coerenza tutto progetto via LoRA tuo.
- Pose control preciso ControlNet.
- Tinkerer-friendly.

**Verdict**: ⭐⭐⭐⭐⭐ char + style, ⭐⭐⭐ animation skeleton-rigged.

---

### 2.4 — Layer.ai Image-to-Sprite

**Tipo**: Web tool + REST API. Community MCP esiste (low maintenance).

**Capabilities**:
- Image upload → pixel art sprite
- Preserves silhouette + core features
- Pay-per-use, no subscription

**Quality**:
- Reference fidelity: ⭐⭐⭐⭐ buono
- Pixel art native: ⭐⭐⭐⭐
- Animation: limitata

**Costo**: ~$0.10-0.50/sprite.

**Verdict**: ⭐⭐⭐⭐ alternativa a PixelLab Pro per char gen reference-based.

---

### 2.5 — AutoSprite

**Tipo**: Web tool. Upload sprite singolo → genera moveset.

**Capabilities**:
- Upload single sprite → seleziona moveset (idle/walk/attack)
- Export Unity/Godot/Phaser ready
- Preserves char design tra frame

**Quality**:
- Char preservation: ⭐⭐⭐⭐ ottimo
- Animation moveset: ⭐⭐⭐⭐
- Pixel art: ⭐⭐⭐⭐

**Costo**: pay-per-use, ~$1-3/anim set.

**Use case ideale**:
- **PERFETTO per workflow tuo**: rip sprite ufficiale Digimon → upload → ottieni moveset esteso.

**Verdict**: ⭐⭐⭐⭐⭐ per extend animation da sprite ufficiali.

---

### 2.6 — Aseprite MCP (pixel-mcp / Aseprite MCP Pro)

**Tipo**: MCP server controlla Aseprite via API. **NON genera AI**.

**Capabilities**:
- Drawing primitives, frame management, palette tools.
- Reference analysis (palette extract, edge detect).
- Frame interpolation auto in-between.
- Spritesheet export (PNG, GIF, Godot format).
- Aseprite MCP Pro: 121 tool, $10 una-tantum.

**Quality**:
- Cleanup automation: ⭐⭐⭐⭐⭐
- Frame interpolation tween: ⭐⭐⭐⭐
- Export pipeline: ⭐⭐⭐⭐⭐

**Costo**: $0-10 + Aseprite license $20.

**Use case ideale**:
- Post-process automation any AI output.
- Frame tween manual lavoro automatizzato.
- Production pipeline finale.

**Verdict**: ⭐⭐⭐⭐⭐ complementare obbligatorio a qualsiasi AI gen.

---

### 2.7 — pixelforge-mcp

**Tipo**: MCP server, usa Gemini Nano Banana sotto.

**Capabilities**:
- Image gen via Gemini.
- Reference matching.
- Smart bg removal, auto-crop.

**Quality**: ⭐⭐⭐⭐⭐ (eredita Nano Banana).

**Costo**: free OSS + Gemini API ~$0.04/img.

**Verdict**: ⭐⭐⭐⭐⭐ alternative MCP a `shinpr/mcp-image` per Nano Banana via Claude Code.

---

### 2.8 — Other paid services

| Service | Strength | Weakness | Cost |
|---------|----------|----------|------|
| **Midjourney v7** | Anime/illustration top tier | No pixel art native, no API, $$ | $10-30/mo |
| **NovelAI** | Anime-specialized best | Anime only, no pixel | $15-25/mo |
| **GPT-Image-2** | Text-to-image solid | Char drift dopo 5-6 batch | $0.04-0.17/img |
| **Scenario.gg** | Style fine-tune cloud | Subscription, no pixel art | $20-30/mo |
| **Ludo.ai** | Character animation | Web UI only | Freemium-$20 |
| **God Mode AI** | Sprite + animation | Generic, no pixel focus | Subscription |
| **Leonardo.ai** | Custom models cloud | LoRA-equiv, no pixel | $30/mo |

---

## 3. Sources sprite ufficiali Digimon

### 3.1 — Game sources rip

| Game | Year | Resolution | Anim count | Style fit bevyrogue |
|------|------|------------|------------|---------------------|
| Digimon World (PS1) | 1999 | 64×64 | Idle/walk/atk/hurt/KO | ⭐⭐⭐⭐ |
| Digimon World 3 (PS1) | 2002 | Battle sprite alta-res relativa | 6-10 anim | ⭐⭐⭐⭐⭐ HSR-fit |
| Digimon Story DS | 2007 | 32×32 | Limited | ⭐⭐⭐ |
| Digimon Adventure (PSP) | 2013 | Chibi battle | Battle full | ⭐⭐⭐⭐ |
| Digimon Re:Digitize (PSP) | 2012 | 64×64 | Battle full | ⭐⭐⭐⭐⭐ |
| Digimon Pendulum sim | various | 16×16 | Micro | ⭐⭐ |

### 3.2 — Sources online

- **Spriters Resource** (spriters-resource.com): rip sprite by game.
- **Sprite Database** (sdb.drshnaps.com): collection by Digimon.
- **Wikimon** (wikimon.net): official artwork high-res.
- **Bandai art books**: archive.org scan.

### 3.3 — Quality vs legal

| Use | Verdict |
|-----|---------|
| Fan-game free release | ⭐⭐⭐⭐⭐ standard practice, low-risk |
| Patreon/monetization | ❌ Bandai cease-and-desist quasi-certain |
| Asset pack store | ❌ illegal redistribute |
| Personal prototype | ⭐⭐⭐⭐⭐ no risk |

---

## 4. Workflow comparison matrix

### 4.1 — Workflow A: Sprite rip diretto

```
[1] Rip sprite Re:Digitize/World 3 da Spriters Resource
       ↓
[2] Aseprite import + slice frame
       ↓
[3] Bevy TextureAtlasLayout integrate
```

| Aspetto | Mark |
|---------|------|
| Fidelity char | ⭐⭐⭐⭐⭐ 100% |
| Animation count | ⭐⭐⭐⭐ idle/walk/atk/hurt rip-ready |
| Custom anim flexibility | ⭐ no custom |
| Setup time | ⭐⭐⭐⭐⭐ ~3-4h totali |
| Cost | ⭐⭐⭐⭐⭐ $0 |
| Legal (fan-game) | ⭐⭐⭐⭐ grey OK free |
| Legal (commercial) | ❌ no go |
| Style coerenza inter-char | ⭐⭐⭐⭐⭐ stesso source game |

**Pros**: 100% fidelity, fast, cheap, zero AI hassle.
**Cons**: limit anim al source, no custom skill.

---

### 4.2 — Workflow B: Sprite rip + Aseprite tween manual

```
[1] Rip sprite ufficiali (Workflow A)
       ↓
[2] Aseprite tween interpolation:
    - Frame 1 = sprite ufficiale
    - Frame N = modifica manuale pose B
    - Tween auto in-between
       ↓
[3] Estendi cycle (idle 4 → 8 frame, attack 6 → 10)
       ↓
[4] Bevy integrate
```

| Aspetto | Mark |
|---------|------|
| Fidelity char | ⭐⭐⭐⭐⭐ 100% (parte da rip) |
| Animation extension | ⭐⭐⭐⭐ |
| Custom anim flexibility | ⭐⭐⭐ moderata |
| Setup time | ⭐⭐⭐ ~10-15h totali |
| Cost | ⭐⭐⭐⭐⭐ $0-20 (Aseprite) |
| Skill required | ⭐⭐ pixel art familiarity |
| Quality output | ⭐⭐⭐⭐⭐ pixel-perfect |

**Pros**: fidelity 100% preserved, fully custom control.
**Cons**: skill bottleneck se non sei pixel artist.

---

### 4.3 — Workflow C: Sprite rip + AutoSprite (paid)

```
[1] Rip sprite ufficiali
       ↓
[2] Upload AutoSprite + seleziona moveset
       ↓
[3] Download spritesheet animato
       ↓
[4] Aseprite cleanup palette match
       ↓
[5] Bevy integrate
```

| Aspetto | Mark |
|---------|------|
| Fidelity char | ⭐⭐⭐⭐ ~95% (preserva design) |
| Animation count | ⭐⭐⭐⭐⭐ moveset esteso |
| Custom anim flexibility | ⭐⭐⭐⭐ moveset selezionabile |
| Setup time | ⭐⭐⭐⭐⭐ ~3-5h |
| Cost | ⭐⭐⭐ ~$15-30 totali |
| Skill required | ⭐⭐⭐⭐⭐ minimal |
| Quality output | ⭐⭐⭐⭐ |

**Pros**: rapido, plug-and-play, skill-light.
**Cons**: dipendenza service paid, fidelity ~95%.

---

### 4.4 — Workflow D: Solo AI gen locale (no rip)

```
[1] Setup ROCm + ComfyUI + Pony XL + LoRA Digimon CivitAI
       ↓
[2] Train style LoRA tuo (opzionale)
       ↓
[3] Gen 12 base sprite con char LoRA + ControlNet pose
       ↓
[4] AnimateDiff per cycle idle/walk
       ↓
[5] ControlNet pose grid + Aseprite tween per attack
       ↓
[6] Aseprite cleanup pixel-perfect
       ↓
[7] Bevy integrate
```

| Aspetto | Mark |
|---------|------|
| Fidelity char | ⭐⭐⭐⭐ ~90% (LoRA-locked, drift sub-pixel) |
| Animation count | ⭐⭐⭐⭐ tutti achievable |
| Custom anim flexibility | ⭐⭐⭐⭐⭐ |
| Setup time | ⭐⭐ 4-6h iniziale + 15-25h production |
| Cost | ⭐⭐⭐⭐⭐ $0 ongoing (electricity) |
| Skill required | ⭐⭐⭐ ComfyUI familiarity |
| Quality output | ⭐⭐⭐⭐ post-cleanup |
| Legal | ⭐⭐⭐⭐⭐ tuo asset |

**Pros**: free, full control, infinite iteration, legal-safe.
**Cons**: setup tech alto, fidelity sub-100%, sub-pixel drift.

---

### 4.5 — Workflow E: Solo AI gen paid (no rip)

```
[1] Setup MCP Nano Banana Pro + PixelLab MCP
       ↓
[2] Nano Banana Pro: gen 12 base sprite reference-faithful
       ↓
[3] PixelLab MCP: skeleton-rigged anim (idle/walk template)
       ↓
[4] Manual Aseprite cleanup palette unify
       ↓
[5] Bevy integrate
```

| Aspetto | Mark |
|---------|------|
| Fidelity char | ⭐⭐⭐⭐ ~95% |
| Animation count | ⭐⭐⭐⭐⭐ skeleton-rig auto |
| Custom anim flexibility | ⭐⭐⭐ |
| Setup time | ⭐⭐⭐⭐⭐ ~30 min setup MCP |
| Cost | ⭐⭐⭐ ~$25-40 totali |
| Skill required | ⭐⭐⭐⭐⭐ minimal |
| Quality output | ⭐⭐⭐⭐ |
| Legal | ⭐⭐⭐⭐⭐ tuo asset |

**Pros**: fast, plug-and-play, no tech setup.
**Cons**: paghi ricorrente, fidelity sub-100%, no custom style.

---

### 4.6 — Workflow F: Hybrid (rip + AI custom anim) ⭐ RACCOMANDATO

```
[1] Rip sprite ufficiali Re:Digitize 12 Digimon
    → idle, walk, basic attack, hurt, KO 100% fidelity
       ↓
[2] Aseprite tween extend cycle se needed
       ↓
[3] AnimateDiff locale o AutoSprite per skill custom
    → pose chiave + interpolation, 90% fidelity
       ↓
[4] AnimateDiff o AI gen + heavy cleanup per ult signature
    → 80-90% fidelity (acceptable per ult flair)
       ↓
[5] Aseprite MCP cleanup batch
       ↓
[6] Bevy integrate
```

| Aspetto | Mark |
|---------|------|
| Fidelity char base | ⭐⭐⭐⭐⭐ 100% (rip) |
| Fidelity custom anim | ⭐⭐⭐⭐ ~90% |
| Animation count | ⭐⭐⭐⭐⭐ tutti |
| Custom anim flexibility | ⭐⭐⭐⭐⭐ |
| Setup time | ⭐⭐⭐⭐ ~10-15h totali |
| Cost | ⭐⭐⭐⭐ $0-30 (AutoSprite optional) |
| Skill required | ⭐⭐⭐ medium |
| Quality output | ⭐⭐⭐⭐⭐ post-cleanup |
| Legal (fan-game) | ⭐⭐⭐⭐ grey OK free |
| Style coerenza | ⭐⭐⭐⭐⭐ |

**Pros**: best fidelity/effort ratio, base 100% fidelity, AI solo per gap.
**Cons**: legal grey commerciale, dipendente da disponibilità rip per char target.

---

### 4.7 — Workflow G: Cloud LoRA train + locale inference

```
[1] Cloud RunPod: train multi-char LoRA SDXL (~$1-2, 1-2h)
       ↓
[2] Download LoRA → locale RX 6700 XT
       ↓
[3] ComfyUI inference + LoRA tuo per gen + anim
       ↓
[4] Aseprite cleanup
       ↓
[5] Bevy integrate
```

| Aspetto | Mark |
|---------|------|
| Fidelity char | ⭐⭐⭐⭐ ~95% (LoRA tuo) |
| Style coerenza | ⭐⭐⭐⭐⭐ baked-in |
| Setup time | ⭐⭐⭐ ~12-16h |
| Cost | ⭐⭐⭐⭐⭐ ~$1-5 |
| Skill required | ⭐⭐⭐ |
| Quality output | ⭐⭐⭐⭐ |

**Pros**: cheap, full control, unlimited inference future.
**Cons**: training pain primo run.

---

## 5. Quality marks summary

### Per task

| Task | Locale (SD+LoRA) | Paid (Nano/PixelLab) | Rip ufficiale |
|------|------------------|---------------------|---------------|
| Single char gen high-fidelity | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ 100% |
| Pixel art 64-128px | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Multi-pose batch 10+ stesso char | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ Nano wins | ⭐⭐⭐ limit anim source |
| Walk/idle cycle skeleton | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ PixelLab wins | ⭐⭐⭐⭐⭐ rip |
| Attack 6-8 frame coerente | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ rip |
| Custom skill animation | ⭐⭐⭐⭐ | ⭐⭐⭐ | ❌ no custom |
| Style coerenza 12+ char | ⭐⭐⭐⭐⭐ LoRA tuo | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ stesso game |
| Reference fidelity Agumon | ⭐⭐⭐⭐⭐ LoRA Agumon | ⭐⭐⭐⭐⭐ Nano ref | ⭐⭐⭐⭐⭐ 100% |

### Per workflow

| Workflow | Fidelity | Time | Cost | Quality finale | Verdict |
|----------|----------|------|------|----------------|---------|
| A — Rip diretto | 100% | 3-4h | $0 | ⭐⭐⭐⭐⭐ | Best fidelity, limit anim |
| B — Rip + Aseprite tween | 100% | 10-15h | $0-20 | ⭐⭐⭐⭐⭐ | Best fidelity + custom |
| C — Rip + AutoSprite | 95% | 3-5h | $15-30 | ⭐⭐⭐⭐ | Fast paid hybrid |
| D — Solo AI locale | 90% | 20-30h | $0 | ⭐⭐⭐⭐ | Free, custom, slow |
| E — Solo AI paid | 95% | 5-7h | $25-40 | ⭐⭐⭐⭐ | Plug-and-play |
| **F — Hybrid rip + AI** ⭐ | **98%** | **10-15h** | **$0-30** | **⭐⭐⭐⭐⭐** | **Best ROI** |
| G — Cloud LoRA + locale | 95% | 12-16h | $1-5 | ⭐⭐⭐⭐ | Style coerenza top |

---

## 5b. Top combo concrete da provare (battle-tested)

Combinazioni esplicite che vale provare in ordine di priorità. Ognuna = pipeline end-to-end production.

### Combo 1 — **"Official + AI animation locale"** ⭐⭐⭐⭐⭐

```
Char base:    Rip Re:Digitize/World 3 (100% fidelity)
Idle/walk:    Rip esistente o Aseprite tween manual
Skill custom: SD locale + IPAdapter sprite ref + AnimateDiff
Ult signature: AnimateDiff + heavy cleanup
Cleanup:      Aseprite MCP batch
```

| Metric | Mark |
|--------|------|
| Fidelity | ⭐⭐⭐⭐⭐ 98% |
| Time | 12-18h |
| Cost | $0-20 |
| Quality | ⭐⭐⭐⭐⭐ |
| Risk | basso (rip = fallback safe) |

**Da provare per primo**. Best ROI per fan-game.

---

### Combo 2 — **"Official + AutoSprite paid"** ⭐⭐⭐⭐

```
Char base:    Rip Re:Digitize (100% fidelity)
Idle/walk:    Rip esistente
Skill custom: AutoSprite (upload rip + seleziona moveset) ~$15-30
Ult signature: AutoSprite custom action
Cleanup:      Aseprite manuale palette unify
```

| Metric | Mark |
|--------|------|
| Fidelity | ⭐⭐⭐⭐ 95% |
| Time | 4-6h ⚡ veloce |
| Cost | $15-30 |
| Quality | ⭐⭐⭐⭐ |
| Risk | dipendenza service AutoSprite |

**Per tempo limitato**. Skip technical setup.

---

### Combo 3 — **"LoRA tuo + PixelLab MCP anim"** ⭐⭐⭐⭐

```
Char base:    SD locale + LoRA Digimon CivitAI (no rip)
Idle/walk:    PixelLab MCP skeleton template ~$15 1 mese
Skill custom: SD + ControlNet pose grid + Aseprite tween
Ult signature: AnimateDiff + cleanup
Cleanup:      Aseprite MCP
```

| Metric | Mark |
|--------|------|
| Fidelity | ⭐⭐⭐⭐ 90% (LoRA-locked) |
| Time | 14-20h |
| Cost | $15-30 |
| Quality | ⭐⭐⭐⭐ |
| Risk | medio (skeleton humanoid limit per Digimon T-rex-style) |
| Legal | ⭐⭐⭐⭐⭐ tuo asset, no IP |

**Quando**: vuoi commercial-safe, no rip Bandai. Caveat: PixelLab skeleton humanoid forza pose bipede.

---

### Combo 4 — **"Nano Banana Pro + Aseprite MCP"** ⭐⭐⭐⭐

```
Char base:    Nano Banana Pro + reference upload (image Agumon ref)
Pose chiave:  Multi-turn edit "stesso char ma in [pose]" → 4-8 pose/anim
Inbetween:    Aseprite MCP frame interpolation (tween auto)
Cleanup:      Aseprite MCP palette + outline
```

| Metric | Mark |
|--------|------|
| Fidelity | ⭐⭐⭐⭐ 95% (reference upload) |
| Time | 8-12h |
| Cost | $5-15 (Gemini API) |
| Quality | ⭐⭐⭐⭐⭐ identity preservation |
| Risk | basso |
| Legal | grey (gen char Agumon-ricoscibile = derivative) |

**Quando**: vuoi quick high-fidelity senza setup ROCm. MCP Nano Banana plug-and-play.

---

### Combo 5 — **"LoRA tuo (cloud train) + AnimateDiff locale"** ⭐⭐⭐⭐

```
Setup:        RunPod A6000 train multi-char LoRA tuo (~$1-2, 1-2h)
Char base:    SD locale + LoRA tuo
Idle/walk:    AnimateDiff locale (motion module breathing-idle)
Skill custom: SD + ControlNet pose grid + LoRA + tween Aseprite
Ult signature: AnimateDiff + LoRA + heavy cleanup
Cleanup:      Aseprite MCP
```

| Metric | Mark |
|--------|------|
| Fidelity | ⭐⭐⭐⭐ 92% (LoRA tuo) |
| Time | 16-22h |
| Cost | $1-5 |
| Quality | ⭐⭐⭐⭐ |
| Risk | medio (LoRA training first-time) |
| Legal | ⭐⭐⭐⭐⭐ tuo asset |
| Style coerenza | ⭐⭐⭐⭐⭐ baked-in 12 char |

**Quando**: vuoi maximum control style + commercial path possibile.

---

### Combo 6 — **"Rip + LoRA tuo + AnimateDiff"** ⭐⭐⭐⭐⭐ (per ambiziosi)

```
Char base:    Rip Re:Digitize 12 Digimon (100% fidelity)
LoRA tuo:     Train su rip stessi (style locked al gioco source)
Idle/walk:    Rip esistente
Skill custom: SD + LoRA tuo + ControlNet pose + IPAdapter sprite ref
              → output match perfetto stile rip
Ult signature: AnimateDiff + LoRA tuo + cleanup
Cleanup:      Aseprite MCP
```

| Metric | Mark |
|--------|------|
| Fidelity char | ⭐⭐⭐⭐⭐ 100% (rip + LoRA-style-match) |
| Style coerenza | ⭐⭐⭐⭐⭐ rip + LoRA same style |
| Custom anim flexibility | ⭐⭐⭐⭐⭐ |
| Time | 18-25h |
| Cost | $1-10 |
| Quality | ⭐⭐⭐⭐⭐ |
| Risk | medio-alto (LoRA training + ROCm setup) |

**Quando**: vuoi best-in-class, accetti effort tech-heavy. LoRA trained su rip = style match perfetto.

---

### Combo 7 — **"Nano Banana + PixelLab + Aseprite MCP"** ⭐⭐⭐⭐ (full paid plug-and-play)

```
Char base:    Nano Banana Pro MCP + ref upload Agumon
Pose chiave:  Nano Banana multi-turn edits
Idle/walk:    PixelLab MCP skeleton anim (su char gen Nano)
Skill custom: Nano Banana frame sequence + tween Aseprite MCP
Cleanup:      Aseprite MCP automation
```

| Metric | Mark |
|--------|------|
| Fidelity | ⭐⭐⭐⭐ 90-95% |
| Time | 6-10h ⚡ |
| Cost | $30-50 |
| Quality | ⭐⭐⭐⭐ |
| Risk | basso |
| Skill | ⭐⭐⭐⭐⭐ plug-and-play |

**Quando**: time-poor, money-OK, max convenience. Tutto via Claude Code MCP.

---

### Combo 8 — **"Pure Aseprite manuale (no AI)"** ⭐⭐⭐⭐⭐ (purist)

```
Char base:    Rip Re:Digitize 100% fidelity
Animations:   Aseprite manual frame-by-frame
              - tween built-in per inbetween auto
              - reference esistenti come keyframe
              - palette indexed enforced
Cleanup:      Continuo durante draw
```

| Metric | Mark |
|--------|------|
| Fidelity | ⭐⭐⭐⭐⭐ 100% |
| Quality | ⭐⭐⭐⭐⭐ pixel-perfect |
| Time | 30-50h ⚠️ |
| Cost | $0-20 |
| Skill | ⭐⭐ pixel art experience required |
| Risk | basso (deterministic) |

**Quando**: skill pixel art esistente, tempo abbondante, perfectionist.

---

### Combo decision tree

```
Vuoi 100% fidelity Digimon?
├─ Sì
│  ├─ Hai tempo (30+h)? → Combo 8 (Aseprite manual)
│  ├─ Vuoi anim custom + AI? → Combo 1 (rip + AI locale) ⭐
│  └─ Vuoi paid fast? → Combo 2 (rip + AutoSprite)
├─ No (90-95% OK)
│  ├─ Style coerenza priority? → Combo 6 (rip + LoRA tuo)
│  ├─ Commercial path needed? → Combo 3 o 5 (LoRA, no rip)
│  ├─ Quick test plug-and-play? → Combo 4 o 7 (Nano Banana)
│  └─ Budget zero? → Combo 5 (cloud train + locale)
```

### Top 3 da provare in ordine

1. **Combo 1** (Official + AI locale) — primo test, best ROI fan-game.
2. **Combo 6** (Rip + LoRA tuo) — se Combo 1 OK e vuoi spingere quality.
3. **Combo 4** (Nano Banana + Aseprite MCP) — fallback fast se locale non setup-able.

---

## 6. Decision matrix

### Per priorità

| Priorità | Workflow |
|----------|----------|
| **100% fidelity Digimon** | A (rip) o B (rip + tween) |
| **Time-to-ship minimo** | E (paid) o C (rip + AutoSprite) |
| **Cost zero** | A o D |
| **Custom skill flexibility** | F (hybrid) o D |
| **Best overall ROI** | **F (hybrid)** |
| **Style unico tuo gioco** | G (LoRA tuo) o D |
| **Solo dev no skill pixel art** | E (paid) |
| **Tinkerer enthusiast** | D (locale full) |

### Per scope progetto

| Scope | Workflow |
|-------|----------|
| Prototipo personale | A o E |
| **Fan-game indie shippable** ⭐ | **F (hybrid)** |
| Indie commercial | G + spot artist freelance |
| AAA tier | irrealistico solo dev |

---

## 7. Costi totali realistici per 12 Digimon

| Workflow | Cost totale | Time totale | Quality |
|----------|-------------|-------------|---------|
| A | $0 | 3-4h | ⭐⭐⭐⭐⭐ fidelity, limit anim |
| B | $0-20 | 10-15h | ⭐⭐⭐⭐⭐ fidelity + custom |
| C | $15-30 | 3-5h | ⭐⭐⭐⭐ |
| D | $0 + electricity | 20-30h | ⭐⭐⭐⭐ |
| E | $25-40 | 5-7h | ⭐⭐⭐⭐ |
| **F** | **$0-30** | **10-15h** | **⭐⭐⭐⭐⭐** |
| G | $1-5 | 12-16h | ⭐⭐⭐⭐ |
| Spot artist freelance | $2400-3600 | 4-8 settimane | ⭐⭐⭐⭐⭐ AAA |

---

## 8. Tooling stack consigliato (Workflow F)

### Mandatory
- **Aseprite** ($20 una-tantum o build from source $0): cleanup, frame management.
- **Bevy 0.18**: già in uso.

### Recommended
- **Aseprite MCP Pro** ($10 una-tantum): automation cleanup pipeline.
- **AutoSprite** (~$15-30 per progetto): extend animation rip.

### Optional
- **PixelLab MCP** ($15 1 mese): se serve skeleton-rig anim custom.
- **AnimateDiff locale** (gratis, ROCm setup ~3h): per custom motion, idle smooth.
- **Nano Banana MCP** (~$5-10 totali Gemini API): solo per concept exploration o ult signature.

### Skip per Workflow F
- LoRA training tuo (rip basta).
- Cloud RunPod (no LoRA train).
- ComfyUI complex setup (AnimateDiff standalone basta).

---

## 9. VFX + particles (separato da char sprite)

### Stack consigliato
1. **Pack itch.io** ($10-20): CodeManu Pixel VFX, Pixel Frog FX → 100+ effect ready.
2. **`bevy_hanabi`** crate (free): runtime particle system Bevy.
3. **AnimateDiff locale** opzionale: signature ult VFX custom.

### Costo VFX
- ~$10-30 totali.
- ~4-8h setup + custom.

---

## 10. Action plan (Workflow F — raccomandato)

### Fase 1 — Verifica (1h)
1. Lista 12 Digimon target.
2. Spriters Resource: verifica disponibilità sprite rip per ognuno.
3. Decidi game source coerente (Re:Digitize raccomandato).

### Fase 2 — Asset base (3-4h)
1. Download sprite sheet 12 Digimon Re:Digitize.
2. Aseprite import + slice frame.
3. Naming convention `assets/sprites/creatures/{name}/{anim}_{frame}.png`.

### Fase 3 — Animation extension (4-6h)
1. Per anim mancanti (skill, ult): AnimateDiff locale con sprite rip come ref.
2. Aseprite tween per cycle estesi.
3. Cleanup palette unify cross-char.

### Fase 4 — VFX (3-4h)
1. Pack VFX itch.io + drop in `assets/sprites/vfx/`.
2. `bevy_hanabi` config particle 5-8 effect.
3. Custom signature ult opzionale.

### Fase 5 — Bevy integrate (2-3h)
1. `TextureAtlasLayout` per ogni Digimon.
2. `assets/data/units.ron` entry.
3. Animation state machine wire-up.
4. Test combat con team Digimon.

### Total
- **Time**: 13-18h spalmati.
- **Cost**: ~$10-30.
- **Quality**: A-tier indie production-grade.
- **Fidelity**: ~98% perceived.

---

## 11. Constraint legali

### Fan-game free release
- ✅ Standard practice per Digimon community (Re:Digitize Decode, etc).
- ⚠️ Risk cease-and-desist Bandai esistente ma raro.
- Crediti: "sprites from Digimon Re:Digitize © Bandai Namco".

### Commercial / Patreon / monetization
- ❌ No-go con asset Bandai.
- Servirebbe 100% original art commissioned.
- Stima costo: $5000-15000 per replica scope 12 Digimon original.

### Asset pack distribution
- ❌ illegal redistribute sprite Bandai separati.

### Naming
- Nome gioco pubblico ≠ "Digimon" trademark.
- Es: "Mon Tamer Roguelite" pubblicamente, "Digimon fan-game" nei credit.

---

## 12. Riferimenti

### Tool
- [PixelLab](https://www.pixellab.ai/) — MCP nativo
- [Nano Banana docs](https://ai.google.dev/gemini-api/docs/image-generation)
- [shinpr/mcp-image](https://github.com/shinpr/mcp-image) — Nano Banana MCP
- [Aseprite MCP Pro](https://aseprite-mcp.abyo.net/)
- [pixel-mcp](https://github.com/willibrandon/pixel-mcp)
- [pixelforge-mcp](https://github.com/freema/pixelforge-mcp)
- [AutoSprite](https://www.autosprite.io/)
- [Layer.ai](https://www.layer.ai/)
- [CivitAI Digimon LoRA](https://civitai.com/search?query=digimon)

### Sprite sources
- [The Spriters Resource](https://www.spriters-resource.com/)
- [Sprite Database](https://sdb.drshnaps.com/)
- [Wikimon](https://wikimon.net/)

### Bevy
- [bevy_hanabi](https://github.com/djeedai/bevy_hanabi)
- [bevy_enoki](https://github.com/Lommix/bevy_enoki)

---

## 13. Open questions

- [ ] Lista finale 12 Digimon target (6 rookie + 6 adult).
- [ ] Game source rip preferito (Re:Digitize / World 3 / Adventure)?
- [ ] Pixel art vs anime style per bevyrogue (pixel raccomandato per indie scope).
- [ ] Resolution char target (64×64 o 96×96)?
- [ ] Frame count target per anim (idle 4 / walk 6 / attack 6)?
- [ ] Style bible scritto in `assets/style_bible.md` da definire.
