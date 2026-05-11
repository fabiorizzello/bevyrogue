# Continue — bevyrogue session resume

**Data scritta:** 2026-05-11
**Branch:** `master` (commit più recente: `c7cc04d`)
**Modalità sessione:** design conversation + sprite pipeline iteration. Niente file scritti senza allineamento esplicito utente.

---

## Stato attuale (cosa abbiamo appena fatto in questa sessione)

1. **Importato** `~/dev/digimonsprites/sprite_pipeline/` → `tools/sprite_pipeline/` (commit `17bab8a`).
   - Tutti e 6 i Rookie (agumon, gabumon, dorumon, patamon, renamon, tentomon) hanno raw_models / configs / palettes / standards / references.
   - `output/` e `raw_models/**/*.zip` ignorati (vedi `tools/sprite_pipeline/.gitignore`).
2. **Re-export atlas 6 Rookie** (`a63c3e1`) — sweep dopo agumon (`53761df`).
3. **Aggiunto** `docs/future_design_draft/digimon/agumon/` (commit `c7cc04d`) come stress-test §2.2b FSM: identity + 4 markdown per skill + 3 GIF (claw, pepper breath, nova blast).

## Problema attivo (motivo dell'import)

**`docs/future_design_draft/digimon/agumon/gifs/03_ult_nova_blast.gif` ha un pop visivo all'apex → idle.**

Diagnosi (già fatta nel turno precedente, prima dell'import):
- Skill clip = action `chr050_bs01`, frame 1→81 step 5 → 17 frame + 1 frame `idle_tail` appendato = 18 frame nell'atlas.
- L'ultimo frame del clip è l'apex del ruggito (Blender frame 81); poi salta hard al frame neutro di idle.
- **Manca la recovery (return-to-idle) nel render.**

L'utente ha confermato che la causa è l'animazione, non la logica:
> *"nova blast ha ancora pop, non andiamo alla cieca. probabilmente il problema è l'animazione"*

### Quattro strade proposte (decisione in sospeso)

| Opt | Approccio | Costo | Rischio |
|-----|-----------|-------|---------|
| **A** | Estendere `frame_end` in `tools/sprite_pipeline/configs/agumon.json` (skill: 81 → 120 step 5). Se l'action FBX ha la recovery, vinciamo gratis. | bassissimo (1 re-render) | nullo se l'action ha più frame; spreco se no |
| **B** | Append frame interpolati prima di `idle_tail`: 3-4 frame remixati da `skill[8..11]` (le pose intermedie già renderizzate) usate come "recovery" all'indietro. Hacky, zero Blender re-render. | basso (script Python) | recovery poco organica |
| **C** | Cross-fade alpha-blend in image space tra apex e idle[0]. | bassissimo | brutto visivamente |
| **D** | Re-render `bs01` in Blender estendendo timeline / aggiungendo keyframe recovery. Più pulito. | medio (tocco .fbx o action) | richiede inspezione Blender |

**Raccomandazione (mia, da validare):** A prima (`bpy` headless inspect dell'action `chr050_bs01` → vedere se ha frame > 81), poi B se A è vuota.

### Pre-step necessario per Opt A: gotcha paths

I config in `tools/sprite_pipeline/configs/*.json` hanno **path hardcoded a `/home/fabio/dev/digimonsprites/`** — stale ora che il pipeline è dentro bevyrogue. Esempi da `agumon.json`:
```
"model_path":   "/home/fabio/dev/digimonsprites/sprite_pipeline/raw_models/agumon/chr050.fbx",
"texture_path": "/home/fabio/dev/digimonsprites/sprite_pipeline/raw_models/agumon/chr050a01.png",
"output_root":  "/home/fabio/dev/digimonsprites/sprite_pipeline/raw_renders/agumon",
```
**Da sistemare prima di lanciare un re-render dal nuovo path.** Tutte e 6 le config. Sed o piccolo script Python.

## Decisioni di design in sospeso (5 domande aperte pre-import)

L'utente ha richiesto la **visione roster intera** prima del dettaglio. Spalmata su più milestone, non solo M017. In coda:

1. **Mappatura ruoli proposta** (Agumon=Burst Fire / Gabumon=Sustain Ice / Dorumon=Snowball single-target Dark / Patamon=Healer-Cleanse / Renamon=Crit-Status / Tentomon=Battery-SP). OK o ritocchi?
2. **Problema tank**: roster senza tank. Opzioni:
   - (a) Tentomon prende ruolo tank/battery
   - (b) tank esterno (NO — sono 6 Rookie unici)
   - (c) skill-tree branch tank su Patamon/Tentomon (deferred — niente skill-tree v1)
3. **Dorumon vs Renamon**: rischio overlap (entrambi dark-flavor crit-stato). Come differenziarli?
4. **Agumon vs Gabumon**: il Twin Core li lega ma rischia di renderli equivalenti. Come differenziarli?
5. **Roadmap milestone**: ordine di consegna del roster e su quanti milestone spalmarlo.

### Richiesta utente attiva (era IN PROGRESS prima del task import)

> *"Comporre 4-5 team da 4 Digimon con grafo di sinergie, valutando se i 6 Rookie sono abbastanza sinergici / slegati / accavallati."*

Da fare **prima** di rispondere alle 5 domande (le informa). Per ciascun team da 4: composizione · win-condition · perno · sinergie esplicite (chi abilita chi). Concludere con verdetto: sinergici / slegati / accavallati?

## Constraints da non dimenticare

- **Niente decisioni autonome di scope/identità.** Visione → utente valida → dettaglio.
- **Niente file design scritti** finché allineamento non c'è. Risposte in chat o sketch markdown solo se richiesti.
- **`combat_current.md` è proposta**, non oracolo. Valutare criticamente i kit attuali — possono essere riformulati se non rendono giustizia al Digimon.
- **Caveman mode attivo** (K002): terse, sostanza tecnica esatta, no fluff.
- **Headless-first**: ogni decisione FSM/runtime deve girare anche in `cargo test`.
- `10_full_kit_plan.md` è **out-of-scope**, da non riaprire senza richiesta esplicita.
- L'utente è sensibile a rename/cambi-identità non richiesti (rif: `m017_design_draft` → `future_design_draft`). Non toccare scope di artefatti senza autorizzazione.

## Riferimenti chiave

- **Design FSM canon:** `docs/future_design_draft/02-02b_animation_fsm.md`
- **Roster minimal canon:** `docs/future_design_draft/08_roster_minimal.md`
- **Stress test agumon:** `docs/future_design_draft/digimon/agumon/`
- **Pipeline config skill agumon:** `tools/sprite_pipeline/configs/agumon.json` (action `chr050_bs01`, frame 1→81 step 5, skill = nova blast)
- **Atlas live:** `assets/digimon/agumon_atlas.{json,png}`
- **Standards stile:** `tools/sprite_pipeline/standards/agumon.md` + `standards/global.md`
- **K001 — dataset Digimon canonico:** `python3 .claude/skills/digimon/scripts/query.py …` (mai leggere `digimon.json` raw)
- **K002 — caveman mode** attivo per default

## Prossima mossa concreta (quando l'utente rientra)

Due fili aperti:
1. **Pop di `nova_blast`** — decidere strategia A/B/C/D + fixare path hardcoded nei config.
2. **Team comps di 4 Digimon** (richiesta utente pre-import) → poi 5 domande di design.

Probabile ordine utente: prima risolvere il pop (è quello che ha innescato l'import), poi tornare ai team. Ma **chiedere quale dei due prima — non assumere**.
