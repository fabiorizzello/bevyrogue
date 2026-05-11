# S05 Research: Shared-surface CLI proof

## Summary

S05 should be a targeted implementation slice, not a broad architecture redesign. The core shared surfaces already exist and `src/bin/combat_cli.rs` already consumes several of them: it builds `CombatQuerySnapshot` via `build_snapshot_from_ecs_with_sp`, calls `query_action_affordance`, emits `ActionIntent`, registers `register_combat_kernel_runtime`, and logs `CombatEvent` messages. The missing work is to make the real CLI runtime actually run deterministically, emit/verify a validation snapshot from the same world, and add a binary-level proof that observes action query output, canonical events, canonical beats/kernel transitions, and snapshot state without adding CLI-only combat rules.

Big surprise: the current real command `BEVYROGUE_JSONL=1 cargo run --bin combat_cli` panics before any shared-surface proof appears. The panic text is `Message not initialized`, and static inspection shows `advance_turn_system` requires `MessageWriter<ActionValueUpdated>` while `combat_cli` registers `TurnAdvanced`, `ActionIntent`, `FollowUpIntent`, `FollowUpTrace`, and `CombatEvent` only. `src/main.rs` registers `ActionValueUpdated`, so the CLI has drifted from the headless app wiring. Also note the process returned exit 0 even though the worker thread panicked, so S05 tests must assert absence of `panicked` / `Message not initialized`, not just success status.

## Requirements Targeted

- **Owns R096 / R098:** prove `combat_cli` consumes shared action query, events, beats, kernel-observable state, and snapshots through the real binary.
- **Supports R094:** keep generic kernel/event/snapshot authority visible in the CLI proof.
- **Supports R095 / S04 output:** CLI must not parse `animation_sequence`, `qte`, beat metadata wording, or presentation trigger text for gameplay outcomes.
- **Supports R100:** keep verification deterministic/headless; final full-suite baseline remains S06.
- **Does not own R097 final closure:** S06 still owns full `cargo test` / no-fail-fast green baseline and broad stale fixture repair.

## Skill / Process Notes

- Required skill `api-design` was loaded. It is mostly a negative finding here: there is no HTTP/GraphQL API, but its “caller contract” principle still applies. Treat the CLI proof output as a stable machine-readable caller contract: honest status, named output markers, and no hidden 0-exit panic.
- Required skill `grill-me` was loaded. No user questions are needed because the repo already answers the root decisions: real CLI proof, shared surfaces only, and no presentation metadata authority.
- Skill discovery ran for core tech. Installed globally for downstream units:
  - `apollographql/skills@rust-best-practices` (`rust-best-practices`, high relevance for Rust code/tests).
  - `bfollington/terma@bevy` (`bevy`, directly relevant but low install count; the skills CLI reported “Gen Critical Risk”, so review before relying on it for privileged guidance).

## Implementation Landscape

### `src/bin/combat_cli.rs`

Current responsibilities:
- Synchronously loads ally roster from `assets/data/units.ron` in `load_ally_roster()` using a cwd-relative path.
- Builds the Bevy app with `MinimalPlugins`, `AssetPlugin::default()`, `DataPlugin`, combat resources, and messages.
- Calls `register_combat_kernel_runtime(&mut app)`.
- Registers CLI systems in a chained `Update` tuple.
- `player_action_system` builds a `CombatQuerySnapshot` with `build_snapshot_from_ecs_with_sp`, builds action entries with `query_action_affordance`, prints `Action affordances`, and in non-interactive mode chooses the first enabled Basic target via `first_enabled_target_id`.
- `event_logger_system` prints `CombatEventKind` debug output; `jsonl_logger_system` emits serializable event JSON when `BEVYROGUE_JSONL` is set.

