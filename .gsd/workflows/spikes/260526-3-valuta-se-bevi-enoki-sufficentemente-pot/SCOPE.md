# Scope — bevy_enoki / extension-first / editor readiness

## Questioni da investigare

Questo spike deve rispondere a **tre domande collegate ma distinte**:

1. **bevy_enoki è abbastanza espressivo** per costruire VFX con il tipo di read / layering / timing che vogliamo, prendendo come target qualitativo effetti in stile **Honkai: Star Rail** e **Digimon Survive / Time Stranger**?
2. **L’architettura attuale del progetto** — in particolare anim graph, stance graph, registri VFX, presentation cues e plugin per-Digimon — è già abbastanza **extension-first** da permettere di aggiungere la specifica di un Digimon come modulo isolato, invece di cablare nel motore logica e presentazione specifiche del personaggio?
3. **La forma attuale dei dati e delle API** lascia una strada credibile verso un **editor in-process** futuro, oppure ci sono ancora accoppiamenti / hardcode che bloccherebbero tuning visuale e authoring data-driven?

## Cosa deve contenere una risposta utile

La risposta finale deve fornire:

- **Verdetto separato per area**:
  - VFX capability: `sì / sì con limiti / no`
  - Extension-first architecture: `sì / parzialmente / no`
  - Editor readiness: `pronto / quasi / ancora lontano`
- **Matrice di valutazione** con criteri espliciti.
- **Evidenza concreta dal repo**: documenti architetturali, asset attuali, registri/plugin per-Digimon, bridge fra cue/presentation e renderer.
- **Gap analysis**: cosa manca per arrivare al livello target, distinguendo:
  - limiti del middleware (`bevy_enoki`)
  - limiti del nostro layer di integrazione
  - limiti di pipeline / authoring / tooling
- **Raccomandazione operativa**:
  - continuare con `bevy_enoki` così com’è
  - continuare con `bevy_enoki` ma restringendo l’ambizione visiva
  - continuare con `bevy_enoki` aggiungendo un layer proprietario sopra
  - oppure cambiare approccio / backlog
- **Next steps concreti** verso editor e onboarding di nuovi Digimon.

## Criteri di valutazione

### 1) Potenza VFX / expressiveness

Valutare `bevy_enoki` e la nostra integrazione attuale rispetto a:

- capacità di fare **burst, trail, projectile, impact, persistent emitter**
- controllo di **spawn timing** e sync con anim graph / beat / cue
- controllo di **anchor / locus / motion** rispetto a caster, target, hit point, mouth / claw / ground, ecc.
- layering sufficiente per effetti “ricchi” ma leggibili
- possibilità di combinare particelle, sprite, bloom / overbright, shake, hold, trail
- adeguatezza per **stile stylized 2D combat VFX**, non photorealismo
- limiti noti per effetti “signature” molto authored / cinematici

### 2) Flessibilità architetturale per-Digimon

Valutare se il motore supporta davvero:

- aggiunta di un Digimon come **plugin/extension** con modifiche minime al core
- registrazione isolata di:
  - stance graph
  - skill graph / presentation cue
  - atlas / clip
  - VFX ids / handles / lifecycle
  - eventuale logica custom di blueprint
- assenza di branching hardcoded nel kernel o nel renderer per singolo Digimon
- separazione tra:
  - **gameplay authority**
  - **presentation metadata**
  - **presentation runtime**
- costo reale di aggiungere il “settimo Digimon” rispetto a quanto promette la documentazione

### 3) Editor readiness

Valutare se la struttura corrente è compatibile con un editor futuro:

- dati visuali e di tuning in **RON / asset typed / hot-reloadable**
- shape dei dati abbastanza stabili da essere editabili via tool
- assenza di numeri / path / wiring critici sepolti in Rust
- punti dove servirebbe introdurre:
  - cataloghi dati (`vfx.ron`, graph/cue metadata, params)
  - reflection / inspector exposure
  - validatori / contract tests boot-time
  - separazione asset-authoring vs runtime wiring

## Vincoli / assunzioni

- Questo spike **non deve produrre feature shipping**: output principale = conoscenza.
- Il target visivo è una **comparazione di classe qualitativa**, non “replicare 1:1 HSR”.
- La valutazione deve distinguere tra **limite del motore open-source** e **limite temporaneo del nostro utilizzo attuale**.
- L’editor è un obiettivo **futuro**, quindi il criterio non è “editor già pronto”, ma “architettura che non blocca un editor serio”.

## Angoli di ricerca

### Angolo 1 — `bevy_enoki` come backend VFX

Obiettivo: capire se il backend scelto è abbastanza potente per il nostro target stilistico.

Domande guida:

- Quali primitive VFX sono già coperte nel progetto?
- Quali pattern target (charge orb, ember swirl, traveling projectile, slash burst, detonate burst, layered impact) sono facili vs difficili?
- Dove il collo di bottiglia è `bevy_enoki`, e dove invece è il nostro authoring/runtime?

Output atteso: valutazione capability + limiti + rischio.

### Angolo 2 — Stato reale dell’architettura extension-first

Obiettivo: verificare se la promessa “aggiungi un Digimon come extension” è vera anche sul lato presentation/windowed, non solo sul lato combat kernel.

Domande guida:

- Quanto del setup per-Digimon è isolato in moduli dedicati?
- Quanto wiring richiede toccare sistemi condivisi?
- Il bridge `data -> cue -> renderer -> VFX` è già generic enough?

Output atteso: audit dei confini del core e dei moduli per-Digimon.

### Angolo 3 — Prontezza per editor e authoring data-driven

Obiettivo: capire cosa manca per passare da wiring manuale a authoring/editing di alto livello.

Domande guida:

- Quali parti sono già asset-driven?
- Quali parti sono ancora registrate a mano in Rust?
- Quale roadmap minima renderebbe sostenibile un editor in-process?

Output atteso: gap list + sequenza consigliata di evoluzione.

## Evidenza iniziale già identificata

- `docs/combat_current.md`
- `docs/future_design_draft/02-04_strong_typing.md`
- `docs/future_design_draft/02-05_tunable_catalog.md`
- `src/windowed/render.rs`
- `src/windowed/digimon/agumon/mod.rs`
- `src/windowed/digimon/renamon/mod.rs`
- `assets/data/digimon/agumon/skills.ron`
- `assets/digimon/agumon/*.particle.ron`
- `tests/windowed_only/*`

## Decision format finale

La raccomandazione finale userà questo schema:

1. **Executive summary**
2. **Comparison matrix**
3. **Recommendation**
4. **Fallback / alternative path**
5. **What would change the recommendation**
6. **Concrete next steps**
