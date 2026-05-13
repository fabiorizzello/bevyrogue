# Milestone Portfolio — M017 → M029

**Status:** definito, non ancora pianificato (solo M017 sarà aperto via `gsd_plan_milestone`).
**Ultimo update:** 2026-05-13.
**Scopo:** allineare il codice (`src/combat/`, `src/data/`, `src/ui/`) al canon design `docs/future_design_draft/` (round-3) attraverso 13 milestone piccoli e sequenzialmente verificabili. Visual tracer end-to-end Agumon entro M023 — niente "headless first, visual last".

**Vincoli generali:**
- ID milestone strettamente incrementali (no lettere/suffissi).
- Headless first per kernel; `cargo check` (no feature) e `cargo run` (no feature) devono restare verdi a ogni milestone.
- `windowed` feature è additiva. Architettura plugin: `CombatPlugin` (core gameplay headless-safe) + `RenderPlugin` + `UiPlugin` (gated `#[cfg(feature = "windowed")]`). Vedi `D008`.
- Test integration restano in `tests/` (headless).
- Out of scope di tutto il portfolio: meta-loop (StS-like), encounter chain, save/load, evolution Champion/Ultimate, enemy AI revisione, balance pass. Sono milestone M030+.

**Spike che hanno informato lo scope:** SP1–SP5 in `.gsd/spikes/` (closed).
**Decisioni già registrate:** D002–D008 in `.gsd/DECISIONS.md`.

---

## DAG dipendenze

```
M017 ─→ M018 ─→ M019 ─→ M020 ─┐
                                ↓
                              M021 ─→ M022 ─→ M023 ─┐
                                                    ↓
                              M024 ─→ M025 ─→ M026 ─→ M027 ─→ M028 ─→ M029
```

- **M017–M020** (foundation kernel): sequenziali, stesso modulo, no parallelizzazione utile.
- **M021** può iniziare in parallelo a M020 (registry ortogonale a reactive bus).
- **M022** può iniziare in parallelo a M021 (asset pipeline indipendente dal trait).
- **M023** richiede **M021 + M022** entrambi chiusi.
- **M024–M028** richiedono il loro foundation specifico (M027 → M018, M028 → M019), e il visual stack di M023.
- **M029** può anche essere drip-feed dentro M024–M028.

---

## Foundation kernel (4 milestone, sequenziali)

### M017 — Status taxonomy v0 rewrite
**Obiettivo:** rimpiazzare la tassonomia attuale (`Burn`/`Freeze`/`Shock`/`DeepFreeze`/`Confused`) con i 5 status canon §H.1.
**Scope:**
- Introdurre `Heated`, `Chilled`, `Paralyzed`, `Slowed`, `Blessed`.
- Regola single-instance + `refresh_max_dur` (no additive).
- `Heated` multi-stack con cap definito da skill (non globale).
- Migrare tutti i test esistenti che referenziano la vecchia tassonomia.
- Aggiornare `assets/data/skills.ron` perché le `Effect::ApplyStatus` usino i nuovi id.
- JSONL log usa nomi canon.
**Demo:** test headless deterministici, status tick + decay verificati, log JSON pulito.
**Riferimenti:** D004 (`D-M017-STATUS-REWRITE`), `docs/future_design_draft/.../09_ui_surface.md §H.1`.

### M018 — Time-manipulation split + TargetShape resolver expansion
**Obiettivo:** abilitare le primitive che alimentano gran parte delle skill identity.
**Scope:**
- Sostituire `TurnAdvance` signed con `AdvanceTurn(pct)` / `DelayTurn(pct)`, cap ±50% per chiamata, clamp [0,200] dopo somma. No AV accumulator pre-cap.
- Estendere `TargetShape` resolver oltre `Single`: `Blast`, `AoE`, `Bounce(N)` con tie-break `slot_index asc`.
- Selectors estesi: `AdjLowest`, `LowestHpPctAlive`, `RandomEnemyAlive{seed}`, `SingleAlly`.
**Demo:** CLI scripted scenario per advance/delay + Bounce chain; tie-break determinismo verificato.
**Riferimenti:** D003 (`D-M017-TIMEMANIP-SPLIT`), D006 (`D-M017-TARGETSHAPE-EXPAND`).

