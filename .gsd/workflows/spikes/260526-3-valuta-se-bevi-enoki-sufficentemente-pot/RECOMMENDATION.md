# Recommendation — bevy_enoki / extension-first / editor readiness

## Executive summary

La risposta breve è:

- **`bevy_enoki` sì, ma con limiti.** È abbastanza potente come **backend particle 2D stylized** per un combat game come questo, soprattutto se lo combini con anim graph, cue system, HDR bloom, shake, freeze e sprite authored. **Non va interpretato come l’unico strato capace di produrre da solo VFX “alla HSR” completi e cinematografici.**
- **L’architettura è già sorprendentemente buona sul lato extension-first**, soprattutto nel combat kernel. Sul lato windowed/presentation la direzione è giusta e in larga parte reale, ma non ancora completamente “drop-in”: esistono ancora choke point manuali di bootstrap, registrazione e catalog discovery.
- **L’editor futuro è plausibile**, ma prima va chiusa una scelta architetturale: oggi il repo ha una schema VFX typed/editor-ready (`VfxAsset`) molto promettente, però il runtime effettivo usa ancora soprattutto `.particle.ron` di enoki + wiring Rust. Finché esistono due linguaggi di authoring paralleli, l’editor rischia di nascere zoppo.

La raccomandazione complessiva quindi è:

> **continua con `bevy_enoki`, non cambiare backend ora; però trattalo come backend di rendering particellare dentro un layer proprietario più alto.** La priorità non è sostituire enoki: è **unificare il source-of-truth presentation/VFX** e completare la transizione verso una vera architettura Digimon-as-extension.

## Comparison matrix

| Area | Verdict | Stato reale | Implicazione |
|---|---|---|---|
| Potenza `bevy_enoki` | **Sì con limiti** | buono per particle VFX 2D stylized, hot reload, custom material, bloom-friendly | ottimo backend, non linguaggio completo di cinematic VFX |
| VFX tipo HSR / Digimon Survive | **Parzialmente** | raggiungibile nel “feel class” se layeri più sistemi; non 1:1 solo con particles | serve composizione: particles + sprite + cues + shaders + camera language |
| Extension-first kernel | **Sì** | boundary combat molto buono | Digimon-specific gameplay già ben isolabile |
| Extension-first windowed/presentation | **Parzialmente sì** | registri generic buoni, ma bootstrap/cataloghi ancora manuali | aggiungere Digimon è possibile ma non ancora economico come dovrebbe |
| Editor readiness | **Quasi** | typed schemas e hot reload ci sono; source-of-truth unico no | prima unificare il modello, poi costruire l’editor |

## Recommendation

## 1) Non cambiare `bevy_enoki` adesso

### Motivo

Dallo spike non emerge un motivo abbastanza forte per buttare via `bevy_enoki`.

Anzi, emerge il contrario:

- è compatibile con Bevy 0.18 / crate attuale
- ha hot reload
- ha editor dedicato
- permette custom material/shader
- nel repo è già integrato con lifecycle utili (`PersistentEmitter`, `Projectile`, `OneShot`)
- è già il backend unico del path windowed

Quindi il collo di bottiglia principale **non è il crate in sé**.

Il collo di bottiglia è più spesso:

- il layer di orchestration sopra
- la frammentazione dell’authoring
- la copertura incompleta del roster

## 2) Ridimensiona correttamente l’ambizione assegnata a `bevy_enoki`

Usalo come:

- **backend particle 2D**
- layer per burst, aura, projectile, impact, swirl, spark, detonation
- componente di una grammatica VFX più ampia

Non usarlo mentalmente come:

- “engine VFX totale”
- sostituto di staging, slash shapes, authored overlays, beam language, distortion, cinematic compositing

Per gli effetti più “signature” la strategia giusta è probabilmente:

- particelle enoki per il corpo energetico / debris / sparks / glow
- sprite authored / atlas tricks per la silhouette leggibile
- cue/camera/freeze/bloom per l’impatto
- eventualmente custom material WGSL per i casi hero

## 3) Investi prima nell’unificazione dell’authoring, poi nell’editor

Questa è la vera priorità emersa.

Oggi hai:

- una lingua typed/editor-ready (`src/animation/vfx_asset.rs`)
- una lingua runtime effettiva (`.particle.ron` enoki + registri Rust)

Se costruisci l’editor prima di decidere quale delle due è il **source-of-truth**, rischi di fare UI sopra un modello che poi devi rifare.

### Raccomandazione concreta

Scegli una di queste due strade e convergi:

#### Strada A — `VfxAsset` come lingua canonica del progetto

