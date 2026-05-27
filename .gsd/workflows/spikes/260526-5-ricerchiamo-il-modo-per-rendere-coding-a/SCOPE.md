# SCOPE — Coding agent esperto in VFX anime cel-shading per bevy_enoki

**Data:** 2026-05-26
**Template:** spike (low complexity)
**Branch:** master

## La domanda

Come rendiamo un coding agent (PI / Claude / Codex) **affidabilmente esperto** nel
produrre VFX in stile **anime cel-shading** (riferimenti: *Digimon Survive / Time
Stranger*, *Honkai: Star Rail*) e nell'applicarli al backend **`bevy_enoki`** già
usato in questo progetto — incluso il giudizio su **quando servono asset hand-authored
e quando bastano le primitive `bevy_enoki` + curve + math**?

Sotto-domande:

1. **Cosa esiste già** (qui e online) da cui attingere, per non riscrivere conoscenza
   VFX generica da zero?
2. **Cosa è specifico di `bevy_enoki`** e del look anime-cel di questo progetto, che
   nessuna skill generica può sapere?
3. **In che forma** va impacchettata questa conoscenza (skill project-local? subagent?
   reference docs?) perché si auto-carichi e sia davvero usata dall'agent al momento giusto?

## Perché ora

- Il VFX di Agumon (`baby_flame`) è in pieno redo procedural-first (vedi memoria
  `project-baby-flame-vfx-redo`); a breve arriveranno altri Digimon con VFX nuovi.
- Lo spike 3 ha già concluso: **`bevy_enoki` = buon backend particellare 2D, NON
  linguaggio VFX cinematico completo** → il valore è nel *layer di orchestration e nelle
  decisioni di authoring sopra il crate*, esattamente la conoscenza che un agent sbaglia
  più spesso.
- Esiste una skill `vfx-realtime` generica (Niagara/VFX Graph/Godot) ma **non parla
  `bevy_enoki`, non parla anime-cel, non conosce le convention di questo repo**.

## Criteri di successo (cosa deve contenere una risposta utile)

Una raccomandazione utile deve produrre:

- [ ] **Inventario delle fonti** riusabili: skill esistenti (locali + online), reference
  di art-direction anime-cel, breakdown VFX HSR/Digimon, doc/esempi `bevy_enoki`.
- [ ] **Decision rule "asset vs primitiva"** esplicita e testabile, calibrata sulla scala
  reale del gioco (particelle 14–34px, 12fps, HDR+bloom).
- [ ] **Una scelta di formato** go/no-go: skill `.claude/skills/<name>/` vs subagent
  `.gsd/agents/` vs entrambi — con motivazione.
- [ ] **Bozza concreta** della struttura della skill/agent (frontmatter, trigger,
  reference files, esempi), pronta da implementare in un follow-up.
- [ ] Cosa NON fare (anti-pattern: aspettarsi cinematic HSR 1:1 dal solo enoki, ecc.).

## Vincoli

- Deve integrarsi con il path `windowed` esistente (`EnokiPlugin`, `EnokiVfxRegistry`,
  `VfxAsset`, lifecycle `PersistentEmitter/Projectile/OneShot`, anchor semantici).
- Deve rispettare l'art direction già decisa: anime cel (bande piatte + outline, core
  white-hot, HDR overbright), **procedural-first / pochi asset**.
- No production code in questo spike — output = conoscenza + bozza skill.
- Deve essere coerente con le conclusioni degli spike 2 e 3 (non ri-litigarle).

## Angoli di ricerca

- **ANGLE-1 — Fonti & prior art**: skill/agent VFX esistenti (locale `vfx-realtime` +
  cosa c'è online da cui attingere), reference di anime cel-shading VFX, breakdown
  tecnici HSR / Digimon Survive. Cosa è riusabile e cosa manca.
- **ANGLE-2 — bevy_enoki + decision rule**: mappare capacità/limiti enoki (da spike 3) e
  l'art direction del repo su una **regola operativa asset-vs-primitiva** che l'agent
  possa applicare deterministicamente.
- **ANGLE-3 — Forma del deliverable**: skill vs subagent vs reference; meccanica di
  auto-discovery (`skill-discovery.ts`), trigger keywords, struttura file, e bozza.

## Formato della decisione

Go/no-go sul **creare una skill project-local `bevy-enoki-vfx`** (e/o subagent), con
struttura proposta e prossimi passi. Decision-only fallback: one-liner in `.gsd/DECISIONS.md`.
