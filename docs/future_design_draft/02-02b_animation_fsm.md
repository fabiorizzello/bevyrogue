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
| `AdvanceTurn { actor, pct_param }` | gameplay | avanza gauge `TurnOrder` di `actor` di `pct%` del valore turn full | `KernelEffect::AdvanceTurnGauge` (kernel-owned `TurnOrder`) | Renamon `koyosetsu`, `kitsune_grace` |
| `DelayTurn { target, pct_param }` | gameplay | ritarda gauge `TurnOrder` di `target` di `pct%` | `KernelEffect::DelayTurnGauge` | Renamon `tohakken` |
| `ApplyBuff { id, dur_param, kind: (Buff\|Debuff\|DR\|Aura\|Mark), target }` | gameplay | unifica `EmitStatus` con flag `kind` esplicita; status registry applica regole cleanse-eligibility per `kind:Buff` ∉ cleansable (vedi §2.8 §H) | `KernelEffect::ApplyStatus` con `BuffKind` field | Renamon `tohakken` (Blessed), Patamon `holy_aegis` (Aura), Gabumon `fur_cloak` (DR) |
| `EmitSpGrant { amount_param, target }` | gameplay | aggiunge SP a SP pool del team di `target`; emette `SpGranted` event; **cap-aware sul lato ricevente** (`SpPool.add` clamp al cap, vedi `src/combat/sp.rs`), **non passa** dal contatore `RoundSpTracker.max_non_basic_per_round` (grant ≠ spend) | `KernelEffect::GrantSp` | Gabumon `blue_cyclone` (ult, team +1), Tentomon `electrical_discharge` (ult, team +1), override `+2 SP` Tentomon basic data-side via `units.ron.sp_gen_per_basic` |
| `Reposition { anchor, target }` | gameplay | sposta `target` a posizione `anchor` sulla combat line (riusa §02-02c §D dematerialize pattern, ora promosso) | `KernelEffect::Reposition` | Dorumon `dash_metal` follow-up (chiude §02-02c §D dangling) |

**Sugar form, NO command separato:** `ApplySelfBuff { id, dur_param, kind }` è alias di `ApplyBuff { target: EntityRef::Self_ }`. Vocabolario kernel resta minimal — il blueprint può scrivere la forma corta come zucchero sintattico nel proprio executor, ma `tick_fsm` emette solo `ApplyBuff`. Stesso pattern di `TargetCenter ↔ EntityCenter(Primary)` in §2.2d §B.

**Relazione `EmitStatus` ↔ `ApplyBuff`:** `ApplyBuff` è **strict superset** di `EmitStatus` (§C base) — aggiunge il campo `kind`. M017+ usa `ApplyBuff` ovunque; `EmitStatus` resta nel vocabolario §C base come **forma degenerate** (`kind` defaulta a `Debuff`) per compat lazy delle skill già scritte. Nessuna doppia logica kernel-side: `EmitStatus { ... }` → `ApplyBuff { ..., kind: Debuff }` a parse-time del RON.

**Tutti i 7 verbi sono kernel-known per design.** Test discriminante: il Command tocca state condiviso oltre la singola entity blueprint (TurnOrder resource, status registry con cleanse-eligibility, SP pool, hp altrui). Tenerli come custom blueprint code drifterebbe la semantica tra digimon (es. `heal` Patamon vs `heal` futuro Vaccine-healer si scollerebbe; cleanse FIFO/LIFO/Random si bifurcherebbe). Il prezzo "tocchi kernel per aggiungere verbo" si paga **una volta in M017**, poi è costante.

**Blueprint-local resta:** trigger predicates e state reads dentro Forma C FSM passive (es. Dorumon `predator_loop` `hp_pct < 0.5`, Agumon `twin_core_fire` `partner.has_status(Chilled)`). Blueprint legge stato → emette Commands kernel-known sugli eventi. Boundary chiara: **blueprint sa leggere, kernel sa scrivere.**

**Espansioni candidate aggiornate (post-C2, non in M017):** `KnockBack`, `Move`, `Teleport`, `SummonMinion`, `Mark` restano candidati come §C base; rivalutazione round-4+.

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

Sketch sintetico per orientamento — il **worked example full-featured** (con super_charge + triple_hit + counter_window + QTE amplify) è in §2.9.

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

## Riferimenti

- §2.1 (data/logic separation): `skills.ron` invariato, `signal_bindings.ron` invariato
- §2.2 (animation manifest flat): superseded dal grafo per `clipmontage.ron`, `clip.ron` invariato
- §2.5 (tunable catalog): `skill_tree.ron` aggiunto come item #13, `effects.ron` (cost/cooldown) aggiunto come item #14
- §2.6 (suspend/resume): `StartQTE` Command usa il meccanismo esistente, nessuna estensione
- §2.7 (SkillBehavior trait): la FSM vive `self.fsm_rt` nel behavior, `execute()` la tick-a
- §2.8 (effect cascade): invariata, i `KernelEffect` emessi dal blueprint executor entrano nella cascade standard
- §2.9 (worked example): sostituito con full-featured AnimGraph di Baby Flame (4 unlock variant + counter reattivo + QTE amplify)
