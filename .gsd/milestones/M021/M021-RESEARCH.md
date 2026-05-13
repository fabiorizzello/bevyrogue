# M021 — Research & Audit: Kernel ⇄ Digimon Identities Decoupling

**Data audit:** 2026-05-13
**Branch:** `milestone/M017`
**Scope:** valutare lo stato attuale del coupling kernel↔digimon, definire l'architettura target ("kernel non sa nulla di Twin Core / Battery Loop / ..."), proporre la slicing completa di M021.

> **Update 2026-05-13 (post-M018 S03):** questa research copre solo la **Fascia B** di M021 (Blueprint trait + plugin split + kernel decoupling). La **Fascia A** (Skill trait + `SkillCtx` + `Intent`, vedi **D010**) è stata aggiunta dopo, ed è descritta in `M021-CONTEXT.md`. Le slice S01–S08 qui sotto corrispondono a B1–B8 in CONTEXT; l'ordine finale interlea A e B (vedi §"Ordine slice" in CONTEXT).

---

## 1. Evidence — stato attuale

### 1.1 `kernel.rs` contiene domain digimon-specifici

`src/combat/kernel.rs` = **1393 LOC**. Esso definisce, oltre al core (Strain/Flow/Fatigue/Tag/Beat/TacticalCycle), enum e struct **per ogni mechanic identitaria**:

| Mechanic | Digimon | Tipi nel kernel | Linee |
|---|---|---|---|
| Twin Core | Agumon + Gabumon | `TwinCoreSignal`, `TwinCoreTransition` | 433, 445 |
| Battery Loop | Tentomon | `BatteryLoopChargeKind`, `BatteryLoopBlockedReason`, `BatteryLoopStep`, `BatteryLoopSignal`, `BatteryLoopTransition` | 509–549 |
| Precision Mind Game | Renamon | `PrecisionMindGamePhase`, `PrecisionMindGameRejectReason`, `PrecisionMindGameStep`, `PrecisionMindGameTransition` | 667–695 |
| Predator Loop | Dorumon | `PredatorLoopCapKind`, `PredatorLoopBlockedReason`, `PredatorLoopStep`, `PredatorLoopSignal`, `PredatorLoopTransition` | 747–787 |
| Holy Support | Patamon | `HolySupportSignal`, `HolySupportStep`, `HolySupportRejectReason`, `HolySupportTransition` | 913–941 |

E sopra a tutti, l'**enum chiuso** che il kernel routa internamente (`kernel.rs:889`):

```rust
pub enum CombatKernelTransition {
    TacticalCycle(_), Strain(_), Flow(_), Fatigue(_), Tag(_), Beat(_),
    TwinCore(_), BatteryLoop(_), HolySupport(_), PredatorLoop(_), PrecisionMindGame(_),
}
```

→ **Aggiungere un Digimon = aprire `kernel.rs` ed editare enum + apply system.** Non è plug-and-play.

### 1.2 Registrazione plugin asimmetrica

`src/combat/blueprints/mod.rs:110-135`:

```rust
const BLUEPRINTS: &[BlueprintRegistration] = &[
    BlueprintRegistration { owner: agumon::OWNER, dispatch: agumon::dispatch },
    BlueprintRegistration { owner: gabumon::OWNER, dispatch: gabumon::dispatch },
    ... // 6 entry hardcoded
];
```

→ const array, non un registry runtime. Compile-time OK ma:

- non c'è `trait Blueprint` → ogni file blueprint inventa la propria forma (`fn dispatch(...) -> Vec<CombatKernelTransition>`).
- `dispatch` produce **già** `CombatKernelTransition::*` — il blueprint **deve conoscere** che esiste una variant kernel apposita per la sua mechanic.

### 1.3 Asimmetria struttura plugin

```
src/combat/blueprints/
├── agumon/        ← mod.rs + identity.rs + signals.rs  (plugin idiomatico)
├── patamon/       ← mod.rs + identity.rs + signals.rs  (plugin idiomatico)
├── dorumon/       ← mod.rs + identity.rs + signals.rs + hooks.rs (plugin idiomatico)
├── gabumon.rs     ← 63 LOC flat — niente Plugin Bevy
├── renamon.rs     ← 40 LOC flat — niente Plugin Bevy
└── tentomon.rs    ← 70 LOC flat — niente Plugin Bevy
```

