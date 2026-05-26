# Angle 2 — stato reale dell’architettura extension-first

## Obiettivo

Verificare se l’architettura attuale consente davvero di aggiungere Digimon come **extension** invece che come branching hardcoded nel motore, con attenzione sia al combat kernel sia al layer windowed/presentation.

## Evidenza raccolta

File chiave:

- `docs/combat_current.md`
- `src/combat/blueprints/`
- `src/windowed/render.rs`
- `src/windowed/digimon/mod.rs`
- `src/windowed/digimon/agumon/mod.rs`
- `src/windowed/digimon/renamon/mod.rs`
- `tests/windowed_only/renamon_extension_contract.rs`
- `tests/windowed_only/agumon_module_extraction.rs`
- `tests/windowed_only/digimon_sprite_cue_dispatch.rs`

## Cosa funziona bene

### 1. Il combat kernel è davvero molto vicino al modello extension-first

`docs/combat_current.md` è abbastanza netto:

- il runtime `src/combat/api/` è generic kernel
- la logica specifica per Digimon vive in `src/combat/blueprints/<name>/`
- le estensioni passano da registri typed (`ExtRegistries`)
- la mutation path è chiusa (`Intent -> IntentQueue -> intent_applier`)
- il kernel non deve conoscere i singoli Digimon

Per il gameplay/combat puro, l’architettura è **forte**.

### 2. Anche il layer windowed ha fatto un buon passo verso registri generici

Nel render path windowed ci sono seam generiche molto sane:

- `EnokiVfxRegistry`
- `OnEnterEffectRegistry`
- `SkillReleaseEffectRegistry`
- `SkillStartNodeRegistry`
- `SpritePresentationRegistry`
- `CueRegistry`

In `src/windowed/render.rs` il sistema generic non cerca `agumon` o `renamon` con branch di specie: legge dati dai registri.

In particolare:

- `DigimonSprite` porta `stance_graph_id` e `skill_graph_id` come **data fields**, non come const di specie
- `presentation_entry_for_unit()` cerca per `unit_id` dentro `SpritePresentationRegistry`
- `spawn_effect_by_id()` fa lookup su `EnokiVfxRegistry`
- il layer di skill bridge usa registri `skill_id -> node` e `particle_name -> effect ids`

Questa è esattamente la direzione giusta.

### 3. I test pinzano esplicitamente il boundary extension-first

`tests/windowed_only/renamon_extension_contract.rs` verifica che:

- `src/windowed/render.rs` e `src/windowed/mod.rs` restino species-agnostic rispetto a Renamon
- il modulo `src/windowed/digimon/renamon/mod.rs` possieda i suoi dati
- l’engine consulti `SpritePresentationRegistry` invece di accessi hardcoded tipo `entries[0]`

Questo è importante perché non è solo “buona intenzione architetturale”: è un **contratto testato**.

## Limiti e punti dove l’estensibilità è ancora incompleta

### 1. L’aggregazione dei Digimon è ancora manuale

`src/windowed/digimon/mod.rs` contiene:

- `mod agumon;`
- `mod renamon;`
- `agumon::register(app);`
- `renamon::register(app);`

Quindi aggiungere un Digimon nuovo richiede ancora almeno:

1. creare il modulo
2. editare l’aggregatore centrale

Questo è accettabile, ma significa che non siamo ancora a un modello “drop folder / discover automatically”.

### 2. Il bootstrap asset per grafi/clip è ancora hardcoded

`src/animation/plugin.rs` e `src/animation/registry.rs` hanno ancora default catalog hardcoded:

- `DEFAULT_ANIM_GRAPH_PATHS`
- `DEFAULT_ANIM_CLIP_PATHS`
- `DEFAULT_ANIM_STANCE_PATHS`

Esempi attuali:

- graph paths di default: solo Agumon + Renamon
- clip paths di default: solo Agumon + Renamon
- stance paths di default: solo Agumon

Renamon deve addirittura fare push manuale in `AnimationStancePaths` a build time.

Questo è un segnale chiaro che:

- il runtime **supporta** più Digimon
- ma il **catalog discovery/bootstrap** non è ancora data-driven

### 3. La copertura del seam presentation è ancora parziale

Nel repo attuale:

- blueprint combat disponibili: `agumon`, `dorumon`, `gabumon`, `patamon`, `renamon`, `tentomon`
- moduli windowed presentation disponibili: solo `agumon`, `renamon`

