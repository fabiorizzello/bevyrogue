# S01: Kernel framework primitives + CombatPlugin extract

**Goal:** Estrarre `CombatPlugin` come Plugin Bevy dedicato senza dipendenze winit/render/egui, e introdurre `src/combat/api/` con le primitive del framework: `Intent` enum + `intent_applier`, `SkillCtx<'a>` (read-only + enqueue), `ExtPoint`+`Registry<E>`+`ExtRegistries` Resource (7 assi), `SignalBus` Resource, `Clock { HeadlessAuto, Windowed }`, RNG seeded SplitMix64. `CastId(NonZeroU32)` aggiunto a `CombatEvent` e propagato da `pipeline::step_app` a tutti i call-site emit. Fine slice: i tipi esistono, sono Bevy-agnostic dove possibile, il plugin si registra senza importare winit/wgpu/egui, le 74+ test integration restano verdi, una variant Intent (DealDamage) è wired end-to-end al damage system esistente come canary. Le altre variant Intent espongono il dispatch ma delegano alle implementazioni esistenti (no migration ancora — quella è S05+). Demo: cargo check headless + windowed puliti; CombatPlugin in main.rs; src/combat/api/ con i 7 file primitive; cast_id su CombatEvent; canary Intent::DealDamage end-to-end via intent_applier emette CombatEvent con cast_id corretto.
**Demo:** cargo check headless + windowed puliti; CombatPlugin in main.rs; src/combat/api/ con i 7 file primitive; cast_id su CombatEvent; canary Intent::DealDamage end-to-end via intent_applier emette CombatEvent con cast_id corretto.

## Must-Haves

- `cargo check` headless + `cargo check --features windowed` puliti, niente warning nuovi
- `cargo test` 74+ test integration verdi end-to-end
- `rg "use bevy::winit|use bevy::render|use bevy_egui" src/combat/` → 0 righe (eccetto subdir blueprints/)
- Modulo `src/combat/api/` esiste con: `intent.rs` (`Intent` enum chiuso ~18 variant + `CastId(NonZeroU32)`), `registry.rs` (`ExtPoint` trait + `Registry<E>` + `ExtRegistries` Resource), `signal.rs` (`SignalBus` Resource scaffold), `clock.rs` (`Clock` enum), `rng.rs` (SplitMix64 deterministico), `skill_ctx.rs` (`SkillCtx<'a>` + `SkillCtxMode`), `applier.rs` (`intent_applier` FIFO drain)
- `CombatPlugin: Plugin` re-exported da `src/lib.rs`; `src/main.rs` lo usa al posto di `register_combat_kernel_runtime` diretto
- `CombatEvent` contiene `cast_id: CastId` e tutti i call-site di emit lo settano coerentemente
- DealDamage canary: un test emette `Intent::DealDamage` via `ctx.enqueue` e osserva il danno applicato; verifica `CastId` propagato sull'evento
- Unit test `Registry<E>` lookup (hit, miss); unit test `Rng::splitmix64` determinismo per stesso (cast_id, beat, hop, salt)
- I tipi pubblici in `src/combat/api/` non importano `bevy::winit`/`bevy::render`/`bevy_egui` (grep)

## Proof Level

- This slice proves: Contract + integration. Contract: ogni primitive (Intent variants visibility, Registry lookup, RNG determinism, Clock enum, SignalBus scaffold) coperto da unit test brevi. Integration: la suite 74+ esistente regge perché il dispatcher delega alle code path attuali; canary DealDamage prova il path Intent→applier→damage system→CombatEvent con cast_id. Verifica statica via grep gates su import vietati e su presenza file/symbol attesi.

## Integration Closure

Tutti i call-site di `CombatEvent` emit (~50) aggiornati per accettare/propagare `cast_id`. JSONL logger e observability parsano CombatEvent senza modifiche schema-breaking — `cast_id` aggiunto come campo nuovo con default `CastId::ROOT` (NonZeroU32::new(1).unwrap()) per call-site pre-cast. `CombatPlugin` montato in `main.rs` sostituisce `register_combat_kernel_runtime` mantenendo lo stesso ordine system.

