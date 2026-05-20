# S03: Section 9 phase strip live (event-driven)

**Goal:** Make the Section 9 phase strip live in the windowed UI by deriving its display state from CombatEvent::OnCombatBeat messages only, while proving the phase-strip UI path does not mutate combat state.
**Demo:** §9 phase strip updates from EventReader<CombatEvent>; a structural test asserts the UI path never mutates combat state.

## Must-Haves

- Windowed UI shows a top-center phase strip/banner that updates from `MessageReader<CombatEvent>` / Bevy event-reader semantics and maps `CombatBeatId` payloads to player-facing phase labels.
- The phase-strip display state is isolated to UI-owned resource state; it never writes `CombatState`, `Unit`, turn queues, or other combat resources/components.
- A feature-gated integration/structural test asserts the phase-strip system can run over fake combat events without changing combat state, and includes a compile-time read-only system-param proof for the event-to-display path.
- Headless `cargo test` remains green and windowed build/tests compile without leaking egui/windowed dependencies into non-windowed code.

## Proof Level

- This slice proves: Contract plus integration proof. Real runtime display is wired through the actual windowed `UiPlugin`; automated proof is `cargo test --test phase_strip_readonly --features windowed`, `cargo test`, and `cargo build --features windowed`. Human/UAT is optional for visual polish and not required to satisfy the slice contract.

## Integration Closure

Consumes existing `CombatEventKind::OnCombatBeat { beat }` from `src/combat/observability/events.rs`, `CombatBeatId` from `src/combat/kernel/primitives.rs`, and the existing windowed `UiPlugin` in `src/windowed/mod.rs`. Introduces `src/ui/phase_strip.rs` plus `src/ui/mod.rs` export and `UiPlugin` wiring. Downstream S05/S06 can add reactive labels and final windowed smoke; S03 closes the read-only phase indicator contract.

## Verification

- Adds an inspectable UI-owned phase display resource and pure label helpers so future agents can verify phase mapping without a display server. Failure visibility comes from focused tests that feed fake `CombatEvent` messages and assert the derived phase label/state while checking combat state snapshots remain unchanged. No secrets or PII are involved.

## Tasks

- [x] **T01: Add phase-strip display model and pure label contract** `est:45m`
  Why: The phase strip needs a narrow, testable model before egui wiring so display semantics are proven without a GPU or display server. This task follows the planning skills grill-me, tdd, write-docs, bevy, rust-best-practices, and verify-before-complete: define the contract first, implement the minimal pure helpers, and document the boundary in code comments.
  - Files: `src/ui/phase_strip.rs`, `src/ui/mod.rs`
  - Verify: cargo test --features windowed phase_strip

- [x] **T02: Wire EventReader-driven egui phase strip into UiPlugin** `est:1h`
  Why: The slice demo requires the live windowed UI path to consume the combat event stream, not a hardcoded combat-state phase. This task follows grill-me, tdd, write-docs, bevy, rust-best-practices, and verify-before-complete: keep the runtime system narrow, make its system params read-only except UI-owned state, and attach it to the existing egui pass.
  - Files: `src/ui/phase_strip.rs`, `src/windowed/mod.rs`
  - Verify: cargo build --features windowed

- [x] **T03: Prove phase-strip UI path is combat-read-only** `est:1h`
  Why: S03's acceptance hinges on a structural proof that the UI path never mutates combat state. This task follows grill-me, tdd, write-docs, bevy, rust-best-practices, and verify-before-complete: encode the boundary as an executable regression test rather than relying on review.
  - Files: `tests/phase_strip_readonly.rs`, `src/ui/phase_strip.rs`
  - Verify: cargo test --test phase_strip_readonly --features windowed

- [x] **T04: Close S03 verification across headless and windowed gates** `est:30m`
  Why: S03 touches feature-gated UI code, so closeout must prove both the default headless contract and the windowed integration contract. This task follows verify-before-complete plus write-docs by producing fresh verification evidence and leaving concise code comments where the read-only boundary matters.
  - Verify: cargo test

## Files Likely Touched

- src/ui/phase_strip.rs
- src/ui/mod.rs
- src/windowed/mod.rs
- tests/phase_strip_readonly.rs
