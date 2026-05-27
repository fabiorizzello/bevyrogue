# Recommendation — Coding agent esperto in VFX anime cel-shading per bevy_enoki

## Executive summary

**Go: crea una skill project-local `bevy-enoki-vfx`** (in `.claude/skills/`), che
*referenzia* la skill generica `vfx-realtime` per i principi e aggiunge l'unica cosa che
manca davvero: il **mapping backend `bevy_enoki` + la regola "asset vs primitiva"** calibrata
sull'art direction di questo gioco. Non serve importare skill da internet — la ricerca
conferma che esistono principi VFX abbondanti (Disney 12 principi, breakdown HSR, cel-shading)
ma **nessuna skill esistente, locale o online, lega quei principi a `bevy_enoki`** né conosce
le convention del repo. Quel buco è il deliverable.

La decision rule è **effect-agnostic**: parte dal *verbo visivo* (glow, spark, aura,
star-burst, trail, beam, dissolve…), non dal Digimon, quindi vale per qualunque kit — Petit
Thunder, Blue Cyclone, Koyosetsu, Metal Cannon o effetti non ancora autorati. Baby Flame è
solo il primo effetto che capita di avere già su disco, citato come esempio applicato, mai
come template.

La conoscenza si compone già da tre strati presenti: i *principi* (`vfx-realtime`), lo *stile/
decisioni* (memoria `project-baby-flame-vfx-redo`), e le *capacità del backend* (spike 3). La
skill nuova li salda in un punto a trigger preciso, così l'agent — PI, Claude o Codex — carica
la conoscenza giusta esattamente quando tocca un `.particle.ron`/`VfxAsset`. Un subagent
autonomo (`enoki-vfx-iterate`, gemello di `sprite-iterate`) è un follow-up opzionale, non il
primo passo.

## Comparison matrix — forma del deliverable

| Opzione | Auto-load | Costo | Copre il gap? | Verdetto |
|---|---|---|---|---|
| Solo memoria (status quo) | Debole/impreciso | Nullo | No — manca decision rule + mapping enoki | Insufficiente |
| Importare skill online | — | Basso | No — nessuna è enoki/anime-cel specifica | Non disponibile |
| **Skill `bevy-enoki-vfx` (+ reference)** | **Preciso (keyword)** | **Basso** | **Sì** | **Raccomandato** |
| Subagent isolato | Solo dispatch esplicito | Medio | Parziale (loop, non knowledge in-context) | Fase 2 opzionale |
| Skill + subagent | Preciso + loop | Medio | Sì, completo | Obiettivo finale |

## Comparison matrix — la regola asset-vs-primitiva (cuore tecnico)

| Livello | Cosa | Asset | Quando |
|---|---|---|---|
| L0 | Primitiva pura (color/scale + curve + HDR) | No | Glow, spark, flash, shockwave; forma astratta leggibile |
| L1 | + `PlacementParams`/`RotationParams`/`Turbulence` | No | Aura, ember converge, star-burst rotante (look HSR), fan-out impatto — **copre la maggioranza del look a 14–34px** |
| L2 | Sprite cel minimale, 1 atom riusabile | Sì (1, N effetti) | Silhouette riconoscibile: lingua di fiamma, lama, petalo, simbolo |
| L3 | Sprite-sheet / atlas animato | Sì | La forma evolve nel tempo (guizzo, esplosione a keyframe) — hero |
| L4 | Custom `Particle2dMaterial` WGSL | Shader | Dissolve/distortion/rim/gradient-map — solo signature |