### M019 — DR pipeline + heal/cleanse Effects
**Obiettivo:** mitigation modulare + heal/cleanse asse mancante (kit Patamon).
**Scope:**
- `BuffKind::DR` + mitigation layer in `damage.rs`.
- Taxonomy: intra-unit max-replace, cross-unit additive con clamp 0.5.
- `Effect::ApplyBuff { id, target_ref, mul, kind, dur }` con `BuffDuration ∈ {Turns, UntilRoundEnd, Permanent}`.
- `Effect::EmitHeal { amount, target }` + `Effect::EmitCleanse { target, count, filter, priority }`.
**Demo:** CLI scripted scenario con mitigation Gabumon-like + heal Patamon-like.
**Riferimenti:** D005 (`D-M017-DR-MODULE`), SP3 add-now list.

### M020 — Reactive bus uniforme + shim removal
**Obiettivo:** chiudere il contract reactive event canon.
**Scope:**
- Aggiungere `CombatEventKind::StatusApplied` + `UltimateUsed`.
- Estendere payload `UnitDied` con `status_remaining` + `heated_remaining`.
- `OnHitN` rimane FSM-side (canon §C4 row 4, no event).
- Rimuovere shim legacy Digimon-specific su `CustomSignalPayload`.
**Demo:** Twin Core reactive end-to-end senza shim; log JSON con event taxonomy stabile.
**Riferimenti:** D002 (`D-M017-EVENTS-BUS`), SP1 reactive verb table.

---

## Plugin + asset (2 milestone)

### M021 — Blueprint trait + registry + Bevy plugin self-registration
**Obiettivo:** consolidare l'extension pattern e formalizzare `CombatPlugin` separation.
**Scope (slice prevista):**
- **S01** Estrarre `CombatPlugin` da `register_combat_kernel_runtime` (refactor `main.rs` + `headless.rs` + `windowed.rs` a composizione plugin, zero cambio di logica).
- **S02** Definire `trait Blueprint` + `BlueprintRegistry` resource + dispatcher generico in `src/combat/blueprints/api.rs` (no migrate).
- **S03** Migrate Agumon plugin al nuovo trait + self-registration (shim per gli altri 5).
- **S04** Migrate Gabumon (paired Twin Core).
- **S05** Migrate Dorumon + Tentomon.
- **S06** Migrate Patamon + Renamon. Rimozione shim. `CombatKernelTransition` Digimon-specific eliminato.
- **S07** Extension-friendly `RosterEntry`: rimuovere field hard-coded Digimon-specific (`twin_core`, `holy_support`, …) a favore di un blueprint-keyed payload generico. Lo schema roster non deve più conoscere i nomi delle mechanic per essere esteso.
- **S08** `ValidationSnapshot` field nominati: passare dall'attuale shape inline (con `battery_loop` di default) a una struct con field nominati per blueprint key, popolata dal registry. Stabilizza l'osservabilità per asserzioni di test e log JSON.

**Vincoli (da D008):**
- `CombatPlugin` non importa `bevy::winit`, `bevy::render`, `bevy_egui`.
- `cargo check` (no feature) verifica il confinamento.
- Test esistenti restano verdi a ogni slice.

**Demo:** 6 plugin auto-registrati, dispatcher generico, test esistenti verdi.
**Riferimenti:** D007 (`D-M017-BLUEPRINT-TRAIT-REGISTRY`), D008 (`D-M021-ARCH-PLUGIN-SPLIT`), SP2 INTERFACE-OPTIONS.

### M022 — Asset pipeline (loader + validator + hot-reload, Agumon-only)
**Obiettivo:** validare `clip.ron` + `animation_fsm.ron` schema su 1 Digimon, infrastruttura pronta.
**Scope:**
- `AssetLoader<Clip>` per `clip.ron` (geometria lossless da `_atlas.json` + loader-side defaults per `texture_path`/fps/loop).
- `AssetLoader<AnimationFsm>` per `animation_fsm.ron`.
- Validator §L: clip name exists, FSM node/edge consistency, `Commands v0` vocabolario.
- Hot-reload working per Agumon.
**Scope NON incluso:** AnimGraph runtime player (consumer dei file caricati). Quello entra in M023.
**Demo:** `cargo run --features windowed` con Agumon clip + FSM caricati, validator blocca errori, hot-reload aggiorna l'asset senza crash.
**Riferimenti:** SP4 sample files (`.gsd/spikes/spike-asset-schema/`).

---

## Visual tracer end-to-end Agumon (1 milestone critico)

