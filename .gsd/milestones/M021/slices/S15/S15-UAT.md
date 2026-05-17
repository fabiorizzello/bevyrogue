# S15: Final milestone closeout evidence — UAT

**Milestone:** M021
**Written:** 2026-05-17T14:11:02.780Z

# S15 UAT

## Objective
Confirm the integrated M021 tree has fresh runtime-closeout evidence and that any remaining architecture-boundary mismatches are recorded explicitly instead of being misreported as passing.

## Steps
1. Run `cargo test` on the integrated tree.
2. Run `cargo check`.
3. Run `cargo check --features windowed`.
4. Run `rg -n -e "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/ --glob '!blueprints/**'` and record whether shared-naming hits remain.
5. Run `rg -n "enum Effect" src/data/skills_ron.rs` and confirm it returns no matches.
6. Run `rg -n "use bevy" src/combat/blueprints/` and record whether direct blueprint Bevy imports remain.

## Expected Results
- The three runtime commands exit 0.
- The `enum Effect` audit returns no matches.
- Any remaining shared-name or blueprint Bevy-import hits are captured explicitly as open architecture-boundary limitations rather than treated as hidden regressions.

## Evidence
- `cargo test` exited 0 after the harness fixes.
- `cargo check` exited 0.
- `cargo check --features windowed` exited 0.
- Shared-name audit still reports matches in shared combat modules.
- `enum Effect` audit reports no matches.
- Blueprint Bevy-import audit still reports direct imports under `src/combat/blueprints/`.