**Bias di default: il livello più basso che regge.** Sali solo sui trigger di escalation
("non si legge a 14–34px" → L2; "la forma deve evolvere" → L3; "look materico che il colore
piatto non dà" → L4).

## Raccomandazione

1. **Crea `.claude/skills/bevy-enoki-vfx/`** con la struttura proposta in ANGLE-3 (SKILL.md
   + 4 reference: `decision-rule.md`, `enoki-cookbook.md`, `anime-cel-principles.md`,
   `wgsl-hero.md`).
2. **SKILL.md snella**: identità procedural-first, *link* a `vfx-realtime` per i principi
   generali (no duplicazione), la decision rule L0–L4, la checklist anime-cel, le convention
   del repo (anchor, `on_expire` chaining, no-hardcoding tests, 12fps/14–34px, atom in
   `assets/vfx/`), gli anti-pattern.
3. **Reference densi e repo-specifici**: `enoki-cookbook.md` deve citare i `.particle.ron`
   reali del repo come esempi vivi (al momento gli unici autorati sono sotto
   `assets/digimon/agumon/`, ma vanno trattati come *un* caso applicato, non come template) e
   i verbi reali (`RotationParams`, `EnokiLifecycle`, `PlacementAnchor`).
4. **Frontmatter con trigger keyword effect-agnostic** keyed sul *verbo visivo*, non sul
   Digimon (`enoki`, `vfx`, `particle`, `.particle.ron`, `VfxAsset`, `EnokiLifecycle`, `cel`,
   `glow/spark/aura/star-burst/trail/beam`, `charge/projectile/impact/detonate`) per auto-load
   preciso su qualunque effetto.

## Alternativa se la primaria non regge

Se durante l'uso emerge che il look "signature" sfugge alla decision rule (troppi effetti
finiscono a L3/L4), **non** cambiare backend: aggiungi un **hero layer** documentato nel
reference `wgsl-hero.md` e, se il tuning manuale diventa il collo di bottiglia, promuovi il
subagent `enoki-vfx-iterate` (loop autora→build→review) a fase 1. Questo è coerente con la
"alternative path" dello spike 3 (enoki per il 70–80%, hero layer per il resto).

## Cosa cambierebbe la raccomandazione

- **Target visivo che si sposta a cinematic vero** (beam/ribbon/distortion/full-screen
  compositing pesanti): allora il centro del sistema non sarebbe più enoki e la skill andrebbe
  riscritta attorno a un hero-renderer, non al particle backend.
- **`VfxAsset` diventa il source-of-truth canonico** (Strada A dello spike 3): la skill dovrà
  parlare `VfxAsset`-first e trattare `.particle.ron` come target di compilazione.
- **Doppione con `vfx-realtime`**: se in pratica si duplica invece di referenziare, la skill
  diventa debito → tenere la disciplina "principi linkati, mapping locale aggiunto".

## Next steps se accettata

1. Implementare la skill `bevy-enoki-vfx` (follow-up di authoring, fuori da questo spike).
2. Aggiornare la memoria `project-baby-flame-vfx-redo` con un link `[[bevy-enoki-vfx]]`.
3. (Opzionale, fase 2) subagent `enoki-vfx-iterate` sul modello di `sprite-iterate`.
4. Validare i trigger: aprire un file VFX e verificare che la skill si auto-carichi.

## Final call (one line)

> **Sì — il modo per rendere l'agent esperto NON è cercare una skill online (non esiste per
> enoki/anime-cel), ma scrivere una skill project-local snella che salda i principi VFX
> esistenti al backend `bevy_enoki` e codifica la regola asset-vs-primitiva L0→L4 del progetto.**

---

### Fonti

- [HSR VFX Style Study — Nathan Lacsamana](https://nathanlacsamana.com/hsr-vfx-style-study)
- [HSR-inspired Unity VFX — 80.lv](https://80.lv/articles/get-these-honkai-star-rail-inspired-hit-impact-unity-made-vfx)
- [Real-Time VFX Fundamentals for UE5 — Gnomon](https://www.thegnomonworkshop.com/workshops/real-time-vfx-fundamentals-for-unreal-engine-5)
- [Cel Shading expert guide — Wayline](https://www.wayline.io/blog/cel-shading-a-comprehensive-expert-guide)
- [bevy_enoki — GitHub / README](https://github.com/Lommix/bevy_enoki)
- [anthropics/skills — formato skill ufficiale](https://github.com/anthropics/skills)
- [travisvn/awesome-claude-skills](https://github.com/travisvn/awesome-claude-skills) · [VoltAgent/awesome-agent-skills](https://github.com/VoltAgent/awesome-agent-skills)
