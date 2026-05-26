# Angle 1 — bevy_enoki come backend VFX

## Obiettivo

Valutare se `bevy_enoki` è abbastanza potente per supportare il tipo di VFX combat stylized che vogliamo, usando come benchmark qualitativo effetti in stile **HSR** o **Digimon Survive / Time Stranger**.

## Evidenza raccolta

### Evidenza esterna: crate `bevy_enoki`

Query usata: `bevy_enoki Particle2dEffect Bevy 0.15/0.16 features limitations GPU 2D particles examples projectile one-shot persistent emitter`

Fonti principali:

- `https://github.com/Lommix/bevy_enoki/blob/master/README.md`
- `https://docs.rs/bevy_enoki/latest/bevy_enoki/`

Punti emersi:

- `bevy_enoki` è un **particle system 2D** per Bevy.
- È descritto come **CPU calculate + GPU instancing**.
- Supporta:
  - asset `.particle.ron` hot-reloadabili
  - texture custom
  - sprite-sheet animation over lifetime
  - `OneShot`
  - `ParticleEffectInstance` per override runtime per-spawner
  - **custom material / custom fragment shader**
  - editor dedicato (`enoki2d_editor`)
- Lo schema `Particle2dEffect` pubblico esposto nella documentazione include soprattutto:
  - `spawn_rate`, `spawn_amount`
  - `emission_shape` (`Point`, `Circle`)
  - `lifetime`
  - speed / acceleration / direction
  - scale/color curves
  - gravity / damping

### Evidenza interna: integrazione nel repo

File chiave:

- `Cargo.toml`
- `src/windowed/render.rs`
- `src/windowed/digimon/agumon/mod.rs`
- `assets/digimon/agumon/*.particle.ron`
- `tests/windowed_only/enoki_impact_render.rs`
- `tests/windowed_only/vfx_windowed_contracts.rs`

Punti emersi:

- `bevy_enoki = { version = "0.6", optional = true }` dietro feature `windowed`.
- Nel render path windowed, `EnokiPlugin` è registrato e il vecchio quad system è stato rimosso: **enoki è il solo particle backend**.
- Il runtime locale aggiunge sopra a enoki un piccolo layer di orchestration:
  - `EnokiVfxRegistry`
  - `EnokiLifecycle::{PersistentEmitter, Projectile, OneShot}`
  - anchor `Mouth / CasterCenter / TargetCenter`
  - `advance_enoki_projectiles()` per far viaggiare il projectile e chainare l’impatto
- Agumon dimostra già una grammatica VFX non banale:
  - charge orb persistente
  - ember swirl persistente
  - projectile caster->target
  - impact burst
  - slash burst
  - detonate burst
- Il path windowed usa anche:
  - `Hdr`
  - `Bloom::NATURAL`
  - `Tonemapping::TonyMcMapface`
  - `DebandDither::Enabled`
  - `Color::linear_rgba(...)`

Questo significa che il backend non è solo “particles su schermo”, ma è già messo dentro un render path che favorisce **overbright + bloom**, cioè proprio una parte importante del look JRPG stylized.

## Cosa `bevy_enoki` fa bene

### 1. Copre bene il core dei VFX 2D stylized

Per il tipo di VFX già presenti o desiderati nel progetto, `bevy_enoki` copre bene:

- burst / impact
- emitter persistenti
- projectile / comet
- swirl / radial motion
- timing guidato dal gameplay o dall’anim graph
- hot reload asset-based
- texturing / sprite sheet animation
- bloom-friendly output

Tradotto: per **fireball, slash, spark burst, aura, detonation, heal motes, shockwave semplice**, il backend è plausibilmente sufficiente.

### 2. È già integrabile con un layer authored sopra

Nel repo il vero punto forte non è enoki “da solo”, ma enoki + il layer locale:

- sync con anim graph / cue barrier
- lifecycle esteso (`Projectile`, `PersistentEmitter`, `OneShot`)
- anchoring semanticamente utile (`Mouth`, `TargetCenter`)
- release timing esplicito

Questo compensa alcune primitive non-native del crate.

### 3. Ha una strada chiara verso tooling

Il fatto che enoki abbia:

- file `.particle.ron`
- hot reload
- editor dedicato
- custom material

lo rende una base concreta per una pipeline iterabile, specialmente se l’obiettivo è **2D stylized authored FX**, non simulazione complessa.

