# ANGLE-3 — Forma del deliverable (skill vs subagent vs reference)

## Obiettivo

Decidere **in che forma** impacchettare la conoscenza VFX-enoki perché (a) si auto-carichi
al momento giusto, (b) sia realmente usata dall'agent, (c) sia manutenibile.

## Le tre forme disponibili in questo ambiente

| Forma | Dove | Carica quando | Costo | Adatta a |
|---|---|---|---|---|
| **Skill project-local** | `.claude/skills/<name>/SKILL.md` (+ references/, scripts/) | Auto-discovery sul match della `description` (trigger keywords) — come `digimon`, `sprite-iterate` già presenti | Basso | Conoscenza + decision rule + reference, in-context nella sessione principale |
| **Subagent** | `.gsd/agents/<name>.md` (project) o `~/.gsd/agent/agents/` | Dispatch esplicito (`subagent`/`Agent`), context window isolato | Medio | Loop autonomi / task self-contained (es. autorare un VFX e auto-rivederlo) |
| **Reference docs** | file `.md` dentro la skill | Letti on-demand dalla skill | Bassissimo | Tabelle dense, esempi `.ron`, snippet WGSL |

Evidenza: il repo usa **già** entrambi i pattern — `.claude/skills/digimon` (skill con
script `query.py`) e `.claude/skills/sprite-iterate` (skill che *spawna un subagent* per il
loop render→review→edit). Il pattern vincente locale è quindi: **skill come entry-point,
subagent solo per il loop autonomo.**

## Perché una SKILL (non solo un agent, non solo memoria)

- **Auto-discovery**: la `description` con keyword *verb/backend-based* (`enoki`, `vfx`,
  `particle`, `.particle.ron`, `VfxAsset`, `EnokiLifecycle`, `cel`, `glow/spark/aura/burst/
  beam/trail`, `charge/projectile/impact/detonate`) la carica esattamente quando l'agent tocca
  un effetto qualsiasi — non un singolo Digimon. La memoria `project-baby-flame-vfx-redo`
  (utile come stile) non garantisce la stessa precisione di trigger.
- **Composable col prior art**: la skill può **referenziare** `vfx-realtime` per i principi
  generali e aggiungere solo il *backend-mapping enoki* + *decision rule* (ANGLE-2). Resta snella.
- **Reference files**: la decision table, gli esempi `.particle.ron` reali del repo e gli
  snippet WGSL stanno in reference letti on-demand → SKILL.md corto, dettaglio profondo.

## Perché ANCHE un (mini) subagent — opzionale, fase 2

Speculare a `sprite-iterate`: un subagent `enoki-vfx-iterate` che, dato un effect-id,
auтора/tunи il `.particle.ron`, builda windowed, e (se c'è un proof headless/screenshot)
auto-rivede contro i principi anime-cel. Non necessario al primo giro; **la skill viene prima.**

## Bozza struttura proposta — `bevy-enoki-vfx`

```
.claude/skills/bevy-enoki-vfx/
├── SKILL.md
└── references/
    ├── decision-rule.md      # ANGLE-2: regola asset-vs-primitiva + decision table
    ├── enoki-cookbook.md     # campi .particle.ron, lifecycle, anchor, RotationParams,
    │                         #   esempi reali da assets/digimon/agumon/*.particle.ron
    ├── anime-cel-principles.md  # ANGLE-1.B/C tradotti su enoki (timing, impact frames,
    │                            #   value contrast, star-burst, shatter, HDR core)
    └── wgsl-hero.md          # quando/come scrivere un Particle2dMaterial custom
```

### Frontmatter proposto

```yaml
---
name: bevy-enoki-vfx
description: >
  Anime cel-shading VFX (Digimon Survive / Honkai Star Rail look) authored on
  bevy_enoki for this project. Use when working on any particle effect, .particle.ron,
  VfxAsset, EnokiLifecycle, charge/projectile/impact/detonate bursts, glow/spark/aura/
  star-burst/trail/beam visual verbs, or deciding whether an effect needs a hand-authored
  asset vs bevy_enoki primitives. Encodes the L0-L4 asset-vs-primitive decision rule
  (effect-agnostic, keyed on the visual verb) and the repo's procedural-first art
  direction. Builds on the generic `vfx-realtime` skill.
---
```

### Contenuto SKILL.md (outline)

1. **Identità + bias**: backend-first, procedural-first, "resta al livello più basso che
   funziona". Rimanda a `vfx-realtime` per i principi generali.
2. **Cosa enoki fa / non fa** (3 righe + link a `enoki-cookbook.md`).
3. **La decision rule L0→L4** (link a `decision-rule.md`) — il cuore.
4. **Anime-cel checklist**: value contrast prima del colore, impact frame, anticipation
   (charge), follow-through (residue), star-burst/shatter shape, HDR white-hot core.
5. **Convention del repo**: anchor semantici, `on_expire` chaining, no-hardcoding contracts
   in `tests/animation/`, 12fps / scala 14–34px, atom riusabili in `assets/vfx/`.
6. **Anti-pattern**: aspettarsi cinematic HSR 1:1 dal solo enoki; trail/beam/distortion non
   nativi; saltare a L4 senza motivo.

## Pro / Contro / Confidence

**Pro:** una skill è il formato a costo minore con auto-load preciso; il repo prova già che
il pattern skill(+subagent) funziona; composable con `vfx-realtime` e con la memoria.

**Contro / rischi:** doppione concettuale con `vfx-realtime` se non si è disciplinati nel
*referenziare* invece di duplicare; le keyword di trigger vanno scelte bene per non
sovra/sotto-attivare.

**Confidence: Alta.** La forma "skill project-local + reference, subagent opzionale dopo" è
direttamente supportata da pattern già presenti nel repo.
