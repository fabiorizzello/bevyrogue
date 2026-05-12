# §2.2b — Animation FSM — clipmontage come grafo orchestratore

**Stato:** amendment a §2.2 (clipmontage flat). Sostituisce la *forma* del file `clipmontage.ron` mantenendo invariati `clip.ron` (§2.2), `skills.ron` (§2.1), `signal_bindings.ron` (§2.1) e i boundary §2.7/§2.8.

**Decisione:** `clipmontage.ron` non è più una **lista piatta di notify ai frame** — diventa un **grafo finito (FSM)** di nodi, dove ogni nodo:

- referenzia un sub-range della clip (`frames: (start, end)`),
- emette **Commands** dichiarativi `on_enter` (intent astratti, no `KernelEffect` diretti),
- transita verso altri nodi via **edges con predicati `when:`** (time, kernel event, unlock).

La FSM è **opt-in**: una skill senza grafo dichiarato usa fallback "linear playback della clip, no decorations" — equivalente al `clipmontage.ron` con bindings vuoti di §2.2.

---

## A — Perché un grafo, non una lista

| Capacità | Lista piatta (§2.2) | Grafo (§2.2b) |
|---|---|---|
| VFX/Sfx a frame fisso | ✅ | ✅ (`on_enter` su nodo) |
| Modifier playhead (Hold/SpeedMul/Loop) | ✅ | ✅ (campo `modifier` su nodo) |
| **Variante visiva da unlock skill-tree** | ❌ (richiederebbe N file paralleli) | ✅ (edge `when: Unlock(id)`) |
| **Branching reattivo** (counter su attacco in arrivo durante windup) | ❌ | ✅ (edge `when: KernelEvent(IncomingDamage)`) |
| **QTE come gate** (skill bivalente: success → potenziamento, fail → ramo base) | ❌ (la lista non sa interrompere il playhead) | ✅ (nodo con edge su `UserInput(QteSuccess/Fail)`) |
| **Cancel su stun caster** | ❌ (notify rimangono pending) | ✅ (edge implicito `* → Cancel`) |
| **Generazione AI** (agent scrive il file) | Hard (notify accavallati richiedono contesto globale) | Friendly (nodi sono unità leggibili, edges esplicite) |

I 6 esempi che hanno scatenato l'amendment ("triple_hit allunga open mouth", "super_charge freeza su charge", "counter_breath", branching path, QTE amplify, granted variant) cadono **tutti** in casi che la lista piatta non sa rappresentare senza duplicare file.

---

## B — Boundary: cosa cambia rispetto a §2.2

**§2.2 invariato:**
- `clip.ron` resta source-of-truth dei frame range (lossless dal json).
- I notify trigger restano cosmetici (Particle, Sfx, Shake, Flash, ScreenFreeze).
- I modifier restano modifier del playhead (Hold, SpeedMul, Loop).
- La regola §2.2 "rimuovere il file lascia il combat invariato ma muto" **non vale più**: la FSM partecipa al gameplay (sequenzia damage emission, gate QTE). Vedi §G.

**§2.2 esteso:**
- Il file `clipmontage.ron` ora descrive **un grafo**, non una lista. La sintassi nodo-edge è più verbose ma assorbe la lista come degenerate case (vedi §J migrazione).
- Le entry diventano `Command` (vocabolario chiuso §C), non più `Notify` raw — il blueprint le traduce in `KernelEffect`/Notify presentation.

**Cosa NON cambia:**
- `clip.ron` è invariato (asset di animazione, no logica).
- `skills.ron` è invariato (identità + numeri base, no condizionali — §2.1).
- `skill_tree.ron` (file nuovo, §I) è il **gemello gameplay** di clipmontage: patch numerici sui params di `skills.ron` quando un nodo skill-tree è sbloccato. Non interagisce con la FSM se non come **input statico** (vedi §F).
- `signal_bindings.ron` (§2.1) è invariato — glue skill ↔ kernel signal, ortogonale alla FSM.

---

## C — Vocabolario Commands (chiuso, 6 verbi base M017)

La FSM emette **Commands** in `on_enter` del nodo. Il blueprint del Digimon proprietario li riceve (`SkillBehavior::execute` → §2.7) e li traduce in `KernelEffect` (gameplay) o notify presentation (cosmetic). Vocabolario minimale, espandibile solo con bump decision:

| Command | Tipo | Esegue | Traduzione blueprint |
|---|---|---|---|
| `EmitDamage { hits, mul_param, status?, chance_pct?, dur? }` | gameplay | applica N danni al target con scaling da params | `KernelEffect::Damage` × N (+ opzionale `ApplyStatus`) |
| `EmitStatus { id, dur_param, chance_param, target }` | gameplay | applica status a target | `KernelEffect::ApplyStatus` |
| `SpawnParticle { name, origin: VfxLocus, motion: VfxMotion }` | cosmetic | crea VFX al frame corrente, posizionato via Locus + animato via Motion (vedi §2.2d) | notify presentation bus (`NotifyParticle`) |
| `Shake { intensity, duration_ms }` | cosmetic | screen shake | notify presentation bus (`NotifyShake`) |
| `Hold { extra_frames }` | playhead | pausa playhead nel nodo corrente | modifier playhead UI animator |
| `StartQTE { kind, window_param, headless_default_param }` | suspend | apre QTE, sospende kernel (§2.6) | `Suspend(YieldReason::QuickTimeEvent)` |

**Regole di vocabolario:**

1. **Chiuso**: nuove Command richiedono review + bump major. Niente "Command::Custom(string)" — sarebbe scripting language nascosto.
2. **Numeri via reference, non literal**: `mul_param: "atk_mul"` legge `skills.ron::params["atk_mul"]`, eventualmente patchato da `skill_tree.ron`. Mai `mul: 1.6` nel grafo (rompe §2.1: dati = numeri, FSM = logica).
3. **Cosmetic ≠ gameplay**: `SpawnParticle` non emette mai `KernelEffect`. `EmitDamage` non spawna mai particelle direttamente — i particle reattivi al damage vengono da `OnKernelEvent(DamageDealt)` (vedi §D edge predicates).

**Espansioni candidate (non in M017):** `KnockBack`, `Move`, `Teleport`, `SummonMinion`, `Mark` (per setup→payoff). Aggiungere solo quando una skill concreta li richiede.

---

## C2 — Vocabolario Commands esteso (M017+ kernel-known)

Round di review-3 del roster (Agumon/Dorumon/Gabumon/Tentomon/Renamon/Patamon) ha promosso **7 verbi candidati a approved kernel-known**. Skill concrete del roster M017 li richiedono. Tutti rispettano le tre regole §C (chiuso / numeri via reference / cosmetic ≠ gameplay) e tutti vivono kernel-side perché toccano state condiviso oltre la singola entity blueprint (TurnOrder, status registry, SP pool, hp altrui).

