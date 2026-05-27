# Combat review — Combat Sandbox (`cargo winx`)

**Data:** 2026-05-27
**Fonte:** registrazione `Video del 2026-05-27 14-00-27.webm` (26.7s), analizzata a 6 fps (160 frame) e incrociata col codice di `src/windowed/render/`.
**Scena:** Agumon `UnitId(1)` + Renamon `UnitId(7)` (Team::Ally) vs Agumon Dummy `UnitId(9001)` (Team::Enemy).

Cinque problemi rilevati, in ordine di priorità.

---

## 1. VFX Baby Flame mira l'alleato sbagliato (headline)

**Problema.**
Il danno colpisce il dummy ma il fuoco non lo raggiunge mai. Nei frame 33–39 l'Agumon in basso-sinistra carica l'orb alla bocca (corretto), poi al lancio il proiettile/impatto viaggia in una colonna verticale lungo il lato **sinistro**, tra i due alleati (frame 37: blob da y≈190 a y≈360), senza mai arrivare al dummy sulla destra. Il kernel invece applica il danno al bersaglio giusto: `[1] Hit(UnitId(7)→UnitId(9001)) 18 Normal`, HP dummy 50→…→0.

Danno (kernel) e VFX (presentation) divergono: traiettoria osservata = caster basso-sx → alleato in alto-sx, **non** → nemico a destra.

Seam coinvolto: `nearest_opposing_target_xy` (`src/windowed/render/playback.rs:741`), alimentato da `team_of` (`playback.rs:136`) e `sprite_positions` (`playback.rs:144`):

```rust
caster_team.and_then(|ct| nearest(&move |team| team != ct))   // ramo opposing-team
    .or_else(|| nearest(&|_| true))                            // fallback: nearest non-caster
```

Il fuoco finisce sull'alleato ⇒ il ramo opposing-team **fallisce** e si cade nel fallback `nearest non-caster`. Con due alleati nella stessa colonna stretta (~150 px) contro il dummy (~330 px), il più vicino è l'altro alleato. È esattamente il caso che il commento "S08 multi-ally fix" (`playback.rs:733-740`) dichiara risolto, ma in questa composizione non lo è.

**Sospetto secondario — binding sprite ↔ unit-id incrociato.**
Il kernel caster è `UnitId(7)` = Renamon (`src/windowed/digimon/renamon/mod.rs:24`, Team::Ally), ma lo sprite che carica e lancia è l'**Agumon** in basso. `barrier_targets_sprite` (`playback.rs:647`) seleziona lo sprite con `unit_id == status.source`, quindi lo sprite con unit_id 7 presenta il cast — ma visivamente è Agumon, non Renamon. Se identità o slot sono incrociati, l'origine di ogni VFX è sbagliata a monte del targeting.

Verificato che `Unit` e `Team` stanno sulla **stessa** entità (`src/combat/encounter/bootstrap.rs:159-172`, `def.team` alla riga 172): `team_of` *dovrebbe* contenere `9001 → Enemy`. Quindi il fallimento è più sottile di un componente mancante.

**Raccomandazione.**
1. Aggiungere log debug di `(caster unit_id, caster_team, target risolto unit_id + xy)` nel ramo `should_spawn_node_vfx` (`playback.rs:288`) e al release del proiettile (`playback.rs:409`).
2. Eseguire `cargo winx` e leggere se `caster_team == None` oppure se il ramo opposing ritorna `None` (nessuno sprite Enemy in `sprite_positions`).
3. In parallelo verificare il binding sprite↔unit-id: confermare che lo sprite che presenta il cast è davvero l'entità di `status.source`.
4. Aggiungere un test mirato su `nearest_opposing_target_xy` per la composizione 2-ally + 1-enemy in colonna.

---

## 2. La morte non persiste (regressione)

**Problema.**
Il dummy va KO (`[5] KO(UnitId(9001))`) ma non muore visivamente. Sequenza: frame 128 idle vivo → frame 131 collasso di **un solo frame** con numero "9" → frame 134/137/144 di nuovo **in piedi in idle, completamente opaco**, sotto il banner "Victory". Nessun fade-out, nessun despawn: resta un cadavere in piedi.

