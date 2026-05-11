# §2.3 — Blueprint plugin extension (kernel estensibile, blueprint isolati)

**Obiettivo dichiarato (vincolo, rilassato):** aggiungere o rimuovere un Digimon deve essere **isolato a una sola directory** `blueprints/<name>/` + **una riga** nel `BlueprintsPlugin` group. Niente touch nel kernel, niente match esaustivi da aggiornare. Il blueprint è un *plugin* Bevy self-contained.

**Cos'è un `*Transition` (primer per chi legge a freddo):** oggi `combat/kernel.rs` definisce un enum `CombatKernelTransition` con una variant per ogni Digimon (`BatteryLoop` per Tentomon, `HolySupport` per Patamon, ecc.). Un blueprint produce un'istanza della *sua* variant; il kernel `match`a su tutte le variant per applicarle. Conseguenza: aggiungere un Digimon = nuova variant + nuovo ramo nel kernel. Violazione del vincolo. `KernelEffect` (sotto, opzione α) sostituisce le 5 variant con un enum di **effetti generici** (`DamageDealt`, `ApplyStatus`, `GrantSp`, `Revive`, `QueueFollowUp`, ...) che il kernel sa applicare senza sapere chi li ha emessi.

**Stato attuale (problema diagnosticato):**

`src/combat/kernel.rs` (1392 righe) contiene 5 transition type Digimon-specifici, e i 5 file di subsystem vivono in `combat/` root (non `combat/blueprints/`):

| Transition type (kernel.rs) | Owner | File subsystem (combat/) | LoC |
|---|---|---|---|
| `BatteryLoopTransition` | Tentomon | `battery_loop.rs` | 261 |
| `HolySupportTransition` | Patamon | `holy_support.rs` | 254 |
| `PredatorLoopTransition` | Renamon o Dorumon (TBD) | `predator_loop.rs` | 510 |
| `PrecisionMindGameTransition` | Renamon o Dorumon (TBD) | `precision_mind_game.rs` | 211 |
| `TwinCoreTransition` | Gabumon | `twin_core.rs` | 253 |

Totale: **1.489 LoC blueprint-specific** lekati nel kernel + variant in `CombatKernelTransition` chiuse a compile time. Il `BLUEPRINTS` registry in `blueprints/mod.rs:107-132` è un `const &[…]` hard-coded che ogni nuovo Digimon deve toccare.

Quindi violiamo il vincolo in **3 punti**:
1. **Variant enum chiusa:** `CombatKernelTransition::BatteryLoop(…)` aggiunge un nuovo Digimon → editi `kernel.rs`.
2. **Registry centrale:** `BLUEPRINTS = &[…]` → editi `blueprints/mod.rs`.
3. **File subsystem fuori posto:** la state machine vive in `combat/<name>_loop.rs` invece che in `blueprints/<name>/`.

---

**Soluzione adottata: α — Bevy Plugin + `KernelEffect` bus.**

Il kernel **non** ha più `CombatKernelTransition::BatteryLoop(…)`. Espone invece **un bus di effetti generici** (`KernelEffect`) e **un bus di signal** (`BlueprintSignal`). Ogni blueprint è un `impl Plugin for TentomonPlugin` che:
- registra le sue Resource interne (`BatteryLoopState`)
- aggiunge un system che `EventReader<BlueprintSignal>` (filtra per owner) e `EventWriter<KernelEffect>`
- non aggiunge nulla al kernel; il kernel applica solo `KernelEffect` (damage, status, sp grant, ult charge, toughness, follow-up, revive, …)

```rust
// blueprints/tentomon/mod.rs — UNICO file da editare per Tentomon
pub struct TentomonPlugin;
impl Plugin for TentomonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BatteryLoopState>()
           .add_systems(Update, tentomon_dispatch.in_set(BlueprintDispatch));
    }
}

// blueprints/mod.rs — UN file che enumera i 6
pub struct BlueprintsPlugin;
impl Plugin for BlueprintsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            AgumonPlugin, GabumonPlugin, PatamonPlugin,
            TentomonPlugin, RenamonPlugin, DorumonPlugin,
        ));
    }
}
// main.rs registra UNA volta: app.add_plugins(BlueprintsPlugin)
```

**Pro:** idiomatico Bevy. Test isolabili (`App::new().add_plugins(TentomonPlugin)`). Niente fn pointer, niente `Box<dyn>` runtime. Aggiungere/rimuovere Digimon = directory + 1 riga in `BlueprintsPlugin`.

**Contro:** il kernel deve avere `KernelEffect` abbastanza espressivo da coprire tutti i casi attuali (sp grant, ult grant, status apply, toughness mod, follow-up queue, custom resource bump, revive, …). Costo: rivedere le 5 transition esistenti e mapparle su `KernelEffect`. Lavoro 1 volta, paga sempre.

**Adjunct nativo Bevy 0.18 (gratis):** [`reflect_auto_register`](https://github.com/bevyengine/bevy/pull/15030) auto-registra i tipi `#[derive(Reflect)]` non-generici in `AppTypeRegistry`. Già nel default di Bevy 0.18, nessuna dep aggiuntiva. Da usare per i blueprint data type (`BatteryLoopState`, `PredatorTargetState`, …) se annotati `Reflect` → si pagano gratis con l'inspector/editor (vedi §2.5).

**Migrazione (strategia coesistenza→cleanup):**
1. S01 definisce `KernelEffect` esaustivo + ponte `From<*Transition> for KernelEffect` (legacy convivono, test passano senza modifiche).
2. S02 fa il flip definitivo: kernel emette solo `KernelEffect`, blueprint diventano `Plugin`, le 5 `*Transition` spariscono. ~60 file di test integrazione adattati ad assertare effetti invece di transition specifiche.

**Ownership predator + precision (deciso):**

| File | LoC | Owner | Tests | Destinazione |
|---|---|---|---|---|
| `predator_loop.rs` | 510 | **Dorumon** (esclusivo) | `tests/dorumon_predator_runtime.rs`, `tests/predator_loop_kernel.rs` | `blueprints/dorumon/predator_loop.rs` |
| `precision_mind_game.rs` | 211 | **Renamon** (esclusivo) | `tests/renamon_precision_runtime.rs` | `blueprints/renamon/precision_mind_game.rs` |

Zero call site esterno al rispettivo blueprint. Entrambi migrano dentro la directory dell'owner come parte di S02. Nessun `blueprints/_shared/`.