- editor custom parla `VfxAsset`
- runtime compila/adatta da `VfxAsset` a enoki
- enoki resta backend, non modello authoring finale

**Questa è la strada che consiglierei** se il tuo obiettivo forte è davvero un editor custom coerente.

#### Strada B — enoki `.particle.ron` come lingua canonica bassa + metadata layer nostro

- enoki editor per l’effetto singolo
- nostro catalogo typed solo per mapping skill/graph/anchor/lifecycle

È più economica nel breve, ma più frammentata nel lungo periodo.

## 4) Porta a compimento il modello “Digimon as extension” anche sul windowed

Qui la raccomandazione non è cambiare idea, ma finire il lavoro.

Le priorità sono:

1. eliminare bootstrap/cataloghi hardcoded dove possibile
2. sostituire singleton poco scalabili (`DetonateEffectRegistry`) con registri keyed
3. rendere uniformi i dati presentation nel roster
4. aumentare il numero di Digimon che passano davvero nel seam windowed generic

In altre parole:

> la direzione è giusta; va resa più sistemica e meno “Agumon-first”.

## Alternative path if primary recommendation fails

Se, durante l’implementazione dei prossimi Digimon o dei prossimi VFX hero, emergesse che `bevy_enoki` limita troppo gli effetti signature, la strada alternativa migliore **non** sarebbe subito “cambiare particle crate”.

Sarebbe invece:

1. tenere enoki per il 70–80% degli effetti comuni
2. aggiungere un **hero layer** dedicato per i casi speciali:
   - sprite authored overlay
   - shader/material custom
   - eventuale beam/slash bespoke renderer
3. usare il particle backend solo dove è davvero il fit giusto

Questo riduce il rischio di riscrivere troppo.

## What would change the recommendation

Cambierei raccomandazione se succedesse una di queste cose:

### 1. Cambio di target visivo

Se il target diventasse:

- VFX molto più cinematici
- forte uso di distortion / beams / ribbon trails / full-screen authored compositing
- “replicare davvero” HSR invece di stare nella stessa famiglia di leggibilità

allora `bevy_enoki` da solo diventerebbe probabilmente troppo stretto come centro del sistema.

### 2. Attrito grave di tooling

Se il team scoprisse che:

- i `.particle.ron` enoki sono troppo bassi livello per iterare bene
- il ponte tra skill graph e VFX è troppo verboso
- l’editor enoki non si incastra bene col resto della pipeline

allora converrebbe accelerare la trasformazione di `VfxAsset` in modello canonico.

### 3. Problemi di scala del seam windowed

Se al terzo/quarto Digimon presentation-first iniziasse a emergere troppo codice shared modificato per ogni onboarding, allora il giudizio “extension-first abbastanza buono” andrebbe abbassato e servirebbe una refactor più urgente del bootstrap/catalog system.

## Concrete next steps

## Step 1 — Decisione architetturale VFX

Prendere una decisione esplicita:

- `VfxAsset` canonico sopra enoki
- oppure enoki canonico + metadata layer nostro

Se vuoi l’editor custom, la mia raccomandazione è:

> **`VfxAsset` canonico, `bevy_enoki` backend.**

## Step 2 — Catalogo discovery unificato

Rimuovere il più possibile i default hardcoded tipo:

- `DEFAULT_ANIM_GRAPH_PATHS`
- `DEFAULT_ANIM_CLIP_PATHS`
- `DEFAULT_ANIM_STANCE_PATHS`

Sostituendoli con un catalogo o discovery per-Digimon più data-driven.

## Step 3 — Chiudere i registri non ancora scalabili

In particolare riesaminare:

- `DetonateEffectRegistry`
- eventuali altri seam singleton residuali

Obiettivo: tutto keyed per skill / effect / presentation owner, non singleton globali quando la semantica è per-Digimon.

## Step 4 — Portare almeno altri 1–2 Digimon nel path windowed completo

Prima dell’editor, conviene fare una prova di scala reale:

- un Digimon con VFX principalmente projectile/ranged
- un Digimon con VFX più aura/buff/AoE

Questo dirà molto meglio di un design doc se il seam regge davvero.

## Step 5 — Solo dopo, costruire l’editor

Quando:

- il source-of-truth è unico
- il catalogo è discoverable
- il seam è provato su più Digimon

allora l’editor diventa un acceleratore.

Prima, rischia di essere un moltiplicatore di debito.

## Final recommendation in one line

> **Sì, `bevy_enoki` è abbastanza potente per questo progetto — ma come backend, non come intera filosofia VFX. Il vero lavoro ora è completare la transizione verso authoring unificato + Digimon-as-extension, così da rendere credibile anche l’editor futuro.**
