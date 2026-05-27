# VFX — prossimi step (Baby Flame e dintorni)

Stato a fine sessione 2026-05-27. Cattura il lavoro VFX già fatto e cosa resta, così
la prossima sessione riparte senza ricostruire il contesto dalla chat.

## Dove siamo

Il sistema VFX nuovo (bevy_enoki `.particle.ron` + `EnokiVfxRegistry`) è in piedi. Il
vecchio quad system (`VfxAsset`/`vfx.ron`) è stato rimosso. Sul Baby Flame di Agumon è
stata costruita la catena a 5 stadi (charge → fiamma definita → sputo+trail → impact →
dissolvenza), con:

- soft particle material condiviso (`assets/vfx/soft_particle.png`) per i layer-glow;
- flipbook fiamma `assets/vfx/flame_sheet.png` (4×4, generato da `scripts/gen_flame_sheet.py`)
  instradato solo sui layer-forma (flames / projectile / impact) via
  `EnokiEffect.material_override` + predicato `uses_flame_flipbook()`;
- fix `spawn_rate` (è un intervallo in secondi, non un rate) e `scale_curve` (sovrascrive
  la scala, non moltiplica) su tutti gli asset Baby Flame;
- calibrazione glow-first (HDR core ~5, impact HDR sopra 1.0, bloom intensity 0.30 nel
  viewer e nel render).

Tutto verificato headless/windowed-check; suite verdi. **Il giudizio estetico è
windowed-only (K001) → lo fa l'utente** lanciando il viewer.

## Prossimi step

1. **UAT visivo Baby Flame** (utente, K001).
   ```
   cargo run --features windowed --bin vfx_viewer
   ```
   Scegliere i 3 preset `[composite] Baby Flame…` (charge body / spit+trail /
   impact+dissolve), registrare un `.webm`, poi tarare i numeri (densità, HDR core,
   scale_curve in px, bloom). Probabili 1–2 giri.

2. **Dissolvenza come effetto separato chainato.** Oggi lo stadio 5 è ripiegato dentro
   `impact` (flash → dissolvenza nella stessa curva). Per una dissolvenza distinta serve
   allargare `on_arrival` da `String` a lista — tocca anche Renamon e il suo contract test.

3. **Commit del lavoro VFX su branch dedicato** (siamo su `master`). Include: rimozione
   editor in-repo, soft material, fix spawn_rate, layering, flipbook, refinement skill.

## Debito noto

- **Split `render.rs` non committato** nel working tree (`render/spawn.rs|playback.rs|
  effects.rs|clock.rs|feedback.rs` untracked). Alcuni grep-test fanno
  `include_str!(".../render.rs")` e cercano funzioni ora spostate → falliscono finché gli
  `include_str!` non puntano ai nuovi `render/*.rs`. Da sistemare insieme al commit.
- **Flipbook procedurale** = placeholder onesto. Il tetto qualitativo resta EmberGen o
  disegno a mano (4×4). La "fiamma definita" vera passa da un asset migliore.

## Riferimenti

- Clip target Digimon Survive: `/home/fabio/Video/digi_ref_vfx/` (+ `gif/`).
- Skill: `.agents/skills/bevy-enoki-vfx/` (engine-generic; ricette glow/layering/flipbook).