| Command | Tipo | Esegue | Traduzione blueprint | Skill source roster |
|---|---|---|---|---|
| `EmitHeal { amount_param, amount_kind: (HpPctMax\|HpPctMissing\|Flat), target }` | gameplay | applica heal a target; emette `Healed` event sul bus (alimenta Patamon ult charge `+25/heal event`) | `KernelEffect::Heal` | Patamon `patapata_hover` / `sparking_air_shot` |
| `EmitCleanse { count_param, selector: (FIFO\|LIFO\|Random), target }` | gameplay | rimuove fino a `count` status `kind:Debuff` da target via selector; emette `StatusRemoved` per ognuno | `KernelEffect::RemoveStatus` × N | Patamon `patapata_hover` / `sparking_air_shot` |
| `AdvanceTurn { actor, pct_param }` | gameplay | avanza gauge `TurnOrder` di `actor` di `pct%` del **next-action gauge** (NON del `speed` stat); cap ±50%/call, clamp gauge `[0, 200]` (§C2.1) | `KernelEffect::AdvanceTurnGauge` (kernel-owned `TurnOrder`) | Renamon `koyosetsu`, `kitsune_grace` |
| `DelayTurn { target, pct_param }` | gameplay | ritarda gauge `TurnOrder` di `target` di `pct%` del **next-action gauge** (NON del `speed` stat); cap ±50%/call, clamp gauge `[0, 200]` (§C2.1) | `KernelEffect::DelayTurnGauge` | Renamon `tohakken` |
| `ApplyBuff { id, dur_param, kind: (Buff\|Debuff\|DR\|Aura\|Mark), target }` | gameplay | unifica `EmitStatus` con flag `kind` esplicita; status registry applica regole cleanse-eligibility per `kind:Buff` ∉ cleansable (vedi §2.8 §H) | `KernelEffect::ApplyStatus` con `BuffKind` field | Renamon `tohakken` (Blessed), Patamon `holy_aegis` (Aura), Gabumon `fur_cloak` (DR) |
| `EmitSpGrant { amount_param, target }` | gameplay | aggiunge SP a SP pool del team di `target`; emette `SpGranted` event; **cap-aware sul lato ricevente** (`SpPool.add` clamp al cap, vedi `src/combat/sp.rs`), **non passa** dal contatore `RoundSpTracker.max_non_basic_per_round` (grant ≠ spend) | `KernelEffect::GrantSp` | Gabumon `blue_cyclone` (ult, team +1), Tentomon `electrical_discharge` (ult, team +1), override `+2 SP` Tentomon basic data-side via `units.ron.sp_gen_per_basic` |
| `Reposition { anchor, target }` | gameplay | sposta `target` a posizione `anchor` sulla combat line (riusa §02-02c §D dematerialize pattern, ora promosso) | `KernelEffect::Reposition` | Dorumon `dash_metal` follow-up (chiude §02-02c §D dangling) |
| `BlockReaction { kind, target_ref, damage_mult, dur }` | gameplay (pre-DR pipeline) | applica `damage_mult` (es. 0.50) a `IncomingDamage` su `target` **prima** del cascade DR standard; emette `BlockReactionTriggered` event (§R-Events) post-mitigation. **Canon FSM pattern + stack rules: `tentomon/04 §1.5/§4`** (X10 consolidation, round-3 2026-05-12). Argomenti (`BlockReactionArgs`): vedi `agumon/04 §9 G-Verbs` per la firma | `KernelEffect::BlockReaction` | Tentomon `battery_loop` (FSM `BlockProc.on_enter`) |

**Sugar form, NO command separato:** `ApplySelfBuff { id, dur_param, kind }` è alias di `ApplyBuff { target: EntityRef::Self_ }`. Vocabolario kernel resta minimal — il blueprint può scrivere la forma corta come zucchero sintattico nel proprio executor, ma `tick_fsm` emette solo `ApplyBuff`. Stesso pattern di `TargetCenter ↔ EntityCenter(Primary)` in §2.2d §B.

**Relazione `EmitStatus` ↔ `ApplyBuff`:** `ApplyBuff` è **strict superset** di `EmitStatus` (§C base) — aggiunge il campo `kind`. M017+ usa `ApplyBuff` ovunque; `EmitStatus` resta nel vocabolario §C base come **forma degenerate** (`kind` defaulta a `Debuff`) per compat lazy delle skill già scritte. Nessuna doppia logica kernel-side: `EmitStatus { ... }` → `ApplyBuff { ..., kind: Debuff }` a parse-time del RON.

**Cross-ref status registry (round-3, 2026-05-12, X8):** il vocabolario `id` di `EmitStatus`/`ApplyBuff` è il **`StatusKind` enum chiuso** definito in `02-08 §H.1` (Heated/Chilled/Paralyzed/Slowed/Blessed + riservati Burn/Shock). Il vocabolario `kind` di `ApplyBuff` è il **`BuffKind` enum chiuso** definito in `02-08 §H.2` (Buff/Debuff/DR/Aura/Mark). Validator `02-02b §L` rigetta a load-time qualsiasi `id` non in `StatusKind` o `kind` non in `BuffKind`. Regole cleanse-eligibility, stack policy, DR stacking intra/cross-unit, lifecycle Aura/Mark sono **tutte** in `02-08 §H` — non duplicare qui.

**Tutti i 7 verbi sono kernel-known per design.** Test discriminante: il Command tocca state condiviso oltre la singola entity blueprint (TurnOrder resource, status registry con cleanse-eligibility, SP pool, hp altrui). Tenerli come custom blueprint code drifterebbe la semantica tra digimon (es. `heal` Patamon vs `heal` futuro Vaccine-healer si scollerebbe; cleanse FIFO/LIFO/Random si bifurcherebbe). Il prezzo "tocchi kernel per aggiungere verbo" si paga **una volta in M017**, poi è costante.

**Blueprint-local resta:** trigger predicates e state reads dentro Forma C FSM passive (es. Dorumon `predator_loop` `hp_pct < 0.5`, Agumon `twin_core_fire` `partner.has_status(Chilled)`). Blueprint legge stato → emette Commands kernel-known sugli eventi. Boundary chiara: **blueprint sa leggere, kernel sa scrivere.**

**Espansioni candidate aggiornate (post-C2, non in M017):** `KnockBack`, `Move`, `Teleport`, `SummonMinion`, `Mark` restano candidati come §C base; rivalutazione round-4+.

---

## C2.1 — Time-manip metric: `% gauge`, no flat speed-stat (X11, 2026-05-12)

**Regola canonica** (canon: `renamon/00 §8 D1`): `AdvanceTurn`/`DelayTurn` operano **esclusivamente** su `pct%` del **next-action gauge** del target (`TurnGauge` field-per-entity, runtime), **mai** sul `speed` stat (invariante di unit, definito in `units.ron`).

| Aspetto | Decisione |
|---|---|
| Unit della `pct_param` | percentuale del next-action gauge `[0, 200]` |
| Direzione `Advance` | sottrae `pct` al gauge → entity agisce prima (gauge `0` = ready) |
| Direzione `Delay` | aggiunge `pct` al gauge → entity agisce dopo |
| Cap per singola call | `±50%` (clamp resolver-side, **non** edge-side) |
| Clamp globale gauge | `[0, 200]` post-applicazione |
| Stacking multi-edge | additivo (es. due `kitsune_grace` proc nello stesso tick = +20% advance), clamp finale gauge |
| `speed` stat impact | **nessuno** — `speed` resta tier-level invariant per turn-order init; Commands `AdvanceTurn`/`DelayTurn` NON lo modificano |

**Rationale (riassunto da `renamon/00 §8 D1`):**
- Leggibilità HSR-style: turn-order tracker UI mostra shift `%` del gauge, non delta `speed` astratto.
- Evita interazioni opache con `speed` stat (es. "Renamon koyosetsu rende permanentemente più veloce Renamon?" — risposta: no, mai).
- Match identity §5 (`AdvanceTurn(target, pct)` / `DelayTurn(target, pct)`).

