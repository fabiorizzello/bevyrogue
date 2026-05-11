# Combat Current State

This is the current combat architecture entrypoint after M016. It replaces the older historical design/status docs as the first read for new combat work.

## Current authority stack

```text
RON data and typed custom signals
-> per-Digimon Rust blueprint logic (6 migrated identities, see roster below)
-> generic CombatKernelTransition values and shared hooks
-> canonical ECS state, CombatEvent stream, and ValidationSnapshot
-> CLI, UI, tests, logs, and presentation consumers
```

Current rule: gameplay authority lives in typed Rust combat paths. RON declares data and typed intent. It does not enumerate one Rust signal variant per Digimon. Consumers observe and render shared surfaces; they do not decide legality or outcomes.

## Canonical boundaries

- **RON:** owns numbers, targeting declarations, costs, metadata, presentation metadata, and typed `custom_signals`. It is not a gameplay scripting engine, and it does not need a static Digimon-by-Digimon signal enum.
- **Blueprints:** unique Digimon behavior belongs in per-Digimon Rust modules. The pattern is applied across 6 migrated identities (Patamon, Dorumon, Tentomon, Renamon, Agumon, Gabumon). Later roster identities follow the same blueprint+signal registration, never a shared-system character branch.
- **Kernel/hooks:** shared mechanics mutate canonical state through generic transitions and hook-owned resources.
- **Events/beats:** `CombatEventKind::OnCombatBeat` and `OnKernelTransition` are live combat output. Presentation consumes them; presentation does not author them.
- **Snapshots:** `ValidationSnapshot` is diagnostic truth over live state, not a second gameplay source.
- **CLI/UI:** must use shared action query, events, beats, kernel state, and snapshots. No CLI/windowed skill-ID-specific legality logic.

## Current proof surfaces

- Authority map: `docs/contracts/combat_authority_map.md`.
- Blueprint runtime proofs: `tests/dorumon_blueprint.rs`, `tests/dorumon_predator_runtime.rs`, `tests/tentomon_blueprint.rs`, `tests/renamon_precision_runtime.rs`, `tests/twin_core_integration.rs`, `tests/twin_core_roster_contract.rs`, `tests/battery_loop_kernel.rs`, `tests/predator_loop_kernel.rs`.
- Presentation boundary: `docs/contracts/presentation_metadata_boundary.md`, `tests/presentation_metadata_boundary.rs`.
- CLI shared-surface proof: `docs/contracts/combat_cli_shared_surface_proof.md`, `tests/combat_cli_shared_surface.rs`.
- UI/CLI legality presentation: `docs/contracts/combat_ui_readiness_gap_matrix.md`, `docs/contracts/skill_legality_contract.md`.
- M016 closure ledger: `docs/contracts/m016_closure_ledger.md`. GSD workflow trail in `.gsd/milestones/M016/`.

## Latest baseline

M016 closed with these checks green:

```bash
cargo test --no-run
cargo test --no-fail-fast
cargo test
cargo test --test combat_cli_shared_surface --test presentation_metadata_boundary --test event_stream --test patamon_blueprint_seam
cargo test --test tentomon_blueprint --test renamon_precision_runtime \
  --test twin_core_integration --test battery_loop_kernel --test predator_loop_kernel
BEVYROGUE_JSONL=1 BEVYROGUE_CLI_PROOF=1 cargo run --bin combat_cli
```

## Migrated identities (M015 + M016)

- **Patamon** — Holy Support (`src/combat/blueprints/patamon.rs`)
- **Dorumon** — Predator Loop (`src/combat/blueprints/dorumon.rs`)
- **Tentomon** — Battery Loop (`src/combat/blueprints/tentomon.rs`)
- **Renamon** — Precision/MindGame (`src/combat/blueprints/renamon.rs`)
- **Agumon, Gabumon** — Twin Core (`src/combat/blueprints/agumon.rs`, `src/combat/blueprints/gabumon.rs`)

Each blueprint owns signal interpretation; shared mechanic primitives (`holy_support.rs`, `predator_loop.rs`, `battery_loop.rs`, `precision_mind_game.rs`, `twin_core.rs`) own canonical kernel mutation.

## Open future work

- Full 12-Digimon roster lock (6/12 migrated).
- Playable CLI UX beyond proof binary.
- Windowed UI presentation completeness audit.
- Fatigue run-loop integration (first cross-encounter mechanic).
- Boss conversion policy.
- Heavy taxonomy formalization.

## Next milestone candidates

M016 closed (2026-05-11). Candidates for the next milestone:

1. Roster expansion (next 6 Digimon) following the same blueprint-owned pattern. Each new identity adds RON `custom_signals` + blueprint module + tests; no shared-system branches.
2. CLI UX completeness pass — promote `combat_cli` from proof binary to playable run-loop.
3. Fatigue run-loop integration.
4. Boss conversion policy + Heavy taxonomy lock.

Acceptance for each migrated Digimon:

- RON declares typed custom-signal intent only.
- Per-Digimon Rust blueprint owns unique interpretation.
- Shared kernel transition/hook owns canonical mutation.
- `CombatEvent` and `ValidationSnapshot` expose the result.
- Action query and CLI proof observe shared surfaces.
- Presentation metadata remains non-authoritative.
