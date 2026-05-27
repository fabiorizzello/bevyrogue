# ANGLE-1 — Fonti & prior art (cosa esiste già da cui attingere)

## Obiettivo

Inventariare la conoscenza VFX riusabile — locale e online — per non riscrivere da zero,
e isolare con precisione **cosa manca** che giustifica una skill nuova.

## A) Skill/agent già presenti in locale

| Asset | Dove | Cosa copre | Gap rispetto al nostro target |
|---|---|---|---|
| `vfx-realtime` | `~/.agents/skills/vfx-realtime/` | VFX AAA generico: Shape/Timing/Color, anticipation, secondary motion, overdraw, LOD, juice. Reference: `patterns.md`, `sharp_edges.md`, `validations.md`. Tooling citato: Niagara, VFX Graph, Godot. | **Non parla `bevy_enoki`, non parla anime-cel, non conosce questo repo.** Ottima base di *principi*, zero conoscenza di *backend* e *convention*. |
| `vfx-realtime/references/patterns.md` | idem | Pattern di composizione effetti | Riusabile as-is come fondamenta concettuali |
| Memoria `project-baby-flame-vfx-redo` | `.claude/.../memory/` | Art direction decisa: anime cel (bande piatte + outline, core white-hot, HDR overbright), 12fps, scala 14–34px, **procedural-first / ~2 sprite** | È la "ground truth" di stile, ma è una memoria, non una skill che si carica sul trigger giusto |
| Skill `digimon` | `.claude/skills/digimon/` | Roster/evoluzioni — utile per coerenza tematica degli effetti | Non VFX |
| Skill `sprite-iterate` | `.claude/skills/sprite-iterate/` | Loop render→review→edit per sprite pixel-art | Pipeline 3D→pixel, non particle VFX |

**Conclusione A:** esiste un'ottima base di *principi* (`vfx-realtime`) e un'ottima base di
*stile/decisioni* (memoria baby-flame), ma **nessuna skill lega i principi al backend
`bevy_enoki` né codifica la regola asset-vs-primitiva**. Questo è esattamente il buco.

## B) Fonti online — principi anime-cel VFX

- **Disney 12 principi applicati al real-time VFX** (Gnomon "Real-Time VFX Fundamentals
  for UE5"): anticipation, slow-in/slow-out, timing, impact frames. Il look anime nasce
  da *timing + impact frames*, non dal solo materiale.
- **Cel shading = bande di colore piatte + contrasto netto + outline**: coerente con la
  nostra art direction. Il "cartoon look" arriva da value contrast e abrupt transitions,
  non da gradienti morbidi.
- Ricorrente: **sprite + mesh + animated material in multi-stage**; effetti anime sono
  *sequenze* (charge → release → impact → residue), non un singolo burst.

## C) Fonti online — breakdown HSR / Digimon Survive (tecniche concrete)

Da *HSR VFX Style Study* (Nathan Lacsamana) e raccolte Unity HSR-style:

1. **Particle indexing & rotation a offset** → segmenti a stella che si separano e ruotano
   verso l'esterno (la nostra `RotationParams::Radial`/`TowardTarget` mappa esattamente qui).
2. **Shape language a stella/kaleidoscope + motivo "glass break/shatter"**; **evitare
   decal**, puntare su burst core con particelle.
3. **Colore = gradienti + significato + alto contrasto**; impatti direzionali con colore
   ottimizzato per leggibilità (value-first, come dice `vfx-realtime`).
4. **Timing**: danno iniziale rapido e forte + follow-through che persiste; camera in stage.
5. **Texture custom per varietà** + **shader per la rifinitura** + **sorting layer** per il
   depth ordering.
6. **Block-out tecnico prima del full detail** — evita timeline ingestibili su VFX anime
   ornati.

> Insight chiave: il look "HSR/Digimon Survive" è **composizione + timing + shape language
> + value contrast**, NON un singolo trucco di rendering. Combacia con la conclusione dello
> spike 3 ("enoki è backend, il look è nel layer sopra").

## D) Fonti online — ecosistema skill da cui attingere/strutturare

- **`anthropics/skills`** — formato di riferimento ufficiale (SKILL.md + frontmatter +
  reference files + script). Modello strutturale, non contenuto VFX.
- **`VoltAgent/awesome-agent-skills`**, **`travisvn/awesome-claude-skills`**,
  **`ComposioHQ/awesome-claude-skills`**, **`hesreallyhim/awesome-claude-code`** — cataloghi
  community. Esistono skill 3D/game-dev/shader (es. "3D design team" con subagent), ma
  **nessuna trovata specifica per `bevy_enoki` o anime-cel particle VFX** → confermato che
  va scritta, non importata.
- Pattern riusabile da quei repo: **skill di principi + reference files densi + opzionale
  subagent per loop autonomi** (cfr. il nostro `sprite-iterate`).

## Pro / Contro / Confidence

**Pro (di costruire sopra il prior art):**
- `vfx-realtime` dà i principi gratis → la skill nuova può *referenziarla* e restare snella.
- Le tecniche HSR concrete mappano 1:1 su verbi già esistenti nel repo (`RotationParams`,
  `Turbulence`, anchor, lifecycle).
- Formato skill standard ben documentato (`anthropics/skills`).

**Contro / rischi:**
- `vfx-realtime` cita tooling sbagliato (Niagara/VFX Graph) → rischio che l'agent suggerisca
  feature non disponibili in enoki se non c'è una skill che "atterra" i principi su enoki.
- Le fonti HSR sono Unity-centriche → vanno tradotte, non copiate.
- Nessuna skill pronta da importare → tutto il valore è nello scrivere la mappatura repo-specifica.

**Confidence: Alta.** Il panorama è chiaro: principi abbondanti, backend-mapping assente.