**Vocabolario vincolato:** `pct_param` accetta solo valori `[0, 50]` (cap per singola call). Validator §L rigetta a load-time `pct_param` > 50 o riferimenti a `speed`-derived ParamRef per `AdvanceTurn`/`DelayTurn`.

**Race con `TurnOrder` (`renamon/02 §T2`):** il kernel applica `AdvanceTurn`/`DelayTurn` solo dopo che la FSM corrente exit (`Recovery.exit`); mid-FSM è atomicamente queued. Conforme a identity §5 ("modifiche atomiche dopo resolution; nessuna reorder mid-action").

---

## C3 — `TargetShape` enum (consolidato, blueprint-side resolver)

`TargetShape` è il **vocabolario chiuso** dei selettori di target per Commands gameplay (`EmitDamage`, `EmitHeal`, `EmitCleanse`, `EmitStatus`, `ApplyBuff`, `AdvanceTurn`, `DelayTurn`, `EmitSpGrant`, `Reposition`). Definito qui come fonte canonica unica — rimpiazza i frammenti sparsi nei roster doc (`agumon/04 §G-Sel`, `gabumon/02 §F3`, `tentomon/02 §C1`, `tentomon/03 §D1`).

```rust
pub enum TargetShape {
    // === Single target (1 entity) ===
    Primary,                                              // primary target of the action
    Self_,                                                // caster
    AdjLeft,                                              // left neighbour on combat line
    AdjRight,                                             // right neighbour on combat line
    SingleAlly { slot: Option<u8> },                      // None = chooser via UI / AI
    AdjLowest { metric: HpPctMin | HpMin | RawHpMin,
                side: Side },                             // metric-selected adj
    LowestHpPctAlive { side: Side },                      // global lowest HP% in side
    NextAliveAdj { side: Side, scan: ClockWise | CounterClockWise },  // bounce-hop helper
    RandomEnemyAlive { seed: SeedSource },                // SeedSource ∈ {TurnRng, CombatRng}

    // === Multi target (>1 entity) ===
    Blast(TargetRef),                                     // primary + 2 adj (3 entities)
    AoE { side: Side, exclude_dead: bool },               // tutti i target del side
    Bounce { hits: u8, selector: Box<TargetShape> },      // chain N hits, re-resolve ogni hop
}

pub enum Side { EnemyTeam, AllyTeam, BothTeams }
pub enum SeedSource { TurnRng, CombatRng }
```

**Resolver contract:**

```rust
/// Blueprint-side pure function. Returns 1..N TargetRef.
/// Empty Vec ⇒ Command is no-op (silent drop, no panic).
fn resolve_shape(shape: TargetShape, ctx: &CommitCtx) -> Vec<TargetRef>;
```

**Regole:**

1. **Blueprint-side resolver, NOT kernel-side.** `TargetShape` è vocabolario blueprint (§5/§6 commit-time resolver). Il blueprint chiama `resolve_shape(shape, ctx) -> Vec<TargetRef>` e emette **N Command separati**, uno per `TargetRef` risolto. Il kernel vede solo Commands con `target: TargetRef` concreto, mai `TargetShape` direttamente. Coerente con `agumon/03 §6 scelta A`.

2. **Snapshot-once.** `resolve_shape` viene chiamato a `commit_action`, il Vec risolto è snapshot frozen per la durata della skill (no re-resolve mid-cascade, eccetto `Bounce` che re-risolve il selector interno ad ogni hop, vedi punto 4).

3. **Failure modes.** `TargetRef` despawned/dead post-resolution ⇒ il singolo Command per quel target è droppato silently dal kernel (no panic, no fallback). Skill multi-target degrada graciously (es. `AoE` su 4 nemici di cui 2 morti mid-cascade → 2 hit applicati).

4. **`Bounce` semantica.** Re-risolve il `selector` interno ad ogni hop — supporta "chain bounce a target diverso ogni volta". Cap hit count = `hits`. Se il selector non trova target valido al hop N (es. tutti morti), la chain si interrompe (no panic). Esempio: `Bounce { hits: 3, selector: NextAliveAdj { side: EnemyTeam, scan: ClockWise } }` → fino a 3 hit, ognuno sul next alive enemy in senso orario.

5. **`RandomEnemyAlive` determinismo.** Usa `TurnRng` di default (seedato dal turn counter, deterministico per replay). `CombatRng` solo per random fuori-turno (rari, e.g. proc passivi triggerati out-of-turn). Headless test: seed esplicito via `combat_seed` fixture.

6. **Side rules.** `AllyTeam` include `Self_`? **No** per default — `AoE { side: AllyTeam }` esclude il caster (use `AoE { side: AllyTeam, exclude_dead: true }` + `Self_` separato se servono entrambi). Eccezione: `SingleAlly { slot: None }` può risolvere a self se il chooser lo permette (UI policy).

**Source-of-truth roster:**

| Skill | Shape canon | Doc |
|---|---|---|
| Agumon `baby_flame` (basic) | `Primary` | agumon/02 |
| Agumon `baby_burner` (ult) | `Blast(Primary)` | agumon/03 |
| Gabumon `gabumon_shot` (skill) | `AdjLowest { metric: HpPctMin, side: EnemyTeam }` | gabumon/02 |
| Gabumon `blue_cyclone` (ult) | `AoE { side: EnemyTeam, exclude_dead: true }` | gabumon/03 |
| Dorumon `dash_metal` (skill) | `LowestHpPctAlive { side: EnemyTeam }` | dorumon/02 |
| Renamon `koyosetsu` (skill) | `AoE { side: EnemyTeam, exclude_dead: true }` + `Self_` (advance) | renamon/02 |
| Renamon `tohakken` (ult) | `AoE { side: EnemyTeam, exclude_dead: true }` + `AoE { side: AllyTeam }` (Blessed) | renamon/03 |
| Tentomon `petit_thunder` (skill) | `Bounce { hits: 3, selector: NextAliveAdj { side: EnemyTeam, scan: ClockWise } }` | tentomon/02 |
| Tentomon `electrical_discharge` (ult) | `RandomEnemyAlive { seed: TurnRng }` × N | tentomon/03 |
| Patamon `patapata_hover` (skill) | `SingleAlly { slot: None }` | patamon/02 |
| Patamon `holy_aegis` (passive) | `AoE { side: AllyTeam }` | patamon/04 |

**Espansioni candidate (post-M017):** `Mark`/`Tracked` (auto-target su mark applicato, e.g. Dorumon `predator_loop` `tracked_target`), `FurthestFromCaster`, `HighestThreat`. Aggiungere quando la prima skill concreta li richiede.

---

## C4 — Reactive signature → FSM mapping (`08 §8.1` reverse table)

Il roster `08 §8.1` enumera 4 reactive signature v0 (`OnKill→Detonate`, `OnStatusApplied→Echo`, `OnKill→Chain`, `OnHitN→Apply`). Non sono primitive runtime — sono **shorthand di design** per pattern FSM edge + Command. Questa tabella formalizza la traduzione canon per dare ai blueprint M017 un riferimento unico.

