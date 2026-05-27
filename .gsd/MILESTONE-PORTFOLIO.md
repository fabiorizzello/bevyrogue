# Milestone Portfolio — M002 → M009

> **⚠️ Documento storico (pianificazione 2026-05-18).** Lo stato corrente e la
> sequenza milestone reale vivono in `.gsd/PROJECT.md` e nei `M0xx-ROADMAP.md`.
> Questo portfolio NON è più la fonte di verità: la numerazione milestone qui
> sotto (M004=Gabumon, M005=Dorumon, M006=Tentomon, M007=Renamon, M008=Patamon)
> è **divergente** dall'esecuzione reale (M003=render Agumon, M004=VFX
> data-driven, M005=visual feedback, M006=refactor extension-first + Renamon).
> Tienilo come traccia dell'intento roster originale, non come roadmap attiva.

**Status (storico):** scritto quando M002 era attivo e M003 era il prossimo da
pianificare. **Stato reale oggi: M002–M005 chiusi, M006 attivo** — vedi `PROJECT.md`.
**Ultimo update:** 2026-05-18.
**Nota canon:** lo scope M002 sotto usa il label "animation_fsm.ron" — il nome normativo è `clipmontage.ron` (§2.2b), vedi `M002-CONTEXT` decisione di naming.
**Scopo:** allineare il codice (`src/combat/`, `src/data/`, `src/ui/`) al canon design `docs/future_design_draft/` (round-3). Visual tracer end-to-end Agumon entro M003 — niente "headless first, visual last".

**Vincoli generali:**
- ID milestone strettamente incrementali (no lettere/suffissi).
- Headless first per kernel; `cargo check` e `cargo run` (no feature) verdi a ogni milestone.
- `windowed` additiva. Plugin split: `CombatPlugin` (core headless-safe) + `RenderPlugin` + `UiPlugin` gated `#[cfg(feature = "windowed")]`. Vedi `D008`.
- Integration test in `tests/` (headless), nomi funzionali, deterministici.
- Out of scope portfolio: meta-loop (StS-like), encounter chain, save/load, evolution Champion/Ultimate, enemy AI, balance pass → M030+.

**Spike:** SP1–SP5 in `.gsd/spikes/` (closed). **Decisioni:** D002–D010 in `.gsd/DECISIONS.md`.

---

## Completato (non ripianificare)

**Foundation kernel M017–M021 — tutti ✅ validati** (vedi STATE.md, `.gsd/milestones/M021/`):

| ID | Consegnato |
|---|---|
| M017 | Status taxonomy v0 (`Heated`/`Chilled`/`Paralyzed`/`Slowed`/`Blessed`, single-instance + refresh). |
| M018 | `AdvanceTurn`/`DelayTurn` split + `TargetShape` resolver (Blast/AoE/Bounce) + selectors estesi. |
| M019 | DR pipeline + `Effect::ApplyBuff`/`EmitHeal`/`EmitCleanse`. |
| M020 | Reactive bus canon (`StatusApplied`/`UltimateUsed`), shim legacy rimosso. |
| M021 | `trait Skill`/`SkillCtx`/`Intent` + `trait Blueprint`/registry + plugin split; kernel digimon-free, 5 enum Digimon-specific eliminati, 237 test verdi. |

**Hardening infra (refactor branch, 2026-05-18)** — non un milestone portfolio ma base per la verifica di M002–M003:
- RNG deterministico `bevy_rand` (`WyRand`/`CombatEntropy`, seed pinnato, roll-vector contract).
- Errori dominio tipizzati `thiserror` (`DataError`), RON parse failure tipizzato.
- Tracing strutturato JSON (`BEVYROGUE_TRACE_FORMAT`), nextest profilo `agent` (fail-fast off + JUnit), insta snapshot filtrati. Workflow: `docs/agent-testing.md`.

---

## DAG dipendenze (residuo)

```
M021 ✅ ─→ M002 ─→ M003 ─┐
                          ↓
            M004 ─→ M005 ─→ M006 ─→ M007 ─→ M008 ─→ M009
```

- **M002** e **M003** sequenziali (M003 consuma gli asset caricati da M002).
- **M004–M008** richiedono il loro foundation specifico (già chiuso M017–M021) + visual stack di M003.
- **M009** può essere drip-feed dentro M004–M008.

---

## Asset + visual tracer (2 milestone)

