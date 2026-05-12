# §2.2c — Dematerialize/Rematerialize — bypass del movement blending

**Stato:** decisione di linguaggio visivo (2026-05-11). Amendment a §2.2b. Sostituisce la necessità di blending skeletal tra clip atomiche con un trucco di presentation coerente con il tema Digimon (creature digitali).

**Decisione:** quando un'unità deve agire fuori dalla sua posizione di idle, **non si muove sulla mappa**. Esegue una transizione *dematerialize → teleport logico → rematerialize* via shader, mantenendo ogni clip d'animazione **atomica** (idle, attack, skill, ult, hit, ko) senza alcun cross-fade o IK locomotion.

Applicato **a tutte le skill** (basic, skill, ultimate) come default uniforme. Variante camera-cut (Honkai-style) e ibridi sono **deferred** — riapribili solo se il default risulta ripetitivo dopo playtest.

---

## A — Razionale

Il blending skeletal tra clip eterogenee (idle→walk-to-target→attack→walk-back→idle) richiede:
- IK per posa di arrivo coerente col target,
- locomotion blendtree (run/walk cycle) o root-motion estratto,
- transition graph con timing fine,
- gestione path/collision sulla mappa di combattimento.

Costo alto, valore di gameplay basso (il combat è turn-based, non posizionale). Il trucco shader **collassa tutto questo** in: dissolve out → instant teleport → dissolve in. Ogni clip resta una sequenza di frame indipendente.

**Coerenza tematica:** i Digimon sono creature digitali. Dematerializzarsi in pixel-noise / data-stream è in-universe, non una scorciatoia camuffata. Vicino a effetti già canonici della franchise (digi-evolution, data-stream attacks).

---

## B — Sequenza per skill (uniforme)

Per ogni azione che richiederebbe movimento (basic attack incluso, salvo skill ranged pure tipo Baby Flame):

| Fase | Durata indicativa | Cosa succede |
|---|---|---|
| 1. **Dematerialize @ source** | 200–300 ms | Shader dissolve sull'unità nella sua posizione di idle. Clip = ultimo frame idle freeze. |
| 2. **Reposition (logico)** | 1 frame | Transform spostato al punto d'azione (arena center / davanti al target). Sprite invisibile. |
| 3. **Rematerialize @ action point** | 150–200 ms | Shader dissolve inverso. Clip = primo frame della clip d'azione (windup o attack). |
| 4. **Play action clip** | clip-dependent | Attack/skill/ult playthrough atomico (no blend). FSM §2.2b sequenzia `EmitDamage`/`SpawnParticle`. |
| 5. **Dematerialize @ action point** | 200–300 ms | Shader dissolve sull'ultimo frame della clip d'azione. |
| 6. **Reposition (logico) back** | 1 frame | Transform riportato in idle position. |
| 7. **Rematerialize @ source** | 150–200 ms | Shader dissolve inverso. Clip = primo frame idle. |

**Total overhead per skill:** ~700–1000 ms cumulato di shader transition, sopra il tempo della clip d'azione vera e propria. Accettabile per combat turn-based.

**Eccezione (basic attack ranged):** se la skill non richiede prossimità (es. Baby Flame, già ranged), si **salta** dematerialize: unit resta in idle position e spara il proiettile. Criterio: `requires_close_range: bool` in `skills.ron` (campo nuovo, default `true`). Vedi §E.

---

## C — Shader

**Forma scelta:** pixel-noise dissolve (compatibile con stile pixel-art del progetto, basso costo). Alternative valutate e scartate per ora: data-stream verticale (più caro, più "Tron"), polygon-shatter (skeletal mesh-dependent, non applicabile a sprite 2D).

**Parametri esposti:**
- `dur_out_ms`, `dur_in_ms` — timing default per fase
- `noise_scale` — granularità del pixel-noise
- `tint` — colore additivo durante la transizione (default: bianco/cyan tenue, varia per attributo? deferred)

**Implementazione:** material custom su `Sprite`, soglia di dissolve animata via `Time::elapsed_seconds()` o componente `MaterializeState { phase, t }`. Headless-safe: il sistema gameplay non aspetta lo shader, è puramente presentation.

---

## D — Integrazione con FSM §2.2b

La FSM non orchestra direttamente lo shader. Il **blueprint del Digimon** (o un sistema globale `presentation_dispatcher`) intercetta i Commands gameplay `EmitDamage` / `EmitStatus` e li wrappa nella sequenza dematerialize/rematerialize se la skill è `requires_close_range`.

**Effetto sulla FSM:**
- I nodi FSM **non hanno bisogno** di stati `Dematerialize` / `Rematerialize` espliciti. Restano clip-driven (windup, hit, recovery).
- Il presentation layer aggiunge il "guscio" shader **attorno** alla FSM, non dentro.
- **Eccezione opzionale:** se una skill ha più punti d'azione (es. Renamon multi-step teleport tra target), la FSM può emettere un Command nuovo `Reposition { anchor }` come trigger del cycle dematerialize/rematerialize tra un nodo e il successivo. **Deferred:** introdurre `Reposition` solo se almeno una skill del roster minimal §8 lo richiede davvero.

---

## E — Campo in `skills.ron`

Aggiungere campo opzionale al record skill:

```ron
(
    id: "agumon_sharp_claws",
    requires_close_range: true,   // default true se omesso
    // ... altri campi
)
```

Skill con `requires_close_range: false` saltano dematerialize/rematerialize → unit resta in idle position, esegue la clip d'attacco (es. Baby Flame = ranged, animazione di spit dal posto).

Decisione su quali skill del kit minimal sono ranged vs melee: **deferred a quando entriamo nel design dettagliato del kit per Digimon**.

---

## F — Cosa NON cambia

- `clip.ron` resta atomico (idle/attack/skill/ult/hit/ko sono clip separate, no transition baking).
- `clipmontage.ron` (FSM §2.2b) resta presentation-aware ma agnostico rispetto allo shader.
- `skills.ron` aggiunge solo `requires_close_range: bool`.
- Headless gameplay: nessun impatto. Lo shader è puramente cosmetic; i test integration in `tests/` non lo toccano.

---

## G — Cosa è deferred / da rivisitare

- **Camera cut** (Honkai-style) per ultimate: possibile aggiunta futura per "wow moment" sugli ult. Ortogonale a dematerialize — non si esclude, si stratifica.
- **Shader variant per attributo** (Vaccine = bianco, Virus = viola, ecc.): valutabile in fase di polish UI.
- **Multi-hop reposition** (skill con più punti d'azione): introdurre Command `Reposition` solo se serve davvero.
- **Hit reaction targeting**: il target che subisce un colpo deve dematerializzare? Probabilmente no — basta hit clip atomica in place. Da confermare con prima skill testabile in M017.

---

## H — Rischio principale

**Ripetitività visiva.** Ogni skill ha lo stesso "stacco" dematerialize/rematerialize. Mitigazione:
- timing leggermente diverso per categoria (basic ~150 ms, skill ~250 ms, ult ~400 ms con flash più marcato),
- particle/sfx unique per ogni clip d'azione (la FSM §2.2b lo permette già via `SpawnParticle`/`Shake`),
- futuro: shader variant per attributo o per Digimon (deferred).

Se dopo playtest del primo kit §8 (Agumon) la ripetitività è bloccante, si valuta camera-cut su skill/ult e si torna allo split A/C originale. Trigger di re-design esplicito.
