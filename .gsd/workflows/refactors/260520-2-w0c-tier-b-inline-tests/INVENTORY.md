# Inventory — W0c Tier-B inline `mod tests` relocate

Created: 2026-05-20
Parent workflow: `260520-1-reduce-loc-tests` (closed; backlog item #1)
Scope: 7 inline `#[cfg(test)] mod tests` blocks, 30–46 LOC each, all <100 LOC (below R003 hard cap but accumulated tail).

## Goal

Apply the established Tier-A relocate pattern (`tests/<name>_internals.rs`) to the next 7 inline-test blocks. Pure relocate where possible; promote visibility only when unavoidable. Three files are gated `#[cfg(feature = "windowed")]` — the relocated test files must carry the same gate.

## Candidates (verified 2026-05-20 against HEAD `781e5d1`)

| # | Source file | Inline block lines | LOC | Feature gate | Notes |
|---|---|---|---|---|---|
| 1 | `src/combat/runtime/registry.rs` | 190–234 | 45 | — | Largest target; verify symbol visibility for kernel registry handles. |
| 2 | `src/combat/state.rs` | 107–149 | 43 | — | Core combat state — check whether internal fields are touched. |
| 3 | `src/windowed/render.rs` | 313–355 | 43 | `windowed` | Relocated file must be `#[cfg(feature = "windowed")]`. |
| 4 | `src/combat/mechanics/sp.rs` | 69–108 | 40 | — | SP mechanics — public surface already broad. |
| 5 | `src/combat/runtime/event_filter.rs` | 95–132 | 38 | — | Event filter internals. |
| 6 | `src/combat/mechanics/modifiers.rs` | 151–184 | 34 | — | Modifier ledger — fold-order pattern (see KNOWLEDGE.md). |
| 7 | `src/windowed/mod.rs` | 290–320 | 31 | `windowed` | Relocated file must be `#[cfg(feature = "windowed")]`. |

Total: **~274 LOC** to move out of `src/` (matches SUMMARY backlog "~281 LOC" within counting tolerance).

## Reference pattern (from Tier-A)

Tier-A reference commits, applied verbatim where the test body is pure:

- `aa22a1f` W0c-1 `CastRng` → `tests/cast_rng_internals.rs` (74 LOC pure relocate)
- `a3b53aa` W0c-2 `TempoResistance`
- `ff6e069` W0c-3 `Energy/RoundEnergyTracker`
- `ba3c9f1` W0c-4 `SignalBus` — required rewrite of private `try_consume` assertion as integration via emit/observe (precedent for visibility-promotion exception)

Rule of thumb (inherited from parent workflow):
1. If every symbol the test touches is already `pub` / `pub(crate)`: pure `git mv` of the block.
2. If a symbol is private but trivially promotable to `pub(crate)`: promote and note in commit body.
3. If the test reaches into a truly private invariant (like `SignalBus::try_consume`): rewrite as integration via the public surface — do NOT widen visibility just to keep the test shape.

## Out of scope

- Tier-C (~159 LOC across 6 files, all 20–30 LOC). Diminishing returns.
- Impl-coupling debt (typed test-API surface) — separate architectural slice.
- Backward `tests/` → `src/` relocate for pure-function unit tests — separate slice.

## Acceptance

- All 7 source files no longer contain an inline `#[cfg(test)] mod tests` block.
- Corresponding `tests/<name>_internals.rs` exists for each relocated block (or a target shows a justified delete/integration-fold).
- `cargo test --tests` and `cargo test --features windowed --tests` green.
- `cargo check --tests` clean on changed files.
- `scripts/check_loc_cap.sh` still passes.
