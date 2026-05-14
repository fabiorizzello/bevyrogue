# M019: DR pipeline + Heal/Cleanse primitives + PerHop guard

**Gathered:** 2026-05-14
**Status:** Ready for planning

## Project Description

Introduce il primo layer numerico di mitigazione difensiva (`BuffKind::DR`) nel combat, aggiungere `Heal` e `Cleanse` come `Effect` del DSL skill, e una guardia `PerHop` che previene la doppia applicazione dello stesso status nello stesso "hop" di una catena di follow-up/counter. Tutto headless-first, deterministico, integration-tested.

## Why This Milestone

Oggi la mitigazione del damage vive **solo** sull'asse qualitativo: triangle attribute (Vaccine/Virus/Data, ±11/13%) e tag resists (×0.75 se il tag è in `Unit::resists`). Non esiste alcuna stat numerica difensiva — niente `defense`, niente `armor` su `Unit`. Senza un asse numerico di mitigazione non si possono costruire kit support credibili (Patamon `holy_aegis`, Gabumon `fur_cloak` previsti nei design draft) e i support attuali possono solo buffare offensivamente.

`Heal` e `Cleanse` mancano del tutto come `Effect` del DSL — i blueprint support sono bloccati. `PerHop` serve perché la pipeline follow-up FIFO può causare re-apply patologici (es. counter che ri-applica lo stesso debuff appena messo da un altro hop nella stessa catena), e la roadmap M020+ aprirà chain più complesse.

Ora perché: M020+ (blueprint support) dipende strutturalmente da queste tre primitive. Non procediamo sui kit senza la base.

## User-Visible Outcome

### When this milestone is complete, the user can:

- Eseguire `cargo test` e vedere i nuovi integration test su DR aggregation, Heal targeting (incluso filtro KO), Cleanse ordering, e PerHop dedup passare deterministicamente.
- Eseguire `cargo run --bin combat_cli` e osservare nel JSONL log eventi `DamageDealt` con campi `pre_dr` e `final` distinti quando una unità ha buff DR attivi.
- Definire in `assets/data/skills.ron` skill con `Effect::Heal { amount, target }` ed `Effect::Cleanse { count, target }` e vederle risolversi correttamente in combat.

### Entry point / environment

- Entry point: `cargo test` (integration suite in `tests/`) + `cargo run --bin combat_cli` per smoke determinismo.
- Environment: local dev, headless (no feature `windowed`).
- Live dependencies involved: none (puro engine, no I/O, no RNG nuovi).

## Completion Class

- **Contract complete means:** tutti gli integration test nuovi passano deterministicamente (DR aggregation con 0/1/N buff + clamp 0.5, Heal su Single KO = IllegalTarget, Heal su AllAllies skippa KO, Cleanse ordering, PerHop dedup). Identità preservata: combat senza buff DR / senza skill Heal-Cleanse produce gli stessi JSONL trace del pre-milestone.
- **Integration complete means:** DR è instradato attraverso TUTTI i call-site di `calculate_damage` — incluso DoT tick (`status_effect.rs`) e damage diretto da `resolution.rs`. Nessun bypass. `CombatEvent::DamageDealt` espone `pre_dr` e `final` per observability.
- **Operational complete means:** N/A (headless, no lifecycle, no servers).

## Final Integrated Acceptance

Per chiamare M019 completo, dobbiamo provare:

- Un encounter scriptato in cui un Gabumon con DR 50% attivo subisce un colpo break-amplified e il damage finale è il break-amplified × 0.5 (entrambi i moltiplicatori applicati nell'ordine corretto), tracciato in JSONL con `pre_dr` e `final` distinti.
- Una skill `Heal` su `AllAllies` con un alleato KO e due vivi: i due vivi guariscono dell'amount, l'evento Heal NON viene emesso per il KO, il `hp_current` del KO resta 0.
- Una skill `Cleanse { count: 2 }` su una `StatusBag` con 5 debuff non-immuni di durate diverse: i due con `remaining` più lungo vengono rimossi (tie-break su slot_index ascendente), emissione di eventi `StatusExpired` deterministici.
- Una catena follow-up in cui A applica Debuff X su B, B counter-attacca con effect "apply X to A": il secondo apply viene bloccato da `PerHop` con un evento di tracing (`StatusBlockedPerHop` o simile).

## Architectural Decisions

### DR è applicato come step esterno post-`calculate_damage`, non dentro la formula

**Decision:** `calculate_damage` in `damage.rs` resta invariato e ritorna il `DamageBreakdown` "lordo" attuale. DR è applicato come step separato in `resolution.rs` (o nuovo modulo `mitigation.rs` se emergono altri layer):

```
pre_dr_damage = calculate_damage(...).final_damage
dr_sum        = aggregate_dr(defender_status)        // intra-source max-replace, cross-source additive, clamp [0.0, 0.5]
final_damage  = ((pre_dr_damage as f32) * (1.0 - dr_sum)).round() as i32
final_damage  = final_damage.max(0)
```

**Rationale:**
- Tiene separati i due assi: qualitative (triangle/tag/break/amp) dentro `calculate_damage`, numerical mitigation (DR) fuori. Break amplifica (offensivo); DR mitiga (difensivo). Testabili in isolamento.
- Observability gratis: `CombatEvent::DamageDealt` porta `pre_dr` e `final` come campi distinti senza inquinare `DamageBreakdown`.
- DR si applica *anche* al break-amplified — coerente con design HSR-like "DR è scudo finale del difensore".
- DoT tick beneficia banalmente: stesso instradamento `compute → apply_dr → write hp`.
- Mantiene la regola del success criterion: "step moltiplicativo, no log space, no cap diverso dal clamp di aggregazione".

**Alternatives Considered:**
- DR dentro `calculate_damage` come ulteriore moltiplicatore — accoppia mitigation a una funzione che oggi è puramente qualitativa; richiede campo extra in `DamageBreakdown`; rende più rumoroso il tracing.
- DR come stat numerica permanente su `Unit` — fuori scope del milestone, e contraddice il design "DR vive come buff temporaneo" della roadmap.

### Aggregazione DR: intra-source max-replace, cross-source additive, clamp [0.0, 0.5]

**Decision:** Per ogni `BuffKind::DR(pct)` nel `StatusBag` del difensore:
1. Raggruppa per `source_id` (l'entità che ha applicato il buff).
2. Intra-source: prendi il `max(pct)` per ogni source — un Patamon non può stackare due DR su se stesso, l'ultimo (più alto) sostituisce.
3. Cross-source: somma i max dei diversi source — DR da Patamon + DR da Gabumon = additivo.
4. Clamp finale: `dr_sum.min(0.5).max(0.0)`.

**Rationale:** Replica taxonomy del progetto già usata per altri buff. Il clamp 0.5 è il design choice del milestone (no immortalità da multi-support). Zero buff DR → `dr_sum = 0.0` → moltiplicatore 1.0 → **identità neutra** sul comportamento attuale (success criterion "trade-off neutrality").

**Alternatives Considered:**
- Tutto additivo (no max-replace intra-source) — permette self-stacking che rompe il design del kit support.
- Tutto max-replace (anche cross-source) — sopprime il valore della composizione team multi-support.

### `Heal` filtra KO a monte; su `Single` KO la skill è `IllegalTarget`

**Decision:** Resolver di `Effect::Heal`:
- `target: Single(id)` con `unit.hp_current == 0` → la skill non parte, emette `CombatEvent::IllegalTarget { reason: "target_ko" }`. Stessa policy dei damage effect su target morto (da verificare in `resolution.rs` durante l'implementazione e replicare il pattern esistente).
- `target: AllAllies` → filtra i KO a monte; gli alleati vivi guariscono, i KO non emettono evento Heal.
- Nessuna skill nel game design corrente intende **rianimare** via Heal; la rez è un'operazione separata (fuori scope M019).

**Rationale:** Consistente con come oggi `resolution.rs` tratta target morti su damage skill. Evita comportamenti emergenti tipo "heal-rez accidentale".

**Alternatives Considered:**
- Heal che rianima — cambia profondamente il design del KO state; out of scope.
- Heal emette evento `Healed { amount: 0 }` su KO — rumore senza significato, e crea ambiguità nel log.

### `Cleanse` ordering: durata residua decrescente (più lunghi prima), tie-break slot_index ascendente

**Decision:** `Effect::Cleanse { count: Option<u32>, target }` con `count = Some(N)`:
1. Filtra i debuff non-immuni dalla `StatusBag`.
2. Ordina per `remaining` (durata residua) **decrescente**.
3. Tie-break: `slot_index` ascendente (deterministico via insertion order della `StatusBag`).
4. Rimuovi i primi N. Per ogni rimozione, emetti `CombatEvent::StatusExpired { target, status_id, reason: "cleansed" }` (o evento dedicato — decisione durante implementazione).
5. `count = None` → rimuove tutti i debuff non-immuni.

**Rationale:** Massimizza il valore del Cleanse player-side (toglie i debuff più persistenti). Tie-break su slot_index è deterministico, replay-safe, e indipendente da hash/random.

**Alternatives Considered:**
- FIFO (i più vecchi prima) — soggetto a "race condition" semantica: un debuff appena applicato a lunga durata non viene rimosso preferenzialmente.
- Per severity (potency) — non c'è oggi un campo `potency` uniforme su tutti i debuff; rimandato a milestone futura se introduciamo severity tagging.

### `PerHop` scope: per "hop" di catena follow-up, non per round

**Decision:** Un "hop" è **una singola risoluzione di skill** all'interno di una catena follow-up/counter. La guardia `PerHop` traccia `HashSet<(target_entity, status_kind_id)>` resettato **all'inizio di ogni nuovo hop** (cioè ogni volta che la pipeline pesca il prossimo elemento dalla FIFO follow-up queue).

**Rationale:** Round-scoped sarebbe troppo aggressivo (impedirebbe ri-applicazione legittima tra turni diversi nella stessa round); skill-cast-scoped (sub-hop) sarebbe troppo permissivo (non blocca counter→counter→counter loops sullo stesso debuff). Hop-scoped è il livello giusto: catena di reazioni della stessa azione originale.

**Alternatives Considered:**
- Round-scoped — blocca pattern legittimi di re-applicazione tra player phases successive.
- Cast-scoped — non risolve il counter loop, che è il pattern problematico documentato.
- Globalmente "no double apply" — equivalente a immunità permanente, rompe il game design.

## Error Handling Strategy

Tre classi di errore, tutte non-panicking:

- **Skill illegale su target invalid** (Heal su Single KO): la skill non altera stato, emette `CombatEvent::IllegalTarget`, l'SP/turn cost va decisa durante il planning slice (probabilmente: skill non parte → no consumo). Allinea con il pattern esistente per damage skill su morti.
- **DR aggregation overflow** (`dr_sum > 0.5`): clamp silenzioso a 0.5. Nessun warning a log: è comportamento documentato del clamp.
- **PerHop block**: emette `CombatEvent::StatusBlockedPerHop { target, status_kind, reason: "perhop" }` per tracing. Non è un errore — è behavior atteso. Il source skill non considera questo un failure.

Niente unwrap su path comuni, niente expect su path runtime. Test deterministici per ogni branch error.

## Risks and Unknowns

- **DR routing completeness:** Ogni call-site di `calculate_damage` deve passare attraverso `apply_dr`. Oggi ci sono almeno due: damage skill diretto in `resolution.rs` e DoT tick in `status_effect.rs`. Va fatto un grep esaustivo durante T01 — se ne manchiamo uno, DR è bypassabile su un asse di damage. Mitigazione: integration test che chiama ogni path noto con DR attivo e verifica `final < pre_dr`.
- **PerHop boundary su counter chains:** il termine "hop" è chiaro per follow-up FIFO esplicita, ma counter (es. retaliate) potrebbe attraversare lo stesso layer. Va deciso durante S04 (PerHop guard slice) se counter è "stesso hop" del trigger o un nuovo hop. Lean: counter = nuovo hop (così la guardia non blocca counter legittimi).
- **Eventi Healed/Cleansed esistono già?** Da verificare in `events.rs`. Se no, vanno aggiunti — ma è un'aggiunta meccanica, non architetturale.
- **Identità neutra dei trace JSONL:** combat senza buff DR e senza skill Heal/Cleanse deve produrre lo stesso log JSONL del pre-milestone. Vincolo forte di non-regressione. Mitigazione: golden test su trace fixture.

## Existing Codebase / Prior Art

- `src/combat/damage.rs` — `calculate_damage`, `DamageBreakdown`. Resta intatto.
- `src/combat/resolution.rs` — entry point per `apply_dr`. Da estendere.
- `src/combat/status_effect.rs` — DoT tick path; deve essere instradato in `apply_dr`. Anche home di `StatusBag` e `BuffKind` (aggiungere `DR(f32)` con `source_id` tracking).
- `src/data/skills_ron.rs` — `Effect` enum; aggiungere variant `Heal { amount, target }` e `Cleanse { count: Option<u32>, target }`.
- `src/combat/events.rs` — `CombatEvent`; estendere `DamageDealt` con `pre_dr`, aggiungere `IllegalTarget`, `StatusBlockedPerHop`, eventualmente `Healed`/`Cleansed` se non esistono.
- `src/combat/follow_up.rs` — pipeline FIFO; hook per scope `PerHop`.
- `assets/data/skills.ron` — definizione test skill che usano DR/Heal/Cleanse.
- `tests/` — pattern esistenti per integration deterministici (vedi `follow_up_triggers.rs`).

## Relevant Requirements

Da popolare in fase di planning slice se i requirement R### esistono — il context milestone non li auto-mappa; vedere `.gsd/REQUIREMENTS.md`.

## Scope

### In Scope

- `BuffKind::DR(f32)` nel `StatusBag` esistente con `source_id` tracking e taxonomy max-replace intra-source + additive cross-source + clamp 0.5.
- Step `apply_dr` esterno a `calculate_damage`, instradato su tutti i call-site (damage diretto + DoT tick).
- `Effect::Heal { amount, target }` con KO filter (Single = IllegalTarget; AllAllies = skip silenzioso).
- `Effect::Cleanse { count: Option<u32>, target }` con ordering durata-decrescente + slot_index tiebreak.
- `PerHop` guard hop-scoped su `(target, status_kind)` con event tracing.
- Integration tests in `tests/` per ogni branch (DR aggregation, KO targeting, Cleanse ordering, PerHop dedup, identità neutra).
- Eventi `CombatEvent` estesi: `pre_dr`/`final` su `DamageDealt`, `IllegalTarget`, `StatusBlockedPerHop`, e `Healed`/`Cleansed` se non esistono.

### Out of Scope / Non-Goals

- Stat numerica `defense`/`armor` permanente su `Unit` — DR resta puramente buff-based in questo milestone.
- Rianimazione via Heal — KO recovery è una primitiva separata, fuori M019.
- Severity tagging dei debuff per Cleanse priority — useremo durata, non potency.
- UI/egui per visualizzare DR sui sprite — feature `windowed` non toccata.
- Blueprint Patamon `holy_aegis` / Gabumon `fur_cloak` finali — M020+.
- Counter mechanic refactor — `PerHop` non rifà la pipeline counter, la osserva.

## Technical Constraints

- Headless first: zero dipendenze da `winit`/`wgpu`/`egui` fuori da `feature = "windowed"`.
- Determinismo: nessun RNG senza seed, nessun wall-clock, nessun `HashMap` iteration su path che produce eventi (usare `BTreeMap` o ordering esplicito).
- Bevy 0.18, toolchain in `rust-toolchain.toml`.
- Non aggiungere unit test inline in `src/` salvo `#[cfg(test)] mod tests` brevi — integration in `tests/`.
- Skill DSL: schema cambi solo in `src/data/skills_ron.rs`, dati in `assets/data/skills.ron`.
- `CombatEvent` è single-source-of-truth: UI/log leggono eventi, non mutano stato.

## Integration Points

- `damage.rs::calculate_damage` ← invariato; chiamato da resolution + DoT tick.
- `resolution.rs` ← nuovo `apply_dr` step inserito dopo `calculate_damage`, prima di scrivere `hp_current` ed emettere `DamageDealt`.
- `status_effect.rs` (DoT tick) ← stesso instradamento, condivide `apply_dr`.
- `events.rs` ← estensione enum `CombatEventKind`.
- `follow_up.rs` ← hook per scope `PerHop` (boundary check su pull dalla FIFO).
- `skills_ron.rs` ← variant nuovi in `Effect`.
- `jsonl_logger.rs` / `observability.rs` ← consumano eventi nuovi automaticamente se sono nel bus.

## Testing Requirements

Integration tests deterministici in `tests/`, naming funzionale (NO `s##_…`):

- `dr_aggregation.rs` — 0/1/N buff DR, intra-source max-replace, cross-source additive, clamp 0.5, `pre_dr`/`final` nel trace.
- `dr_dot_path.rs` — DoT tick attraversa `apply_dr`.
- `heal_targeting.rs` — Single KO = IllegalTarget; AllAllies skippa KO; nessun rez accidentale.
- `cleanse_ordering.rs` — durata-decrescente, tie-break slot_index, count = None / Some(N), debuff immuni saltati.
- `perhop_guard.rs` — counter ri-applicante stesso debuff bloccato; debuff diverso passa; reset all'inizio di nuovo hop.
- `m019_identity_neutral.rs` — golden test JSONL: combat scriptato senza DR/Heal/Cleanse produce lo stesso trace del pre-milestone (modulo eventuali campi nuovi nullable/default).

Coverage minima: ogni `Decision` ha un test che la prova; ogni branch error ha un test che lo trigghera.

## Acceptance Criteria

Per-slice, da finalizzare durante `gsd_plan_milestone`. Indicative:

- **S01 (DR primitive + aggregation):** `BuffKind::DR` esiste, aggregation function ha integration coverage, taxonomy max-replace + additive + clamp dimostrata.
- **S02 (DR routing):** `apply_dr` instradato su damage diretto e DoT tick; `DamageDealt` espone `pre_dr`/`final`; identity test passa.
- **S03 (Heal + Cleanse Effect):** entrambi gli `Effect` parsano da RON, risolvono in combat, KO filter Heal e Cleanse ordering testati.
- **S04 (PerHop guard):** scope hop-based, event tracing, counter loop test passa.

I nomi slice sono indicativi; la roadmap finale è prodotta da `gsd_plan_milestone`.

## Open Questions

- **Counter vs hop boundary:** un counter è un nuovo hop o lo stesso hop del trigger? Current thinking: nuovo hop (la guardia osserva, non blocca counter legittimi). Da finalizzare in S04.
- **Eventi `Healed`/`Cleansed` aggregati o granulari?** Cleanse rimuove N status → un evento `Cleansed { target, removed: Vec<StatusId> }` oppure N eventi `StatusExpired { reason: "cleansed" }`? Current thinking: N eventi `StatusExpired` (riusa l'evento esistente, ridotto rumore di tipo). Da decidere durante S03.
- **Skill che fallisce su IllegalTarget consuma SP/turn?** Current thinking: non consuma (replica il pattern di damage skill su KO; da verificare in `resolution.rs` durante T01 e replicare deterministicamente).
- **`source_id` su buff: campo nuovo o riusabile?** Va verificato durante T01 se `StatusBag` già traccia source per altri scopi (es. dispel-by-source). Se no, lo aggiungiamo.
