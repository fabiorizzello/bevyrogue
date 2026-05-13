# Project

## What This Is

**bevyrogue** è un roguelite RPG monster-taming turn-based in Rust + Bevy 0.18, headless-first by default, with optional `windowed` egui UI.

Core value: una run giocabile end-to-end dove combat, party build, e futuri layer UI/CLI leggono una sola combat authority.

## Current State

M018 è completato. Il combat kernel ora supporta AdvanceTurn/DelayTurn split e TargetShape resolver expansion (Blast, AoE(All), Bounce(N)), le due foundation primitives che sbloccano la maggior parte delle skill identity del roster in arrivo.

Latest combat baseline:

- All M018 slices (S01–S03) are complete: AdvanceTurn(u32)/DelayTurn(u32) split with ±50% cap at emission; SlotIndex(u8) component + pure resolve_targets(); Blast (primary + adjacent slot±1), AoE(All) alias for AllEnemies, Bounce(N) with BounceSelector/RepeatPolicy/DamageCurve.
- `cargo test` green: 201 tests across all integration and lib targets, 0 failed, 0 ignored.
- All M017 regression tests (status_slowed_delay, tempo_resistance, turn_advance_split) remain green.
- CLI scenario `advance-delay-cap`: turn order recalculated step-by-step with cap enforcement (80→50) and floor clamp (delta=0 at AV=0) visible in JSONL.
- CLI scenario `aoe-blast`: Blast targeting deterministic across 10 runs (byte-for-byte identical JSONL).
- Bounce hop loop rebuilds TargetableSnapshot each hop; KO'd units excluded from candidate pool in subsequent hops.

## Architecture / Key Patterns

- **Headless-first Bevy:** default features avoid UI/windowing.
- **Combat authority:** action query + turn pipeline + resolution + kernel/hooks decide legality, timing, damage, and state.
- **Typed kernel:** Tactical Cycle, Strain, Flow, Fatigue, beats, tags, and mechanic transitions live in typed Rust.
- **Content layer:** RON owns numbers, tags, target shape, scaling, sp/ult costs, and presentation metadata — **no skill logic**. Skill behavior lives in Rust (target post-M021: `trait Skill::resolve(&mut SkillCtx, &Params)`, see D010). Custom-signal handling is a blueprint concern in Rust, not a RON layer.
- **Blueprint seam:** unique Digimon behavior lives in per-Digimon Rust modules that produce generic kernel intents via `SkillCtx` (target post-M021); kernel resta unico esecutore degli `Intent` (formula damage, mitigation, break, status tick).
- **Event bus:** `CombatEvent` is the canonical consumer stream.
- **Validation snapshots:** diagnostic state surface for tests, CLI, UI, and future tools.
- **Legality contract:** shared query vocabulary in `docs/contracts/skill_legality_contract.md` and `docs/contracts/combat_ui_readiness_gap_matrix.md`; no skill-ID-specific CLI/windowed legality rules.
- **StatusBag:** per-unit consolidated component with single-instance-per-kind enforcement at apply(). BuffKind-classified cleanse (Buff entries immune by default). Reserved Burn/Shock variants declared but rejected at load-time by RON allow-list.
- **Status semantics (§H.1):** Heated = DoT 4 Fire + fire amp%; Chilled = −20% AV (derived-read at AV-gain site) + ice amp%; Paralyzed = action-dispatch gated in process_turn_advanced_system; Slowed = DelayTurn{30} on first apply; Blessed = ×1.15 damage dealt + +1 Ult charge per action + cleanse-immune.
- **TargetShape resolver (M018):** pure `resolve_targets(TargetableSnapshot)` fn handles Single/Blast/AllEnemies; pure `select_bounce_hop(TargetableSnapshot)` fn handles Bounce selector dispatch. Both are ECS-free and directly unit-testable. SlotIndex(u8) component assigned post-spawn by apply_composition.
- **Turn manipulation (M018):** `AdvanceTurn(u32)` + `DelayTurn(u32)` replace `TurnAdvance(i32)`. Cap ±50% enforced at emission; consumers never see an unclamped value. Resource consumption (SP/ult/streak) hoisted before per-target loop — consumed once per cast regardless of fan-out width.

## Capability Contract

See `.gsd/REQUIREMENTS.md`. Active requirements: none. Current validated baseline: M018 AdvanceTurn/DelayTurn split + TargetShape resolver expansion. M016 per-Digimon blueprint migration, M017 status taxonomy, and M018 turn/targeting foundations are all complete.

## Milestone Sequence

M016 (blueprint migration) → M017 (status taxonomy §H.1) → M018 (turn manipulation + targeting expansion) → **next**

## Recommended Next Milestone

**M019: DR pipeline + Heal/Cleanse Effects**, or **M020: reactive event variants**.

Per la boundary map di M018: il passo successivo naturale è M019 (DR pipeline BuffKind::DR + Heal/Cleanse Effects come Effect variant nel DSL RON), oppure M020 (reactive event variants tipizzati per follow-up triggers). Entrambi consumano la foundation multi-target di M018.

Known deferred items from M018 (plan into a follow-up slice):
- Per-hop CombatEvent emission for Bounce (UI/log observability of intermediate hop state)
- OnActionFailed on Bounce pool exhaustion (currently silent truncation)
- DamageCurve::PerHop runtime length guard in the kernel hop loop

## Operational Notes

- Use `docs/combat_current.md` as first read for combat work.
- Use `scripts/verify_combat_authority_audit.py` after changing authority docs or seams.
- Use `scripts/verify_m015_failure_ledger.py` after changing M015 closure proof docs.
- Use `cargo test` for broad verification before claiming baseline health.
- Status taxonomy reference: `src/combat/status_effect.rs` (StatusEffectKind enum, StatusBag, apply/tick/expire).
- RON status id allow-list: `src/data/skills_ron.rs` (`validate_skill_book_on_load`, 5 valid ids).
- TargetShape resolver: `src/combat/resolution.rs` (resolve_targets), `src/combat/turn_system/pipeline.rs` (Bounce hop loop), `src/combat/action_query.rs` (select_bounce_hop).
- Turn manipulation: `src/combat/av.rs` (AdvanceTurn/DelayTurn applicators with cap/floor).
