---
id: T02
parent: S08
milestone: M011
key_files:
  - src/combat/kit.rs
  - src/combat/round_flags.rs
  - src/combat/state.rs
  - src/data/skills_ron.rs
  - src/data/units_ron.rs
  - src/combat/events.rs
  - src/combat/resolution.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/turn_system/mod.rs
  - src/combat/follow_up.rs
  - src/combat/bootstrap.rs
  - src/headless.rs
  - src/windowed.rs
  - src/bin/combat_cli.rs
  - assets/data/units.ron
  - assets/data/skills.ron
  - tests/form_identity.rs
key_decisions:
  - GrantEnergy handled in step_app via separate Query<&mut Energy> to avoid breaking 15 resolution_tests.rs callsites
  - FormIdentityKit as separate ECS component (not extending UnitSkills) to avoid 25+ construction sites
  - OnFirstHitVsTagThisRound matches any OnDamageDealt {amount>0} from unit — tag specificity deferred to T03
  - triggered_this_frame HashSet guard prevents duplicate form identity scheduling within a single listener invocation
  - form_identity_used flag set by resolve_follow_up_action_system post-execution to enforce once-per-round
duration: 
verification_result: passed
completed_at: 2026-04-28T10:12:38.177Z
blocker_discovered: false
---

# T02: Form Identity infrastructure built end-to-end: FormIdentityKit component, GrantEnergy effect, EnergyGained event, once-per-round listener, Greymon canonical demo wired in data + 3 integration tests green.

**Form Identity infrastructure built end-to-end: FormIdentityKit component, GrantEnergy effect, EnergyGained event, once-per-round listener, Greymon canonical demo wired in data + 3 integration tests green.**

## What Happened

Built the full Form Identity pipeline from schema to integration test.

Schema layer: Added FormIdentityTrigger enum (4 variants), FormIdentityConfig struct, and FormIdentityKit ECS component to kit.rs. Added form_identity_used bool field to RoundFlags. Added energy_grant: i32 to ResolvedAction. Added GrantEnergy(i32) to the Effect enum in skills_ron.rs and EnergyGained { unit_id, amount } to CombatEventKind. Added form_identity: Option<FormIdentityConfig> to UnitDef with serde(default).

Effect application: Added skill_grant_energy helper in resolution.rs and wired energy_grant into resolve_action. Added 0-damage guard in apply_effects to prevent spurious OnDamageDealt events for modifier-only skills. Added energy_q: &mut Query<&mut Energy> parameter to step_app in pipeline.rs with the energy grant block after apply_effects. Updated resolve_action_system in turn_system/mod.rs to pass energy_q and reset form_identity_used alongside break_sealed in advance_turn_system.

Listener/pipeline: Added FollowUpOriginKind enum (FollowUp/FormIdentity) to follow_up.rs. Added form_identity_listener_system that builds fi_snapshots + target_snapshots once, uses triggered_this_frame: HashSet guard, and emits FollowUpIntent with origin_kind: FormIdentity. Updated resolve_follow_up_action_system to accept energy_q, pass it to step_app, and set form_identity_used = true after resolving FormIdentity intents. Updated bootstrap.rs to spawn FormIdentityKit when def.form_identity.is_some(). Registered form_identity_listener_system in headless.rs, windowed.rs, and combat_cli.rs.

Data: Added form_identity trigger OnFirstHitVsTagThisRound(Fire) → greymon_form_identity to Greymon in units.ron. Added greymon_form_identity skill (GrantEnergy(5), no damage) to skills.ron.

Tests: Created tests/form_identity.rs with 3 integration tests: first fire hit grants +5 Energy + sets form_identity_used=true; second hit in same round is blocked (energy stays at 5); resetting form_identity_used (simulating advance_turn_system) allows the grant to fire again next round. Fixed existing tests in bootstrap_spawn_composition.rs, roster_smoke.rs, tempo_resistance.rs, follow_up_chains.rs that construct UnitDef directly — added form_identity: None to each literal.

Key design decisions: energy grant handled in step_app via separate Query<&mut Energy> (not inside apply_effects) to avoid breaking 15 resolution_tests.rs callsites. FormIdentityKit as separate component (not extending UnitSkills) to avoid touching 25+ UnitSkills construction sites. OnFirstHitVsTagThisRound matches any OnDamageDealt { amount > 0 } from the unit — tag specificity deferred to T03 since OnDamageDealt doesn't carry DamageTag yet.

## Verification

cargo check (clean, warnings only, pre-existing) and cargo test (all passing, 0 failures)

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check` | 0 | Clean compile, warnings only (pre-existing) | 1480ms |
| 2 | `cargo test` | 0 | All test suites pass including 3 new form_identity tests | 4200ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/combat/kit.rs`
- `src/combat/round_flags.rs`
- `src/combat/state.rs`
- `src/data/skills_ron.rs`
- `src/data/units_ron.rs`
- `src/combat/events.rs`
- `src/combat/resolution.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/follow_up.rs`
- `src/combat/bootstrap.rs`
- `src/headless.rs`
- `src/windowed.rs`
- `src/bin/combat_cli.rs`
- `assets/data/units.ron`
- `assets/data/skills.ron`
- `tests/form_identity.rs`