Gaps:
- Missing `.add_message::<ActionValueUpdated>()` import/registration. This blocks `advance_turn_system` at runtime.
- No call to `capture_validation_snapshot` / `format_validation_snapshot`, so CLI cannot currently prove the snapshot surface.
- Asset paths are cwd-sensitive: `load_ally_roster()` reads `assets/data/units.ron`, and `AssetPlugin::default()` expects `assets/` under the process cwd. This is explicitly named in the M015 ledger as S05-owned.
- `DataReady` is set by `DataPlugin` after roster + party load only; skill-book readiness is not part of `DataReady`. `player_action_system` has a fallback `SkillBook(Vec::new())` and prints `[QUERY] Skill book unavailable; using query MissingSkill affordances.`. The CLI proof should fail if this fallback appears, because S05 needs real shared action query over loaded skills.
- Non-interactive timeout is fixed at ~6s (`counter.0 > 60` at 10Hz). For a fast integration proof, add a proof-mode exit after snapshot or an env-controlled tick budget.

### `src/main.rs` / `src/headless.rs`

Useful reference patterns:
- `src/main.rs` registers `ActionValueUpdated`; copy that message registration into the CLI.
- `src/headless.rs` has `headless_validation_snapshot_once(world: &mut World)` using `capture_validation_snapshot(world)` and `format_validation_snapshot(&snapshot)`. The CLI can use the same exclusive-system pattern for proof-mode snapshot emission.
- `src/headless.rs` also shows the intended app-level wiring: add common messages/resources, call `combat::kernel::register_combat_kernel_runtime`, then register combat systems.

### `src/combat/action_query.rs`

This is the legality/query authority for CLI/UI consumers:
- `build_snapshot_from_ecs_with_sp` is explicitly documented as the UI/CLI preflight path using real `SpPool.current`.
- `query_action_affordance`, `query_intent_legality`, and `first_enabled_target_id` are the shared surfaces CLI should continue using.
- Do not add target legality or SP/ultimate rules in `combat_cli.rs`; route through these functions.

### `src/combat/turn_system/mod.rs` and `src/combat/turn_system/pipeline.rs`

Canonical runtime emissions:
- `resolve_action_system` performs early legality validation with `query_intent_legality` when the skill book is available.
- `emit_combat_beat` emits both `CombatEventKind::OnCombatBeat` and mirrored `CombatKernelTransition::Beat` through `emit_kernel_transition`.
- `pipeline::step_app` emits `CombatBeatId::Impact`/`Damage` and dispatches blueprint transitions only after `outcome.succeeded`.
- `advance_turn_system` needs `ActionValueUpdated`; this is the current CLI panic source.

### `src/combat/events.rs` / `src/combat/jsonl_logger.rs`

The canonical event proof surface is already serializable:
- `CombatEventKind` includes `OnActionDeclared`, `OnActionPreApp`, `OnCombatBeat`, `OnKernelTransition`, `OnActionApplied`, `OnActionResolved`, damage/cast events, and mechanic-specific resolved events.
- `jsonl_logger_system` prints each `CombatEvent` as one JSON line when `BEVYROGUE_JSONL` is set. This is the best real-binary assertion surface for S05.

### `src/combat/observability.rs`

Snapshot proof surface:
- `capture_validation_snapshot(world)` requires live resources including `CombatState`, `SpPool`, `ActionLog`, `TwinCoreState`, unit components, and mechanic resources.
- `format_validation_snapshot` prints `phase`, `sp`, `twin_core`, `holy_support`, `predator_loop`, `precision`, `turn_preview`, `action_log_tail`, `floating_live`, and units.
- Since `register_combat_kernel_runtime` initializes `HolySupportState`, `PredatorLoopState`, etc., a CLI snapshot should include `holy_support=grace=...` instead of `holy_support=none` when runtime wiring is correct.

### Existing Tests / Docs

- `tests/action_affordance_consumers.rs` contains source-scan guards that `src/bin/combat_cli.rs` must not reintroduce hardcoded skill/enemy/KO rules. However this file is currently among broad `SkillDef` constructor-drift blockers, so S05 should not rely on running it until S06 repairs broad fixtures.
- `tests/presentation_metadata_boundary.rs` is the runtime pattern for asserting events/snapshots ignore presentation metadata.
- `tests/patamon_blueprint_seam.rs` proves the Patamon/Holy Support seed in tests; S05 does not need to reimplement that logic in CLI.
- `docs/combat_authority_map.md`, `docs/combat_mixed_pattern_drift_ledger.md`, and `docs/m015_failure_ledger.md` already name CLI proof as pending S05 work and should be updated when S05 closes it.
- `scripts/verify_combat_authority_audit.py` currently expects D9 to say “downstream CLI proof”; S05 will likely need to update this verifier if the docs change D9 to closed/normalized-by-S05.
- `scripts/verify_m015_failure_ledger.py` currently expects the marker `S05 (CLI / asset-path / consumer proof repair)`. Either preserve this historical marker in the ledger while adding closed evidence, or update the verifier alongside the ledger.