| Reactive signature | Trigger (FSM edge predicate `§D`) | Effetto (Command `on_enter` su nodo Reactive) | Skill source (M017) |
|---|---|---|---|
| `OnKill→Detonate(status)` | `KernelEvent(UnitDied { target == strike_target })` | `EmitStatus { id: status, target: AoE { side: EnemyTeam, scope: Adj }, dur: <residuo strike target> }` | Agumon `baby_burner` (Heated spread sui 2 adj) |
| `OnStatusApplied→Echo(status)` | `KernelEvent(StatusApplied { status_id == target_status, target == strike_target })` con `caster_node == "Strike"` (no chain) | `EmitStatus { id: status, target: AdjLowest { metric: HpPctMin, side: EnemyTeam }, dur: <stesso del primary> }` | Gabumon `gabumon_shot` (Chilled echo) |
| `OnKill→Chain` | `KernelEvent(UnitDied { target == strike_target })` con `chain_count < max_chain` | re-emit `EmitDamage { ... }` su `LowestHpPctAlive { side: EnemyTeam }`, `chain_count += 1` | Dorumon `dash_metal` Predator state (max 1 chain) |
| `OnHitN→Apply(status)` | `TimeInNode` su nodo `Hop{N}` (l'`N`-esimo hit della Bounce chain ha già emesso `EmitDamage` in `on_enter`) | `EmitStatus { id: status, target: Primary, dur_param: ... }` sul nodo Hop{N} stesso, **dopo** `EmitDamage` (Command ordering: damage then status per leggere kernel event `DamageDealt` correttamente) | Tentomon `petit_thunder` (Paralyzed al hop 3) |

**Pattern shape:**

Ogni reactive signature si traduce in un **nodo Reactive aggiuntivo** nell'FSM (o, per `OnHitN`, in un Command extra sul nodo Hop{N} esistente). La topology standard FSM 3-nodi (Windup → Strike → Recovery) diventa 4-nodi quando la skill ha reactive signature:

```
Windup → Strike → Reactive → Recovery → Exit
                     ↑
                     └─ edge predicate: KernelEvent(...)
```

- **`Reactive` nodo:** clip frame range tipicamente sovrapposto a `Strike` (es. `(s, e)` con `s = strike.end`, `e = strike.end + 2` per dare 2 frame di VFX); `on_enter` emette il Command reattivo.
- **Edge `Strike → Reactive`:** predicate kernel-event. Se l'evento non arriva entro `TimeInNode(strike.frames)`, fallback edge `Strike → Recovery` con `priority: 0` lower del kernel-event edge.
- **Edge `Reactive → Recovery`:** `TimeInNode` o `Always` per chiusura standard.

**Esempio Gabumon `gabumon_shot`:**

```ron
"gabumon_gabumon_shot": AnimGraph(
    clip: "skill",
    entry: "windup",
    nodes: {
        "windup":   Node(frames: (0, 6)),
        "strike":   Node(frames: (6, 10),
                         on_enter: [
                             EmitDamage { hits_param: "hits", mul_param: "atk_mul", ... },
                             EmitStatus { id: "Chilled", target: Primary, dur_param: "chilled_dur" },
                         ]),
        "echo":     Node(frames: (10, 12),
                         on_enter: [
                             EmitStatus { id: "Chilled",
                                          target: AdjLowest { metric: HpPctMin, side: EnemyTeam },
                                          dur_param: "chilled_dur" },
                         ]),
        "recovery": Node(frames: (12, 16)),
    },
    transitions: [
        Edge(from: "windup",   to: "strike",   when: TimeInNode),
        Edge(from: "strike",   to: "echo",     when: KernelEvent(StatusApplied { status_id: "Chilled" }), priority: 10),
        Edge(from: "strike",   to: "recovery", when: TimeInNode, priority: 0),
        Edge(from: "echo",     to: "recovery", when: TimeInNode),
        Edge(from: "recovery", to: Exit,       when: TimeInNode),
    ],
)
```

**Regole di traduzione:**

1. **Reactive signature NON è un Command kernel-side.** Il blueprint executor `08 §8.1` short-hand non esiste a runtime — è una notazione di design che descrive shape FSM riusabili. Il blueprint M017 implementa il pattern via edge predicate + Command, niente "reactive signature" runtime field.
2. **Re-entry prevention.** La reactive signature NON deve auto-triggerare la propria edge (es. `OnStatusApplied→Echo(Chilled)` che applica `Chilled` non deve far ri-scattare `echo` infinito). Filtro canon: edge predicate include `caster_node == "Strike"` (vedi esempio sopra) o flag `once_per_skill: true` sull'edge (estensione §D candidata).
3. **Param naming convention.** Skill con reactive signature in `skills.ron::params` usa keys `reactive_signature_*` come metadati identificativi (es. `params: { "reactive_signature": "OnStatusApplied→Echo", ... }`); il blueprint executor li legge solo per debug/log. Il **vero comportamento** vive nell'FSM RON, non nel params.
4. **Patamon eccezione.** `08 §8.1` documenta Patamon "senza reactive signature" — la sua FSM 3-nodi standard non ha nodo Reactive aggiuntivo. Coerente con l'identità "support affidabile".

**Espansioni candidate (post-M017):** `Splash`, `Escalate`, `ShapeOverride` (rinviati in `08 §8.1`). Ognuno mapperà a un pattern FSM analogo quando il primo concreto emerge.

---

## D — Edge predicates (predicato chiuso)

Una transition `Edge(from, to, when)` valuta `when` ogni tick di playhead. Predicato chiuso:

```rust
pub enum Predicate {
    /// Playhead ha consumato tutti i frame del nodo corrente.
    TimeInNode,

    /// Un CombatEvent matcha il filter. Riusa EventFilter di §2.7.
    KernelEvent(EventFilter),

    /// Un input utente è arrivato (QTE outcome, branch picker).
    UserInput(InputFilter),

    /// Uno skill-tree node è unlocked. Risolto **snapshot-once** al commit (§F).
    Unlock(NodeId),

    /// Combinatori.
    And(Box<Predicate>, Box<Predicate>),
    Or(Box<Predicate>, Box<Predicate>),
    Not(Box<Predicate>),

    /// Default-true (per Exit edge incondizionato).
    Always,
}
```

**Priority resolution.** Edge uscenti dallo stesso nodo possono matchare contemporaneamente (es. `TimeInNode` ∧ `Unlock("super_charge")`). Si ordinano per `priority: u8` discendente dichiarata sull'edge; tie-break su ordine di dichiarazione nel RON.

**Esempio:**

```ron
Edge(from: "windup", to: "charged_extended",
     when: Unlock("super_charge"), priority: 10),
Edge(from: "windup", to: "charged",
     when: TimeInNode, priority: 0),
```

Quando il playhead esaurisce `windup`: se l'unlock è attivo, va in `charged_extended`; altrimenti `charged`. Senza priority esplicita, ordine di dichiarazione vince — leggibile ma fragile, **priority è obbligatoria** se più edge possono essere simultanee.

**Predicati ammessi e non:**

| Forma | Ammessa? | Note |
|---|---|---|
| `TimeInNode` | ✅ | playhead-driven, deterministic |
| `KernelEvent(StatusApplied { kind: "Stun" })` | ✅ | reagisce al bus |
| `UserInput(QteSuccess)` | ✅ | risolve `Suspend(QTE)` |
| `Unlock("super_charge")` | ✅ | snapshot-once |
| `KernelEvent(DamageDealt) and Unlock("triple_hit")` | ✅ | composizione |
| ~~`HP < 30%`~~ | ❌ | promuovi a kernel event `OnLowHpEntered` emesso dal bus, listen quello |
| ~~`params["hits"] > 1`~~ | ❌ | il grafo non legge numeri di gameplay |
| ~~Closure / RON expression~~ | ❌ | rompe determinismo + hot-reload |

---

## E — Topologia: i tre layer

```
                ┌────────────────────────────────────────────────────┐
                │  INPUTS (snapshot-once o live)                     │
                │   • UnlockedPassives  (static @ commit)            │
                │   • CombatEvents       (live, bus §2.8)            │
                │   • UserInput          (live, via §2.6 yield)      │
                └────────────────────────────────────────────────────┘
                                       │
                                       ▼
                ┌────────────────────────────────────────────────────┐
                │  AnimGraph FSM (clipmontage.ron)                   │
                │   - sequenzia nodi                                  │
                │   - emette Commands                                 │
                │   - gestisce QTE/branching/cancel                   │
                │   (interprete: anim_fsm_runtime.rs)                │
                └────────────────────────────────────────────────────┘
                                       │
                          Commands (intent dichiarativi)
                                       │
                                       ▼
                ┌────────────────────────────────────────────────────┐
                │  SkillBehavior (executor, §2.7)                    │
                │   - risolve params (skills.ron + skill_tree.ron)   │
                │   - traduce Commands in KernelEffect/Notify        │
                │   - emette via ctx.emit() (§2.7)                   │
                └────────────────────────────────────────────────────┘
                                       │
                          KernelEffect (mutazioni stato)
                                       │
                                       ▼
                ┌────────────────────────────────────────────────────┐
                │  Kernel (combat state, §2.8 cascade)               │
                │   - applica effetti                                 │
                │   - emette CombatEvent (DamageDealt, ...)          │
                │   - reactive hook dispatcher (§2.7 C2, §2.8)       │
                └────────────────────────────────────────────────────┘
                                       │
                          CombatEvent (live feedback)
                                       │
                                       └─► back into AnimGraph (edge `KernelEvent(...)`)
```

**Tre responsabilità separate:**

| Layer | Decide | Non decide |
|---|---|---|
| AnimGraph | **quando** emettere un intent (sequencing) | quanti danni / contro chi (lascia al blueprint) |
| Blueprint executor | **come** un intent diventa effetto (numeri risolti) | se l'effetto si applica davvero (lascia al kernel) |
| Kernel | **se** l'effetto applica (target vivo, immunità, stun) | la sequenza di azioni della skill (è dell'AnimGraph) |

