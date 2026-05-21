# M002 Final Regression Matrix — S06/T03

Date: 2026-05-21
Scope: R016 invariant gate at M002 closeout. Mechanical proof that R002 (headless-first), R004 (determinism via existing harnesses), R005 (windowed dep-gating), R006 (repo hygiene), and I3 two-clock + cue handshake parity all hold.

Live windowed UAT is performed by the user per K001 — the captured log under `uat-evidence/` is the R014 evidence and is independent of this regression matrix.

## Command Matrix

| # | Command | Tests | Exit | Wall | Verdict |
|---|---------|-------|------|------|---------|
| 1 | `cargo test` (headless, default features) | full default suite incl. M001 + M002 headless harnesses; 56-test final binary tail (sp_economy, ultimate_meter, turn_system_*, etc.); doc-tests empty | 0 | 24.5s | PASS |
| 2 | `cargo test --features windowed --test windowed_only` | 23 passed; 0 failed | 0 | 1.4s | PASS |
| 3 | `cargo test --test timeline` | 47 passed; 0 failed (incl. `timeline_two_clock_parity`, `timeline_loop_hop_cue_parity`, `timeline_cue_barrier_pipeline` — the I3 extended parity proof) | 0 | 0.2s | PASS |
| 4 | `cargo test --features windowed --test bootstrap_encounter` | 16 passed; 0 failed; 1 ignored | 0 | 0.3s | PASS |
| 5 | `cargo test --test digimon_kits` | 70 passed; 0 failed | 0 | 0.2s | PASS |
| 6 | `cargo build --no-default-features` | headless build clean | 0 | 0.2s | PASS |
| 7 | `cargo build --features windowed` | windowed build clean | 0 | 0.3s | PASS |

## R016 Structural Checks

### R005 — Windowed dep gating

Command:

```
grep -rn 'use winit\|use wgpu\|use bevy_egui\|use egui' src/ \
  | grep -v '#\[cfg(feature = "windowed")\]' \
  | grep -v '/windowed/' \
  | grep -v '/ui/'
```

Output:

```
src/combat/runtime/mod.rs:20://! No `use bevy::winit`, `use bevy::render`, or `use bevy_egui` in this module
```

Verdict: **PASS** — sole match is a doc-comment forbidding such imports, not an actual `use` statement. No real winit/wgpu/egui/bevy_egui leakage outside `src/windowed/` or `src/ui/`.

### R006 — Repo hygiene (no `.md` in repo root)

Command:

```
find . -maxdepth 1 -name '*.md' -not -path './node_modules/*'
```

Output: *(empty)*

Verdict: **PASS** — zero markdown files in repo root. Docs continue to live under `docs/` and `.gsd/`.

### I3 — Two-clock + cue handshake parity (R016 extension)

Files present and exercised by command #3:

- `tests/timeline/timeline_two_clock_parity.rs` — present
- `tests/timeline/timeline_loop_hop_cue_parity.rs` — present
- `tests/timeline/timeline_cue_barrier_pipeline.rs` — present

All three are linked into the `timeline` harness (command #3) and passed in the 47-test green run. Together they cover the cue handshake extension R016 mandates (suspend-on-cue, release semantics, headless/windowed equivalence under the barrier).

Verdict: **PASS**

## Summary

All 7 cargo commands exit 0. All 3 R016 structural checks PASS. No blockers; M002 closure may proceed to slice/milestone validation. Live windowed smoke remains a user-driven UAT per K001 and is recorded separately under `uat-evidence/` per T01.
