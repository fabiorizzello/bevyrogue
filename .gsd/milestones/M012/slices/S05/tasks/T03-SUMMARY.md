---
id: T03
parent: S05
milestone: M012
key_files:
  - src/combat/follow_up.rs
  - src/combat/turn_system/pipeline.rs
  - src/combat/turn_system/mod.rs
  - tests/form_identity.rs
  - tests/resource_caps.rs
key_decisions:
  - (none)
duration: 
verification_result: mixed
completed_at: 2026-05-01T08:21:53.491Z
blocker_discovered: true
---

# T03: Partially wired canonical Form Identity follow-up execution through the cap-aware pipeline, but same-entity self-target application and DORUgamon trigger handling still need follow-up work.

**Partially wired canonical Form Identity follow-up execution through the cap-aware pipeline, but same-entity self-target application and DORUgamon trigger handling still need follow-up work.**

## What Happened

I updated the Form Identity runtime path so follow-up declarations can identify self-only hidden effects, and I taught the trigger listener to consult the skill book for cast-tag matching. I also added production-like RoundEnergyTracker fixtures to the Form Identity tests and started a canonical resource-cap regression that exercises Greymon grants across tracker resets. During verification, the suite still fails: Greymon’s hidden self-only GrantEnergy follow-up schedules but does not apply energy, and DORUgamon’s canonical cast-trigger still does not land its toughness follow-up. The remaining work is to finish same-entity self-target application in the action pipeline and reconcile the DORUgamon trigger path with the tested canonical behavior before the slice can be closed.

## Verification

Fresh diagnostics were clean on the edited files, but the required focused suite is still red. `cargo test --test form_identity --test resource_caps` currently fails in `tests/form_identity.rs` on Greymon energy assertions and DORUgamon toughness assertions; the latest trace check confirmed Greymon Form Identity is at least being scheduled, so the remaining gap is in same-entity execution/application rather than pure trigger discovery.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo test --test form_identity --test resource_caps` | 101 | ❌ fail | 21200ms |
| 2 | `cargo test --test form_identity greymon_first_fire_hit_grants_energy -- --nocapture` | 101 | ❌ fail | 20100ms |

## Deviations

I expanded the work beyond the original narrow declaration gate because the canonical self-only follow-up was still blocked by same-entity execution semantics. I also added a targeted `FollowUpTrace` assertion in the Greymon test to separate scheduling from application behavior.

## Known Issues

`tests/form_identity.rs` still fails for Greymon/Garurumon/Kabuterimon/Kyubimon/DORUgamon canonical cases; `tests/resource_caps.rs` was updated with the new canonical tracker regression but has not yet been re-verified against passing runtime behavior.

## Files Created/Modified

- `src/combat/follow_up.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/turn_system/mod.rs`
- `tests/form_identity.rs`
- `tests/resource_caps.rs`
