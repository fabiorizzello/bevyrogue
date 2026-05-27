# ANGLE-2 — bevy_enoki + la regola "asset vs primitiva"

## Obiettivo

Trasformare le capacità/limiti di `bevy_enoki` (già caratterizzati nello spike 3) e
l'art-direction del repo in una **regola operativa deterministica** che l'agent possa
applicare: *quando bastano primitive enoki + curve + math, e quando serve un asset
hand-authored (texture/atlas) o uno shader custom.*

## Capacità reali del backend (verificate)

### `.particle.ron` (autorabile senza codice)

`spawn_rate`, `spawn_amount`, `emission_shape` (Point | Circle), `lifetime`,
`linear_speed`, `linear_acceleration`, `direction`, `angular_speed/accel`,
`gravity_direction/speed`, `linear_damp`, `angular_damp`, `scale`, `color`,
`scale_curve`, `color_curve` (MultiCurve f32 + LinearRgba). Tutti i numerici con `Rval`
(± random). Sprite-sheet animation over lifetime via `SpriteParticle2dMaterial` (frame
H×V). Hot-reload.

### Layer locale sopra enoki (verbi del repo)

- **`PlacementParams`**: `ConvergeInward`, `FanOut`, `ArcLaunch`, `Turbulence`, `Static`
- **`RotationParams`**: `Static`, `Radial{offset,omega}`, `TowardTarget{offset,omega}`,
  `Fixed{angle,omega}` ← mappa **esattamente** sul "particle indexing/rotation a offset"
  delle tecniche HSR (ANGLE-1.C.1)
- **`PlacementAnchor`**: `Mouth`, `CasterCenter`, `TargetCenter`
- **`EnokiLifecycle`**: `PersistentEmitter`, `Projectile{...on_arrival}`, `OneShot`
- **`on_expire` / `ImpactSpawnPlan`**: chaining multi-stage (charge→release→impact→residue)
- Render path: `Hdr` + `Bloom::NATURAL` + `Tonemapping::TonyMcMapface` → l'overbright
  white-hot core "gratis" via bloom.

### Cosa richiede CODICE/shader (non autorabile in `.ron`)

- Logica fragment shader → `Particle2dMaterial` trait custom (WGSL): dissolve, distortion,
  fresnel/rim, mask/erosion, gradient-map procedurale.
- Verbi di movimento non nativi (es. il `Projectile` caster→target è già orchestrato dal
  nostro `advance_enoki_projectiles`, non dal `.ron`).
- Trail/ribbon/beam mesh, sub-emitter complessi, screen-space compositing → **non
  primitive native** (confermato spike 3).

## La regola "asset vs primitiva" (deliverable centrale)

Ordine di preferenza, dal più economico al più costoso. **Sali di livello solo quando il
livello sotto non regge il look richiesto.**

### Livello 0 — Primitiva pura (preferito di default)
Particella enoki con `color`/`scale` solidi + `color_curve`/`scale_curve` + bande HDR per
il cel look. **Nessun asset.**
→ Usa per: glow core, spark, motes, debris puntiforme, flash, shockwave radiale semplice.
Condizione: la silhouette è leggibile come **forma astratta** (punto/cerchio/streak).

### Livello 1 — Primitiva + math di placement/rotation
Aggiungi `PlacementParams` + `RotationParams` + `Turbulence` per dare *intenzione*
(converge, fan-out, radial spin, toward-target). **Ancora nessun asset.**
→ Usa per: aura swirl, ember convergente, shard a stella che ruotano (look HSR star-burst),
ventaglio di impatto. Questo livello copre **la maggior parte del look anime-cel** del gioco
data la scala 14–34px.

### Livello 2 — Asset sprite cel minimale (1 atom riusabile)
Una texture cel piccola, single-element, **canonicamente orientata** (es. `flame_tongue`
points-up), luminanza→alpha, riusata su più effetti via rotation/scale.
→ Usa **solo quando** serve una **silhouette riconoscibile** che la primitiva non dà:
fiamma a lingua, foglia, lama, petalo, artiglio, fulmine stilizzato, simbolo grafico.
Regola: *1 atom, N effetti* — lo stesso sprite, ruotato/scalato, serve molti kit. (Esempio
applicato: l'art direction procedural-first del progetto punta a ~2 sprite totali sull'intero
roster, non per-effetto.)

### Livello 3 — Sprite-sheet / atlas animato
`SpriteParticle2dMaterial` con flipbook, quando la *forma cambia nel tempo* in modo che
curve+rotation non riescono a esprimere (es. fiamma che guizza, esplosione a frame chiave).
→ Costo medio-alto (autoring frame-by-frame). Usa per **hero moment** non per VFX comuni.

### Livello 4 — Custom `Particle2dMaterial` (WGSL)
Solo per **signature/hero**: dissolve, distortion, rim, gradient-map procedurale,
mask erosion. Massima potenza, massimo costo di manutenzione.

### Trigger di escalation (quando salire di livello)
- "Non si capisce cosa sia" a 14–34px → potresti aver bisogno di L2 (silhouette).
- "Serve che la forma evolva" → L3.
- "Serve un look materico/energetico che il colore piatto non dà" → L4.
- Altrimenti: **resta al livello più basso che funziona** (principio `vfx-realtime`:
  "l'effetto che non c'è è il più economico").

## Decision table sintetica

| Effetto target | Livello consigliato | Asset? |
|---|---|---|
| Glow core / white-hot center | L0 | No (bloom fa il lavoro) |
| Spark / debris / motes | L0–L1 | No |
| Aura / swirl / ember converge | L1 | No |
| Star-burst shard rotanti (HSR) | L1 | No (`RotationParams::Radial`) |
| Projectile caster→target | L1 (lifecycle `Projectile`) | Opzionale comet sprite |
| Fiamma a lingua / lama / petalo leggibile | L2 | Sì, 1 atom cel |
| Esplosione/guizzo a frame chiave | L3 | Sì, sprite-sheet |
| Dissolve / distortion / rim hero | L4 | Shader WGSL |

## Pro / Contro / Confidence

**Pro:** la regola è ancorata a verbi reali del repo e alla scala reale → l'agent può
deciderla senza "inventare" feature. Spinge verso procedural-first (allineato all'art
direction e a "fewer artists").

**Contro / rischi:** la soglia L1→L2 ("silhouette leggibile") resta qualitativa; va
calibrata guardando in-game a 12fps. Rischio che l'agent sovra-stimi e salti a L3/L4 per
moda → la skill deve enfatizzare il bias verso il livello basso.

**Confidence: Alta** sulle capacità (verificate in codice + README), **Media** sulla
soglia esatta L1/L2 (dipende da giudizio visivo).
