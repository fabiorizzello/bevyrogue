---
estimated_steps: 12
estimated_files: 4
skills_used: []
---

# T04: Close S03 verification across headless and windowed gates

Why: S03 touches feature-gated UI code, so closeout must prove both the default headless contract and the windowed integration contract. This task follows verify-before-complete plus write-docs by producing fresh verification evidence and leaving concise code comments where the read-only boundary matters.

Do:
1. Run `cargo test` to prove headless-first behavior remains intact and no windowed dependencies leaked into the default build.
2. Run `cargo test --test phase_strip_readonly --features windowed` to prove the structural event-reader/read-only contract.
3. Run `cargo build --features windowed` to prove real windowed wiring compiles. Do not rely on bare `cargo run --features windowed`; if a visual smoke is attempted, use `cargo run --features windowed --bin bevyrogue` and treat missing `DISPLAY`/`WAYLAND_DISPLAY` as environment-limited per project memory.
4. If failures occur, fix within S03 scope only; do not broaden to S04/S05 reactive labels or Agumon full-kit behavior.

Done when: All three automated gates pass, or any visual run limitation is documented as display-environment limited while the automated gates pass.

Threat Surface (Q3): Verification only; no new runtime attack surface.
Requirement Impact (Q4): Re-verifies R002/R004/R005 and S03 acceptance; no decisions revisited.
Failure Modes (Q5): Headless failures indicate cfg leakage; windowed build failures indicate egui/plugin wiring drift; phase-strip test failures indicate boundary regression.
Load Profile (Q6): Build/test workload only.
Negative Tests (Q7): Covered by `phase_strip_readonly` non-beat/no-event cases and full test suite regression coverage.

## Inputs

- `src/ui/phase_strip.rs`
- `src/ui/mod.rs`
- `src/windowed/mod.rs`
- `tests/phase_strip_readonly.rs`

## Expected Output

- Update the implementation and proof artifacts needed for this task.

## Verification

cargo test

## Observability Impact

Produces fresh command evidence for default and windowed gates; no code observability changes beyond earlier tasks.
