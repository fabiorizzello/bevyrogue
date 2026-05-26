# Angle 3 — editor readiness e authoring data-driven

## Obiettivo

Valutare se la struttura attuale del progetto è compatibile con un futuro **editor in-process**, oppure se esistono ancora hardcode e split di responsabilità che renderebbero fragile o costoso costruirlo.

## Evidenza raccolta

File chiave:

- `docs/future_design_draft/02-05_tunable_catalog.md`
- `docs/future_design_draft/02-04_strong_typing.md`
- `docs/future_design_draft/02-02b_animation_fsm.md`
- `src/animation/plugin.rs`
- `src/animation/registry.rs`
- `src/animation/vfx_asset.rs`
- `src/animation/placement.rs`
- `assets/digimon/agumon/vfx.ron`
- `tests/animation/vfx_asset_schema.rs`
- `tests/animation/vfx_asset_load.rs`

## Segnali positivi

### 1. L’intenzione architetturale è già molto corretta

`docs/future_design_draft/02-05_tunable_catalog.md` è chiarissimo:

- i dati ad alta frequenza di iterazione dovrebbero vivere in **RON asset typed**
- il runtime dovrebbe leggere da asset hot-reloadabili
- Rust dovrebbe contenere **regole**, non valori di tuning

Questa è esattamente la premessa giusta per un editor serio.

### 2. La pipeline animation è già vicina al modello editor-friendly

`src/animation/plugin.rs` usa:

- `RonAssetPlugin::<AnimGraph>`
- `RonAssetPlugin::<Clip>`
- `AssetServer::load(...)`
- tracking di `AssetEvent::LoadedWithDependencies` / `Modified`
- registri separati per skill graph e stance graph

`src/animation/registry.rs` inoltre clona `ResolvedAnimGraph` in snapshot per disaccoppiare player runtime e hot reload.

Questa parte è tecnicamente molto promettente.

### 3. Esiste già una schema VFX typed e editor-ready

`src/animation/vfx_asset.rs` è forse il segnale più forte in assoluto.

Contiene una schema fortemente typed che deriva:

- `Asset`
- `Serialize`
- `Deserialize`
- `Reflect`

e modella:

- `VfxAsset`
- `EffectDef`
- `Placement`
- `Appearance`
- `ScaleCurve`
- `ColorCurve`
- `RotationParams`
- `PlacementParams`

Il commento del file dice esplicitamente che questa surface è pensata per essere:

- introspectable
- editor-ready
- closed-vocabulary

Quindi il progetto **ha già una lingua typed per un editor futuro**.

### 4. Esistono test espliciti sulla readiness della schema

`tests/animation/vfx_asset_schema.rs` e `tests/animation/vfx_asset_load.rs` verificano:

- roundtrip RON lossless
- `deny_unknown_fields`
- presenza di `Reflect`
- parsing dell’asset reale `assets/digimon/agumon/vfx.ron`
- validazione delle placement verbs

Questo è ottimo: non è solo una bozza di schema, è una base tecnica concreta.

## Gaps reali che oggi bloccano o rallentano un editor

### 1. La schema VFX typed esiste, ma oggi non è il source-of-truth runtime

Questo è il gap più importante emerso nello spike.

Audit pratico:

- `assets/digimon/agumon/vfx.ron` è referenziato **solo nei test**
- non emerge un path runtime che lo carichi e lo usi come source-of-truth
- il runtime windowed effettivo usa invece:
  - `bevy_enoki` `.particle.ron`
  - registrazione Rust manuale in `src/windowed/digimon/agumon/mod.rs`

Quindi oggi convivono due mondi:

1. **mondo typed/editor-ready** (`VfxAsset`) — molto promettente
2. **mondo runtime effettivo** (`Particle2dEffect` enoki + registri Rust) — quello realmente usato

Per un editor, questa duplicazione è pericolosa: rischi di editare un asset “bello” che però non guida davvero il runtime.

### 2. L’authoring VFX è spezzato in troppi punti

Per un singolo Digimon oggi il wiring visuale è distribuito tra:

- `skills.ron` (cue/presentation parziale)
- `anim_graph.ron`
- `stance.ron`
- `clip.ron`
- `.particle.ron` enoki
- costanti Rust nel modulo per-Digimon
- registrazione Rust di effect ids / anchors / lifecycle

Questo non impedisce un editor, ma significa che l’editor dovrebbe sapere orchestrare **molti artefatti e molto wiring implicito**.

### 3. Alcuni cataloghi sono ancora hardcoded, non discoverable

`src/animation/plugin.rs` / `src/animation/registry.rs` hanno ancora:

- `DEFAULT_ANIM_GRAPH_PATHS`
- `DEFAULT_ANIM_CLIP_PATHS`
- `DEFAULT_ANIM_STANCE_PATHS`

Quindi il set di asset noti al runtime non nasce ancora da un catalogo dati unico.

Per un editor questo è un problema pratico:

- un asset nuovo potrebbe esistere su disco ma non entrare nel boot/runtime finché non editi codice
- l’editor non ha un “index” dichiarativo chiaro del roster visuale

### 4. La copertura data-driven è ancora disomogenea sul roster

Stato attuale `assets/digimon/`:

- Agumon: `anim_graph.ron`, `clip.ron`, `stance.ron`, `vfx.ron`, vari `.particle.ron`
- Renamon: `anim_graph.ron`, `clip.ron`, `stance.ron`
- gli altri Digimon hanno ancora solo atlas `.json/.png`

Stato attuale `assets/data/digimon/*/skills.ron`:

- solo Agumon usa `presentation: Some(...)`
- tutti i `vfx` in quei blocchi sono `vfx: None`
- gli altri Digimon non hanno quasi nulla di presentation metadata

Quindi l’editor oggi non avrebbe ancora una superficie omogenea da esporre per tutto il roster.

### 5. L’editor ideale non può appoggiarsi direttamente solo a Enoki Editor

L’editor di `bevy_enoki` è utile per il **particle asset singolo**, ma il tuo problema è più grande:

- sequencing per skill
- mapping cue -> effect
- anchor/lifecycle semantico
- graph/stance sync
- per-Digimon package boundary

Quindi il tool di enoki è una **parte della soluzione**, non la soluzione dell’authoring totale.

## Cosa significa davvero “editor-ready” qui

Se l’obiettivo futuro è un editor custom, la domanda non è “posso modificare un particle asset?”.

La domanda vera è:

> posso esprimere in dati editabili l’intera relazione tra skill, graph, cue, VFX, atlas, anchor e varianti per-Digimon?

La risposta attuale è:

- **quasi sì** sul piano delle intenzioni e delle types
- **non ancora** sul piano del source-of-truth runtime unificato

## Scelta architetturale implicita da risolvere

Lo spike evidenzia una biforcazione che prima o poi va chiusa.

### Opzione A — `VfxAsset` typed come source-of-truth

Modello:

- l’editor parla la lingua di `VfxAsset`
- il runtime traduce/compila da `VfxAsset` a enoki spawners / instances / helper entities
- `.particle.ron` enoki diventa implementazione interna o formato derivato

Pro:

- perfetto per editor typed
- coerenza con `Reflect`
- vocabolario stabile del progetto

Contro:

- devi mantenere un adapter serio verso enoki
- parte delle capability native di enoki diventano “backend details” da mappare

### Opzione B — `.particle.ron` enoki come source-of-truth basso livello + metadata layer nostro

Modello:

- enoki editor gestisce l’effetto singolo
- il progetto aggiunge un catalogo typed sopra per anchor/lifecycle/mapping/skill integration

Pro:

- sfrutti tooling esistente di enoki
- meno trasformazione

Contro:

- il modello complessivo dell’editor resta split in due linguaggi
- più difficile offrire un editor unico coerente per il designer

## Giudizio pratico

Per il tuo obiettivo (“in futuro creare un editor”), oggi il repo è in uno stato che definirei:

> **quasi editor-ready a livello di direzione, ma non ancora unificato a livello di source-of-truth.**

Non sei bloccato.

Però sei ancora in una fase in cui devi scegliere **qual è la vera lingua dell’authoring**.

## Tradeoff matrix

| Criterio | Valutazione | Note |
|---|---|---|
| Asset typed / hot reload per anim | Alto | base tecnica già solida |
| Schema typed / Reflect per VFX | Alto | `vfx_asset.rs` molto promettente |
| Runtime usa davvero la schema typed VFX | Basso | oggi no, quasi solo test |
| Uniformità roster | Basso-Medio | coverage ancora incompleta |
| Catalog discoverability | Medio-Basso | bootstrap ancora hardcoded |
| Possibilità di costruire editor futuro | Alto | sì come direzione |
| Possibilità di costruirlo senza refactor | Medio-Basso | serve unificazione |

## Verdict dell’angolo

### Risposta breve

L’architettura è **abbastanza buona da non bloccare un editor futuro**, ma **non ancora abbastanza consolidata da renderlo economico oggi**.

La readiness reale è:

> **quasi**, con un refactor chiave ancora da fare: unificare il source-of-truth dell’authoring VFX/presentation.

## Conclusione operativa

### Raccomandazione

Prima di investire seriamente nell’editor, conviene fare un piccolo passaggio architetturale intermedio:

1. scegliere il **source-of-truth unico** per i VFX authored
2. rendere discoverable i cataloghi di asset/graph/clip/stance
3. spostare altro wiring today-in-Rust verso cataloghi dati keyed
4. solo dopo costruire UI editor custom sopra quella lingua

## Confidence

**Alta** sul fatto che la direzione sia giusta, **media-alta** sul dettaglio della soluzione finale.

Motivo:

- i segnali tecnici sono forti e concreti (`Reflect`, typed RON, hot reload, registri)
- ma oggi il runtime effettivo e la schema più editor-friendly non coincidono ancora pienamente