Solo 3/6 sono Bevy `Plugin`. Riprova in `src/combat/kernel.rs:1088-1092`:

```rust
app.add_plugins((AgumonPlugin, PatamonPlugin, DorumonPlugin,))
```

Battery Loop e Precision Mind Game (Tentomon, Renamon) sono cablati altrove come system standalone (kernel.rs:1069-1070 + `precision_mind_game.rs`).

### 1.4 Shim compat ancora vivi

`src/combat/mod.rs`:
```rust
pub use blueprints::agumon::identity as twin_core;
// + 2 simili: predator_loop, holy_support
```

Tag `Q10` nel changelog locale dice "rimossi quando i Q9 sono chiusi". Q9 = migrazione gabumon/renamon/tentomon a plugin. Non ancora chiuso.

### 1.5 Roster + ValidationSnapshot conoscono i nomi delle mechanic

- `RosterEntry` (presumibilmente `src/data/units_ron.rs`) ha field hardcoded `twin_core`, `holy_support`, `battery_loop`, … invece di un blueprint-keyed payload.
- `ValidationSnapshot` ha field inline (es. `battery_loop`) — aggiungere un nuovo digimon richiede aggiornare lo struct.

### 1.6 Dead/scaffolding presente ma non bloccante per il refactor

Da QQ5-b skippato: `PredatorLoopDesignTag`, `BatteryLoopDesignTag`, `CombatKernelHookDomain` enum, vari ctor mai chiamati. Vivono nello stesso domain del refactor → **verranno cancellati o resi vivi dalla migrazione naturale** in M021, no action separata necessaria.

---

## 2. Target architecture — è possibile arrivare a kernel agnostico al 100%?

**Risposta corta:** sì, con un caveat su un sotto-insieme di feature di dominio.

### 2.1 Cosa il kernel **deve** sapere (= concept invariant del combat)

- Turn order, AV gauge, TacticalCycle (windup/strike/recovery), Beat
- Strain (toughness/break), Flow (SP), Fatigue (Ult charge) — risorse universali
- Tag bus generico (CombatTag) → meccanismo, non lista chiusa
- Status taxonomy canon §H.1 (5 status) — vocabolario condiviso, non identitario
- Custom signal dispatch infrastructure (envelope `Blueprint { owner, signal, payload }`)

→ **questo è il "core invariant"**. Resta dentro `kernel.rs`.

### 2.2 Cosa il kernel **non deve** sapere

- Twin Core: heated stack rules, cross-resonance — è Agumon+Gabumon, non kernel
- Battery Loop: charge kinds, transfer rules — è Tentomon
- Predator Loop: berserk cap, target tracking — è Dorumon
- Holy Support: grace gauge, martyr light — è Patamon
- Precision Mind Game: phase ladder — è Renamon

→ tutta questa roba **trasloca dentro `blueprints/<digimon>/`**.

### 2.3 Architettura target (sketch)

```rust
// src/combat/blueprints/api.rs — il contract

pub trait Blueprint: Send + Sync + 'static {
    fn owner(&self) -> &'static str;

    /// Decodifica un custom signal → emette transition opache.
    fn dispatch(
        &self,
        signal: &SkillCustomSignal,
        action: &ResolvedAction,
    ) -> Result<Vec<BlueprintTransition>, CustomSignalDispatchError>;

    /// Bevy plugin hooks: lo blueprint registra i propri system + componenti.
    fn build(&self, app: &mut App);
}

/// Payload opaco — il kernel non lo guarda, lo routa al blueprint owner.
pub struct BlueprintTransition {
    pub owner: &'static str,
    pub payload: Box<dyn Any + Send + Sync>,
}

// Registry runtime, popolato in `BlueprintsPlugin::build`.
pub struct BlueprintRegistry(HashMap<&'static str, Box<dyn Blueprint>>);
```

`CombatKernelTransition` perde **5 variant** digimon-specifiche:

