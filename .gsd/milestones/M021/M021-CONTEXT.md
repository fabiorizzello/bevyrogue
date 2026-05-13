# M021 — Skill trait + SkillCtx + Blueprint trait + Plugin self-registration

## Obiettivo

Consolidare l'extension pattern del kernel su due fronti che ora viaggiano insieme:

1. **Skill API**: passare da enum `Effect` data-driven (v0, M017→M020) a `trait Skill::resolve(&mut SkillCtx, &Params)` in Rust, con split netto query (read-only) / enqueue `Intent` (write-deferred). Vedi **D010**.
2. **Blueprint API + plugin split**: formalizzare `trait Blueprint` + `BlueprintRegistry` + `CombatPlugin` separation (scope storico M021 da portfolio). Vedi **D007 + D008**.

I due refactor condividono lo stesso modulo target (`src/combat/blueprints/`, `src/combat/resolution.rs`, kernel surface) e lo stesso vincolo (kernel unico esecutore, P001). Tenerli separati in milestone diversi raddoppierebbe il churn sul kernel surface — meglio un unico milestone di refactor che consegna entrambi i contract stabili pre-roster.

## Scope

### Fascia A — Skill trait + SkillCtx (nuovo, post-M018)

- **A1. Design `SkillCtx`**: API read-only query (`predict_damage`, `adjacents`, `can_target`, `next_adjacent_alive`, `sp_available`, `alive_enemies`, `unit_state`, `peek_pending`, …) + API write-deferred enqueue (`ctx.enqueue(Intent::…)`). Single source di verità per *cosa* una skill può chiedere e produrre.
- **A2. Design `Intent` enum**: varianti minime canon (`DealDamage`, `ApplyStatus`, `FollowUp`, `AdvanceTurn`, `DelayTurn`, `Heal`, `Cleanse`, …). Lista chiusa, crescibile.
- **A3. `trait Skill`**: `fn id(&self) -> SkillId; fn resolve(&self, ctx: &mut SkillCtx, params: &Params);` + `SkillRegistry` resource Bevy.
- **A4. Kernel `Intent` resolver**: pipeline che drena la coda `Intent` post-resolve della skill, applica via il damage/status/turn-order esistente. Niente duplicazione formula.
- **A5. Migrate skill esistenti**: Bounce (S03 M018, già selector-tipizzato), Blast, AoE, Single, Heal, Cleanse, status applies → tutte sotto `trait Skill`. Drop di `enum Effect` quando vuoto.
- **A6. RON ridotto**: `skills.ron` tiene solo `id`, numeri (dmg, hops, sp_cost, scaling), `target_shape` base, tag. Niente logica. `units.ron` invariato (è già numeri).
- **A7. Test pattern**: ogni skill = test `assert_eq!(ctx.drain_intents(), expected)`. Test integration esistenti restano verdi (re-cablati sui nuovi tipi).

### Fascia B — Blueprint trait + plugin split (scope storico portfolio)

- **B1.** Estrarre `CombatPlugin` da `register_combat_kernel_runtime` (refactor `main.rs` + `headless.rs` + `windowed.rs` a composizione plugin). Zero cambio di logica. Vedi D008.
- **B2.** `trait Blueprint` + `BlueprintRegistry` resource + dispatcher generico in `src/combat/blueprints/api.rs`. Vedi D007.
- **B3.** Migrate Agumon plugin al nuovo trait + self-registration (shim per gli altri 5).
- **B4.** Migrate Gabumon (paired Twin Core).
- **B5.** Migrate Dorumon + Tentomon.
- **B6.** Migrate Patamon + Renamon. Rimozione shim. `CombatKernelTransition` Digimon-specific eliminato. Migration delle 5 famiglie enum (`TwinCoreSignal`, `BatteryLoopTransition`, `HolyAegisTransition`, `KitsuneGraceTransition`, `PredatorLoopState`) dentro `kernel.rs`.
- **B7.** Extension-friendly `RosterEntry`: rimuovere field hard-coded Digimon-specific (`twin_core`, `holy_support`, …) a favore di blueprint-keyed payload generico.
- **B8.** `ValidationSnapshot` field nominati per blueprint key, popolata dal registry.

### Ordine slice

Fascia B1–B2 prima (plugin split + Blueprint trait) — abilita injection pulita di `SkillRegistry`. Poi A1–A6 (Skill trait + Intent + migrate skills) — ortogonale ai blueprint, gira sul kernel ripulito. Poi B3–B6 a coppie (blueprint migration sfrutta `SkillCtx`/`Intent` per accodare follow-up senza toccare il kernel). Chiusura con B7–B8 (cleanup roster/snapshot).

## Vincoli

- **P001**: kernel resta unico esecutore. Skill produce `Intent`, kernel risolve. Blueprint produce signals/follow-up via `ctx.enqueue`, mai mutazione diretta.
- **D008**: `CombatPlugin` non importa `bevy::winit`, `bevy::render`, `bevy_egui`. `cargo check` (no feature) verifica il confinamento.
- **Determinismo**: tests headless restano deterministici. `Intent` order definito da resolve order; nessuna dipendenza da `HashMap` iteration.
- **Test esistenti verdi a ogni slice.** Migration incrementale, non big-bang. Le skill già migrate convivono con quelle ancora su `Effect` durante la transizione (registry doppio temporaneo, rimosso a fine A5).
- **No data-DSL Turing-completo**, no scripting embedded (Rhai/Rune): D010.

## Demo

- 6 plugin auto-registrati, dispatcher Blueprint generico, `CombatKernelTransition` Digimon-specific rimosso.
- Tutte le skill canon (Bounce, Blast, AoE, Heal, Cleanse, status apply) eseguite via `trait Skill` su `SkillCtx`; `enum Effect` rimosso.
- `skills.ron` contiene solo numeri/tag.
- Test integration suite verde end-to-end senza modifiche di shape pubbliche oltre tipo skill.

## Riferimenti

- **D007** — Blueprint API: trait Blueprint + BlueprintRegistry + plugin self-registration.
- **D008** — `CombatPlugin` separation headless/windowed.
- **D010** — Skill API: `trait Skill` + `SkillCtx` (query/enqueue split). Pre-M021.
- **K-P001** — Kernel generico, specifiche fuori dal kernel (aggiornato con `SkillCtx`/`Intent` rule).
- **SP2 INTERFACE-OPTIONS** — spike Blueprint API.
- **SP-skill-dsl-coverage** — `.gsd/spikes/spike-skill-dsl-coverage/` — 24/24 skill canon Effect-expressible; conferma curva pattern chiusa.
- **M020 §entry portfolio** — rimanda esplicitamente le 5 famiglie enum Digimon-specific alla migration di M021 S05–S06 (qui rinumerate B5–B6).

## Dipendenze

- **Richiede chiuso**: M018 (foundation primitive shape: Blast/AoE/Bounce + selectors), M019 (DR pipeline), M020 (reactive bus uniforme + shim removal kernel-side).
- **Sblocca**: M022 (asset pipeline, ortogonale ma può partire in parallelo), M023+ (visual stack), M024–M028 (roster identity migration — nascono direttamente sopra `trait Skill` + `trait Blueprint`, niente debito).

## Non-scope (deferred)

- Escape hatch modding via scripting embedded → post-1.0.
- Stack-aware status numerici (Heated × N DoT scaling) → resta differito (D009).
- Hot-reload skill logic → non richiesto (vincolo utente esplicito: logica in Rust).