## Verification

- `CombatEvent.cast_id: CastId` aggiunto al JSONL output (campo nuovo, additivo non-breaking). Nessun nuovo event type in S01. `ExtRegistries` aggiunto come Resource ma vuoto (built-in arrivano in S05).

## Tasks

- [x] **T01: Create src/combat/api/ skeleton — primitive types (Intent, Registry, SignalBus, Clock, RNG)** `est:M`
  Introdurre il modulo src/combat/api/ con i tipi base del framework, niente wiring Bevy (solo Resource markers). Definisce CastId(NonZeroU32)+ROOT, Intent enum chiuso ~18 variant (incl. BlueprintSignal/SetBlueprintState/Reject), trait ExtPoint + Registry<E> + ExtRegistries Resource (7 assi placeholder), SignalBus Resource scaffold, Clock enum, CombatRng SplitMix64 deterministico. Vincolo: nessun import bevy::winit/render/bevy_egui in src/combat/api/. Unit test brevi inline per Registry lookup (hit/miss) e RNG determinism.
  - Files: `src/combat/api/mod.rs`, `src/combat/api/intent.rs`, `src/combat/api/registry.rs`, `src/combat/api/signal.rs`, `src/combat/api/clock.rs`, `src/combat/api/rng.rs`, `src/combat/mod.rs`
  - Verify: cargo check (headless) + cargo check --features windowed puliti. rg 'use bevy::winit|use bevy::render|use bevy_egui' src/combat/api/ → 0. cargo test --lib api::registry::tests e cargo test --lib api::rng::tests verdi. rg 'pub mod api' src/combat/mod.rs → 1.

- [ ] **T02: SkillCtx<'a> + intent_applier dispatcher (DealDamage canary wired)** `est:M`
  Aggiungere SkillCtx + dispatcher Intent. In S01 il dispatcher è scheletro: route per variant esiste, ma solo DealDamage è wired al damage system esistente come canary. Altre variant: log::warn! + delega alla code-path attuale. SkillCtxMode {DryRun, Execute, Preview} (Default=Execute). SkillCtx<'a> con caster, primary_target, cast_id, pending VecDeque<Intent>. Resource IntentQueue + system intent_applier exclusive. Test canary tests/intent_applier_canary.rs: spawn 2 unit, enqueue DealDamage, tick, asserisce HP ridotto + CombatEvent::OnDamageDealt + cast_id propagato (finalizzato dopo T03).
  - Files: `src/combat/api/skill_ctx.rs`, `src/combat/api/applier.rs`, `src/combat/api/mod.rs`, `tests/intent_applier_canary.rs`
  - Verify: cargo check (headless + windowed) puliti. cargo test --test intent_applier_canary verde (finalizzato dopo T03). rg 'fn intent_applier' src/combat/api/applier.rs → 1. rg 'use bevy::winit|use bevy::render|use bevy_egui' src/combat/api/skill_ctx.rs src/combat/api/applier.rs → 0.

