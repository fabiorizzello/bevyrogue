# bevyrogue — Agent Onboarding

Roguelite RPG monster-taming turn-based. Rust + Bevy 0.18 (headless default), bevy_egui dietro feature `windowed`. Combat ispirato a Honkai: Star Rail; meta-loop tipo Slay the Spire / Aethermancer.

## Build & Test

```bash
cargo check                   # default = headless
cargo test                    # full integration suite (tests/)
cargo run                     # headless run
cargo run --features windowed # con UI egui
```

Toolchain: vedi `rust-toolchain.toml`. Dev profile usa `cranelift` (vedi `Cargo.toml`).

## Layout

```
src/
  lib.rs              → re-exports moduli pubblici (combat, data, party_validation)
  main.rs             → App builder, plugin registration, headless/windowed branching
  party_validation.rs → validazione PartyConfig contro UnitRoster
  combat/             → core gameplay (vedi sotto)
  data/               → caricatori RON (units, skills, party)
  ui/                 → bevy_egui combat panel (solo feature windowed)
assets/data/          → RON: units.ron, skills.ron, party.ron
tests/                → integration tests (headless, no UI)
docs/                 → current docs + prior_art/ per design storici
.gsd/                 → workflow GSD (PROJECT.md, REQUIREMENTS.md, DECISIONS.md…)
```

## Combat module map (`src/combat/`)

- `state.rs` — `CombatState`, `CombatPhase` (enum fasi)
- `unit.rs`, `types.rs`, `team.rs`, `speed.rs`, `kit.rs` — components/types base
- `turn_order.rs` — `TurnOrder`, `TurnAdvanced` event
- `turn_system.rs` — `advance_turn_system`, `resolve_action_system`, `check_victory_system`
- `resolution.rs` — applicazione effetti skill
- `damage.rs`, `toughness.rs`, `stun.rs` — danno + break/stun
- `status_effect.rs` — buff/debuff + tick
- `sp.rs`, `ultimate.rs` — economie risorse (SP pool, Ult charge)
- `follow_up.rs` — reazioni follow-up FIFO
- `enemy_ai.rs` — AI nemica (decision routing)
- `bootstrap.rs` — spawn composizione encounter
- `events.rs` — `CombatEvent`/`CombatEventKind` (event bus)
- `log.rs`, `observability.rs`, `jsonl_logger.rs` — logging + snapshots
- `floating.rs` — floating damage display

## Convenzioni

- **Headless first:** ogni system deve girare senza `windowed`. Gating: `#[cfg(feature = "windowed")]` solo per egui/winit.
- **Tests:** integration in `tests/`. Naming **funzionale** (es. `follow_up_triggers.rs`, non `s10_…`). Non aggiungere unit test inline in `src/` salvo `#[cfg(test)] mod tests` brevi.
- **Skill DSL:** RON in `assets/data/skills.ron`, schema in `src/data/skills_ron.rs` (`SkillDef`, `Effect`, `TargetShape`).
- **Eventi:** `CombatEvent` è il bus single-source-of-truth. UI/log leggono eventi, non mutano stato.
- **Determinismo:** tests devono essere deterministici (no wall-clock, no RNG senza seed).

## Where to look

| Vuoi… | File |
|-------|------|
| Modificare bilanciamento | `assets/data/units.ron`, `skills.ron` |
| Aggiungere skill effect | `src/data/skills_ron.rs` (`Effect`) + `src/combat/resolution.rs` |
| Cambiare turn flow | `src/combat/turn_system.rs` |
| Wiring nuovo system | `src/main.rs` (plugin registration) |
| Design intent corrente | `docs/combat_current.md` |
| Stato roadmap | `.gsd/PROJECT.md`, `.gsd/REQUIREMENTS.md` |

## Don't

- Non toccare `Cargo.lock` a mano.
- Non aggiungere dipendenze winit/wgpu/egui fuori da `windowed` feature gate.
- Non scrivere su `assets/data/*.ron.bak` (backup manuali).
- Non riempire root con `.md` — vanno in `docs/` o `.gsd/`.
