# S04 Research: DamageCurve::PerHop runtime length guard

**Depth:** Light — established local pattern, single small hop kernel, decision is binary (fail-fast vs clamp).

## Summary

Add a runtime guard in the Bounce hop kernel for the case `DamageCurve::PerHop(v).len() < hops_planned`. Today the load-time validator in `skills_ron.rs:416-423` rejects mismatched PerHop curves at RON parse, but a dynamically constructed `ResolvedAction` (e.g. from a future Digimon blueprint emitter or test fixture) bypasses that gate. The runtime kernel currently silently clamps the index to the last element via `hop.min(v.len().saturating_sub(1))` (resolution.rs:328-332) — no panic, but no diagnostic either. The slice must either keep clamping with a diagnostic event, or fail-fast (abort/truncate the bounce loop) with a diagnostic event, and record the chosen policy in DECISIONS.md.

## Recommendation

**Adopt fail-fast-with-truncate + diagnostic event** at the start of the Bounce branch in `pipeline.rs` (pre-loop), not inside `compute_hop_damage`:

1. After resolving `curve = &inflight.action.damage_curve` and `hops` (pipeline.rs:801, 812), insert a gate: if `let DamageCurve::PerHop(v) = curve` and `v.len() < hops as usize`, emit a diagnostic `CombatEvent` and clamp `effective_hops = v.len()` (truncate the loop) so the action still resolves to completion without panic.
2. Keep `compute_hop_damage`'s defensive `.min()` clamp as a belt-and-suspenders safeguard but fix its doc-comment (currently lies about debug panic).
3. Decision recorded in DECISIONS.md: "PerHop runtime mismatch → diagnostic event + truncate to v.len(); kernel never panics; loader gate remains primary defence."

Rationale for fail-fast-truncate over silent clamp:
- Silent clamp masks bugs in blueprint emitters that M020+ will introduce — exactly what M018-LEARNINGS warns against.
- Truncating preserves total resolution (no half-applied action, no panic, the action still bounces what it can) and matches the existing `select_bounce_hop → None ⇒ break` ("pool exhausted, silent truncate") pattern at pipeline.rs:844.
- A diagnostic event makes the bug observable in JSONL traces, which the project values (combat events are the single source of truth).

Event choice: reuse `CombatEventKind::OnActionFailed { reason: String }` with `reason: "DamageCurve::PerHop length {n} < hops {h}"` — keeps the event taxonomy stable and aligns with how other kernel anomalies are reported (SP shortfall, attacker stunned/KO). Adding a dedicated variant is possible but unnecessary churn for a kernel-only invariant.

## Implementation Landscape

### Files to touch

- **`src/combat/turn_system/pipeline.rs`** — Bounce branch (~lines 674-812). Add pre-loop length check on `PerHop`. Compute `effective_hops`. Emit `OnActionFailed` (or a dedicated diagnostic if planner chooses) once before the loop.
- **`src/combat/resolution.rs:309-334`** — `compute_hop_damage`. Fix the doc-comment lie about debug panic. Keep the defensive `.min()` clamp as a second line of defence. Optionally promote to `debug_assert!(idx < v.len())` if the kernel-level gate is in place upstream — but the test should drive this.
- **`tests/perhop_guard.rs`** *(new)* — Integration test per the demo line: construct a skill / ResolvedAction with `DamageCurve::PerHop(vec![30, 20])` and `hops=3`, run through the bounce pipeline, assert (a) no panic, (b) diagnostic event present in CombatEvent stream, (c) only 2 hops resolved with damages `[30, 20]`, (d) no fourth-hop "ghost" damage.

### Files NOT to touch

- `src/data/skills_ron.rs` validator (lines 416-423) — load-time gate stays as primary defence; no changes needed.
- `src/combat/damage.rs`, `events.rs` if reusing `OnActionFailed`. Add a new variant only if planner picks a dedicated event name (e.g. `OnDamageCurveTruncated`) — not recommended.

### Natural seams (potential single task or split into 2)

