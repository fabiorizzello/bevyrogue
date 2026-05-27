# SCOPE — Test utility audit: prune visual + impl-coupling tests

**Date:** 2026-05-26 · **Branch:** master · **Complexity:** low

## The Question

Which tests in the suite are **low-value** and should be deleted? Two target
classes the user named:

1. **Purely visual tests** — tests that try to assert something only a human eye
   can actually judge (K001 says visual quality is manual-only), so the
   automated version proves little.
2. **Impl-coupling tests that churn** — tests that know *where types live / what
   the source text says* rather than *what the system observably does*, so any
   refactor breaks them even when behavior is unchanged.

Essentially: find the tests that cost maintenance without buying confidence,
and cut them.

## What a good answer includes

- A **categorized inventory** of the 943 test fns by value-class, not a per-test
  dump — grouped by the signature that makes them low-value.
- A **kept / cut / rewrite** disposition per category, with rationale.
- For every CUT candidate: what boundary (if any) it currently protects, and
  whether that boundary is already covered by a stronger test — so deletion
  removes churn, not coverage.
- A concrete **delete list** (file/function level) ready to act on, plus an
  estimate of how many test fns / files it removes.

## Constraints

- **Do not weaken real invariants.** R002 (headless-first), R004 (determinism),
  R005/R016 (dep-gating: no windowed deps leak headless) must stay enforced by
  *something*. Cutting a churny test is fine only if its real boundary survives.
- **K001 is the frame for "visual":** auto-mode cannot run the windowed binary,
  so anything claiming to verify on-screen look is already manual. The question
  for those is whether the headless proxy asserts a real contract or just noise.
- Spike output is **knowledge + a delete list**, no code shipped this phase.
- R003 test layout (19 scope harnesses) stays; this is about *which cases*, not
  the harness structure.

## Research Angles

### Angle 1 — Source-token structural guards (highest-churn suspect)
~14 assertions that `include_str!("../../src/**.rs")` then `.contains("token")`,
concentrated in `tests/windowed_only/` (render.rs ×6, windowed mod ×3, agumon/
renamon modules). These pin source *text*; M006 history shows them inverted/
rewritten in S01/S03/S04. Classify each: does it guard a true boundary
(engine-stays-species-agnostic, dep-gating, no-VFX-kind-dispatch) that couldn't
be expressed behaviorally — or does it just freeze current code shape?

### Angle 2 — Impl-coupled internal tests
The `*_internals.rs` files relocated out of `src/` per R003, plus the large
`validation_snapshot` golden-string tests. Do they assert observable combat
behavior, or private helpers / internal field layout that drifts on refactor?
Separate behavior-anchored from structure-anchored.

### Angle 3 — Visual / windowed-proxy tests
The windowed VFX cluster (`vfx_asset_impact_render`, `vfx_windowed_contracts`,
HDR/overbright "acceptance", `enoki_*_parses`). Which are genuine data/parse
contracts (keep) vs headless stand-ins for a look only K001-manual can judge
(cut candidates)?

## Decision format

Tradeoff matrix + an explicit delete list with cut/keep/rewrite per category and
a coverage-preservation note for each cut.
