# PLAN — tests files unification

## Wave structure

- **One wave = one scope.** Atomic, revertable. Independent verification per scope.
- **Verification after every wave:** `cargo build --tests` (must compile). Full `cargo nextest run` only after every 3–4 scopes or at end (full suite is ~minutes).
- **Pilot first** to validate the pattern with the smallest no-common-deps scope.
- **Cargo.toml + snapshots last** to avoid churning entries while files are still moving.

## Migration mechanics (per scope)

For each scope `<scope>` with files `[A.rs, B.rs, ...]`:

1. `mkdir tests/<scope>/`
2. `git mv tests/A.rs tests/<scope>/A.rs` (for each file)
3. For files containing `mod common;` → rewrite to remove the declaration; rely on the harness's `#[path = "common/mod.rs"] mod common;` and replace `use common::X` with `use crate::common::X`.
4. Create `tests/<scope>.rs` harness. **Important:** `tests/<scope>.rs` is the crate root of the integration test binary, so a bare `mod A;` looks for `tests/A.rs`, NOT `tests/<scope>/A.rs`. Every case `mod` (and `common`) must use `#[path = "..."]`:
   ```rust
   //! Aggregated harness for the <scope> domain. See .gsd/KNOWLEDGE.md R003.
   #[path = "common/mod.rs"]
   mod common;

   #[path = "<scope>/A.rs"]
   mod a;
   #[path = "<scope>/B.rs"]
   mod b;
   // ...
   ```
5. If any moved file is referenced by a `[[test]]` entry in `Cargo.toml`, **leave the entry for now** (Wave-final cleanup removes them all together). Cargo will emit both binaries until cleanup but that's harmless.
6. Run `cargo build --tests`. Fix any breakage.
7. Commit: `refactor(tests): aggregate <scope> into single harness`

### Edge cases

- **`tests/windowed_only.rs`**: harness leaves the case `mod` declarations unconditional. The case files retain their `#![cfg(feature = "windowed")]` inner attribute, which means the module body compiles to nothing under the headless feature set — the harness binary is emitted but has no test functions registered. Acceptable; equivalent to current behavior.
- **`encounter_bootstrap_windowed.rs`**: name is misleading (no `cfg` gate inside). Goes into `bootstrap_encounter`, NOT `windowed_only`.
- **`follow_up/triggers.rs`** (the snapshot owner): snapshot file relocated in the same commit as the move, so `cargo nextest run --test follow_up` finds it on first run.

## Wave order

Pilot smallest no-common no-snapshot scope → then ordered by risk (size + complexity).

| Wave | Scope | N | Has `mod common;` | Notes |
|---|---|---|---|---|
| W1 | **effects_kernel** (pilot) | 3 | 0 | No common, no snapshots, validates pattern |
| W2 | invariants | 2 | 0 | Self-contained (proptest) |
| W3 | windowed_only | 2 | 0 | `cfg(windowed)` per file, validate that feature path still compiles |
| W4 | passives_infra | 4 | 1 | Tests `passive_reactive_canon` uses common |
| W5 | blueprints_infra | 3 | 1 | `form_identity` uses common |
| W6 | follow_up | 3 | 2 | **Snapshot relocation here** for `follow_up_triggers` |
| W7 | target_shape | 5 | 2 | `combat_resolution_bounce`, `combat_resolution_targets` |
| W8 | action_query | 5 | 0 | Self-contained per agent classifier |
| W9 | tempo_toughness | 6 | 1 | `tempo_resistance` uses common |
| W10 | assets_data | 7 | 0 | RON parsing, self-contained |
| W11 | bootstrap_encounter | 7 | 0 | Heavy (combat_cli_shared_surface spawns binary) |
| W12 | animation | 8 | 0 | All animation-specific |
| W13 | damage_resolution | 8 | 5 | High common usage |
| W14 | turn_economy | 10 | 2 | `combat_resolution_streak`, `turn_system_av` |
| W15 | status_effects | 11 | 4 | `status_*` cluster |
| W16 | preview_ai | 6 | 2 | `enemy_ai`, `scenario_ttk` |
| W17 | runtime_events_obs | 13 | 0 | Mostly `*_internals.rs` |
| W18 | timeline | 16 | 3 | Largest; high pipeline_dispatch coupling |
| W19 | digimon_kits | 16 | 4 | Largest; per-Digimon cluster |
| W20 | **Cargo.toml + R003 + README** | — | — | Remove 18 `[[test]]` entries, update `KNOWLEDGE.md` R003, update `tests/README.md` |

**Total waves: 20** (19 scope migrations + 1 cleanup). Each commits in conventional format: `refactor(tests): aggregate <scope>` or `refactor(tests): cleanup Cargo.toml + KNOWLEDGE`.

## Verification gates

- **After each wave:** `cargo build --tests` must succeed.
- **After waves W6, W11, W15, W19:** `cargo nextest run --test <scope>` for the just-migrated scope (smoke).
- **After Wave W20:** full `cargo nextest run` + `cargo build --tests --features windowed` + grep for orphan references to old paths.

## Rollback

Each wave is one commit. `git revert` works at scope granularity. The 29-file `mod common;` rewrite is the only non-rename change; isolated per case file, easy to undo per scope.

## Risks & mitigations

| Risk | Mitigation |
|---|---|
| Insta snapshot path divergence (only 1 snapshot) | Move snapshot in same commit as `follow_up/triggers.rs` (W6). Verify `cargo nextest run --test follow_up` matches. |
| Hidden symbol-name collisions across cases (e.g., two `mod tests {}` blocks with same fn name in different cases) | Each case is a `mod`, so functions are namespaced. Collisions surface as compile errors per scope, caught by W's `cargo build --tests`. |
| `default = ["dev"]` regression under `--no-default-features` | Out of scope for this refactor; W3 (windowed_only) and W20 verify windowed feature still compiles. |
| Test count drop between before/after (silent skip) | W20 final verification compares `cargo nextest list \| wc -l` before vs after. |
