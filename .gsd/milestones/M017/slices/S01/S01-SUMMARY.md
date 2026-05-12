---
id: S01
parent: M017
milestone: M017
provides:
  - StatusKind canon vocabulary (Heated/Chilled/Paralyzed/Slowed/Blessed + reserved Burn/Shock)
  - RON validator enforcing 5-id allow-list at load-time
  - Clean test suite baseline for S02-S05 semantic expansion
requires:
  []
affects:
  []
key_files:
  - src/combat/status_effect.rs
  - src/data/skills_ron.rs
  - assets/data/skills.ron
  - assets/data/units.ron
  - src/combat/speed.rs
  - src/combat/battery_loop.rs
  - src/combat/rng.rs
  - src/combat/observability.rs
  - src/combat/kernel.rs
  - src/combat/turn_system/mod.rs
  - src/combat/turn_system/tests.rs
  - tests/status_effect_apply.rs
  - tests/status_effect_integration.rs
  - tests/status_effect_turn_tick.rs
  - tests/combat_coherence.rs
  - tests/status_accuracy.rs
  - tests/follow_up_chains.rs
  - tests/form_identity.rs
key_decisions:
  - Reserved Burn/Shock declared in StatusEffectKind enum but rejected at load-time by the RON validator — not silently no-op, fail-fast per §H.1
  - Legacy semantic test assertions (DoT, SpeedModifier, action cancel) removed rather than #[ignored] — delete-and-rewrite-fresh approach for S03-S05
  - assets/ cosmetic matches (skill name 'Freeze Fang', units.ron role comments) confirmed out-of-scope per slice DoD which targets src/ and tests/
  - 0 ignored tests — all lifecycle assertions (apply/tick/expire) updated to canon names and kept green
patterns_established:
  - Status taxonomy vocabulary split: 5 active canon variants vs 2 reserved gas-era variants (Burn/Shock) declared but not applicable in v0
  - Load-time validator pattern: RON ids validated against explicit allow-list at load time, rejecting both legacy and reserved ids with a clear error message
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-12T16:51:44.121Z
blocker_discovered: false
---

# S01: Enum rewrite + RON migration + tests cascade

**Replaced legacy Burn/Freeze/Shock/DeepFreeze status taxonomy with the 5 canon §H.1 variants (Heated/Chilled/Paralyzed/Slowed/Blessed) + reserved Burn/Shock; migrated all src/, tests/, and assets/; cargo check + cargo test + smoke CLI all green.**

## What Happened

S01 replaced the entire legacy status taxonomy (Burn/Freeze/Shock/DeepFreeze) with the §H.1 canon vocabulary across the full codebase in six sequential tasks.

**T01** rewrote `StatusEffectKind` in `src/combat/status_effect.rs`: replaced legacy variants with `Heated`, `Chilled`, `Paralyzed`, `Slowed`, `Blessed` plus `Burn` and `Shock` declared as reserved §H.1 (no-op, documented inline). apply/refresh/tick skeleton preserved as single-instance per (target,kind) with refresh_max_dur semantics (no per-status effects — those are S03-S05 scope).

**T02** updated `src/data/skills_ron.rs`: `Effect::ApplyStatus` now accepts the 5 canon ids ('heated'/'chilled'/'paralyzed'/'slowed'/'blessed') and hard-rejects legacy ids at load-time with a clear error listing all valid ids. Reserved 'burn'/'shock' are also rejected (not applicable in v0).

**T03** migrated `assets/data/skills.ron` (11 occurrences) and `assets/data/units.ron` (3 occurrences) using the canonical map Burn→Heated, Freeze→Chilled, Shock→Paralyzed, DeepFreeze→Slowed. Skill names containing "Freeze" (e.g. "Freeze Fang") were untouched — those are cosmetic names, not status ids.

**T04** cascade-renamed 8 src/combat files: `speed.rs`, `battery_loop.rs`, `rng.rs`, `observability.rs`, `kernel.rs`, `turn_system/mod.rs`, `turn_system/tests.rs`. Reserved Burn/Shock match arms in the turn_system pipeline left intact as no-op branches per §H.1.

**T05** migrated 7 test files. For status semantic tests (status_effect_apply.rs, status_effect_integration.rs, status_effect_turn_tick.rs, status_accuracy.rs) all assertions on per-status effects (DoT damage, SpeedModifier, action cancel) were removed as S03-S05 scope — lifecycle assertions (apply/tick/expire) remain. No `#[ignore]` was needed; the approach was delete-and-rewrite-fresh as specified. follow_up_chains.rs, combat_coherence.rs, and form_identity.rs had their incidental references updated with no logic change.

**T06** confirmed the grep guard, full test suite, and smoke CLI all pass. Zero legacy references in src/ and tests/. The `assets/` residuals (units.ron thematic comments, skills.ron "Freeze Fang" skill name) are out-of-scope per slice DoD. All tests pass with 0 failed and 0 ignored. `cargo run --bin combat_cli` emits correct combat events with no panics.

## Verification

1. Grep guard: `grep -rEn '\b(Burn|Freeze|Shock|DeepFreeze)\b' src/ tests/` — only reserved Burn/Shock variants remain in src/ (enum declaration, inline unit tests for reserved roundtrip, validator guard, no-op match arms in turn_system). tests/ is clean (zero matches).
2. `cargo check` — exit 0, no warnings about unknown variants or missing match arms.
3. `cargo test` (full integration suite) — all targets pass: 0 failed, 0 ignored, 0 filtered. All legacy semantic test assertions removed cleanly as S03-S05 scope.
4. `cargo run --bin combat_cli` — exit 0, headless smoke run completes, combat events flowing (OnKernelTransition, OnDamageDealt, OnHitTaken, UltGain), no panics, no status-taxonomy-related errors.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

None.

## Follow-ups

["S02: Apply/refresh_max_dur/cleanse policy — single-instance enforcement and Debuff-only cleanse filter", "S03: Heated DoT amp% pipeline + Chilled −20% speed via SpeedModifier", "S04: Paralyzed skip-turn + Slowed delay-on-apply", "S05: Blessed buff dealt + Ult charge + cleanse-immune", "S06: Observability — JSONL log + ValidationSnapshot emitting canon names"]

## Files Created/Modified

None.
