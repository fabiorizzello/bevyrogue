---
id: T04
parent: S01
milestone: M021
key_files:
  - src/combat/plugin.rs
  - src/combat/api/clock.rs
  - src/combat/mod.rs
  - src/lib.rs
  - src/main.rs
  - src/bin/combat_cli.rs
key_decisions:
  - CombatPlugin::build calls register_combat_kernel_runtime then appends framework Resources — preserves existing kernel wiring without duplication
  - insert_resource(CombatRng::from_seed(0xDEAD_BEEF)) used for canonical seeding; headless.rs keeps its init_resource which would be a no-op after plugin runs (Bevy init_resource skips if already present)
  - Clock enum gained Resource derive in api/clock.rs; inserted as Clock::default() (HeadlessAuto) — windowed branch can overwrite via insert_resource if needed in future
  - CombatRng ownership moved to CombatPlugin; removed init_resource::<CombatRng>() from combat_cli.rs
duration: 
verification_result: passed
completed_at: 2026-05-14T23:30:46.413Z
blocker_discovered: false
---

# T04: CombatPlugin extracted as Bevy Plugin wrapper — mounts 6 framework Resources, delegates to register_combat_kernel_runtime, wires intent_applier; main.rs and combat_cli.rs updated to use add_plugins(CombatPlugin)

**CombatPlugin extracted as Bevy Plugin wrapper — mounts 6 framework Resources, delegates to register_combat_kernel_runtime, wires intent_applier; main.rs and combat_cli.rs updated to use add_plugins(CombatPlugin)**

## What Happened

Created src/combat/plugin.rs with CombatPlugin implementing Plugin::build. The build fn calls register_combat_kernel_runtime (existing kernel wiring preserved) then mounts the M021 framework Resources: init_resource for ExtRegistries, SignalBus, CastIdGen, IntentQueue; insert_resource for Clock::default() (HeadlessAuto) and CombatRng::from_seed(0xDEAD_BEEF); and registers intent_applier as an exclusive Update system. Added Resource derive to Clock enum in api/clock.rs (was missing). Added pub mod plugin + pub use plugin::CombatPlugin to combat/mod.rs. Added pub use combat::CombatPlugin to lib.rs so binary crates can reference it via bevyrogue::CombatPlugin. Updated main.rs: removed init_resource::<CastIdGen>() (now owned by plugin) and replaced register_combat_kernel_runtime call with .add_plugins(CombatPlugin). Updated combat_cli.rs: replaced use bevyrogue::combat::kernel::register_combat_kernel_runtime with use bevyrogue::CombatPlugin, removed init_resource::<CombatRng>() (now owned by plugin), replaced register_combat_kernel_runtime(&mut app) with app.add_plugins(CombatPlugin). All 208+ integration tests remain green.

## Verification

rg forbidden imports in src/combat/ (excluding blueprints) → 0 real matches (only a comment in api/mod.rs). rg 'CombatPlugin' src/lib.rs → 1 match. rg 'add_plugins.*CombatPlugin' src/main.rs → 1 match. rg 'register_combat_kernel_runtime' src/main.rs → 0 matches. cargo check (headless) → Finished with no errors. cargo check --features windowed → Finished with no errors. cargo test → 208+ passed, 0 failed.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `rg 'use bevy::winit|use bevy::render|use bevy_egui' src/combat/ --glob '!blueprints/**'` | 0 | 0 forbidden imports (only comment in doc string) | 50ms |
| 2 | `cargo check` | 0 | headless build clean | 2260ms |
| 3 | `cargo check --features windowed` | 0 | windowed build clean | 2090ms |
| 4 | `cargo test` | 0 | 208+ passed, 0 failed | 5000ms |

## Deviations

none

## Known Issues

headless.rs still calls init_resource::<CombatRng>() (Default seed 42) before CombatPlugin runs; since plugin uses insert_resource it overwrites to 0xDEADBEEF. Functionally correct but headless.rs carries a redundant call — not in Expected Output so left unchanged.

## Files Created/Modified

- `src/combat/plugin.rs`
- `src/combat/api/clock.rs`
- `src/combat/mod.rs`
- `src/lib.rs`
- `src/main.rs`
- `src/bin/combat_cli.rs`