- **Seam A (kernel guard + doc fix):** edit pipeline.rs pre-loop gate; fix resolution.rs doc-comment; ensure `compute_hop_damage` keeps clamping. ~30 lines.
- **Seam B (test):** new `tests/perhop_guard.rs`. Construct skill via RON or directly via `ResolvedAction` to bypass the loader gate (per M018-LEARNINGS, the runtime guard exists precisely for dynamically constructed actions — the test must construct directly without going through `validate_skill_book`).

Single task is fine for low-risk slices like this; split only if the planner wants test-first.

### First proof / biggest unblocker

The test in `tests/perhop_guard.rs`. Once that test is red against the current `pipeline.rs`, the implementation is mechanical. Bypassing the RON validator is the key construction trick — call `apply_effects` or the pipeline entry point with a hand-built `ResolvedAction { damage_curve: DamageCurve::PerHop(vec![30, 20]), target_shape: TargetShape::Bounce { hops: 3, .. }, .. }` directly.

## Verification

- `cargo test --test perhop_guard` — new test (red → green).
- `cargo test` — full integration suite remains green (no regression on existing `target_shape_bounce_chain.rs` cases that use well-formed PerHop or Falloff curves).
- `cargo check --tests` — no exhaustiveness fallout (no new enum variants added if `OnActionFailed` is reused).

## Existing Code / Prior Art

- Hop loop: `src/combat/turn_system/pipeline.rs:812-873`.
- Damage curve resolution: `src/combat/resolution.rs:309-334` (`compute_hop_damage`).
- Load-time validator: `src/data/skills_ron.rs:416-423`.
- Pool-exhaustion truncate pattern: `pipeline.rs:844` (`break` on `select_bounce_hop → None`). Mirror this for length-truncate.
- Diagnostic event pattern: `OnActionFailed { reason }` used for "Attacker is stunned", "SP shortfall" (pipeline.rs:705-770).
- Test pattern: `tests/target_shape_bounce_chain.rs:343-400` builds a Bounce+PerHop skill end-to-end — direct template for constructing a length-mismatched fixture, but the test must skip `validate_skill_book` (or build `ResolvedAction` directly) to reach the runtime path.
- Project memory MEM003: integration tests use direct `apply_effects` calls (no Bevy world spin-up) — applies here.

## Active Requirements Touched

`.gsd/REQUIREMENTS.md` does not contain a requirement specifically for the runtime guard; the closest is the milestone-level success criterion ("DamageCurve::PerHop guard runtime: ... fail-fast con event diagnostico o clamp — decisione di slice"). No requirement-update needed by the planner unless one is to be created.

## Forward Intelligence (for the planner)

- **Decision to lock in DECISIONS.md before T01:** truncate-with-event (recommended) vs silent clamp + event vs hard abort with `OnActionFailed` and zero hops resolved. Recommend the planner make this an explicit one-line decision write before tasks are emitted.
- **Trap:** The doc-comment on `compute_hop_damage` claims debug-panic; the implementation does not. If the planner picks "hard abort" semantics, do not switch `compute_hop_damage` to panic — keep the kernel non-panicking; do the abort upstream by skipping the loop entirely.
- **Trap (test construction):** Going through `SkillBook` + `validate_skill_book` will be rejected at load time — the test will never reach the runtime path. The test MUST construct `ResolvedAction` directly (or use a permissive path that skips validation) to exercise the dynamic-emission scenario M018-LEARNINGS describes.
- **Watch-out:** If the planner adds a new event variant instead of reusing `OnActionFailed`, exhaustive matches in `follow_up.rs`, `jsonl_logger.rs`, `observability.rs`, and `log.rs` need verification — most use wildcards but not all.
- **Out of scope:** PerHop curves with `v.len() > hops` (extra coefficients ignored — already harmless, loop just stops). Don't add a guard for that direction unless the planner explicitly chooses to.

## Skills Discovered

None installed. The work is local kernel hardening; existing project conventions (headless-first, deterministic integration tests, CombatEvent-as-bus) cover it. The bundled `bevy-ecs-expert` and `tdd` skills are tangentially relevant but not load-bearing for a ~30-line guard plus a single test file.
