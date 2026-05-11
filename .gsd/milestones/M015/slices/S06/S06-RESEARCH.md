# S06 Research — Regression baseline and M013 closure packaging

## Summary

S06 is the closure slice for M015. It owns the final proof that M015 really supersedes M013’s incomplete closure rather than leaving a classified-red baseline. The work is not another architecture redesign: S03/S04/S05 already established the Patamon blueprint seam, presentation metadata non-authority, and real `combat_cli` shared-surface proof. S06 should focus on (1) making the whole Rust suite compile/run green, (2) updating the failure/audit ledgers from “S06 still owns this” to “S06 closed or explicitly split this,” and (3) producing truthful M013/M015 closure artifacts.

The highest-risk implementation seam is the transition from targeted green to full-suite green. Prior slices intentionally left broad fixture/schema drift and docs artifacts for S06. The likely first blockers are stale integration-test declarations, `SkillDef` / `UnitDef` constructor drift in integration tests, duplicate fields from partially-updated fixtures, and the missing UI readiness gap matrix doc included by `tests/ui_readiness_gap_matrix_docs.rs`.

Important process note: in this research session, shell/`gsd_exec` command cwd appeared to resolve to the base checkout rather than the intended M015 worktree, while GSD/read context had the M015 artifact view. Treat any command evidence from this scout as diagnostic only; executors should run fresh commands in their actual worktree before changing code. The preloaded S03/S04/S05 summaries and GSD DB status are the authoritative prior-slice state.

## Requirements focus

S06 directly owns:

- **R089** — Closure artifacts must truthfully supersede M013 gaps rather than silently repairing history.
- **R091** — Final closure requires a green full Rust suite and real `combat_cli` shared-surface proof.
- **R099** — M013 validation/artifact gaps must be repaired or superseded without rewriting history as if evidence existed in M013.
- **R100** — Verification must remain deterministic and headless-first.

S06 supports/consumes:

- **R090** — Preserve the failure-ledger discipline while retiring blockers; classify before fixing/deleting.
- **R092–R098** — Do not regress the authority map, Patamon seam, RON boundary, kernel authority, presentation non-authority, or CLI proof validated by S02–S05.

Validated requirement updates at the end should be made with `gsd_requirement_update`, not by hand-editing `.gsd/REQUIREMENTS.md`.

## Skills activated and how they apply

- **api-design**: no HTTP API is involved, but the skill’s “caller contract” principle applies to the shared CLI/query/event/snapshot surfaces. Do not change consumer-visible semantics or reason-code shapes merely to satisfy stale tests.
- **design-an-interface**: S06 should avoid inventing a new deep interface unless full-suite failures force it. If test helpers are introduced, keep them narrow and boring; this is a closure slice, not an abstraction slice.
- **grill-me**: major decisions are already resolved in D016–D018. Do not reopen final-bar or history-supersession questions unless a genuinely rewrite-scale blocker appears and must be split forward.
- **observability**: the durable signals are `docs/m015_failure_ledger.md`, `docs/combat_authority_map.md`, `docs/combat_mixed_pattern_drift_ledger.md`, and verifier scripts. Update them with exact commands, exits, blocker classifications, and failure modes — not generic “it works” prose.
- **write-docs**: closure docs must serve a fresh future agent. Name what M013 proved, what M013 missed, what M015 fixed, what remains future work, and how to re-run the proof.

## Skills discovered

Installed globally for later units:

- `apollographql/skills@rust-best-practices` — relevant to Rust execution/review in S06.
- `affaan-m/everything-claude-code@rust-testing` — relevant to full-suite repair and test-first verification.

Search results also showed Bevy-specific skills (`bfollington/terma@bevy`, `sickn33/antigravity-awesome-skills@bevy-ecs-expert`, etc.), but they had low install counts and S06 is mainly fixture/docs/verification closure, not new Bevy API design. No RON-specific skill was relevant.

## Implementation landscape

### GSD / closure artifacts