```rust
// PRIMA: 11 variant
pub enum CombatKernelTransition {
    TacticalCycle(_), Strain(_), Flow(_), Fatigue(_), Tag(_), Beat(_),
    TwinCore(_), BatteryLoop(_), HolySupport(_), PredatorLoop(_), PrecisionMindGame(_),
}

// DOPO: 7 variant — 6 core + 1 opaque blueprint passthrough
pub enum CombatKernelTransition {
    TacticalCycle(_), Strain(_), Flow(_), Fatigue(_), Tag(_), Beat(_),
    Blueprint(BlueprintTransition),
}
```

Il sistema `apply_combat_kernel_transitions` matcha solo le 6 core; per `Blueprint(_)` chiama `registry.dispatch_apply(owner, payload)` che ritorna al plugin del digimon.

Componenti di stato delle mechanic (heated stack count, charge_kind, predator target) → `Component` Bevy registrato dal plugin del digimon, **non** field dentro `CombatState` o `Unit`.

### 2.4 Caveat — interaction cross-mechanic

Twin Core cross-resonance (Agumon ↔ Gabumon) e Battery Loop transfer (Tentomon → ally) implicano che un blueprint legge state di un altro. Soluzioni:

1. **Bevy ECS naturale**: ogni mechanic ha il suo `Component`, i system cross-mechanic fanno `Query<&TwinCoreState>` dall'altro blueprint. Coupling esiste ma è a livello plugin↔plugin, non kernel↔plugin.
2. **Event bus condiviso**: il blueprint A emette `CombatEvent::BlueprintEmitted { owner: "agumon", kind: "twin_core_cross_resonance" }`, blueprint B lo consuma. Disaccoppiato ma stringly-typed.

Preferenza: **(1)** per cross-resonance Twin Core (Agumon + Gabumon condividono uno stesso `Component` `TwinCoreState` co-registrato dal "Twin Core mechanic", che è un mini-plugin sotto il dominio di entrambi i blueprint). Pattern: il "shared mechanic" diventa un blueprint a sé senza un `owner` digimon, ma owner = "twin_core". Agumon e Gabumon ci si appoggiano.

→ riconciliabile: anche le shared mechanic restano *fuori dal kernel*.

### 2.5 Verdetto: kernel-agnostic al 100% **è raggiungibile**.

Costi:
- ~30 sites di `CombatKernelTransition::TwinCore(...)` ecc. da migrare a `CombatKernelTransition::Blueprint(...)` (grep mostra le occorrenze).
- Boxing dei payload (`Box<dyn Any>`) = piccolo overhead alloc, irrilevante (transition events sono frequenza turn-tick).
- `Box<dyn Blueprint>` registry runtime invece di `const &[_]` = perdita di static-dispatch ma irrilevante a 6–20 plugin.

Tradeoff esplicito: il typesafety puntuale ("so a compile-time che `BatteryLoopTransition` ha campo `kind`") è sostituito da un downcast `payload.downcast_ref::<BatteryLoopApply>()` **dentro il plugin Tentomon**. Type safety preservata dentro il dominio, persa solo all'envelope.

---

## 3. moonshine_kind — stato

Già adottato dove serve: `Query<Instance<Unit>>` in `headless.rs:210`, `ui/combat_panel.rs:234-235`, `bin/combat_cli.rs:135`.

Residui legittimi:
- `Query<(Entity, &FloatingDamage)>` in `floating.rs:22` — usato solo per despawn, OK.

Residui da convertire (piccolo QQ post-M021):
- `av.rs:56` `pub unit_entity: Entity` in payload tipato → `Instance<Unit>`
- `turn_system/mod.rs:380` payload con `entity: Entity` → idem

**Non bloccante M021.** Cleanup ortogonale.

---

## 4. Slicing proposta M021 (vertical, in priority order)

Aggiorna lo sketch in `MILESTONE-PORTFOLIO.md` (S01–S08) con criteri di accettazione concreti.

### S01 — `CombatPlugin` extract (refactor zero-logic)

**Goal:** estrarre `register_combat_kernel_runtime` in un Bevy `Plugin` neutro headless/windowed.