## Recommended Build Order

1. **Fix real CLI startup drift first.**
   - Import `ActionValueUpdated` into `src/bin/combat_cli.rs`.
   - Add `.add_message::<ActionValueUpdated>()` next to the other message registrations.
   - Re-run `BEVYROGUE_JSONL=1 cargo run --bin combat_cli` and assert there is no panic text.

2. **Make CLI asset loading cwd-stable.**
   - Add a small helper using `env!("CARGO_MANIFEST_DIR")` to locate the repository root/assets in dev builds.
   - Use it in `load_ally_roster()` instead of cwd-relative `assets/data/units.ron`.
   - Configure `AssetPlugin` with an absolute `file_path` pointing at `<manifest>/assets` so `DataPlugin` loads `data/*.ron` even when the binary is run from another cwd.
   - This is control-plane/runtime setup, not gameplay logic.

3. **Add a proof-mode snapshot/exit surface.**
   - Add a `CliProofConfig` or simple env gate such as `BEVYROGUE_CLI_PROOF=1`.
   - Reuse the headless exclusive-system pattern: once `DataReady` exists, units are spawned, and `ActionLog` is non-empty, call `capture_validation_snapshot(world)` and print a stable marker such as `[CLI_PROOF] validation_snapshot: {formatted}`.
   - In proof mode, optionally exit successfully immediately after snapshot emission. This keeps the integration test fast and avoids waiting for the fixed 6s timeout.
   - Treat `[QUERY] Skill book unavailable` as proof failure. Either ensure `DataReady` includes skills or make the CLI proof wait until `SkillBookHandle` resolves to an asset before acting.

4. **Add a real-binary integration test.**
   - New file suggestion: `tests/combat_cli_shared_surface.rs`.
   - Use only `std::process::Command`; no dev dependency is present and none is needed.
   - Run `env!("CARGO_BIN_EXE_combat_cli")` with `BEVYROGUE_JSONL=1` and proof-mode env enabled.
   - Set `current_dir` to a non-root existing dir such as `<manifest>/target` to prove the asset-root fix.
   - Assert combined stdout/stderr contains:
     - `Action affordances` (shared action query consumer)
     - `OnCombatBeat` (canonical beat event)
     - `OnKernelTransition` and/or serialized `"Beat"` transition (kernel beat mirror)
     - `OnActionResolved` or a damage/cast event (canonical event stream)
     - `[CLI_PROOF] validation_snapshot` and `holy_support=grace=` (snapshot + kernel runtime resources)
   - Assert it does **not** contain:
     - `panicked`
     - `Message not initialized`
     - `[QUERY] Skill book unavailable`
   - Keep the test targeted; do not depend on broad stale fixture tests compiling.

5. **Update S05 docs/verifiers.**
   - Add a small durable doc, e.g. `docs/combat_cli_shared_surface_proof.md`, explaining the proof env vars and exactly which shared surfaces the CLI consumes.
   - Update `docs/combat_authority_map.md`: change the CLI row from “proof pending” to “S05 proof via real binary”, while preserving the broader “full CLI/future UI migration not complete” boundary if the verifier still needs the phrase.
   - Update `docs/combat_mixed_pattern_drift_ledger.md`: close D3/D9 as S05-normalized for CLI runtime/snapshot proof; keep broad fixture/full-suite work assigned to S06.
   - Update `docs/m015_failure_ledger.md`: add the S05 CLI proof command as ✅ pass, mark the runtime/consumer gap closed by S05, and leave broad `cargo test --no-run` blockers under S06.
   - Update `scripts/verify_combat_authority_audit.py` and/or `scripts/verify_m015_failure_ledger.py` if their marker expectations conflict with the new closed S05 wording.

## Natural Seams for Planning