**Diagnosi.**
La pipeline di morte esiste ed è corretta: `drive_death_reactions` (`src/windowed/render/feedback.rs:189`) inserisce `DeathExiting`, `advance_digimon_presentation` salta la riconciliazione di modo (`playback.rs:165`) e all'uscita del nodo semina `FadeOut` (`playback.rs:472`). Il nodo `death` esiste (`assets/digimon/agumon/stance.ron:13`, transizione `death → Exit` su `TimeInNode`).

Poiché il dummy **rientra in idle**, `death_exiting.is_none()` era vero ⇒ `drive_death_reactions` **non ha mai inserito** `DeathExiting`. Il collasso di un frame (131) è solo il flinch `hurt` dell'`OnHitTaken` letale (`hurt → idle`, `stance.ron:22`). Causa probabile: il dummy non riceve un `CombatEvent::UnitDied` sul bus di presentation — KO registrato solo lato kernel, oppure l'evento viene consumato/drenato prima del tick di presentation al passaggio a Victory.

**Raccomandazione.**
Verificare che il KO del dummy emetta `UnitDied` sul `MessageWriter<CombatEvent>` letto da `drive_death_reactions`, e che la transizione a Victory non azzeri il bus eventi prima che i sistemi di presentation girino. Possibile legame col problema #4 (vedi sotto).

---

## 3. Hit-react quasi assente

**Problema.**
Il dummy resta statico mentre l'HP scende 50→0; flinch praticamente assenti sui colpi multipli (log `[1]…[5]`). Si legge come un sacco da boxe.

**Causa.**
`drive_hurt_reactions` è gated ai soli sprite in `Idle` (`feedback.rs:150`) e fa dedup per-target nella stessa finestra (`feedback.rs:127-131`). I colpi multipli ravvicinati collassano a ≤1 flinch.

**Raccomandazione.**
Decidere se il feedback deve scattare per ogni colpo (rimuovere/allentare il dedup e il gate Idle) o se va bene un flinch per finestra. È una scelta di design, non un bug netto.

---

## 4. Doppia barra HP desincronizzata

**Problema.**
Il dummy mostra una barra principale "8/100" e una barra arancio "x/50" (WEAK: Ice). A Victory la principale è **8/100** (≠ 0) mentre l'arancio è 0/50. Il KO è scattato sul pool arancio con 8 ancora sulla barra principale. Non è chiaro quale sia l'HP reale.

**Raccomandazione.**
Chiarire il modello a due pool: cosa rappresenta la barra "50" (seconda HP? stagger/break? scudo elementale?) e quale pool decide il KO. È fonte plausibile anche del problema #2: se il kernel valuta `hp_current = 8 > 0`, l'evento `UnitDied` potrebbe legittimamente non partire pur mostrando "KO" nel log.

---

## 5. Numero di danno staccato dal bersaglio

**Problema.**
Il "9" fluttua in alto a sinistra, lontano dal corpo che collassa (frame 131).

**Causa.**
Il numero è ancorato al centro dello sprite **pre-hit** più un offset Y fisso (`DAMAGE_NUMBER_SPAWN_OFFSET_Y_PX`, `feedback.rs:268`) e non segue lo sprite quando si muove/collassa.

**Raccomandazione.**
Minore. Eventualmente agganciare il numero alla posizione live dello sprite per la durata dell'animazione, o ridurre l'offset. Bassa priorità.

---

## Nota positiva

Sprite distinti e corretti (Renamon biped vs Agumon dino, nessun duplicato), telegraph badge e phase chips (Declared→Resolved) chiari, combat log leggibile. La pipeline di morte e di hit-feedback è **implementata correttamente nel codice**; i problemi #1 e #2 sono di *ingaggio* (targeting / consegna eventi), non di logica di rendering.

## Ordine di intervento suggerito

1. #1 — mira VFX + binding unit-id (con log debug prima di toccare il codice)
2. #2 — consegna `UnitDied` al dummy
3. #4 — modello doppia HP (può causare #2)
4. #3 — politica hit-react
5. #5 — ancoraggio numero danno
