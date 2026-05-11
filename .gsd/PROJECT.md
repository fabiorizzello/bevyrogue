# Project

## What This Is

**bevyrogue** è un roguelite RPG monster-taming turn-based in Rust + Bevy 0.18, headless-first by default, with optional `windowed` egui UI.

Core value: una run giocabile end-to-end dove combat, party build, e futuri layer UI/CLI leggono una sola combat authority.

## Current State

M015 è completato e supersede la closure incompleta di M013. M016 has successfully migrated the roster's core combat identity (Battery, Predator, Precision, and Twin Core loops) into per-Digimon Rust blueprints.

Latest combat baseline:

- All M016 slices (S01-S04) are complete, covering Tentomon, Dorumon, Renamon, Agumon, and Gabumon.
- `cargo test` and integration tests for specific loops (Battery, Twin Core) pass.
- Combat authority stack is fully operational: RON data/custom signals -> per-Digimon Rust blueprints -> generic kernel transitions -> canonical state.
- CLI proof continues to pass via `combat_cli`.

## Architecture / Key Patterns

- **Headless-first Bevy:** default features avoid UI/windowing.
- **Combat authority:** action query + turn pipeline + resolution + kernel/hooks decide legality, timing, damage, and state.
- **Typed kernel:** Tactical Cycle, Strain, Flow, Fatigue, beats, tags, and mechanic transitions live in typed Rust.
- **Content layer:** RON owns data, numbers, target declarations, metadata, and typed custom-signal intent; not final gameplay authority.
- **Blueprint seam:** unique Digimon behavior belongs in per-Digimon Rust modules that emit generic kernel transitions.
- **Event bus:** `CombatEvent` is the canonical consumer stream.
- **Validation snapshots:** diagnostic state surface for tests, CLI, UI, and future tools.
- **Legality contract:** shared query vocabulary in `docs/contracts/skill_legality_contract.md` and `docs/contracts/combat_ui_readiness_gap_matrix.md`; no skill-ID-specific CLI/windowed legality rules.

## Capability Contract

See `.gsd/REQUIREMENTS.md`. Active requirements: none. Current validated baseline: M015 Combat Authority Closure Baseline. M016 has advanced the "Full per-Digimon blueprint migration" deferred work.

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
- [ ] M016: Per-Digimon Blueprint Migration and Roster Combat Identity (Validation Pending)
- [ ] M007: Roguelite Loop End-to-End

## Recommended Next Milestone

**M017: Full Roster Validation and Balance Pass**.

Now that the blueprint architecture is in place for the core roster, M017 should focus on verifying all 12 Digimon's behaviors and performing a comprehensive balance pass before proceeding to the roguelite loop.

## Operational Notes

- Use `docs/combat_current.md` as first read for combat work.
- Use `scripts/verify_combat_authority_audit.py` after changing authority docs or seams.
- Use `scripts/verify_m015_failure_ledger.py` after changing M015 closure proof docs.
- Use `cargo test` for broad verification before claiming baseline health.