- **Task A — CLI runtime boot repair:** message registration + cwd-stable asset root + skill readiness guard. This should be built and smoke-run before any docs.
- **Task B — CLI proof observability:** proof-mode config, snapshot emission, optional proof exit. Depends on Task A.
- **Task C — real-binary test:** `tests/combat_cli_shared_surface.rs`. Depends on A+B.
- **Task D — docs/audit gates:** authority map, drift ledger, failure ledger, optional new proof doc, verifier updates. Depends on C evidence.

These seams are mostly sequential because the test and docs need the actual CLI evidence. Docs/verifier updates can be parallelized after the real-binary output is known.

## Verification Plan

Minimum S05 completion command after implementation:

```bash
BEVYROGUE_JSONL=1 BEVYROGUE_CLI_PROOF=1 cargo run --bin combat_cli
cargo test --test combat_cli_shared_surface -- --nocapture
python3 scripts/verify_combat_authority_audit.py
python3 scripts/verify_m015_failure_ledger.py
```

Recommended regression bundle, still targeted and headless:

```bash
cargo test --test combat_cli_shared_surface \
  --test presentation_metadata_boundary \
  --test event_stream \
  --test patamon_blueprint_seam && \
python3 scripts/verify_combat_authority_audit.py && \
python3 scripts/verify_m015_failure_ledger.py
```

Do **not** claim final milestone closure from this. `cargo test --no-run` / `cargo test --no-fail-fast` broad suite remains S06 because broad stale `SkillDef`/`UnitDef` constructors and missing UI docs are already classified in `docs/m015_failure_ledger.md`.

## Risks / Pitfalls

- **Exit code alone is insufficient.** The current CLI panic returned exit 0 in the smoke run. Tests must inspect output for panic markers.
- **Do not add CLI-only combat rules.** It is acceptable to add env-gated proof controls (party/cwd/proof exit/snapshot). It is not acceptable to decide legality, damage, target validity, ultimate readiness, or Holy Support outcomes in CLI code.
- **Skill readiness matters.** A CLI proof that prints `[QUERY] Skill book unavailable` is not proof of the shared query contract; it is the fallback missing-skill path.
- **Snapshot timing matters.** If future proof asserts Holy Support changes, ensure the snapshot is captured after kernel applier systems have processed `OnKernelTransition`. For a minimal S05 proof, seeing initialized `holy_support=grace=0/...` plus beat/kernel events may be enough.
- **Avoid over-expanding into full Patamon scripting unless needed.** Patamon ultimate is not ready at startup and the default non-interactive party does not include Patamon. S03 already proves the Patamon blueprint seam; S05 should prove CLI surface consumption unless the planner explicitly chooses to add env-scripted party/action setup.
- **Broad tests are still blocked.** Adding a new targeted integration test is fine, but running all tests will still hit S06-owned stale fixture drift.

## Sources / Evidence

- `src/bin/combat_cli.rs` — current CLI runtime, query consumer, and missing snapshot/message registration.
- `src/main.rs` — correct app message registration includes `ActionValueUpdated`.
- `src/headless.rs` — existing validation snapshot exclusive-system pattern.
- `src/combat/action_query.rs` — shared UI/CLI query surface.
- `src/combat/turn_system/mod.rs` and `src/combat/turn_system/pipeline.rs` — canonical legality, event, beat, and kernel transition emissions.
- `src/combat/events.rs` and `src/combat/jsonl_logger.rs` — serializable event stream.
- `src/combat/observability.rs` — validation snapshot capture/format surface.
- `src/data/mod.rs` — current `DataReady` tracks roster + party but not skill book readiness.
- `docs/m015_failure_ledger.md` — S05-owned CLI / asset-path / consumer proof gap and S06-owned broad blockers.
- `docs/combat_authority_map.md` / `docs/combat_mixed_pattern_drift_ledger.md` — S03/S04 authority map and D3/D9 S05 handoff.
- `gsd_exec` smoke evidence `2eb27128-e12c-46db-9d54-5b63c5060197`: `BEVYROGUE_JSONL=1 cargo run --bin combat_cli` panicked with `Message not initialized`, emitted no query/event/beat/snapshot markers, and still exited 0.