- `.gsd/REQUIREMENTS.md` currently leaves R089, R091, R099, and R100 active with S06 as primary owner.
- `.gsd/DECISIONS.md` contains the controlling decisions:
  - D016: full Rust suite + real CLI proof required for M015 closure.
  - D017: supersede M013 through M015 artifacts, do not repair M013 in-place as if it closed cleanly.
  - D018: rewrite stale tests to current source-of-truth contracts; remove obsolete expectations only with named replacement coverage.
- `.gsd/milestones/M013/M013-ROADMAP.md`, `M013-DISCUSSION.md`, and `M013-PARKED.md` show M013 was parked after choosing split/replan. Missing M013 validation/summary artifacts should remain part of the closure story.
- No `M015-VALIDATION.md` was found in the pre-S06 state. S06 should create validation through `gsd_validate_milestone` after final verification passes.

### Failure ledger and audit docs

- `docs/m015_failure_ledger.md` is the main S06 working ledger. It currently records targeted S03/S05 passes and says broad full-suite blockers remain S06-owned.
- `scripts/verify_m015_failure_ledger.py` is currently shaped for the pre-S06 boundary. It requires S05 proof markers and S06 ownership markers, and has overclaim guards such as “cargo test --no-run is green/passes.” S06 must update this verifier deliberately when the final baseline becomes green, otherwise truthful final ledger wording may be rejected by the script.
- `scripts/verify_combat_authority_audit.py` checks claim-scoped markers for the authority map, drift ledger, presentation metadata boundary, CLI proof, and S06 boundary. At S06 closure, update only the final-baseline boundary claims; preserve the non-authority and “not full blueprint/UI migration” boundaries.
- `docs/combat_authority_map.md`, `docs/combat_mixed_pattern_drift_ledger.md`, `docs/presentation_metadata_boundary.md`, and `docs/combat_cli_shared_surface_proof.md` are the durable architecture/audit docs. S06 should not rewrite their architecture story unless a full-suite failure proves it false.

### Test and fixture blockers to expect

The ledger already names the broad fixture repair surface. Executor should run fresh `cargo test --no-run` first, but likely batches are:

1. **Stale manifest target**
   - Ensure `Cargo.toml` no longer declares `[[test]] battery_loop_resolution` pointing at missing `tests/battery_loop_resolution.rs`.
   - Replacement coverage is `tests/battery_loop_kernel.rs`; do not restore the obsolete resolution target.

2. **Missing docs artifact**
   - `tests/ui_readiness_gap_matrix_docs.rs` includes `../docs/combat_ui_readiness_gap_matrix.md` and asserts exact vocabulary.
   - The doc must include: `R085`, `D053`, `D054`; status definitions for `Implemented`, `ToFixNow`, `Deferred`, `Hidden`; at least one table row each for `Implemented`, `Deferred`, `Hidden`; boundary text “No CLI/windowed skill-ID-specific legality rules”, “must come from DSL/query output”, and “queryable `Deferred`/`Hidden` declaration”; plus the mechanics and reason codes listed in the test.
   - Use `docs/skill_legality_contract.md` as the existing source of truth for reason-code wording and consumer boundary.

3. **`SkillDef` constructor drift**
   - New/current fields are `custom_signals: Vec<SkillCustomSignal>`, `animation_sequence: Option<Vec<String>>`, and `qte: Option<String>`.
   - For stale inline fixture constructors, prefer either explicit neutral fields:
     - `custom_signals: vec![]`
     - `animation_sequence: None`
     - `qte: None`
     or `..Default::default()` at the end of `SkillDef` when every meaningful field is already set.
   - Do not restore removed Holy Support effect variants, `TargetShape::SelfTarget`, or obsolete Twin Core fields.
   - Ledger-known files include `tests/status_effect_apply.rs`, `tests/engine_legality_integration.rs`, `tests/action_affordance_consumers.rs`, `tests/action_affordance_query.rs`, `tests/combat_coherence.rs`, `tests/boundary_contract.rs`, `tests/status_effect_integration.rs`, `tests/sp_economy.rs`, `tests/patamon_revive.rs`, and `tests/ultimate_meter.rs`. Fresh no-run may expose more once earlier blockers are fixed.

