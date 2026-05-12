# §7 — Definition of done M017

**Plugin extension (§2.3):**
- [ ] Kernel `combat/kernel.rs` non contiene più `BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|TwinCore` Transition (verificato da grep)
- [ ] `combat/kernel.rs` espone un solo extension point: `KernelEffect` enum
- [ ] Ogni Digimon ha la sua state machine sotto `combat/blueprints/<owner>/` e `impl Plugin`
- [ ] `BlueprintsPlugin` in `blueprints/mod.rs` enumera i 6 plugin in **una sola** `add_plugins((...))` — aggiungere/rimuovere un Digimon = directory + 1 riga (grep su `kernel.rs`, `main.rs` mostra zero menzione di nomi Digimon specifici)
- [ ] `predator_loop.rs` e `precision_mind_game.rs` sono spariti da `src/combat/` root e vivono dentro `blueprints/dorumon/` e `blueprints/renamon/` rispettivamente

**Tunable data in RON (§2.5):**
- [ ] `assets/data/unit_stats.ron` esiste, contiene base stats + `GrowthCurve` per i 6 Rookie; `units.ron` ridotto a sola identità (no campi hp/atk/def/spd)
- [ ] `assets/data/signal_bindings.ron` esiste e sostituisce il campo `custom_signals` in `skills.ron` (skills.ron resta solo numeri)
- [ ] `assets/data/encounter_balance.ron` esiste con `WildNerfCurve` + `LevelTrack`; `bootstrap.rs` non contiene numeri hardcoded di nerf
- [ ] `assets/data/status_effects.ron` esiste e parametrizza `Heated`, `Wet`, `Stun` (e altri usati); `status_effect.rs` legge dal RON, non da costanti
- [ ] `assets/data/run_config.ron` esiste e parametrizza encounter count, carryover %, fail condition
- [ ] Tutti i nuovi RON hanno campo `version: 1` come prima riga
- [ ] Hot-reload verificato per `clipmontage.ron`, `encounter_balance.ron`, `skills.ron`, `status_effects.ron` (asset change → effetto visibile al prossimo trigger senza restart)
- [ ] Tutti i nuovi type RON derivano `Reflect` (preparazione editor in M019+)

**Animation manifest (§2.2 + AnimGraph FSM §2.2b):**
- [ ] `assets/digimon/<name>/clip.ron` e `clipmontage.ron` esistono per tutti e 6
- [ ] Agumon ha `clipmontage.ron` come **AnimGraph FSM** popolato (nodi + edges + Commands `on_enter`), reference design `baby_flame` con almeno 2 variant ortogonali da `skill_tree.ron` predicate (anche se `UnlockedPassives` è vuota a runtime M017 — i predicate sono parsati e validati)
- [ ] Gli altri 5 Digimon hanno `clipmontage.ron` come AnimGraph **degenerate** (1 nodo all-clip + Edge → Exit), forma valida per migrazione (§2.2b §N)
- [ ] I `.json` atlas sono rimossi
- [ ] Il caricatore emette `CombatEvent::AnimationNotify` quando un `SpawnParticle`/`Shake` Command è dispatcched (testato headless mockando il playhead)

**AnimGraph FSM runtime (§2.2b — S03f + S03g):**
- [ ] Schema `AnimGraph` (nodi, edges, Commands, Predicate) parsato da `clipmontage.ron`
- [ ] `tick_fsm(rt, kernel_events, user_inputs) -> FsmTickOutput` è puro (no `&mut World`, no RNG global, no wall-clock)
- [ ] Validator contract test `tests/anim_fsm_validation.rs` enforce: entry exists, exit reachable senza unlock, frame range in-bounds, `*_param` esistono in `skills.ron`, priority unique tra edge che possono matchare contemporaneamente
- [ ] Vocabolario Commands chiuso a 6 verbi (EmitDamage, EmitStatus, SpawnParticle, Shake, Hold, StartQTE) — nessuna `Command::Custom(...)`
- [ ] Golden test deterministico `tests/anim_fsm_baby_flame.rs`: dato `(graph, unlocks, kernel_events_sequence)` lo `commands_sequence` emesso è bit-identico across run
- [ ] `UnlockedPassives` resource opzionale (vuota in M017); edge `Predicate::Unlock(...)` parsate ma mai matchate
- [ ] Headless determinism: cosmetic Commands (SpawnParticle/Shake) sono no-op in `#[cfg(not(feature = "windowed"))]` o quando `PresentationBus` assente; `StartQTE` auto-resolve via `headless_default_param`

