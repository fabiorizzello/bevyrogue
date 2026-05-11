---
estimated_steps: 14
estimated_files: 5
skills_used: []
---

# T03: Add data-only custom signals and carry them through resolved actions

Add the RON-side declaration and action metadata plumbing for per-Digimon signals while keeping RON out of final gameplay authority.

Skills expected: `design-an-interface`, `tdd`, `verify-before-complete`.

Steps:
1. Add a typed `SkillCustomSignal` field to `SkillDef` with `#[serde(default)]`, starting with a Patamon-specific enum rather than a line/mechanic-owned enum.
2. Add a Patamon signal to one tracked Patamon skill in `assets/data/skills.ron`, preferring a low-risk Holy Support skill such as `patamon_ult` or `holy_breeze` rather than Angemon's deferred mixed-effect ultimate.
3. Extend `ResolvedAction` and `resolve_action` to copy the resolved skill's custom signals into action metadata.
4. Add/seed `tests/patamon_blueprint_seam.rs` with assertions that tracked RON parses the signal and a resolved action carries it; do not assert Holy Support state yet.

Must-haves:
- The new field defaults for all existing RON/test fixtures, so old skills without signals still deserialize.
- `apply_effects` does not interpret custom signals and does not gain per-Digimon branches.
- The schema shape preserves D011/D012/D013: RON declares typed intent, Patamon Rust logic will own interpretation, kernel transitions remain observable output.

Failure Modes (Q5): malformed RON custom-signal syntax should fail parsing clearly; missing signal fields must default to empty vectors; unknown signal variants should be rejected by serde rather than ignored silently.
Load Profile (Q6): per action copies a small vector of typed signals from the resolved skill; avoid per-action asset re-lookup downstream.
Negative Tests (Q7): tests should include a skill with no custom signals and the Patamon skill with exactly the expected signal.

## Inputs

- ``src/data/skills_ron.rs` — current `SkillDef`, `Effect`, presentation metadata, and serde defaults.`
- ``assets/data/skills.ron` — tracked canonical skill content where the first Patamon signal is declared.`
- ``src/combat/state.rs` — current `ResolvedAction` contract.`
- ``src/combat/resolution.rs` — `resolve_action` copies skill data into resolved action metadata.`

## Expected Output

- ``src/data/skills_ron.rs` — typed data-only custom signal schema with default empty vector.`
- ``assets/data/skills.ron` — at least one Patamon skill declares the first custom signal.`
- ``src/combat/state.rs` — `ResolvedAction` carries copied custom signals.`
- ``src/combat/resolution.rs` — `resolve_action` populates custom signals without changing `apply_effects` semantics.`
- ``tests/patamon_blueprint_seam.rs` — tests parse the Patamon signal and prove resolved-action propagation.`

## Verification

cargo test --test patamon_blueprint_seam custom_signal