4. **`UnitDef` constructor drift**
   - Current/defaulted metadata fields are `twin_core: TwinCoreRosterMetadata` and `holy_support: HolySupportRosterMetadata`.
   - `UnitDef` does not have a simple safe default in the schema; add `twin_core: Default::default()` and `holy_support: Default::default()` to inline fixtures unless the test specifically needs metadata.
   - Ledger-known files include `tests/tempo_resistance.rs`, `tests/follow_up_chains.rs`, and `tests/roster_smoke.rs`. Static/early compile probes may also expose `tests/bootstrap_spawn_composition.rs`, `tests/combat_coherence.rs`, `tests/follow_up_triggers.rs`, `tests/form_identity.rs`, `tests/pipeline_dispatch.rs`, `tests/resource_caps.rs`, and `tests/twin_core_integration.rs` depending on exact worktree state.

5. **Duplicate fixture fields**
   - `tests/follow_up_chains.rs` and `tests/roster_smoke.rs` have been named in the ledger for duplicate `enemy_traits` and `charged_attack` fields. Remove the duplicate pair, preserving the more intentional/non-empty value when there is a difference.

6. **Secondary runtime failures**
   - Once `cargo test --no-run` passes, `cargo test --no-fail-fast` may expose real runtime regressions. Classify each before fixing. Default policy from M015 is to fix blockers by default unless they are genuinely rewrite-scale/out-of-bound and explicitly split forward.

## Natural seams for task planning

### Seam A — Baseline compile unblockers

Scope: `Cargo.toml`, docs include artifact, broad fixture constructors, duplicate fields, `docs/m015_failure_ledger.md`.

Goal: `cargo test --no-run` exits 0. This is the riskiest/unblocking task because every later proof depends on full suite compilation.

Keep it mechanical: neutral defaults for new data-only fields, remove stale manifest target if present, add the missing UI readiness doc, and record each retired blocker in the ledger.

### Seam B — Full-suite runtime classification and repair

Scope: test failures exposed only after no-run passes. Likely files under `tests/` plus possibly small source fixes if a real regression is found.

Goal: `cargo test --no-fail-fast` exits 0. If failures are numerous, update `docs/m015_failure_ledger.md` with a classification table before fixes, then retire rows as fixes land.

### Seam C — Final shared-surface proof and audit verifiers

Scope: `src/bin/combat_cli.rs`, `tests/combat_cli_shared_surface.rs`, S03/S04/S05 targeted tests, verifier scripts, authority/CLI docs.

Goal: preserve S05 contract while converting S06 docs/verifiers from “baseline remains S06-owned” to “baseline is closed by S06.” Run the real CLI proof and ensure forbidden hidden-drift markers remain absent.

### Seam D — Truthful closure packaging

Scope: GSD artifacts and closure docs: `.gsd/PROJECT.md`, `.gsd/REQUIREMENTS.md` via tools, `docs/m015_failure_ledger.md`, likely `M015-VALIDATION.md` via `gsd_validate_milestone`, final S06 summary/UAT via `gsd_slice_complete`, and final milestone summary via `gsd_complete_milestone`.

Goal: a cold future agent can answer: what did M013 prove, what did it miss, what did M015 fix, what remains intentionally future work, and which exact commands prove closure?

## Recommended task order

1. **Repository/worktree sanity check and no-run classification**
   - Run fresh `cargo test --no-run` in the actual M015 worktree.
   - Confirm S03/S04/S05 files from summaries exist (`docs/combat_authority_map.md`, `scripts/verify_m015_failure_ledger.py`, `tests/combat_cli_shared_surface.rs`, etc.). If they do not, stop and resolve worktree/state hydration before code repair.
   - Update the ledger with fresh command ID/output summary before fixes.

2. **Compile unblocker batch**
   - Remove stale `battery_loop_resolution` manifest declaration if present.
   - Create/repair `docs/combat_ui_readiness_gap_matrix.md` to satisfy `tests/ui_readiness_gap_matrix_docs.rs` using `docs/skill_legality_contract.md` as source vocabulary.
   - Batch neutral fixture defaults for `SkillDef` and `UnitDef` drift.
   - Remove duplicate fixture fields.
   - Verify `cargo test --no-run` until exit 0.

