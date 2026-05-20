---
estimated_steps: 12
estimated_files: 2
skills_used: []
---

# T01: Add phase-strip display model and pure label contract

Why: The phase strip needs a narrow, testable model before egui wiring so display semantics are proven without a GPU or display server. This task follows the planning skills grill-me, tdd, write-docs, bevy, rust-best-practices, and verify-before-complete: define the contract first, implement the minimal pure helpers, and document the boundary in code comments.

Do:
1. Create `src/ui/phase_strip.rs` behind `#[cfg(feature = "windowed")]` exports from `src/ui/mod.rs`.
2. Define a small UI-owned display state/resource, e.g. `PhaseStripDisplay`, that stores only the current `CombatBeatId` plus UI timing/visibility fields if needed; it must not reference `CombatState`, `Unit`, `TurnOrder`, or action pipeline types.
3. Add pure helper(s) that map `CombatBeatId` to Section 9 phase labels. Include all current beat variants: `Declared`, `PreApp`, `Impact`, `Damage`, `ExtraHit`, `Applied`, and `Resolved`; S03 can group `Damage`/`ExtraHit` as impact/chain-adjacent display labels but must make that mapping explicit.
4. Add unit tests in the same module or a focused integration test that assert every `CombatBeatId::ALL` value has a stable non-empty label and expected canonical labels for the core cycle (`Declared`, `PreApp`, `Impact`, `Applied`, `Resolved`).

Done when: The pure phase-strip contract compiles with `--features windowed`, is hidden from non-windowed builds, and its tests prove no beat variant silently falls through.

Threat Surface (Q3): No auth, secrets, filesystem, network, or untrusted user input; attack surface is limited to rendering event payload labels from internal enum values.
Requirement Impact (Q4): Supports R002, R004, R005, and M002 S03 acceptance; does not revisit D025. Re-verify headless and windowed tests.
Failure Modes (Q5): If Bevy/egui feature gating is malformed, headless `cargo test` fails; fix by keeping all egui/windowed imports behind `#[cfg(feature = "windowed")]`. If a new beat is added later, the all-beats test should fail until a label is chosen.
Load Profile (Q6): Per-event cost is trivial string/enum mapping; no shared mutable combat resources and no allocation-heavy path required.
Negative Tests (Q7): Empty/no beat state should render no active phase; all known beat variants must produce non-empty labels.

## Inputs

- `src/ui/mod.rs`
- `src/combat/observability/events.rs`
- `src/combat/kernel/primitives.rs`
- `docs/future_design_draft/09_ui_surface.md`

## Expected Output

- `src/ui/phase_strip.rs`
- `src/ui/mod.rs`

## Verification

cargo test --features windowed phase_strip

## Observability Impact

Creates pure phase-label helpers and UI-owned display state that can be asserted in tests without opening the windowed app.