**File toccati:** `src/main.rs`, `src/headless.rs`, `src/windowed.rs`, nuovo `src/combat/plugin.rs`.

**Accept:** `cargo check`, `cargo check --features windowed`, `cargo test` tutti verdi. Zero cambi di logica. Diff confinato a wiring.

**Risk:** basso. **Stima:** 1 h.

### S02 — `trait Blueprint` + opaque `BlueprintTransition` + registry runtime

**Goal:** introdurre l'API target. Nessuna migrazione ancora. Coesistenza con `const BLUEPRINTS` legacy.

**File nuovi:** `src/combat/blueprints/api.rs` (trait + registry), test unit per il registry.

**Accept:** registry vuoto si registra come Bevy `Resource`. Test verifica register+lookup. Suite esistente verde (nessuno usa ancora il registry).

**Risk:** basso. **Stima:** 1.5 h.

### S03 — Agumon migrato al trait + self-registration

**Goal:** prima migrazione completa. Le altre 5 restano sul percorso legacy via shim.

**Accept:** `AgumonBlueprint` impl `Blueprint`. `BlueprintsPlugin::build` la registra. Agumon non emette più `CombatKernelTransition::TwinCore(_)` direttamente — emette `Blueprint(_)` con payload opaco, plugin Agumon decodifica.

**Tests:** twin core cross-resonance test esistenti verdi. Snapshot test su payload flow.

**Risk:** medio (Twin Core ha cross-resonance con Gabumon — serve transizione attenta). **Stima:** 3 h.

### S04 — Gabumon migrato (Twin Core paired)

**Goal:** chiude la coppia Twin Core. Twin Core diventa shared mechanic mini-plugin (owner = "twin_core") dietro Agumon+Gabumon.

**Accept:** test cross-resonance Agumon→Gabumon e viceversa verdi. `TwinCoreSignal`/`TwinCoreTransition` non esistono più in `kernel.rs` — vivono in `blueprints/twin_core/`.

**Risk:** medio-alto (lo split mechanic vs owner è il pattern reference per le altre). **Stima:** 3 h.

### S05 — Dorumon + Tentomon migrati

**Goal:** Predator Loop e Battery Loop migrati. Mechanic standalone (ognuno solo single-owner).

**Accept:** kernel ha perso `PredatorLoopTransition`, `BatteryLoopTransition`, `PredatorLoopSignal`, `BatteryLoopSignal` e tutti gli enum collegati. Suite verde (incluso `predator_loop_runtime.rs`, `battery_loop_*.rs`).

**Risk:** medio (Predator Loop ha kernel hook system, Battery Loop ha charge kinds enum). **Stima:** 3 h.

### S06 — Patamon + Renamon migrati. Rimozione shim. `CombatKernelTransition` digimon-free.

**Goal:** chiusura. Holy Support + Precision Mind Game migrati. `pub use … as twin_core` shim cancellati. Registrazione `app.add_plugins((Agumon, Patamon, Dorumon,))` rimpiazzata da `app.add_plugins(BlueprintsPlugin)` che auto-registra tutti.

**Accept:** grep `^pub (enum|struct) (TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame)` su `src/combat/kernel.rs` → 0 match. `CombatKernelTransition` ha 7 variant invece di 11. `cargo check` + full test suite verdi.

**Risk:** medio. **Stima:** 3 h.

### S07 — `RosterEntry` blueprint-keyed payload (no hardcoded mechanic field)

**Goal:** schema roster non conosce più i nomi delle mechanic.

**Prima:**
```ron
RosterEntry(name: "agumon", twin_core: Some(...), holy_support: None, battery_loop: None, ...)
```

**Dopo:**
```ron
RosterEntry(name: "agumon", blueprint_data: {
    "twin_core": ( ... ),
})
```

**Accept:** aggiungere un nuovo digimon non richiede toccare `units_ron.rs` né `RosterEntry` schema. Schema validato da loader RON via blueprint-owner lookup.

**Risk:** medio (asset migration: `assets/data/units.ron`). **Stima:** 2 h.

### S08 — `ValidationSnapshot` nominato dal registry

**Goal:** observability stabile per blueprint key.