### M023 — AnimGraph runtime player + sprite render + §9 UI core (Agumon-only)
**Obiettivo:** **prima volta che il combat gira sullo schermo.** Reality check completo di tutto lo stack visual.
**Scope (interno suddiviso per de-risking incrementale):**
- **Slice 1** Agumon FSM idle loop on-screen (smoke test render + FSM, no combat).
- **Slice 2** 1 basic attack windup→strike→recovery con telegraph chip.
- **Slice 3** Phase strip §9 (turn order display) aggiornata in tempo reale.
- **Slice 4** Baby Burner reactive detonate con signature flash.
- **Slice 5** `cargo run --features windowed`: Agumon vs Agumon dummy, kit completo (Sharp Claws + Baby Flame + Baby Burner + Twin Core fire side via placeholder ally per Heated).
- **Slice 6** Smoke test windowed end-to-end (no panic, FPS stabile, hot-reload non rompe world state).

**Vincoli (da D008):**
- `RenderPlugin` + `UiPlugin` confinati a `#[cfg(feature = "windowed")]`.
- UI legge `EventReader<CombatEvent>`, non muta state.
- Test headless di Agumon (M022 baseline) restano verdi.

**Rischio:** **milestone più rischioso del piano.** Se va male, replan tocca 1 milestone (5–7 slice), non il resto del portfolio.
**Demo:** Agumon vs Agumon su schermo, sprite animati, telegraph chip prima dell'azione, phase strip live, flash su Baby Burner detonate.

---

## Roster Digimon visual-aware (5 milestone)

Ogni milestone = blueprint plugin + skill RON + FSM RON + sprite + UI cue + integration test headless + demo visiva.

### M024 — Gabumon
**Scope:** kit completo (Claw Attack + Gabumon Shot + Blue Cyclone + Fur Cloak DR passive) + sprite + FSM + UI cue.
**Demo:** Twin Core pair Agumon+Gabumon su schermo (entrambi Heated→Chilled transitions visibili).
**Dipendenze:** M017 (status), M019 (DR), M021 (registry), M023 (visual stack).

### M025 — Dorumon
**Scope:** Bite + Dash Metal + Metal Cannon + Predator Loop passive (migrato a trait nuovo).
**Demo:** Predator Loop tracking + transition visibile.
**Dipendenze:** M020 (reactive), M021, M023.

### M026 — Tentomon
**Scope:** Hard Claw + Petit Thunder + Electrical Discharge + Battery Loop passive (migrato).
**Demo:** Battery Loop charge → discharge visibile, telegraph AoE indicator.
**Dipendenze:** M018 (AoE shape), M021, M023.

### M027 — Renamon
**Scope:** Kōkaishū + Koyōsetsu + Tōhakken + Kitsune Grace reactive passive.
**Demo:** time-manip telegraph (AdvanceTurn ally), Kitsune Grace flash su `UltimateUsed`.
**Dipendenze:** M018 (time-manip), M020 (reactive), M021, M023.

### M028 — Patamon
**Scope:** Tai Atari + Patapata Hover (heal/cleanse) + Sparking Air Shot + Holy Aegis self-included passive.
**Demo:** heal+cleanse cascade visibile, Holy Aegis DR su tutti gli alleati (incluso self).
**Dipendenze:** M019 (DR + heal/cleanse), M021, M023.

---

## Polish finale (1 milestone)

### M029 — UI §9 polish + edge cases
**Scope:**
- Modifier glossary §G (tooltip).
- Edge cases telegraph: Bounce chips, AoE indicator refinement.
- Signature flash cross-Digimon consistency.
- Accessibility: skip animation, motion-reduce.
- Eventuali fix di drift accumulati durante M024–M028.

**Nota:** può anche essere drip-feed durante i roster milestone se serve. Tenuto come milestone separato per chiusura formale del prototype.

---

## Quando si vede cosa

| Dopo | Cosa funziona |
|---|---|
| M020 | Combat kernel canon-completo headless, CLI scripted scenario, JSONL log |
| **M023** | **Agumon gira sullo schermo con sprite + UI canon** — primo reality check visivo |
| M024 | Twin Core pair visibile (skill identity Agumon+Gabumon completa) |
| M028 | Tutti i 6 Digimon giocabili sullo schermo |
| M029 | Demo prototype playable polished |

---

## Note di processo

- I milestone vanno aperti **uno alla volta** tramite `gsd_plan_milestone`, non tutti in batch.
- A ogni chiusura di milestone: `gsd_complete_milestone` + eventuale `gsd_reassess_roadmap` se lo scope downstream cambia.
- Le decisioni emerse durante l'esecuzione di un milestone vanno in `DECISIONS.md` via `gsd_save_decision`.
- Drift dal portfolio (es. uno scope che cambia mid-flight) → aggiornare **questo file** insieme al milestone affected.
