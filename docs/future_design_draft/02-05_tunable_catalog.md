# §2.5 — Tunable data catalog (editor-ready)

**Vincolo dichiarato:** in futuro vogliamo **editor custom in-process** per tunare cose ad alta frequenza di iterazione (animazioni, trigger, vfx, curve stats) **senza ricompilare**. Implicazione architetturale: ogni dato ad alta frequenza di iterazione deve essere **sorgente di verità in RON**, hot-reloadabile via `bevy_asset` change events. Il runtime *legge* dal RON; il codice Rust contiene **solo regole**, non valori.

**Catalogo (M017 scope vs futuro):**

| # | Asset | Stato | Contenuto | Edit ratio | M017 |
|---|---|---|---|---|---|
| 1 | `assets/data/units.ron` | ✅ esiste | identità unit, skill_id list | bassa | resta |
| 2 | `assets/data/skills.ron` | ✅ esiste | damage, sp_cost, ult_cost, target shape | **alta** | resta + audit numeri hardcoded da migrare |
| 3 | `assets/data/party.ron` | ✅ esiste | preset party | bassa | resta |
| 4 | **`assets/data/unit_stats.ron`** | ❌ nuovo | base stats + growth curve per livello, toughness, resistenze | **alta** | **introdurre** |
| 5 | **`assets/data/signal_bindings.ron`** | ❌ nuovo (§2.1) | `{ skill_id → kernel signal }` | media | **introdurre** |
| 6 | **`assets/data/encounter_balance.ron`** | ❌ nuovo (§4.2) | wild nerf curve + per-pack-size + per-level growth | **alta** | **introdurre** |
| 7 | **`assets/data/status_effects.ron`** | ❌ nuovo | duration, tick damage, stack rules per `Heated`/`Wet`/`Stun`/... | **alta** | **introdurre** |
| 8 | **`assets/data/run_config.ron`** | ❌ nuovo (§3.2) | num encounter, HP carryover %, SP reset, encounter slot mix | media | **introdurre** |
| 9 | **`assets/digimon/<name>/clip.ron`** | ❌ nuovo (§2.2) | frame ranges per clip animazione | bassa | **introdurre** (6 file) |
| 10 | **`assets/digimon/<name>/clipmontage.ron`** | ❌ nuovo (§2.2 → forma finale §2.2b) | **AnimGraph FSM** (nodi + edges + Commands `on_enter`); fallback degenerate = 1 nodo all-clip | **alta** | **introdurre** (6 file, solo Agumon popolato in M017 con grafo full-featured Baby Flame §2.9; gli altri 5 = degenerate node) |
| 11 | `assets/digimon/<name>/vfx.ron` | ❌ futuro | definizione particle (color, count, lifetime) | n/a | fuori scope, namespace riservato |
| 12 | AI behavior (`enemy_ai.rs`) | resta in Rust | flow decisionale AI nemica | n/a | logica, non dati |
| 13 | **`assets/data/skill_tree.ron`** | ❌ schema riservato (§2.2b §I) | nodi unlock per Digimon (`cost`, `requires`, `patches: PatchParams/PatchCostEffect`, `kit_swap`) | **alta** quando entra runtime | **schema riservato M017**, runtime resolver implementato col primo unlock concreto (M018+). In M017 il file può esistere vuoto come placeholder o omesso: la FSM legge `UnlockedPassives` empty resource → tutte le `Predicate::Unlock(...)` non matchano |
| 14 | **`assets/data/effects.ron`** | ❌ schema riservato (§2.2b §J) | catalog di `CostEffect`/`CooldownEffect` referenziati da `skills.ron::cost_effect`/`cooldown_effect` | media | **schema riservato M017**, implementato quando il primo `skill_tree.ron` patcha cost. In M017 `skills.ron` può continuare a usare `sp_cost` numero come oggi; migrazione cost-as-effect è hook M018+ |

