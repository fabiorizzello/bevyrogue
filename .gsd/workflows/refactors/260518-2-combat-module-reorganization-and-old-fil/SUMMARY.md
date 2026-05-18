# Summary — Combat module reorganization and old files cleanup

Date: 2026-05-18
Branch: `gsd/refactor/combat-module-reorganization-and-old-fil`

## What changed

Combat root went from **34 flat .rs files with 4 submodules** to **12 root files with 10 submodules**, each scoped by responsibility. The misleading `api/` module was renamed to `runtime/`. Six large files (>870 LOC) were split into submodule directories. Shared demo skills were removed.

## Commits

| Wave | Commit | Description | Files |
|------|--------|-------------|-------|
| 6 | `3e16435` | Move 22 flat files into mechanics/, encounter/, observability/, turn_system/ | 27 |
| 7 | `6bd8c42` | Remove shared demo skills, update consumers, update 3 captures | 9 |
| 8 | `3328f40` | Rename api/ → runtime/ across 103 files | 103 |
| 9 | `cf0effe` | Split pipeline.rs, resolution.rs, action_query.rs, turn_system/mod.rs | 14 |
| 10 | `a805c25` | Split applier.rs and follow_up.rs into submodules | 8 |

## Module structure (post-refactor)

```
src/combat/
├── mod.rs, plugin.rs, preview.rs        # Core wiring
├── types.rs, unit.rs, team.rs, kit.rs   # Vocabulary
├── state.rs, rng.rs, bevy_types.rs      # Resources
├── runtime/                             # Execution engine (was api/)
│   ├── applier/{mod,effects}.rs
│   ├── runner.rs, runner_common.rs
│   ├── intent.rs, signal.rs, skill_ctx.rs
│   ├── timeline.rs, builtins.rs, clock.rs
│   ├── registry.rs, rng.rs
│   ├── blueprint_state.rs, passive_runner.rs
│   └── event_bridge.rs, event_filter.rs
├── kernel/{mod,primitives}.rs           # Kernel primitives
├── turn_system/                         # Turn pipeline + AV
│   ├── mod.rs, types.rs, helpers.rs
│   ├── pipeline/{mod,declaration,application,timeline_exec}.rs
│   ├── av.rs, speed.rs, turn_order.rs, resistance.rs
│   └── tests.rs
├── mechanics/                           # Combat mechanics
│   ├── buffs, damage, energy, modifiers, round_flags
│   ├── sp, status_effect, stun, toughness, ultimate
│   └── follow_up/{mod,types,triggers,form_identity,resolve}.rs
├── resolution/{mod,types,skill_extract,apply}.rs
├── action_query/{mod,types,legality}.rs
├── encounter/{bootstrap,counterplay,enemy_ai}.rs
├── observability/{mod,format,snapshot,events,floating,jsonl_logger,log}.rs
└── blueprints/                          # Per-digimon blueprints
    ├── agumon/, dorumon/, gabumon/, patamon/, renamon/, tentomon/
    └── twin_core/
```

## Captures

All 6 captures verified:
- CAP-8d133d1a (kernel cleanup): **done**
- CAP-af4db4ca (counterplay): **done**
- CAP-7c065a44 pt.1 (asset split): **done**
- CAP-7c065a44 pt.2 (visual arch): deferred → M023
- CAP-159d33b5 (animation sync): deferred → M023
- CAP-b892da3a (fluent DSL): deferred → future milestone

## Follow-up candidates

- `turn_system/pipeline/application.rs` (1963 LOC) — still large, could split per-path (multi-target, bounce, self-target, standard)
- `resolution/mod.rs` (1399 LOC) — tests block is ~1000 LOC, could extract to `resolution/tests.rs`
