---
estimated_steps: 13
estimated_files: 2
skills_used: []
---

# T03: Prove phase-strip UI path is combat-read-only

Why: S03's acceptance hinges on a structural proof that the UI path never mutates combat state. This task follows grill-me, tdd, write-docs, bevy, rust-best-practices, and verify-before-complete: encode the boundary as an executable regression test rather than relying on review.

Do:
1. Add `tests/phase_strip_readonly.rs` gated by `--features windowed` that builds a minimal Bevy `App` with `CombatEvent` messages, `CombatState`, and the phase-strip ingest/render-independent system or a testable ingest-only system from `src/ui/phase_strip.rs`.
2. Include a compile-time read-only param proof for the event-to-display system where feasible (for example, assert the reader/query params satisfy Bevy's read-only system-param traits, or expose an ingest function/system split whose combat-facing params are read-only). Because `ResMut<PhaseStripDisplay>` is intentionally UI-owned mutable state, the proof should specifically cover that no combat resource/component writer is part of the event ingestion contract.
3. Send fake `CombatEvent` messages for at least two beats and assert `PhaseStripDisplay` changes to the latest expected label/state.
4. Snapshot `CombatState` before/after or assert a sentinel field remains unchanged to prove fake UI updates did not mutate combat state. Add a negative case where a non-beat event leaves the phase unchanged.
5. Run the full verification suite for this slice.

Done when: `cargo test --test phase_strip_readonly --features windowed` passes, headless `cargo test` passes, and `cargo build --features windowed` passes.

Threat Surface (Q3): Test-only fake events; no auth/input/data exposure.
Requirement Impact (Q4): Directly validates R002, R004, R005, and S03's structural UI contract. No roadmap reassessment needed unless this proof exposes an impossible Bevy constraint.
Failure Modes (Q5): If `CombatState` is not easily comparable, test a stable sentinel field or use debug snapshot before/after; do not weaken the test to only compile. If Bevy read-only trait proof cannot cover `MessageReader` directly in this version, keep the combat-facing ingest helper pure and compile-proof that helper instead, plus runtime no-mutation assertions.
Load Profile (Q6): Test app processes a tiny synthetic event stream; no scaling risk.
Negative Tests (Q7): Non-beat event ignored; no events leave phase unchanged; combat state unchanged after beat processing.

## Inputs

- `src/ui/phase_strip.rs`
- `src/windowed/mod.rs`
- `src/combat/observability/events.rs`
- `src/combat/kernel/primitives.rs`

## Expected Output

- `tests/phase_strip_readonly.rs`
- `src/ui/phase_strip.rs`

## Verification

cargo test --test phase_strip_readonly --features windowed

## Observability Impact

Adds the regression proof a future agent can run to localize any accidental combat-state mutation introduced by UI changes.