### M002 — Asset pipeline (loader + validator + hot-reload, Agumon-only)
**Obiettivo:** validare `clip.ron` + `animation_fsm.ron` schema su 1 Digimon, infrastruttura pronta.
**Scope:**
- `AssetLoader<Clip>` per `clip.ron` (geometria lossless da `_atlas.json` + loader-side defaults `texture_path`/fps/loop).
- `AssetLoader<AnimationFsm>` per `animation_fsm.ron`.
- Validator §L: clip name exists, FSM node/edge consistency, `Commands v0` vocabolario.
- Hot-reload working per Agumon.
**Scope NON incluso:** AnimGraph runtime player → M003.
**Demo:** `cargo run --features windowed` con Agumon clip + FSM caricati, validator blocca errori, hot-reload aggiorna senza crash.
**Riferimenti:** SP4 sample files (`.gsd/spikes/spike-asset-schema/`).

### M003 — AnimGraph runtime player + sprite render + §9 UI core (Agumon-only)
**Obiettivo:** **prima volta che il combat gira sullo schermo.** Reality check completo dello stack visual.
**Scope (slice per de-risking incrementale):**
- **S1** Agumon FSM idle loop on-screen (smoke render + FSM, no combat).
- **S2** 1 basic attack windup→strike→recovery con telegraph chip.
- **S3** Phase strip §9 (turn order) live.
- **S4** Baby Burner reactive detonate con signature flash.
- **S5** `cargo run --features windowed`: Agumon vs Agumon dummy, kit completo (Sharp Claws + Baby Flame + Baby Burner + Twin Core fire side via placeholder ally per Heated).
- **S6** Smoke windowed end-to-end (no panic, FPS stabile, hot-reload non rompe world state).
**Vincoli (D008):** `RenderPlugin`/`UiPlugin` gated windowed; UI legge `EventReader<CombatEvent>`, non muta state; test headless Agumon (M002 baseline) verdi.
**Rischio:** **milestone più rischioso del piano.** Replan tocca 1 milestone (5–7 slice), non il resto.
**Demo:** Agumon vs Agumon su schermo, sprite animati, telegraph chip, phase strip live, flash su Baby Burner detonate.

---

## Roster Digimon visual-aware (5 milestone)

Ogni milestone = blueprint plugin + skill RON + FSM RON + sprite + UI cue + integration test headless + demo visiva.

### M004 — Gabumon
**Scope:** Claw Attack + Gabumon Shot + Blue Cyclone + Fur Cloak DR passive + sprite + FSM + UI cue.
**Demo:** Twin Core pair Agumon+Gabumon su schermo (Heated→Chilled visibili). **Dip.:** M003.

### M005 — Dorumon
**Scope:** Bite + Dash Metal + Metal Cannon + Predator Loop passive (trait nuovo).
**Demo:** Predator Loop tracking + transition visibile. **Dip.:** M003.

### M006 — Tentomon
**Scope:** Hard Claw + Petit Thunder + Electrical Discharge + Battery Loop passive.
**Demo:** Battery Loop charge → discharge, telegraph AoE indicator. **Dip.:** M003.

### M007 — Renamon
**Scope:** Kōkaishū + Koyōsetsu + Tōhakken + Kitsune Grace reactive passive.
**Demo:** time-manip telegraph (AdvanceTurn ally), Kitsune Grace flash su `UltimateUsed`. **Dip.:** M003.

### M008 — Patamon
**Scope:** Tai Atari + Patapata Hover (heal/cleanse) + Sparking Air Shot + Holy Aegis self-included passive.
**Demo:** heal+cleanse cascade, Holy Aegis DR su tutti gli alleati (incluso self). **Dip.:** M003.

---

## Polish finale (1 milestone)

### M009 — UI §9 polish + edge cases
**Scope:** modifier glossary §G (tooltip); edge cases telegraph (Bounce chips, AoE refinement); signature flash cross-Digimon consistency; accessibility (skip animation, motion-reduce); fix drift accumulati M004–M008.
**Nota:** può essere drip-feed durante i roster milestone; tenuto separato per chiusura formale del prototype.

---

## Quando si vede cosa

| Dopo | Cosa funziona |
|---|---|
| M021 ✅ | Combat kernel canon-completo headless, deterministico, CLI scripted scenario, JSONL log |
| **M003** | **Agumon gira sullo schermo con sprite + UI canon** — primo reality check visivo |
| M004 | Twin Core pair visibile (skill identity Agumon+Gabumon completa) |
| M008 | Tutti i 6 Digimon giocabili sullo schermo |
| M009 | Demo prototype playable polished |

---

## Note di processo

- Milestone aperti **uno alla volta** via `gsd_plan_milestone`.
- A ogni chiusura: `gsd_complete_milestone` + `gsd_reassess_roadmap` se lo scope downstream cambia.
- Decisioni in esecuzione → `DECISIONS.md` via `gsd_save_decision`.
- Drift dal portfolio → aggiornare **questo file** insieme al milestone affected.
