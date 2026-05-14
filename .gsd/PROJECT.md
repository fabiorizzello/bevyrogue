# Project

## What This Is

**bevyrogue** è un roguelite RPG monster-taming turn-based in Rust + Bevy 0.18, headless-first by default, with optional `windowed` egui UI.

Core value: una run giocabile end-to-end dove combat, party build, e futuri layer UI/CLI leggono una sola combat authority.

## Current State

M019 è completato. Il combat kernel ora dispone del primo layer numerico di mitigazione difensiva (BuffKind::DR), delle primitive Effect::Heal e Effect::Cleanse nel DSL RON, e di una guardia runtime per DamageCurve::PerHop.

Latest combat baseline:

- All M019 slices (S01–S04) are complete: BuffKind::DR with unclamped summation and multiplicative step in calculate_damage; Effect::Heal { amount_pct_max_hp } with KO no-op, hp_max cap, and OnHealed event; Effect::Cleanse { count: Option<u8> } with duration-descending ordering, immunity via classify_buff_kind, and OnCleansed event; DamageCurve::PerHop runtime length guard (truncate + OnActionFailed diagnostic).
- `cargo test` green: 668 tests across all integration and lib targets, 0 failed, 0 ignored.
- All M018 regression tests remain green.
- New test files: tests/dr_pipeline.rs (6 tests), tests/heal_effect.rs (5 tests), tests/cleanse_effect.rs (8 tests), tests/perhop_guard.rs (1 test).
- M018 follow-up #3 (PerHop length guard) is closed by S04.

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
- **DR mitigation (M019):** `DrBag` component accumulates DR entries with unclamped summation; `calculate_damage` applies factor `(1.0 - sum_dr).max(0.0)` as a final multiplicative step after break amplification. DR is the defender's last shield — applies to all damage paths including DoT tick. Observability: `CombatEvent::Damage` carries both `pre_dr` and `final` amounts.
- **Heal/Cleanse primitives (M019):** `Effect::Heal { amount_pct_max_hp }` and `Effect::Cleanse { count: Option<u8> }` available in RON DSL. Pipeline dispatches via `apply_heal_only` / `apply_cleanse_only` in resolution.rs. AllAllies fan-out reuses the Blast resource-hoist-then-per-target-dispatch pattern. Mixed Heal+Cleanse on a single skill is rejected by the legality validator (deferred to M021 trait Skill).
- **PerHop guard (M019/D001):** Pre-loop check in pipeline.rs truncates the bounce loop to available coefficients and emits `OnActionFailed` diagnostic when `DamageCurve::PerHop` length < `hops_planned`. Never panics; load-time validator remains the primary defence.

## Capability Contract

See `.gsd/REQUIREMENTS.md`. Active requirements: none. Current validated baseline: M019 DR pipeline + Heal/Cleanse primitives + PerHop guard. M016 per-Digimon blueprint migration, M017 status taxonomy, M018 turn/targeting foundations, and M019 mitigation/support primitives are all complete.

## Milestone Sequence

M016 (blueprint migration) → M017 (status taxonomy §H.1) → M018 (turn manipulation + targeting expansion) → M019 (DR pipeline + Heal/Cleanse + PerHop guard) → **next**

## Recommended Next Milestone

**M020: support blueprint kit implementation** (Patamon holy_aegis, Gabumon fur_cloak) — the first skills using Effect::Heal, Effect::Cleanse, and BuffKind::DR primitives introduced in M019.

Alternatively: **M021: trait Skill + SkillCtx abstraction** (ATK-based Heal scaling, selective cleanse by StatusKind, mixed Heal+Cleanse, custom immunity hooks) — the generalization layer currently deferred from M019.

Deferred items from M019:
- Buff expiry events when DrBag entry ticks to zero (general buff-expiry event system — later milestone)
- RON `Effect::DR` variant (DR lives at component/formula level only in M019)
- Selective cleanse by StatusEffectKind (deferred to M021)
- Mixed Heal+Cleanse on a single skill (deferred to M021)
- Load-time PerHop coefficient count vs hops_planned check (deferred to M021)

## Operational Notes

- Use `docs/combat_current.md` as first read for combat work.
- Use `scripts/verify_combat_authority_audit.py` after changing authority docs or seams.
- Use `scripts/verify_m015_failure_ledger.py` after changing M015 closure proof docs.
- Use `cargo test` for broad verification before claiming baseline health.
- Status taxonomy reference: `src/combat/status_effect.rs` (StatusEffectKind enum, StatusBag, apply/tick/expire).
- RON status id allow-list: `src/data/skills_ron.rs` (`validate_skill_book_on_load`, 5 valid ids).
- TargetShape resolver: `src/combat/resolution.rs` (resolve_targets), `src/combat/turn_system/pipeline.rs` (Bounce hop loop), `src/combat/action_query.rs` (select_bounce_hop).
- Turn manipulation: `src/combat/av.rs` (AdvanceTurn/DelayTurn applicators with cap/floor).
- DR bag: `src/combat/buffs.rs` (DrBag, DrEntry); damage integration: `src/combat/damage.rs`.
- Heal/Cleanse resolution: `src/combat/resolution.rs` (apply_heal_only, apply_cleanse_only); pipeline wiring: `src/combat/turn_system/pipeline.rs`.
- When adding new components to ResolveActorsQuery in resolution.rs, also update follow_up.rs's local query to avoid tuple-arity compile errors.
