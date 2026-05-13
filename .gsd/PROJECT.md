# Project

## What This Is

**bevyrogue** è un roguelite RPG monster-taming turn-based in Rust + Bevy 0.18, headless-first by default, with optional `windowed` egui UI.

Core value: una run giocabile end-to-end dove combat, party build, e futuri layer UI/CLI leggono una sola combat authority.

## Current State

M017 è completato. Il combat kernel ora usa la tassonomia status canon §H.1: 5 status attivi (Heated/Chilled/Paralyzed/Slowed/Blessed) + 2 reserved gas-era (Burn/Shock), con policy single-instance per (target,kind), refresh_max_dur, BuffKind-classified cleanse, e tutte le 5 semantiche per-status cablate. JSONL log e ValidationSnapshot emettono nomi canon. RON loader rifiuta id non-canon a load-time.

Latest combat baseline:

- All M017 slices (S01–S06) are complete: enum rewrite, StatusBag + cleanse policy, Heated DoT + amp%, Chilled −20% AV, Paralyzed skip-turn, Slowed delay-on-apply, Blessed ×1.15 + Ult charge, JSONL/ValidationSnapshot observability.
- `cargo check` + `cargo test` green: 40 test binaries, 0 failed, 0 ignored.
- Zero references to Freeze/DeepFreeze in src/ and tests/; Burn/Shock present only in 7 canonical exempt locations.
- Status foundation is ready for M018 (AdvanceTurn/DelayTurn split + TargetShape expansion), M019 (DR pipeline + Heal/Cleanse Effects), and M020 (reactive event variants).

## Architecture / Key Patterns

- **Headless-first Bevy:** default features avoid UI/windowing.
- **Combat authority:** action query + turn pipeline + resolution + kernel/hooks decide legality, timing, damage, and state.
- **Typed kernel:** Tactical Cycle, Strain, Flow, Fatigue, beats, tags, and mechanic transitions live in typed Rust.
- **Content layer:** RON owns data, numbers, target declarations, metadata, and typed custom-signal intent; not final gameplay authority.
- **Blueprint seam:** unique Digimon behavior belongs in per-Digimon Rust modules that emit generic kernel transitions.
- **Event bus:** `CombatEvent` is the canonical consumer stream.
- **Validation snapshots:** diagnostic state surface for tests, CLI, UI, and future tools.
- **Legality contract:** shared query vocabulary in `docs/contracts/skill_legality_contract.md` and `docs/contracts/combat_ui_readiness_gap_matrix.md`; no skill-ID-specific CLI/windowed legality rules.
- **StatusBag:** per-unit consolidated component with single-instance-per-kind enforcement at apply(). BuffKind-classified cleanse (Buff entries immune by default). Reserved Burn/Shock variants declared but rejected at load-time by RON allow-list.
- **Status semantics (§H.1):** Heated = DoT 4 Fire + fire amp%; Chilled = −20% AV (derived-read at AV-gain site) + ice amp%; Paralyzed = action-dispatch gated in process_turn_advanced_system; Slowed = TurnAdvance −30% on first apply; Blessed = ×1.15 damage dealt + +1 Ult charge per action + cleanse-immune.

## Capability Contract

See `.gsd/REQUIREMENTS.md`. Active requirements: none. Current validated baseline: M017 Status taxonomy v0 rewrite (canon §H.1). M016 per-Digimon blueprint migration and M017 status taxonomy are both complete.

## Milestone Sequence

- [x] M001: Combat core giocabile
- [x] M002: Combat hardening
- [x] M004: Bevy 0.18 + headless adoption
- [x] M005: Combat consolidation + event bus + skill DSL
- [x] M006: Roster completo + Taichi + party selection
- [x] M008: Combat Refinement & Polish
- [x] M009: Digimon Synergy & Combat Coherence Analysis
- [ ] M010: Combat Architecture & Synergistic Roster — interrupted, residue migrated later
- [x] M011: Combat Architecture & Synergistic Roster alignment
- [x] M012: Data-driven skill legality and UI-readiness query surface
- [ ] M013: Combat architecture revision + animation beat pipeline — historical closure incomplete; superseded by M015 proof
- [x] M015: M013 Closure and Combat Architecture Coherence
- [x] M016: Per-Digimon Blueprint Migration and Roster Combat Identity
- [x] M017: Status taxonomy v0 rewrite (canon §H.1)
- [ ] M007: Roguelite Loop End-to-End

## Recommended Next Milestone

**M018: AdvanceTurn/DelayTurn split + TargetShape resolver expansion.**

Per la boundary map di M017: il passo successivo è M018 (AdvanceTurn/DelayTurn split, cap ±50%, gauge clamp [0,200], TargetShape resolver expansion), oppure M019 (DR pipeline BuffKind::DR + Heal/Cleanse Effects come variant), oppure M020 (reactive event variants tipizzati). M018 è la scelta naturale perché il foundation del turn pipeline (Slowed TurnAdvance) è già cablato in M017.

Nota: SC-3 (Chilled) chiude con PARTIAL — l'integration test per il turn-order shift visibile di Chilled è opzionale/deferred. M018 può chiuderlo se il turn pipeline viene refactored.

## Operational Notes

- Use `docs/combat_current.md` as first read for combat work.
- Use `scripts/verify_combat_authority_audit.py` after changing authority docs or seams.
- Use `scripts/verify_m015_failure_ledger.py` after changing M015 closure proof docs.
- Use `cargo test` for broad verification before claiming baseline health.
- Status taxonomy reference: `src/combat/status_effect.rs` (StatusEffectKind enum, StatusBag, apply/tick/expire).
- RON status id allow-list: `src/data/skills_ron.rs` (`validate_skill_book_on_load`, 5 valid ids).
