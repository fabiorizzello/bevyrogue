# §2.4 — Extension-first + strong-typing (kernel chiuso, blueprint aperti)

**Domanda:** col modello α (Bevy Plugin) la logica custom di un Digimon è davvero **isolata in un solo crate logico**? E le stringhe nei RON (signal name, skill_id, status name) restano string-untyped o esiste un percorso a strong-type?

**Risposta breve:** sì, extension-first è centrato su α. Strong-typing si ottiene per gradi: M017 sta su stringhe + contract test al boot.

**A — Extension-first (cosa "vive" dentro `blueprints/<name>/`):**

Tutta la logica custom di un Digimon vive nella sua directory plugin. Il kernel non sa cosa fa il blueprint, sa solo applicare `KernelEffect`. Concretamente, aggiungere il Digimon ipotetico "Wormmon" significa creare:

```
blueprints/wormmon/
  mod.rs              ← impl Plugin for WormmonPlugin
  state.rs            ← Resource interna (es. SilkCounter)
  dispatch.rs         ← system: EventReader<BlueprintSignal> → EventWriter<KernelEffect>
  state_machine.rs    ← logica condizionale interna (se serve, opzionale)
```

più 1 riga in `blueprints/mod.rs::BlueprintsPlugin`. Il kernel resta **invariato**. La logica del Digimon (es. "ogni 3 attacchi base accumula 1 silk; al 5° silk libera un follow-up") vive interamente dentro `wormmon/` come state machine privata che reagisce ai signal kernel e produce `KernelEffect`. **Nessun match esaustivo nel kernel da aggiornare, nessuna variant da aggiungere a un enum centrale.**

Un blueprint può anche introdurre **state-machine custom interne** (es. `BatteryLoopState` con 3 stati: `Idle | Charging | Overdrive`) che non sono visibili al kernel — il kernel vede solo gli effetti che la state machine emette. Questo è ciò che intende "extension-first": estendere il gioco = aggiungere un crate logico, non modificare il core.

**B — Stringhe nei RON: dove fanno male, dove vanno bene.**

| Categoria | Tipo oggi | Type-safety problema? | Frequenza d'uso in Rust | Reco M017 |
|---|---|---|---|---|
| Signal name (`build_exploit`, `apply_heated`) | stringa in `signal_bindings.ron` + `&str` in blueprint Rust | **alta** — typo silenzioso, `EventReader` filtra falso negativo | alta (ogni system blueprint legge signal name) | **stringhe + contract test al boot** (vedi C) |
| Skill ID (`tentomon_thunder`, `agumon_pepper`) | stringa in `skills.ron`, `units.ron`, `party.ron` | **alta** — typo in cross-reference rompe loadup | alta | **stringhe + contract test** (già esistente per units↔skills) |
| Status effect name (`heated`, `wet`, `stun`) | hardcoded in Rust oggi → diventa stringa in `status_effects.ron` (S05) | **media** — set chiuso e piccolo, ma referenziato da `on_apply_signals` cross-RON | alta in `status_effect.rs`, `resolution.rs` | **stringhe + contract test** |
| Blueprint owner ID (`agumon`, `gabumon`) | stringa in `units.ron`, key del registry | **bassa** — già coperto da `BLUEPRINTS` registry static | media | **stringhe** |
| Asset names (`claw_slash` per particle, `claw_hit` per sfx in `clipmontage.ron`) | stringa in `clipmontage.ron` | **bassa** — sono tag verso asset path, non logica | bassa (consumer è UI animator) | **stringhe libere** |
| Notify variant kind (`Particle`, `Hold`, `SpeedMul`) → Command kind (§2.2b: `EmitDamage`, `SpawnParticle`, `Shake`, `Hold`, `StartQTE`, `EmitStatus`) | enum Rust deserializzato da RON | n/a — già enum tipato | alta | **enum tipato** (è già così) |
| Param refs in Command (`mul_param: "atk_mul"`, `dur_param: "burn_duration"` in `clipmontage.ron` FSM nodes) | stringa che indicizza `skills.ron::params` | **alta** — typo silenzia il numero, blueprint legge `None` e applica 0 | media (interprete + blueprint executor) | **stringhe + contract test §L** (vedi §2.2b §L validator: ogni `*_param` deve esistere nei params della skill proprietaria) |
| `Predicate::Unlock(NodeId)` in edges FSM | stringa che indicizza `skill_tree.ron::nodes` | **alta** se skill_tree implementato — typo = edge mai matcha | media (interprete `tick_fsm`) | **stringhe + contract test** quando skill_tree è runtime (deferred §2.2b §I/§O); in M017 niente runtime → only validator warning |

**Regola:** quando il valore è un'**istruzione tipata** ("modifica il playhead", "applica danno") → enum Rust con variant. Quando il valore è un **tag indirizzato** (chiave di lookup verso asset/registry) → stringa. Le stringhe high-risk (signal/skill/status) vengono protette da contract test, non da type system.

**C — Contract test al boot (M017):**

Un solo system di startup `validate_data_contracts` che, quando i RON sono caricati, esegue un audit:

- Ogni `signal_bindings.ron::skill_id` esiste in `skills.ron`
- Ogni `signal_bindings.ron::signal` ha un blueprint subscribed (registry interno)
- Ogni `skills.ron::Effect::ApplyStatus(name)` esiste in `status_effects.ron`
- Ogni `unit_stats.ron::owner_id` esiste in `BLUEPRINTS`
- Ogni `clipmontage.ron::clip_name` esiste in `clip.ron`
- Ogni `units.ron::skill_id` esiste in `skills.ron` (già esiste, lo si estende)
- **FSM cross-validation (§2.2b §L):** per ogni `AnimGraph`, ogni `mul_param`/`dur_param`/`chance_param` in un Command esiste in `skills.ron::params` della skill proprietaria; ogni `Node.frames` range è in-bounds rispetto a `clip.ron::total_frames`; entry node e Exit reachability verificati prima del runtime.

**Failure mode:** in debug → panic con lista dei mismatch. In release → log error + skip dell'unit/skill incoerente (non far crollare la run per un typo). **Bench:** lo stesso pattern già usato da `transitions_for_action_checked` (§2.1). Estensione naturale.

**D — Adjunct gratis: enum-like newtype runtime-registered.**

Compromesso intermedio per M017 senza compile-time codegen: un newtype `SignalId(InternedString)` con costruttore `try_new(&str) -> Result<Self, UnknownSignal>` che valida contro un registry runtime popolato dal contract test. Il consumer scrive `let sig = SignalId::known("build_exploit")` (panic in debug se sconosciuto) ottenendo strong-type *runtime* + autocomplete IDE inesistente ma error early. Costo: ~50 righe di boilerplate. **Opzionale per M017, attivabile in S03 se il volume di signal name è alto.**