3. **Full runtime baseline**
   - Run `cargo test --no-fail-fast`.
   - Classify every failure first in `docs/m015_failure_ledger.md`.
   - Fix in-scope failures by default. Only split a failure forward if it is truly rewrite-scale/unrelated and the split has explicit rationale and an artifact/requirement follow-up.

4. **S03/S04/S05 regression bundle + real CLI proof**
   - Run the targeted bundle from S05: `cargo test --test combat_cli_shared_surface --test presentation_metadata_boundary --test event_stream --test patamon_blueprint_seam`.
   - Run `python3 scripts/verify_combat_authority_audit.py` and `python3 scripts/verify_m015_failure_ledger.py` after updating them for S06 closure semantics.
   - Run `BEVYROGUE_JSONL=1 BEVYROGUE_CLI_PROOF=1 cargo run --bin combat_cli` and require the S05 markers: `Action affordances`, `OnCombatBeat`, `OnKernelTransition`, `OnActionResolved`, `OnDamageDealt`, `OnSkillCast`, `[CLI_PROOF] validation_snapshot:`, `holy_support=grace=`; forbid `panicked`, `Message not initialized`, `[QUERY] Skill book unavailable`, `readiness_timeout`, and snapshot/proof failure markers.

5. **Closure artifacts and GSD completion**
   - Update `docs/m015_failure_ledger.md` final verification table with the full suite, no-run, no-fail-fast, CLI proof, and verifier evidence.
   - Update closure prose to state that M013 was parked/incomplete and M015 supersedes it; do not backfill M013 as if it originally closed.
   - Use `gsd_requirement_update` to validate R089, R091, R099, and R100 with evidence.
   - Complete S06 with `gsd_slice_complete`, validate the milestone with `gsd_validate_milestone`, then complete M015 with `gsd_complete_milestone` if all slices are complete and verification passed.

## Verification contract for S06

Minimum final evidence should be fresh, not inherited from prior slice summaries:

```bash
cargo test --no-run
cargo test --no-fail-fast
cargo test --test combat_cli_shared_surface --test presentation_metadata_boundary --test event_stream --test patamon_blueprint_seam
python3 scripts/verify_combat_authority_audit.py
python3 scripts/verify_m015_failure_ledger.py
BEVYROGUE_JSONL=1 BEVYROGUE_CLI_PROOF=1 cargo run --bin combat_cli
```

If available and not too costly, also run `cargo test` after `--no-fail-fast` passes as a conventional final check; however `--no-fail-fast` is the roadmap-required broad failure-shape command.

All verification must stay headless (`default` features, no `windowed`). Use `gsd_exec` for noisy commands and record command IDs/exit codes in summaries.

## Guardrails and pitfalls

- Do not restore obsolete Holy Support affordance APIs or removed RON variants just to satisfy old tests.
- Do not make `animation_sequence`, `qte`, beat wording, or presentation trigger text authoritative.
- Do not add CLI-only combat logic; CLI must remain a shared-surface consumer.
- Do not silently delete failing tests. If a test is obsolete, name replacement coverage in the ledger.
- Do not hand-edit `.gsd/REQUIREMENTS.md` or completion artifacts that have GSD tools.
- Do not hand-edit `Cargo.lock`.
- Be careful when changing verifier scripts: S05 overclaim guards were correct before final closure but must evolve for S06. Replace them with final-baseline evidence checks rather than simply deleting safeguards.
- Keep docs for a fresh reader. S06 closure prose should be action-oriented: exact proof commands, what passed, what was retired, and what remains intentionally future work.

## Open questions for executors

- What exact failures appear after the first S06 `cargo test --no-run` in the real worktree? The ledger lists likely blockers, but compile fronts can change as earlier blockers are retired.
- Does `cargo test --no-fail-fast` expose runtime regressions beyond fixture/docs drift? If yes, classify before fixing.
- Does any blocker truly require a follow-up split? Current M015 policy says fix by default; splitting requires explicit rationale and should probably involve the user/decision capture.