- [ ] **T03: CastId propagation in CombatEvent + pipeline::step_app + emit sites** `est:L`
  Aggiungere cast_id: CastId come campo di CombatEvent e propagarlo da pipeline::step_app a tutti i call-site emit (~50). Emit pre-cast usano CastId::ROOT. CastIdGen Resource monotonic. Aggiorna tutti i call-site CombatEvent {...} via rg. Aggiorna test pattern-match con .. rest. Test tests/cast_id_propagation.rs: (a) eventi durante cast condividono cast_id; (b) cast-scoped ≠ ROOT; (c) eventi pre-cast = ROOT.
  - Files: `src/combat/events.rs`, `src/combat/api/intent.rs`, `src/combat/turn_system/pipeline.rs`, `src/combat/turn_system/mod.rs`, `src/combat/follow_up.rs`, `src/combat/damage.rs`, `src/combat/resolution.rs`, `src/combat/status_effect.rs`, `src/combat/stun.rs`, `src/combat/toughness.rs`, `src/combat/ultimate.rs`, `src/combat/sp.rs`, `src/combat/kernel.rs`, `src/combat/jsonl_logger.rs`, `tests/cast_id_propagation.rs`
  - Verify: cargo check (headless + windowed) puliti. cargo test full suite (~74) verde. rg 'CombatEvent \{' src/ | rg -v 'cast_id' → 0. cargo test --test cast_id_propagation verde (3 assertion). JSONL output contiene cast_id su ogni evento.

- [ ] **T04: CombatPlugin extract — Bevy Plugin wrapper + Resource init + lib re-export** `est:M`
  Spostare logica di register_combat_kernel_runtime in impl Plugin for CombatPlugin, montare Resource framework (ExtRegistries, SignalBus, Clock, CombatRng seed 0xDEADBEEF, IntentQueue, CastIdGen), registrare intent_applier exclusive, esporre CombatPlugin da lib.rs, aggiornare main.rs + bin/combat_cli.rs. Verifica rg import vietati e sposta dietro #[cfg(feature="windowed")] o in src/windowed.rs se trovati.
  - Files: `src/combat/plugin.rs`, `src/combat/mod.rs`, `src/lib.rs`, `src/main.rs`, `src/bin/combat_cli.rs`
  - Verify: rg 'use bevy::winit|use bevy::render|use bevy_egui' src/combat/ --glob '!blueprints/**' → 0. cargo check (headless + windowed) puliti. cargo run headless boot OK. rg 'CombatPlugin' src/lib.rs → ≥1. rg 'add_plugins.*CombatPlugin' src/main.rs → 1. rg 'register_combat_kernel_runtime' src/main.rs → 0.

- [ ] **T05: Slice verification — grep gates + full suite + decision update** `est:S`
  Verifica finale slice. Esegue tutti i gate cargo + grep, valida che le test esistenti reggono. Cargo check headless + windowed, cargo test full suite (~74 + canary T02 + propagation T03), grep verifiers (no bevy::winit/render/bevy_egui in src/combat/ ex-blueprints, CombatEvent {} con cast_id, pub mod api, CombatPlugin in lib.rs, no register_combat_kernel_runtime in main.rs), smoke run headless + windowed (skip se DISPLAY mancante). Se emerse decisioni non in DECISIONS.md (shape IntentQueue, default seed RNG), appendi via gsd_decision_save.
  - Files: `.gsd/DECISIONS.md`
  - Verify: Tutti i gate verdi. cargo test 0 fail. cargo check --features windowed 0 warning nuovi. rg verifiers come step 3 di T05-PLAN.

## Files Likely Touched

- src/combat/api/mod.rs
- src/combat/api/intent.rs
- src/combat/api/registry.rs
- src/combat/api/signal.rs
- src/combat/api/clock.rs
- src/combat/api/rng.rs
- src/combat/mod.rs
- src/combat/api/skill_ctx.rs
- src/combat/api/applier.rs
- tests/intent_applier_canary.rs
- src/combat/events.rs
- src/combat/turn_system/pipeline.rs
- src/combat/turn_system/mod.rs
- src/combat/follow_up.rs
- src/combat/damage.rs
- src/combat/resolution.rs
- src/combat/status_effect.rs
- src/combat/stun.rs
- src/combat/toughness.rs
- src/combat/ultimate.rs
- src/combat/sp.rs
- src/combat/kernel.rs
- src/combat/jsonl_logger.rs
- tests/cast_id_propagation.rs
- src/combat/plugin.rs
- src/lib.rs
- src/main.rs
- src/bin/combat_cli.rs
- .gsd/DECISIONS.md
