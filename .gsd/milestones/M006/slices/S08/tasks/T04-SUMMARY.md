---
id: T04
parent: S08
milestone: M006
key_files:
  - tests/windowed_only/renamon_extension_contract.rs
key_decisions:
  - D055: Non-idle stance reactions (hurt/death) use shared engine reaction defaults via StanceReaction::stance_node() canonical node names, not species-specific reaction data; each Digimon conforms via its own stance graph asset, preserving S05's zero-engine-edit thesis
  - Added a real-behavior parse-based test (ron::from_str::<AnimGraph>) rather than an include_str! source-contract, because the lib reaction API and the stance asset are both reachable from tests/ — only src/windowed/ binary code requires the source-contract workaround
duration: 
verification_result: passed
completed_at: 2026-05-27T11:19:02.373Z
blocker_discovered: false
---

# T04: Resolved the idle-only-vs-hurt design call as "shared engine reaction defaults" (D055) and locked it with an executable Renamon stance-graph reaction contract test

**Resolved the idle-only-vs-hurt design call as "shared engine reaction defaults" (D055) and locked it with an executable Renamon stance-graph reaction contract test**

## What Happened

Resolved spike-2's open idle-only-vs-hurt design question for Renamon's non-idle reactions. Investigation found the engine reaction path is already fully species-agnostic: drive_hurt_reactions / drive_death_reactions in src/windowed/render.rs drive the target sprite into the node named by the pure lib mapping StanceReaction::stance_node() (canonical "hurt"/"death"), against whatever stance graph the unit registered. Renamon's authored assets/digimon/renamon/stance.ron already names its nodes "hurt"/"death" with the canonical hurt -> idle (TimeInNode) and death -> Exit (TimeInNode) return transitions — structurally identical to Agumon. So the chosen behavior (shared defaults, no species-specific reaction data) is already implemented for free; the work was to make the decision explicit and guard it.

Recorded the decision as D055 (gsd_save_decision): non-idle reactions use shared engine reaction defaults, not species-specific data — a new Digimon gets working hurt/death purely by authoring a stance graph whose node names + return transitions conform to the canonical reaction vocabulary, preserving S05's zero-engine-edit thesis. Marked revisable: add a per-species override seam only when a Digimon genuinely needs a different reaction shape.

Added the executable half of the decision to tests/windowed_only/renamon_extension_contract.rs: renamon_reactions_use_shared_engine_defaults parses Renamon's authored stance.ron into an AnimGraph (lib-reachable, no windowed binary — K001), then asserts (1) every shared StanceReaction::stance_node() resolves to a node Renamon actually authored, and (2) the return transitions match the engine's degrade-to-idle / death-exit contract from drive_stance_reaction (hurt -> Node("idle") on TimeInNode; death -> Exit on TimeInNode). This is a real-behavior test against the lib reaction API + parsed asset, not a source-contract string match.

## Verification

Ran the windowed_only reaction coverage and the full headless suite. New test renamon_reactions_use_shared_engine_defaults passes; all 10 renamon_extension_contract tests pass; full windowed_only binary 75/75 pass; full headless `cargo test` green across all 25 test binaries (no failures). The only warning (unused import BeatEdge) is pre-existing and unrelated to this change.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --features windowed --test windowed_only renamon` | 0 | pass | 1280ms |
| 2 | `cargo test --features windowed --test windowed_only` | 0 | pass | 20000ms |
| 3 | `cargo test` | 0 | pass | 30000ms |

## Deviations

No code implementation was needed: the chosen behavior (shared defaults) was already satisfied by Renamon's authored stance.ron and the species-agnostic engine reaction path. src/animation/reaction.rs and src/windowed/digimon/renamon/mod.rs were read as inputs but required no edits — the task reduced to recording the decision and adding guarding test coverage.

## Known Issues

none

## Files Created/Modified

- `tests/windowed_only/renamon_extension_contract.rs`
