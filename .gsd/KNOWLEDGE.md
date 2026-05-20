# Project Knowledge

Append-only register. Agents read this before every unit.

## Onboarding

```bash
cargo check                   # headless (default)
cargo test                    # integration suite (tests/)
cargo run                     # headless run
cargo run --features windowed # egui UI
```

Toolchain: `rust-toolchain.toml`. Dev profile: cranelift (`Cargo.toml`).
Test workflow (nextest `agent` profile, seeded `bevy_rand`, insta snapshots): `docs/agent-testing.md`.
## Rules

| # | Scope | Rule | Added |
|---|-------|------|-------|
| R001 | Passive bootstrap | Per-Digimon passive listeners bootstrapped from `CombatPlugin` post core-init: fixed canonical `UnitId` owners, shared `kernel/ult_used` triggers, per-blueprint guard keys. Declarative, no per-test scaffolding. | 2026-05-17 |
| R002 | Headless-first | Every system runs without `windowed`. Gate egui/winit only via `#[cfg(feature = "windowed")]`. | 2026-05-18 |
| R003 | Test layout | Integration tests in `tests/`, functional names (`follow_up_triggers.rs`, not `s10_…`). Shared helpers in `tests/common/{units,actions,apply}.rs` (`mod common;`). `rstest` for table cases, `proptest` for invariants (see `tests/README.md`). No `src/` unit tests except short `#[cfg(test)] mod tests`. | 2026-05-18 |
| R004 | Determinism | No wall-clock, no unseeded RNG. Seeded `bevy_rand` + insta depend on it. | 2026-05-18 |
| R005 | Dep gating | No winit/wgpu/egui deps outside `windowed`. | 2026-05-18 |
| R006 | Repo hygiene | No `.md` in repo root — use `docs/` or `.gsd/`. | 2026-05-18 |

## Patterns

| # | Pattern | Where | Notes |
|---|---------|-------|-------|
| P001 | Keep combat dispatch metadata on `InFlightAction`, not `ResolvedAction`. | `src/combat/turn_system/` | `ResolvedAction` stays semantic; routing details for the pipeline live on the in-flight execution wrapper. |
| P002 | Reuse the same compiled timeline resolution/interner path for preview and execute. | `src/combat/preview.rs`, timeline-backed consumers | Preview should run `BeatRunner` in preview mode and return the pending queue without touching `intent_applier`, so preview/execution drift is caught early. |
| P003 | Use owner-gated generic Blueprint envelopes while preserving typed owner-side observability seams. | Blueprint runtime modules + observability surfaces | Raw transport can stay generic without breaking downstream assertions if the owner module preserves the typed resolved-state contract. |
| P004 | Apply damage modifiers in canonical order: Intrinsic → Status → Buff → Passive. | Damage modifier ledger / incoming-damage pipeline | Fixed fold order keeps layered mitigation deterministic and replayable regardless of insertion order. |
| P005 | Blueprint `*Snapshot::last_transition` fields are typed observability contract surfaces, not internal latches. | `src/combat/blueprints/{tentomon,dorumon}/{identity,apply,mod}.rs` | Consumed by `ValidationExt` rows + per-step JSONL transition logging. Tests asserting on these fields are the regression guard for those display surfaces. See D026. |

## Lessons Learned

| # | What Happened | Root Cause | Fix | Scope |
|---|--------------|------------|-----|-------|
| L001 | Passive listeners that enqueue state changes for later predicates can loop forever or read stale state. | Queued intents were not flushed between `BeatRunner` steps, so later predicates observed old state. | Flush queued intents between passive-runner steps; for outer passive timelines, stop when the cursor cycles back to the entry beat while keeping explicit `BeatKind::Loop` bodies on the normal 256-hop breaker. | Passive timelines / follow-up runtime |
| L002 | Pre-damage events like `IncomingDamage` did not affect the current hit even when reactions subscribed to them. | `IncomingDamage` is observational only; modifiers were being armed too late. | Arm target-scoped modifiers in state before the damage intent is processed, then let the damage applier drain the ledger and emit the post-mitigation trigger once. | Incoming-damage / mitigation pipeline |
| L003 | A `#[ignore]`d unit test (`follow_up_low_hp_event_targets_alive_enemy`) claimed the timeline pipeline does not emit `OnAllyLowHp`/`OnEnemyBreak`. Un-ignored, the break test passed; the low-hp one still failed. | The ignore reason was stale. `OnAllyLowHp` (and break) threshold-crossing events ARE emitted for real RON-driven skills (proven by passing `renamon_low_hp_follow_up_targets_the_attacker` / `gabumon_triggers_follow_up_on_ally_low_hp`). The failing test used the synthetic `skill()` builder in `follow_up/mod.rs` tests, whose compiled timelines do not reproduce real-skill damage-derived observability. | Kept the now-passing break unit test; deleted the superseded synthetic low-hp test (behavioral coverage lives in the real-config integration tests). Assert `OnAllyLowHp`/break follow-ups via integration tests with real pilot config, not synthetic-skill unit tests. | Follow-up trigger tests / timeline observability |