## Limiti osservati

## 1. La surface pubblica del particle asset è relativamente stretta

Dalla documentazione pubblica, `Particle2dEffect` espone un set abbastanza classico di parametri di emissione e curve, ma **non emerge** un linguaggio VFX molto ricco del tipo:

- sub-emitters / particle events complessi
- ribbon / trail mesh dedicati
- beam authoring di alto livello
- collision / scene interaction
- distortion / refraction / screen-space compositing built-in
- masking / dissolve / layered post-process authoring
- authored multi-stage cinematic sequencing built into the particle system

Questo non significa che siano impossibili in assoluto, ma che **non sono la primitive di primo livello** che si vede nella documentazione o nello schema base.

## 2. Alcune cose “belle” le stiamo già facendo fuori da enoki

Nel repo:

- il projectile caster->target non è un verb nativo del particle asset: è orchestrato dal nostro `ProjectileFlight`
- il chain su arrivo è gestito dal nostro runtime
- il sync semantico con skill graph / cue barrier è nostro

Quindi il verdetto corretto non è “enoki sa fare tutto”, ma:

> enoki sa fare bene la particella 2D; la grammatica di skill VFX combat la stiamo costruendo noi sopra.

## 3. Non basta da solo per il livello “HSR signature shot”

Se per “tipo HSR” si intende:

- forte layering di sprite authored + particles + screen FX
- silhouette chiarissima del colpo
- staging cinematico
- distorsioni, slash shapes, smears, flares, camera language, timing estremamente curated

allora **enoki da solo non basta**.

Serve almeno la combinazione di:

- particles
- sprite / atlas authored
- anim graph
- cue system
- bloom / camera shake / freeze
- eventualmente custom material / shader specifici

Questa è una distinzione importante:

- **come backend particle**: sì, buono
- **come intero sistema VFX AAA-like**: no, non da solo

## Tradeoff matrix

| Criterio | Valutazione | Note |
|---|---|---|
| Burst / impact 2D | Alto | già provato nel repo |
| Persistent emitters | Alto | charge + ember già presenti |
| Projectile travel | Medio-Alto | ottenuto bene, ma con orchestration custom |
| Sync con anim / cues | Alto | grazie al layer locale, non solo al crate |
| Hot reload / iteration | Alto | `.particle.ron` + file watcher + editor enoki |
| Shader extensibility | Medio-Alto | custom material disponibile |
| Cinematic multi-stage authored FX | Medio | possibile solo con più layer fuori dal particle asset |
| HSR-class signature polish puro | Medio-Basso | manca un linguaggio VFX high-level completo |
| Mobile/wasm friendliness | Alto | claim esplicito del crate |

## Verdettto dell’angolo

### Risposta breve

`bevy_enoki` è **sufficientemente potente** per il nostro obiettivo **se** lo interpretiamo come:

- backend particles 2D stylized
- dentro una pipeline più ampia fatta di anim graph, cue system, bloom, shake, sprite authored

Non è sufficiente se lo interpretiamo come:

- unico strato responsabile di esprimere da solo VFX “alla HSR” completi e cinematici.

## Rischi

1. **Sovra-aspettativa sul crate**: aspettarsi che enoki da solo copra trails/beams/distortion/cinematic choreography.
2. **Duplicazione di logica VFX nel runtime**: più orchestration custom costruiamo, più serve una grammatica pulita sopra il crate.
3. **Editor split-brain**: editor enoki da una parte, authoring skill/cue/graph dall’altra.

## Conclusione operativa

### Raccomandazione

**Continuare con `bevy_enoki`, ma come backend di basso livello / medio livello, non come soluzione totale.**

Strategia consigliata:

1. usare enoki per le particelle 2D vere e proprie
2. continuare a usare anim graph + cues per il sequencing
3. considerare sprite/material/shader authored per gli “hero moments” o slash / beam più signature
4. costruire un layer proprietario di authoring sopra enoki, invece di esporre direttamente il crate come linguaggio VFX finale

## Confidence

**Medio-alta.**

Motivo:

- la documentazione pubblica del crate e il codice del repo danno abbastanza evidenza sulla grammatica disponibile
- manca però una prova visuale diretta comparativa con effetti molto avanzati, quindi il giudizio sul target “HSR-like” resta qualitativo, non benchmarkato frame-by-frame