**Run-loop (§3):**
- [ ] `cargo run --bin combat_cli` esegue una run di 5 encounter end-to-end con AI scriptata default
- [ ] HP carryover parziale (50% missing healed) verificato
- [ ] SP reset a 3, ult charge persiste verificato
- [ ] KO revivable solo via skill in-combat: `patamon_revive` mappa su `KernelEffect::Revive { target, hp_pct: 25 }` e funziona
- [ ] Nessun auto-revive post-combat: Digimon KO al termine dell'encounter restano KO al prossimo
- [ ] Run fail solo con 4/4 KO simultaneo in singolo encounter verificato

**Enemy roster (§4):**
- [ ] `WildPack(Vec<UnitId>)` accetta 1-4 elementi, validazione bootstrap rifiuta 0 e 5+
- [ ] `WildVariant` modifier applica nerf via `nerf_for(pack_size)` senza duplicare unit in `units.ron`
- [ ] Encounter 1-4 usano `WildPack`, 4-5 usano boss hand-crafted (encounter 4 può essere mix)

**Skill-as-Plugin (§2.7):**
- [ ] Trait `SkillBehavior` esiste con metodi `preview` / `legality` / `execute` / `manifest`
- [ ] `SkillExecCtx` non espone `&mut World` né RNG globale; solo `ctx.emit(KernelEffect)` come canale di mutazione
- [ ] Skill registry boot-validato: ogni `skills.ron` entry ha behavior corrispondente registrato
- [ ] `DeclarativeSkill` helper esiste e copre il 70-80% delle skill esistenti
- [ ] 2-3 skill custom Rust-native (es. Tentomon `battery_loop` trigger) esistono per dimostrare extension-first
- [ ] Contract test golden: `preview.targets ⊇ execute_emitted_targets` per ogni skill su seed fisso
- [ ] Contract test: ogni skill dichiara `headless_default_for_yield` se può sospendere

**Kernel suspend/resume (§2.6):**
- [ ] `CombatPhase::AwaitingSkillYield { cursor_id, reason }` esiste
- [ ] Systems `advance_turn` / `enemy_ai` / `status_effect_tick` / `follow_up` sono gated su `CombatPhase != AwaitingSkillYield`
- [ ] `YieldReason` ha 5 varianti: `QuickTimeEvent`, `BlueprintHook`, `AnimationGate`, `PlayerChoice`, `CascadeComplete`
- [ ] Headless default funzionante per ognuna (no test bloccato da yield)
- [ ] `cursor_id` seedato su `(combat_seed, turn_idx, skill_id, action_uid)` → replay stabile

**Cascade & loop prevention (§2.8):**
- [ ] `KernelEffectQueue` come Resource, drain FIFO; ogni effect applicato emette `CombatEvent` corrispondente
- [ ] Reaction registry: ogni blueprint plugin dichiara `ReactionManifest { id, listens_to, cause_filter, idempotency, emits }`
- [ ] `CauseChain { id, kind, parent, depth }` propagato automaticamente dal kernel su ogni `KernelEffect`
- [ ] Contract test al boot: ogni reaction manifest deve avere `cause_filter` esplicito o fail boot
- [ ] Contract test fixture: ogni reaction non innesca loop infiniti su fixture sintetico (cycle < 50 iter → fail CI)
- [ ] `IdempotencyScope` enum chiuso a 4 varianti (`OncePerAction` / `OncePerCause` / `OncePerTarget` / `EveryTime`); kernel traccia `(reaction_id, scope_key)` per drain run
- [ ] Cap diagnostico debug-only (1024 effetti, log-only) attivo in `#[cfg(debug_assertions)]`; release non ha cap
- [ ] `FollowUpIntent` rimosso da `combat/follow_up.rs`; sostituito da `KernelEffect::EnqueueAction` emesso via cascade; test M015 follow-up passano

**Generale:**
- [ ] `tests/run_loop.rs` passa
- [ ] `cargo test` verde su tutti i file integrazione (post-refactor)
- [ ] Test di parità RON: pre/post migrazione dati il combat headless produce gli stessi `CombatEvent` su seed fisso (regression gate per S03/S05/S06)
- [ ] Test di parità follow-up: pre/post S03e il combat headless produce gli stessi `CombatEvent` su seed fisso per gli scenari M015 (regression gate)
- [ ] `docs/combat_current.md` aggiornato con nuovo modello plugin + run-loop + RON catalog + skill-as-plugin + cascade
- [ ] `K###` in `.gsd/KNOWLEDGE.md` documenta: (a) pattern "aggiungi Digimon = `blueprints/<name>/` + 1 riga in `BlueprintsPlugin`", (b) regola "tutti i numeri tunabili stanno in RON, Rust contiene solo regole", (c) pattern "skill = plugin con `SkillBehavior` trait, kernel non ha catalogo di gameplay primitives", (d) regola "reaction deve dichiarare `cause_filter` + `IdempotencyScope` o non compila"
