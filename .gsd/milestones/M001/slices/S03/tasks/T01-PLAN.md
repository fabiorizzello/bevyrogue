---
estimated_steps: 8
estimated_files: 3
skills_used: []
---

# T01: Add pure validation contract and typed diagnostics

Why: S03's load-bearing seam is a generic validator inside `src/animation` that can reason over typed `AnimGraph` and `Clip` without knowing about Digimon, combat, or data modules. This task should be executed test-first using the expected skills: decompose-into-slices, design-an-interface, grill-me, tdd, write-docs, bevy, rust-best-practices, verify-before-complete.

Do: Create `src/animation/validation.rs` and export it from `src/animation/mod.rs`. Design a small deep interface around `validate_anim_graph(&AnimGraph, &Clip, &AnimationValidationCatalogs) -> AnimationValidationReport` plus a blocking convenience wrapper if useful. Include typed diagnostics with severity, check/reason enum, graph context (clip id/node id/edge index/command context where relevant), and human-readable detail. Keep catalogs generic, deterministic, and adapter-owned: at minimum `params: BTreeSet<ParamKey>`, `statuses: BTreeSet<StatusId>`, `particles: BTreeSet<ParticleId>`, and optionally future-safe `skills: BTreeSet<SkillIdRef>`. Implement first checks for: graph clip id exists in `Clip.ranges`; entry node exists; each node frame range is ordered, inside clip total frames, and inside the named clip range; transition `from`/`to Node(...)` references exist; recursive `Predicate::Unlock`, `And`, `Or`, `Not`, `KernelEvent::StatusApplied`, command status/particle/param references are resolved through the graph or catalogs. Preserve advisory-warning room for later reachability/cancel checks by using report diagnostics rather than a single-error-only API.

Done when: Unit/integration tests in `tests/anim_validation.rs` exercise the public validator API with an in-memory valid mini graph and multiple broken mini graphs, and every failure returns typed diagnostics instead of panics or string-only errors.

Q3 Threat Surface: authored local RON data is untrusted input at boot; this task must not introduce filesystem, network, auth, or secret access beyond parsing already-loaded typed assets.
Q4 Requirement Impact: owns R004 and R005, supports R001/R008, and must not invalidate S01/S02 typed schema behavior.
Q5 Failure Modes: if caller passes mismatched graph/clip/catalog data, return blocking diagnostics; if many diagnostics exist, keep collecting enough context rather than failing at first error.
Q6 Load Profile: validation is boot-time over small authored maps/vectors; use deterministic `BTreeSet`/`BTreeMap` lookups so 10x asset count scales by graph size without hidden global scans.
Q7 Negative Tests: missing clip range, missing entry node, missing transition target, node frames outside clip range, unknown particle/status/param references.

## Inputs

- `src/animation/anim_graph.rs`
- `src/animation/clip.rs`
- `src/animation/mod.rs`
- `tests/anim_graph_parse.rs`
- `tests/clip_parse.rs`

## Expected Output

- `src/animation/validation.rs`
- `src/animation/mod.rs`
- `tests/anim_validation.rs`

## Verification

cargo test --test anim_validation

## Observability Impact

Introduces the durable typed diagnostic vocabulary future asset/plugin tests and S04 hot-reload paths can inspect.