Drift tra layer = bug auditable (es. AnimGraph emette `EmitDamage { hits: 3 }` ma blueprint produce 1 `KernelEffect::Damage` → mismatch ispezionabile via log).

---

## F — Snapshot-once vs live inputs

| Input | Quando viene letto | Motivazione |
|---|---|---|
| `UnlockedPassives` (skill-tree) | **Snapshot al `commit_action`** | Stabilità per la durata della skill (un unlock mid-skill non shape-shifta l'animazione). Stesso pattern §2.6 cause snapshotting |
| `active_form` (digivolution future) | Snapshot al commit | Idem; una form change mid-skill non mescola kit |
| `params` risolti (skills.ron + skill_tree.ron patches) | Snapshot al commit | Numeri stabili — il blueprint executor li ha già davanti quando traduce |
| `CombatEvent` (kernel bus) | **Live** ogni tick FSM | È *lo stato* del combat; reagire in tempo reale è il punto degli edge `KernelEvent(...)` |
| `UserInput` (QTE/picker) | **Live** via `Suspend → YieldResolved` (§2.6) | Il giocatore agisce dentro la skill; FSM riprende quando arriva |

**Implicazione test:** golden test `(graph, unlocks, kernel_events_sequence) → commands_sequence` è deterministico — gli unlock sono frozen, kernel events sono input ordinato, output è una sequenza ispezionabile.

---

## G — Headless determinism (la regola più importante)

Con §2.2 la clipmontage era **presentation-only**: ignorata in `cargo test`, il combat headless girava senza. Con §2.2b la FSM **sequenzia gameplay** (emette `EmitDamage` Commands). Implicazione: **la FSM deve girare anche headless**.

Regole di salvaguardia:

1. **Frame counter, no wall-clock.** `Hold/SpeedMul/Loop` sono espressi in frame logici (es. `Hold { extra_frames: 2 }`), non in millisecondi. Eccezione: `StartQTE { window_ms }` — ma window_ms è solo metadata UI; in headless si risolve via `headless_default_param` (§2.6).
2. **Cosmetic Commands sono no-op in headless.** `SpawnParticle`, `Shake`, `Flash` non producono `KernelEffect`. Il blueprint executor li droppa silenziosamente quando `cfg!(not(feature = "windowed"))` o quando il `PresentationBus` non è disponibile.
3. **QTE auto-resolve.** `StartQTE` in headless usa `params["qte_default_headless"]` come outcome (regola §2.6.D — riusa la macchina suspend/resume esistente, niente nuovo meccanismo).
4. **FSM driver headless = frame counter ticker.** Nessun "render loop", nessuna timeline. Il driver avanza il playhead di N frame per chiamata `execute()` finché non incontra una `Suspend` o `Done`. Vedi §H.

Coerente con CLAUDE.md "headless first": ogni system gira senza `windowed`. La FSM ricade dentro questa regola.

---

## H — Contratto interprete (anim_fsm_runtime.rs)

Una sola funzione pura + un driver con state:

```rust
// Pure: filtra edge non applicabili (Unlock missing), calcola priority order.
pub fn resolve_anim_graph(
    graph: &AnimGraph,
    unlocks: &UnlockedPassives,
) -> ResolvedGraph;

pub struct FsmRuntime {
    resolved: ResolvedGraph,
    current_node: NodeId,
    frame_in_node: u32,
    pending_commands: SmallVec<[Command; 4]>,
}

// Tick: avanza playhead di 1 frame, valuta edges, emette on_enter commands.
// Restituisce le commands generate in questo tick + eventuale Suspend.
pub fn tick_fsm(
    rt: &mut FsmRuntime,
    kernel_events_since_last_tick: &[CombatEvent],
    user_inputs: &[UserInput],
) -> FsmTickOutput {
    // 1. Avanza frame_in_node
    // 2. Valuta transition uscenti in priority order:
    //    a. UserInput-matching edges (alta priorità reattiva)
    //    b. KernelEvent-matching edges
    //    c. Unlock-matching edges (già pre-filtrate da resolved)
    //    d. TimeInNode (se frame_in_node ≥ node.frames)
    // 3. Su transition: emetti on_enter del nodo dest, resetta frame_in_node
    // 4. Se on_enter contiene StartQTE → ritorna Suspend
    // 5. Se nodo dest è Exit → ritorna Done
    FsmTickOutput { commands, transition_to: Some(node_id_or_exit), suspend: None }
}
```

**Integrazione `SkillBehavior::execute` (§2.7):**

```rust
impl SkillBehavior for PepperBreath {
    fn execute(&mut self, ctx: &mut SkillExecCtx) -> SkillStepOutcome {
        let out = tick_fsm(&mut self.fsm_rt,
                           ctx.kernel_events_since_resume(),
                           ctx.user_inputs_since_resume());

        for cmd in out.commands {
            self.translate_command(cmd, ctx);   // emette KernelEffect via ctx.emit
        }

        match (out.suspend, out.transition_to) {
            (Some(reason), _) => SkillStepOutcome::Suspend(reason),
            (None, Some(NodeRef::Exit)) => SkillStepOutcome::Done,
            (None, _) => SkillStepOutcome::Continue,   // più frame da consumare
        }
    }
}
```

**Punto cruciale:** `execute()` viene chiamato ripetutamente (frame loop in windowed; loop drain in headless). La FSM è la state machine privata `self.fsm_rt` — coerente con §2.7 "skill è state machine privata interna alla behavior". `tick_fsm` è puro rispetto a `(rt, events, inputs)`, replay-stable.

---

## I — Skill-tree: file dedicato `skill_tree.ron`

Lo skill-tree è il **gemello gameplay** di clipmontage: stessa logica condizionale, ma vive in un file separato perché agisce su numeri (params) e routing (kit_swap), non su animazione.

```ron
// assets/data/skill_tree.ron (nuovo, M018+ — schema riservato in M017)
{
    "agumon": SkillTree(
        nodes: {
            "triple_hit": Node(
                cost: 1,
                requires: [],
                patches: [
                    PatchParams(skill: "agumon_baby_flame",
                                params: { "hits": 3, "atk_mul": 0.7 }),
                ],
            ),
            "super_charge": Node(
                cost: 2,
                requires: ["triple_hit"],
                patches: [
                    PatchParams(skill: "agumon_baby_flame",
                                params: { "atk_mul": 2.4, "sp_cost": 2 }),
                ],
            ),
            "ember_path": Node(
                cost: 3,
                requires: ["super_charge"],
                // Branch path divergente: rimpiazza skill nel kit
                kit_swap: [Swap(slot: 0, to: "agumon_ember_breath")],
            ),
        },
    ),
}
```

**Resolver puro:**

```rust
pub fn resolve_skill_params(
    base: &SkillParams,
    unlocks: &UnlockedPassives,
    tree: &SkillTree,
) -> ResolvedParams;
```

Chiamato a **commit_action**, prima di `behavior.execute()`. Stesso snapshot-once della FSM (§F).

**Niente `params_overrides` in `skills.ron`** — `skills.ron` resta identità + params base, mai condizionali (§2.1 invariato).

**Out of scope M017:** lo schema è riservato nel catalog §2.5 (item #13 "skill_tree.ron") ma l'implementazione attende il primo skill-tree concreto. La FSM in M017 legge `UnlockedPassives` come resource opzionale: in M017 sarà vuota (zero unlock implementati), tutte le edge `Unlock(...)` semplicemente non matchano.

---

## J — Cost / Cooldown come effect (decisione adottata)

Concordato durante il design pass. Sostituzione del modello attuale "campo `sp_cost: 1` letto direttamente in legality":

**Prima (oggi):**
```ron
"agumon_baby_flame": SkillRon(
    params: { "sp_cost": 1, ... },
),
// kernel: if caster.sp < params["sp_cost"] → IllegalReason::NotEnoughSp
```

**Dopo (§2.2b):**
```ron
// skills.ron — il costo è un effect riferito per id
"agumon_baby_flame": SkillRon(
    cost_effect: "cost.sp_basic",   // riferimento all'effect catalog
    cooldown_effect: None,           // opzionale, default = nessun cd
    params: { "atk_mul": 1.6, "hits": 1, ... },
),

// assets/data/effects.ron (file nuovo o sezione esistente)
{
    "cost.sp_basic": CostEffect(kind: Sp, amount: 1),
    "cost.sp_heavy": CostEffect(kind: Sp, amount: 2),
    "cooldown.short": CooldownEffect(turns: 1),
    "cooldown.medium": CooldownEffect(turns: 2),
}
```

**Vantaggio.** Lo skill-tree può patchare il cost-effect come qualsiasi altro effect:

```ron
// skill_tree.ron
"free_first_cast": Node(
    patches: [
        PatchCostEffect(skill: "agumon_baby_flame",
                        cost_override: "cost.sp_zero_once_per_turn"),
    ],
),
```

Senza dover esporre `sp_cost` come param patchable individuale. Coerente con il pattern GAS `GameplayEffect` per Cost/Cooldown — adottato perché ortogonalizza meglio il modding via skill-tree.

**Out of scope M017:** schema riservato. Implementazione cooperata con `skill_tree.ron` quando il primo unlock cost-modifier emerge.

---

## K — GAS comparison (perché 80% più piccolo)

Sintesi delle scelte vs Unreal Gameplay Ability System:

| GAS feature | Nostro approccio | Decisione |
|---|---|---|
| GameplayAbility + AbilityTask (graph in BP) | AnimGraph RON + SkillBehavior trait | ✅ rubato (graph dichiarativo) |
| GameplayEffect | KernelEffect | ✅ già nostro (§2.7) |
| GameplayCue (cosmetic) | Commands cosmetic (`SpawnParticle`/`Shake`/`Flash`) | ✅ già nostro |
| Cost/Cooldown come Effect | `cost_effect`/`cooldown_effect` ref | ✅ rubato (§J) |
| GameplayTag (hierarchical, wildcards) | Flat status tags | ⏸ rimandato — nostro flat per ora |
| AttributeSet (pipeline modifier) | HP/SP diretti | ⏸ rimandato fino a complessità giustificata |
| Tag-based cancellation (`CancelAbilitiesWithTag`) | Cancel via edge `KernelEvent(StatusApplied { Stun })` | ⏸ formalizzazione tag-cancel rimandata |
| Granted abilities | `kit_swap` statico (in skill_tree.ron) | ⏸ pieno granted dinamico rimandato |
| Prediction + replication (networking) | — | ❌ non applicabile (single-player) |
| AbilityTask asincrono con callback | `Suspend(YieldReason)` (§2.6) | ✅ già nostro, più focused |
| GameplayEffectExecutionCalculation (formula custom) | Blueprint executor in Rust | ✅ più semplice da debuggare |
| Editor visuale | Validator + Graphviz dump | ⚠️ pareggio testuale, no UI editor |

**Cosa GAS fa meglio:** tooling (10 anni di UE5 + community), tag hierarchy, attribute pipeline.
**Cosa noi facciamo meglio:** RON testuale (AI-generabile, hot-reloadabile §2.5), determinismo headless (test reproducibility), boundary più stretti (vocabolario Commands chiuso).

---

## L — Validator requirements (essenziale: il grafo lo genera un AI agent)

Se i grafi vengono generati da un agent, servono validatori statici che falliscano al boot prima del runtime. Contract test in `tests/anim_fsm_validation.rs`:

| Check | Errore se… |
|---|---|
| Entry exists | `entry` non è nei `nodes` |
| Reachability | nodi non raggiungibili dall'entry (warning, non error — utili come dead branch) |
| Exit reachable | esiste almeno una sequenza entry → Exit nel grafo dei nodi senza unlock attivi |
| Dangling edges | edge con `from` o `to` che non corrispondono a nodi/Exit |
| Priority unique | due edge uscenti dallo stesso nodo con stessa priority che possono matchare contemporaneamente (verificato per matching combinations su unlocks possibili) |
| Frame range in-bounds | `Node.frames: (s, e)` con `s < e ≤ clip.total_frames` |
| Command params reference exist | `EmitDamage { mul_param: "atk_mul" }` — `atk_mul` deve esistere in `skills.ron::params` della skill proprietaria |
| StartQTE has headless_default | ogni `StartQTE` deve avere `headless_default_param` settato e valido (regola §2.6 estesa) |
| Cancel coverage | warning se nessun edge `KernelEvent(CasterIncapacitated)` esiste (cancel-tag rimandato, ma warning utile) |

**Senza questi check:** un agent produce un grafo plausibile-ma-rotto, te ne accorgi solo durante combat live → debug painful. Validator = boundary tra "agent ha prodotto" e "engine accetta".

---

## M — Esempio shape (Baby Flame senza unlock — minimale)

Sketch sintetico per orientamento — il **worked example full-featured** (con super_charge + triple_hit + counter_window + QTE amplify) è **deferred (post-M017)**: scritto quando il primo skill-tree concreto + QTE entry vengono implementati. La forma sotto + l'AnimGraph effettivo di Baby Flame in `agumon/02_skill_baby_flame.md` bastano per M017.

```ron
"agumon_baby_flame": AnimGraph(
    clip: "skill",   // ref a clip.ron range
    entry: "windup",

    nodes: {
        "windup":   Node(frames: (0, 12)),
        "charged":  Node(frames: (11, 12), modifier: Hold { extra_frames: 3 }),
        "release":  Node(frames: (12, 14)),
        "impact":   Node(frames: (14, 14),
                         on_enter: [EmitDamage { hits_param: "hits",
                                                  mul_param: "atk_mul",
                                                  status: "burn",
                                                  chance_param: "burn_chance_pct",
                                                  dur_param: "burn_duration" }]),
        "particles": Node(frames: (14, 17),
                          on_enter: [SpawnParticle { name: "fireball_explode",
                                                     origin: TargetCenter,
                                                     motion: Static }]),
        "recovery": Node(frames: (14, 17), reverse: true),
    },

    transitions: [
        Edge(from: "windup",    to: "charged",   when: TimeInNode),
        Edge(from: "charged",   to: "release",   when: TimeInNode),
        Edge(from: "release",   to: "impact",    when: TimeInNode),
        Edge(from: "impact",    to: "particles", when: TimeInNode),
        Edge(from: "particles", to: "recovery",  when: TimeInNode),
        Edge(from: "recovery",  to: Exit,        when: TimeInNode),
    ],
)
```

Note:
- L'unico nodo "gameplay" è `impact`: emette `EmitDamage`. Tutti gli altri sono cosmetic/timing.
- `SpawnParticle` su `particles` si attiva una volta sola (1 emit, 1 burst). Se la skill avesse `hits: 3` e volesse 3 particle uno per hit, il design corretto è **non** pinnare il particle a `particles` node ma usare un edge / hook:

```ron
// alternativa: particle pinnato all'evento kernel (1 particle per damage)
// in §2.7 reactive hook, NON in AnimGraph
on_event: |ev| if matches!(ev, CombatEvent::DamageDealt { source: this_action, .. }) {
    ctx.notify(NotifyParticle { name: "fireball_explode", origin: TargetCenter, motion: Static });
}
```

Il particle reattivo a un kernel event vive nel **listener attivo del blueprint** (dual-role §2.7 C2), non nella FSM. Questo mantiene il boundary "FSM sequenzia intent, blueprint reagisce a effetti applicati".

---

## N — Migrazione da clipmontage flat (§2.2)

La lista piatta attuale è il **degenerate case** del grafo: 1 nodo che copre l'intera clip, tutti i notify diventano `on_enter` di quel nodo (per i trigger) o `modifier:` (per Hold/SpeedMul/Loop).

**Esempio Tentomon (montage vuoto oggi):**

```ron
"tentomon_skill": AnimGraph(
    clip: "skill",
    entry: "all",
    nodes: { "all": Node(frames: (0, 999)) },
    transitions: [Edge(from: "all", to: Exit, when: TimeInNode)],
)
```

**Per i 5 Digimon con clipmontage vuoto:** trasformazione meccanica, 1 file per Digimon, niente perdita di info. Per Agumon (Baby Flame con clipmontage popolato) la migrazione è la conversione esplicita degli 8 notify attuali in nodi+edges (vedi §M sketch).

Tooling proposto: script `tools/migrate_clipmontage_to_fsm.py` che converte file-per-file. Decisione su quando eseguirlo (M017 vs M018) lascia al planning.

---

## O — Cosa NON entra in M017 (scope esplicito)

| Item | Stato | Motivazione |
|---|---|---|
| Cancel-tag pattern (GAS-style `cancel_tags: ["status.stun.*"]`) | ⏸ rimandato | Implementabile come edge implicito `* → Cancel when KernelEvent(StatusApplied { Stun })` quando il primo caso emerge. Per ora il blueprint può gestirlo manualmente |
| Granted abilities dinamico (status-gated, form-gated) | ⏸ rimandato | Skill-tree statico (`kit_swap`) basta per ora. Dinamico arriverà con Digivolution / equip system |
| `skill_tree.ron` resolver implementato | ⏸ rimandato | Schema riservato, FSM legge `UnlockedPassives` vuota in M017. Implementazione con primo unlock concreto |
| Cost/Cooldown effect catalog implementato | ⏸ rimandato | Schema riservato (§J), implementato quando il primo skill-tree modifica cost |
| Tag hierarchy (GAS-style) | ❌ fuori scope | Status flat per ora, hierarchy aggiunta solo se ricorrente |
| Attribute pipeline (pre/post modify) | ❌ fuori scope | HP/SP diretti, pipeline solo se complessità giustificata |
| Editor visuale FSM | ❌ fuori scope | RON-first + Graphviz dump per debug |

---

## P — Slice impact

Impatto su §5 slicing (delta vs §2.2 flat):

- **S03b** "SkillBehavior trait + registry + reactive hook dispatcher" — invariato.
- **NUOVA: S03f "AnimGraph FSM parser + interprete + validator"** — schema RON, parser, validator contract test, `tick_fsm` puro, golden test `(graph, unlocks, events) → commands`. Headless-only, niente UI.
- **NUOVA: S03g "AnimGraph integration con SkillBehavior"** — `SkillExecCtx::kernel_events_since_resume()`, `Command::translate_into_kernel_effect()`, blueprint executor per il vocabolario base (6 verbi).
- **S03c** "skill RON v2 + behavior porting" — aggiornato: Baby Flame usa AnimGraph come reference. Le altre 5 Rookie skills hanno AnimGraph degenerate (1 nodo all-clip) finché non emergono complessità.

Migration script (§N) è tooling, non slice gameplay.

---

## Q — Open question (decisione successiva)

1. **Verbose short-form?** Per le skill banali (1 nodo all-clip) lo schema è verbose. Se diventa pain point dopo migrazione, valutare short-form RON che si espande nel grafo a load-time. Per ora paghiamo verbosity per uniformità.
2. **Frame-range overlap.** Cosa succede se due nodi referenziano frame range overlapping (es. `charged: (11,12)` e `release: (11,14)` per super_charge variant)? Decisione: ammesso, è il caso d'uso di override. Il playhead salta a frame del nodo destinazione su transition, anche se "indietro" rispetto al precedente. Documentare in §H l'algoritmo di salto playhead.
3. **`reverse: true` su Node** (es. recovery che riproduce frame al contrario). Notazione attuale è speculativa — valutare se serve come campo Node oppure se basta listare gli indici esplicitamente in un nuovo `frames_explicit: [14,13,12,11]` field. Decidere quando il primo Digimon non-Agumon richiede recovery animation.

---

## R — Kernel events catalog (`§R-Events`)

> **Naming note.** Originariamente programmato come `§G-Events`, rinominato a `§R-Events` per evitare collisione con `§G — Headless determinism`. Refs esterne (`renamon/04_passive_kitsune_grace.md`, `tentomon/04_passive_battery_loop.md`) usano già il nuovo nome.

Estensione del bus `CombatEvent` (definito in `src/combat/events.rs`) con gli eventi richiesti dalle passive listener del roster M017. Sono **kernel events** emessi dalla pipeline gameplay, ascoltabili da `BlueprintListener::on_kernel_event` (§2.7 C2) e da edge FSM `KernelEvent(...)` (§D).

| Event | Payload | Emesso da | Consumato da (M017) |
|---|---|---|---|
| `UltimateUsed` | `{ actor: EntityId }` | Kernel, dopo `commit_action(Ult)` quando l'Ult consuma la bar (post-`Strike.on_enter`, pre-cleanup) — cancellazioni mid-resolve non lo emettono | Renamon `kitsune_grace` (filter `is_ally && !is_self`) |
| `BlockReactionTriggered` | `{ defender: EntityId, attacker: EntityId, mitigated_pct: u8 }` | Kernel, durante `IncomingDamage` cascade pre-step quando defender è in `BlockReady` state e mitigation applica | Tentomon `battery_loop` (FSM `BlockReady → BlockProc` edge), side-channel SP-grant listener |
| `DamageDealt` | `{ source: EntityId, target: EntityId, amount: u32, kind: DamageKind }` | Kernel, post-resolution di ogni `KernelEffect::Damage` | Listener reattivi (dual-role §2.7 C2), esempi: Tentomon `OnAnyAttack` ult-charge gen |
| `StatusApplied` | `{ target: EntityId, status: StatusId, source: Option<EntityId> }` | Kernel, post-`KernelEffect::ApplyStatus` | Edge `KernelEvent(StatusApplied { kind: "Stun" })` per cancel-pattern §D |
| `Healed` | `{ target: EntityId, amount: u32, source: EntityId }` | Kernel, post-`KernelEffect::Heal` | Patamon ult charge `+25/heal event` |
| `SpGranted` | `{ recipient_team: TeamId, amount: u8 }` | Kernel, post-`KernelEffect::GrantSp` (cap-aware) | Gabumon/Tentomon ult team-grant chain |
| `IncomingDamage` *(pre-step)* | `{ attacker: EntityId, defender: EntityId, raw_amount: u32, kind: DamageKind }` | Kernel, **prima** del damage cascade (B4 gap §2.8) — finestra per block/parry reactive | Tentomon `battery_loop` block reaction, future parry mechanics |

**Regole di emissione:**

1. **Post-effect, not pre-effect (eccetto `IncomingDamage`).** Gli eventi sono emessi dopo che il `KernelEffect` ha mutato state, così i listener vedono lo stato consistente. `IncomingDamage` è l'unica eccezione: emessa pre-step per permettere mitigation reattiva nel cascade.
2. **Niente cancellazioni triggerate.** Eventi cancellati a metà cascade (es. `UltimateUsed` su target morto durante windup) non vengono emessi. Listener vedono solo eventi "successful".
3. **Ordering.** Eventi nello stesso tick di cascade sono ordinati per emit-order del kernel; listener vedono la sequenza completa via `kernel_events_since_last_tick` (§H tick contract).

**Espansioni candidate (post-M017):** `LowHpEntered`, `BreakBarOpened`, `RoundStarted`, `RoundEnded`, `EntityDied`. Aggiungere quando la prima skill concreta li richiede.

---

## S — Param reference resolution (`§S-Param`)

> **Naming note.** Originariamente programmato come `§G-Param`, rinominato a `§S-Param` per evitare collisione con `§G — Headless determinism`. Refs esterne (`02-02d §B`, `02-02d §I`) usano già il nuovo nome.

`ParamRef` è il **vocabolario chiuso** per la risoluzione di numeri/identifier nei Commands. Estende la regola §C "numeri via reference, non literal" — il blueprint executor risolve `ParamRef` → valore concreto a `commit_action` time (snapshot-once §F) o on-the-fly per i `BlueprintState` reads.

```rust
pub enum ParamRef {
    /// Lettura statica da `skills.ron::params[key]` della skill proprietaria.
    /// Patchabile da skill_tree.ron (§I). Snapshot-once a commit_action.
    Static(String),

    /// Lettura snapshot-once da uno store ausiliario captured a commit_action
    /// (es. target snapshot per skill multi-hit deterministiche).
    Snapshot(String),

    /// Lettura live da `BlueprintState[key]` del listener proprietario.
    /// NON snapshottato — il valore può cambiare tra tick FSM successivi.
    /// Esempi: Dorumon `predator_loop` `tracked_target`, Twin Core
    /// `partner_status_flag`, Tentomon battery `last_block_attacker`.
    BlueprintState(String),

    /// Costante numerica letterale. Ammessa solo per soglie strutturali
    /// (es. `cap: 50`), MAI per scaling values (vanno in skills.ron).
    Literal(i32),
}
```

**Resolution contract:**

| ParamRef | When read | Lifetime | Patchable da skill_tree? |
|---|---|---|---|
| `Static(k)` | a `commit_action` | snapshot frozen per la skill | ✅ via `PatchParams(skill, params)` |
| `Snapshot(k)` | a `commit_action` (capture phase) | snapshot frozen per la skill | ❌ — runtime capture, non patchable |
| `BlueprintState(k)` | ogni read durante `tick_fsm` | live, mutabile mid-skill | ❌ — listener-owned, fuori scope skill_tree |
| `Literal(n)` | parse-time | invariante | ❌ |

**Failure modes:**

- `Static(k)` con `k` non in `params`: validator §L lo rifiuta a load-time.
- `Snapshot(k)` con `k` non capturato: runtime warning + skip Command (no-op).
- `BlueprintState(k)` con `k` non in schema blueprint: validator warning (vedi §L extension `EntityRef::FromBlueprintState`).

**Relazione con `EntityRef` (`02-02d §B`):** `EntityRef::FromBlueprintState(k)` è il counterpart entity-typed di `ParamRef::BlueprintState(k)` — stesso backing store, stesso failure mode (silent drop).

**Espansioni candidate (post-M017):** `ParamRef::Computed(formula)` per scaling derivati (e.g. `atk_mul * level`) — rifiutato per ora (rompe determinismo + validator). Promuovere via kernel event se necessario (es. `OnLevelUp` ricalcola lo `Static`).

---

## Riferimenti

- §2.1 (data/logic separation): `skills.ron` invariato, `signal_bindings.ron` invariato
- §2.2 (animation manifest flat): superseded dal grafo per `clipmontage.ron`, `clip.ron` invariato
- §2.5 (tunable catalog): `skill_tree.ron` aggiunto come item #13, `effects.ron` (cost/cooldown) aggiunto come item #14
- §2.6 (suspend/resume): `StartQTE` Command usa il meccanismo esistente, nessuna estensione
- §2.7 (SkillBehavior trait): la FSM vive `self.fsm_rt` nel behavior, `execute()` la tick-a
- §2.8 (effect cascade): invariata, i `KernelEffect` emessi dal blueprint executor entrano nella cascade standard
- §2.9 (worked example full-featured Baby Flame: 4 unlock variant + counter reattivo + QTE amplify): **deferred post-M017** — autorizzato col primo skill-tree concreto + QTE entry. Per ora `agumon/02_skill_baby_flame.md` (forma base) e §M sketch coprono lo scope