**Cosa NON va in RON (motivo):**
- **Kernel rules** (turn order, speed math, SP cap formula): cambia raramente, vive bene in Rust con costanti chiare.
- **Blueprint state machine** (`battery_loop.rs`, `predator_loop.rs`, ecc.): è logica condizionale tipata, non dati. Tradurla in RON significa reinventare un linguaggio di scripting → tradeoff perdente.
- **AI decision flow**: condizionali complessi (target select, prioritization). Restano in Rust. Editor RON solo per *weights/priorities*, e solo se diventa bottleneck di tuning (non in M017).

**Forma concreta dei nuovi RON:**

```ron
// unit_stats.ron
{
    "agumon": StatBlock(
        base: Stats(hp: 100, atk: 28, def: 12, spd: 95),
        growth: GrowthCurve(
            hp:  Linear(per_level: 12.0),
            atk: Linear(per_level: 3.2),
            def: Linear(per_level: 1.4),
            spd: Linear(per_level: 0.5),
        ),
        toughness: 60,
        resistances: { Fire: -0.10, Ice: 0.20 },
    ),
    // … 5 altri Rookie
}

// encounter_balance.ron
WildNerfCurve(
    by_pack_size: {
        1: PackNerf(hp_mult: 0.65, atk_mult: 0.75, ult: Suppressed),
        2: PackNerf(hp_mult: 0.75, atk_mult: 0.85, ult: Suppressed),
        3: PackNerf(hp_mult: 0.85, atk_mult: 0.90, ult: Limited(initial_pct: 50, max_casts: 1)),
        4: PackNerf(hp_mult: 0.95, atk_mult: 0.95, ult: Normal),
    },
    // i wild scalano col livello medio del party
    level_track: LevelTrack(
        offset: 0,            // wild = livello party (0 = parità)
        hp_per_level:  0.08,  // +8% HP per livello
        atk_per_level: 0.06,
    ),
)

// status_effects.ron
{
    "heated": StatusEffectDef(
        duration_turns: 3,
        tick: DamageTick(amount: 5, scaling: AtkMult(0.10)),
        stack_rule: RefreshDuration,
        on_apply_signals: ["heated_applied"],
    ),
    // …
}

// run_config.ron
RunConfig(
    encounter_count: 5,
    encounters: [
        Slot(kind: WildPackRange(min: 1, max: 2)),
        Slot(kind: WildPackRange(min: 2, max: 3)),
        Slot(kind: HandcraftedTier(elite: false)),
        Slot(kind: WildPackRange(min: 3, max: 4)),
        Slot(kind: HandcraftedTier(elite: true)),
    ],
    carryover: Carryover(
        hp_missing_healed_pct: 50,
        sp_reset_to: 3,
        ult_charge: Persist,
    ),
    fail_condition: AllPartyKO,
)
```

**Editor strategy (deferred):** gli editor custom (stat curve plotter, animation timeline, encounter balance previewer) sono **fuori scope M017** ma le scelte di shape RON di questa milestone li abilitano. Tutti i type RON devono `#[derive(Reflect, Serialize, Deserialize)]` → `reflect_auto_register` (§2.3 adjunct) li espone gratis a `bevy-inspector-egui` per un editor placeholder windowed. L'editor custom diventerà la sua milestone (M019+).

**Hot-reload:** Bevy `AssetServer` già supporta change-detect dei `.ron` se caricati come asset typed (non come literal `include_str!`). Tutti i nuovi RON di questa tabella devono essere caricati via `AssetServer::load`, non `include_str!`. Test di reload: in S03 verifica che editare un `clipmontage.ron` a runtime aggiorni gli FX al prossimo trigger.

**Migration ordering (rilevante per §5 slicing):**
1. S01-S02 introducono il bus `KernelEffect` (dipendenza solo strutturale, non tocca i RON).
2. **Nuove slice dedicate** ai nuovi RON: una per `unit_stats.ron` (estrarre stats da `units.ron` con growth), una per `status_effects.ron`, una per `encounter_balance.ron`, una per `run_config.ron`. Vedi §5 aggiornato.
