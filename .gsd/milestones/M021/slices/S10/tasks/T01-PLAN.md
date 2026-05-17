---
estimated_steps: 4
estimated_files: 5
skills_used: []
---

# T01: Migrate Patamon Holy Support transport onto the Blueprint owner envelope

Skills used: bevy, rust-best-practices, verify-before-complete.

Why: Patamon still emits `CombatKernelTransition::HolySupport(...)` from custom signal dispatch and from Holy Support hook fan-out, which keeps the shared kernel transition enum in the loop.

Do: update `src/combat/blueprints/patamon/signals.rs` and `src/combat/blueprints/patamon/identity.rs` so Patamon-originated writes use `CombatKernelTransition::Blueprint { owner: "patamon", name, payload }`; keep Holy Support typed state/transition structs owned by the Patamon blueprint module; make the applier decode only Patamon-owned Blueprint envelopes; preserve current grace / martyr-light semantics and deterministic passive behavior; update Patamon-focused tests to assert owner-gated dispatch, malformed-payload rejection, and unchanged runtime outcomes through the blueprint path.

Done when: no Patamon custom-signal path emits the shared Holy Support kernel variant anymore, Patamon runtime state is still deterministic, and the Patamon seam/resolution tests pass against the Blueprint envelope path.

## Inputs

- `src/combat/blueprints/patamon/signals.rs`
- `src/combat/blueprints/patamon/identity.rs`
- `src/combat/blueprints/patamon/mod.rs`
- `tests/patamon_blueprint_seam.rs`
- `tests/holy_support_resolution.rs`
- `assets/data/skills.ron`

## Expected Output

- `src/combat/blueprints/patamon/signals.rs`
- `src/combat/blueprints/patamon/identity.rs`
- `src/combat/blueprints/patamon/mod.rs`
- `tests/patamon_blueprint_seam.rs`
- `tests/holy_support_resolution.rs`

## Verification

cargo test --test patamon_blueprint_seam
cargo test --test holy_support_resolution

## Observability Impact

Patamon diagnostics stay available through blueprint-owned Holy Support state/snapshot inspection instead of shared kernel variant matching.