Quindi la forma architetturale è corretta, ma la prova di scala è ancora limitata.

### 4. Alcuni registri sono generic, altri ancora troppo stretti

Punto positivo:

- `EnokiVfxRegistry` è una mappa per `effect_id`
- `SkillReleaseEffectRegistry` è una mappa per `skill_id`
- `SpritePresentationRegistry` è una lista di entry

Punto sospetto:

- `DetonateEffectRegistry` è **un singolo `Option<String>`**, non una mappa keyed per skill/specie

Questo sembra un residuo “quasi-generic” ma non ancora completamente scalabile. Se domani 3 Digimon avessero semantiche di detonate diverse, questo singleton rischia di diventare una strettoia.

### 5. La dichiarazione presentation nei `skills.ron` è ancora poco usata

Audit veloce dei file `assets/data/digimon/*/skills.ron`:

- solo `agumon/skills.ron` ha `presentation: Some(...)`
- tutti gli `vfx` in quei blocchi sono `vfx: None`
- gli altri Digimon hanno attualmente `presentationSome=0`

Questo significa che oggi la presentazione non è ancora descritta in modo omogeneo nel data layer per tutto il roster.

## Costo reale di aggiungere un “settimo Digimon” oggi

### Sul combat kernel

Costo relativamente buono:

- blueprint nuovo in `src/combat/blueprints/<name>/`
- registrazione estensioni
- dati in `assets/data/...`

Qui il modello tiene.

### Sul layer presentation/windowed

Costo ancora medio:

- modulo nuovo in `src/windowed/digimon/<name>/`
- edit dell’aggregatore `src/windowed/digimon/mod.rs`
- aggiunta dei path ai cataloghi animation bootstrap (direttamente o via push resource)
- registrazione manuale di:
  - stance path
  - sprite presentation
  - skill start nodes
  - eventuali cue ids
  - eventuali effect ids enoki
  - demo entries

Quindi siamo in uno stato:

> **extension-first sì, ma ancora “extension by Rust registration”, non “extension by pure data package”.**

## Tradeoff matrix

| Criterio | Valutazione | Note |
|---|---|---|
| Combat kernel extension-first | Alto | boundary forte e ben documentato |
| Presentation registries generic | Medio-Alto | buon disaccoppiamento raggiunto |
| Nuovo Digimon senza engine branching | Medio-Alto | vero in gran parte, ma non totale |
| Automatic discovery/bootstrap | Basso | ancora hardcoded/manuale |
| Whole-roster proof | Medio-Basso | seam provato solo su Agumon/Renamon |
| Risk of registry bottlenecks | Medio | `DetonateEffectRegistry` è il caso più evidente |
| Test protection del boundary | Alto | ci sono contract test mirati |

## Verdict dell’angolo

### Risposta breve

L’architettura è **davvero extension-first sul lato combat** e **abbastanza extension-first sul lato windowed/presentation**, ma **non ancora totalmente “specifica Digimon come extension package indipendente”**.

La frase più accurata è:

> il progetto ha già un **core generic corretto** e un **layer presentation in buona transizione**, ma ci sono ancora alcuni choke point di bootstrap e catalogo che obbligano a toccare codice shared.

## Rischi

1. **Falsa sensazione di completezza**: il boundary è bello, ma è ancora dimostrato su pochi Digimon.
2. **Registry seams non uniformi**: alcune surface sono map-based e scalabili, altre sono singleton o liste bootstrap hardcoded.
3. **Onboarding cost nascosto**: aggiungere un Digimon nuovo è meno costoso di prima, ma non ancora veramente “drop-in”.

## Conclusione operativa

### Raccomandazione

Conservare questa direzione architetturale: è quella giusta.

Per renderla davvero robusta come “Digimon as extension” servono soprattutto tre mosse:

1. **catalogo asset/data-driven** per graph/clip/stance discovery
2. **registri completamente keyed** dove esistono singleton residuali
3. **presentation metadata più uniforme** nel data layer del roster, non solo per Agumon

## Confidence

**Alta** sul giudizio strutturale.

Motivo:

- il repo contiene sia documentazione architetturale sia contract test che pinzano precisamente questi boundary
- la limitazione principale non è ambigua: è visibile direttamente nei cataloghi hardcoded e nel numero ridotto di moduli windowed presenti