**Prima:** shape inline con `battery_loop` di default (vedi memoria gotcha).

**Dopo:** `ValidationSnapshot { blueprint_states: HashMap<String, BlueprintSnapshot> }` popolata dal registry. Ogni blueprint contribuisce la sua slice di snapshot.

**Accept:** test che fanno snapshot assertion (`predator_loop_runtime_proof`, `battery_loop_*`) usano la nuova shape. Aggiungere un digimon non richiede toccare `ValidationSnapshot` struct.

**Risk:** medio (touch su ~10+ test file). **Stima:** 2.5 h.

---

## 5. Stima totale + sequencing

| Slice | Stima | Risk | Dipende da |
|---|---|---|---|
| S01 | 1 h | basso | — |
| S02 | 1.5 h | basso | S01 |
| S03 | 3 h | medio | S02 |
| S04 | 3 h | medio-alto | S03 |
| S05 | 3 h | medio | S02 (S04 consigliato per pattern) |
| S06 | 3 h | medio | S03+S04+S05 |
| S07 | 2 h | medio | S06 |
| S08 | 2.5 h | medio | S06 |

**Totale:** ~19 h (~2.5 giornate piene), serializzato per ordine di dipendenza. S05 può andare in parallelo a S04 se due agenti.

**Demo finale M021:**
- 6 blueprint Bevy plugin auto-registrati via `BlueprintsPlugin`.
- Grep `kernel.rs` non mostra nomi digimon-specifici.
- Aggiungere un 7° digimon = creare `blueprints/<x>/` con `impl Blueprint`, niente edit a `kernel.rs`/`mod.rs`/`RosterEntry`/`ValidationSnapshot`.
- Full integration suite verde (>60 test).

---

## 6. Out of scope esplicito

- DR pipeline (`BuffKind::DR` clamp 0.5) → M019.
- Reactive bus extension (StatusApplied, UltimateUsed) → M020.
- AdvanceTurn/DelayTurn split + cap → M018.
- TargetShape resolver (Blast/AoE/Bounce) → M018.
- Asset pipeline loader (`clip.ron`, `animation_fsm.ron`) → M022.
- AnimGraph runtime + sprite render → M023.
- Skill RON schema redesign → **dentro M021 fascia A (slice A6, vedi `M021-CONTEXT.md`)**: post-D010, RON viene ridotto a numeri/tag (dmg, hops, sp_cost, scaling, target_shape base) e la logica Effect/scripting esce. Questa nota originale (fuori M021) è obsoleta — era valida pre-D010 quando si pensava solo alla fascia B (Blueprint trait) e si voleva preservare lo schema `Effect` esistente.
- `Entity → Instance<Unit>` cleanup in `av.rs`/`turn_system/mod.rs` payload → QQ post-M021.
- `default_headless_script` esternalizzazione → QQ post-M021.
- Split `kernel.rs` 1393 LOC per topic — **naturalmente risolto da S04+S05+S06** che svuotano il kernel di ~600 LOC digimon-specifici.

---

## 7. Apertura M021

**Prerequisiti per partire M021:**
- M017 chiuso e validato (✅ tutti i 6 slice S01-S06 segnati `[x]` in `M017-ROADMAP.md`).
- Branch corrente: `milestone/M017` → merge in master prima di iniziare `milestone/M021`.
- `cargo check --tests` verde, `cargo test` verde sul branch corrente.

**Open question per il planning meeting:**
1. Confermare il pattern "shared mechanic mini-plugin" per Twin Core (sezione 2.4). Alternativa: Agumon è "owner primario", Gabumon "ospite" che chiede il componente via Query.
2. `Box<dyn Any>` payload OK o preferiamo enum chiusa a livello blueprint-side (es. `BlueprintTransition::Twin(TwinCoreApply)` mantenuta solo dentro `blueprints/twin_core/`)? Tradeoff: tipo-safety dentro plugin vs. zero downcast.
3. `BlueprintRegistry`: `Resource` Bevy single-writer o startup-frozen? Implicazioni hot-reload futuro.
4. Devo creare l'M021-CONTEXT.md + roadmap dopo questa research, o aspettiamo la chiusura M017 in master?
