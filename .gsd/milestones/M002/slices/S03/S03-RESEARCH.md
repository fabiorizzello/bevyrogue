# M002 / S03 — Section 9 phase strip live (event-driven) — Research

**Date:** 2026-05-19

## Summary

S03 wires the §9 "phase indicator" UI from the combat event stream. The phase strip (a small banner at top-center) cycles through action phases (`Windup` → `Strike` → `ReactiveDetonate/Echo/Chain` → `Recovery`) driven purely by reading `CombatEvent::OnCombatBeat` and its nested `CombatBeatId` enum (`Declared`, `PreApp`, `Impact`, `Applied`, `Resolved`). The critical constraint is structural: **the UI path must never mutate combat state**. It reads `EventReader<CombatEvent>`, resolves phase identity from event **payloads only**, and renders a stateless banner. A compile-time or runtime structural test asserts that no `ResMut<CombatState>`, `ResMut<Unit>`, or mutable combat queries exist on the phase-strip system.

This is S03's highest-value de-risker: proving that the UI contract (read-only event consumer) is enforceable and testable before S05–S06 add complexity. S03 depends only on S01 (AnimGraph/stance foundation) and will ship a **first proof** that the windowed UI path stays query-only (no gameplay mutations).

## Active Requirements / Constraints

- **R002 (Headless-first):** phase strip system must compile in headless mode (even if unused); gated behind `#[cfg(feature="windowed")]` for integration.
- **R004 (Determinism):** event stream is deterministic (kernel produces seeded timeline); UI layer adds no wall-clock or RNG.
- **R005 (No windowed deps outside feature):** phase-strip system lives in `src/ui/` and is entirely `#[cfg(feature="windowed")]` with no egui/winit re-exports in headless.
- **R006 (Repo hygiene):** no `.md` files in repo root; RESEARCH artifact in `.gsd/milestones/M002/slices/S03/`.
- **Stand requirement from roadmap (M002 Acceptance Criteria, S03 line):** "§9 phase strip updates from `EventReader<CombatEvent>`; structural test asserts the UI path never mutates combat state."

## Implementation Landscape

### Key Files

| File | Purpose | Status |
|------|---------|--------|
| `src/combat/observability/events.rs` | `CombatEvent` struct + `CombatEventKind` enum (150+ lines); `OnCombatBeat{beat: CombatBeatId}` variant | existing, complete |
| `src/combat/kernel/primitives.rs` | `CombatBeatId` enum (`Declared`, `PreApp`, `Impact`, `Applied`) with `as_str()` method | existing, complete |
| `src/windowed.rs` | `UiPlugin` struct, `build()` impl, `roster_panel` + `turn_order_panel` + `combat_panel` systems | existing; phase strip system attaches here |
| `src/ui/combat_panel/mod.rs` | `PendingAction` + `PreviewDamageCache` resources; conditional compilation `#[cfg(feature="windowed")]` | existing pattern for UI resources |
| `src/ui/combat_panel/render.rs` | `combat_panel` system already reads `CombatState` (includes phase) | existing; phase-strip will be separate |
| `docs/future_design_draft/09_ui_surface.md` | §9.4 specifies phase indicator: small banner top-center, only if phase duration >300ms | design doc |
| **New** `src/ui/phase_strip.rs` | Module containing `PhaseStripSystem` function to read `EventReader<CombatEvent>` and derive current phase (no mutations) | to be created |

### Emitters and Readers (Event Flow)

**Emitters (kernel):**
- `src/combat/turn_system/resolve.rs` and related: emit `CombatEvent::OnCombatBeat{beat}` at lifecycle seams.

**Current readers:**
- Follow-up reactive-signature logic reads `OnCombatBeat` internally; **no existing UI reader** for phase strip.

### Phase Concept: From Events to Display

The `CombatBeatId` enum (`Declared`, `PreApp`, `Impact`, `Applied`, `Resolved`) maps directly to action phases. The phase-strip system reads `EventReader<CombatEvent>`, filters for `OnCombatBeat` variants, tracks elapsed time, and renders an egui banner if duration > 300ms. **No combat state mutation occurs.**

## Natural Seams

### 1. Phase-Strip System

**Location:** `src/ui/phase_strip.rs` (new module).

**Attachment point:** `src/windowed.rs`, `UiPlugin::build()`, `EguiPrimaryContextPass` schedule.

### 2. Phase-Strip Resource

**Optional resource:** `PhaseStripDisplay{current_beat: Option<CombatBeatId>, since_secs: f32}` initialized in `UiPlugin::build()`.

### 3. Minimal Scope

For S03, phase strip shows only the **core tactical cycle beats** (Declared, PreApp, Impact, Applied, Resolved). Reactive-signature labels defer to S05.

## First Proof

**Highest-value de-risker:** A compile-time assertion using `ReadOnlySystemParam` trait bounds on the `phase_strip_system` function, paired with a minimal integration test that sends fake events and verifies zero combat mutations.

**Verification command:**
```bash
cargo test --test phase_strip_readonly --features windowed
```

**One-liner:** Compile-time `ReadOnlySystemParam` trait bounds on `phase_strip_system` function prove the UI path never mutates combat state, validated by integration test.

## Verification

| Command | Expectation | Gate |
|---------|-------------|------|
| `cargo test` (headless) | phase-strip code compiles with cfg guards; no egui/winit in headless | R002, R005 |
| `cargo test --test phase_strip_readonly --features windowed` | `ReadOnlySystemParam` succeeds; integration test asserts no mutations | First Proof |
| `cargo build --features windowed` | full windowed build with phase-strip integrated | integration check |
| `cargo run --features windowed` | phase-strip banner visible at top-center, cycles through beat names | visual smoke test |

## Risks / Unknowns

1. **Event ordering:** Confirm `CombatBeatId::Declared` event timing aligns with "Windup" phase semantics by inspecting `turn_system/resolve.rs`.

2. **Elapsed-time tracking:** Banner visibility depends on phase duration > 300ms. Only `PhaseStripDisplay` resource mutates (not combat state). Ensure time comparison is frame-rate agnostic.

3. **Reactive signature scope:** The design specifies "Reactive: {name}" labels, but those are separate event kinds (`OnStatusApplied`, `OnKill`). S03 covers only core beats; S05 adds reactive labels. Confirm planner understands boundary.

4. **Event reader independence:** Multiple systems reading `EventReader<CombatEvent>` each get independent cursors; no event "stealing" risk.

5. **Test isolation:** A minimal `phase_strip_readonly` test boots only `UiPlugin` with fake events, avoiding full combat pipeline coupling.

## Skills Discovered

| Technology | Skill | Status |
|------------|-------|--------|
| Bevy ECS (system params, read-only queries) | bevy-ecs-expert | installed — for `EventReader`, `ReadOnlySystemParam`, `UiPlugin` integration |
| Rust trait bounds | rust-advanced | existing — for readonly-system proof trait |

## Sources

- **Local code:** `src/combat/observability/events.rs` (CombatEvent + CombatEventKind), `src/combat/kernel/primitives.rs` (CombatBeatId), `src/windowed.rs` (UiPlugin), `docs/future_design_draft/09_ui_surface.md` (§9.4 spec).
- **M002 context:** `.gsd/milestones/M002/M002-CONTEXT.md` (S03 acceptance criteria), `.gsd/milestones/M002/slices/S01/S01-RESEARCH.md` (plugin split pattern).
