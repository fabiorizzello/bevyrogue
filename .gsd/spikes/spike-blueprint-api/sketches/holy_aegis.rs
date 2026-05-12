// THROWAWAY SKETCH — NON-COMPILED, ILLUSTRATIVE ONLY.
// Spike SP2 — proves that Option B's `Blueprint` trait can express
// Patamon's Holy Aegis passive (round-3 canon) cleanly.
//
// Canon source: docs/future_design_draft/digimon/patamon/04_passive_holy_aegis.md
//   On CombatStarted: apply DR 10% Permanent Aura to all allies (incl. self).
//   On UnitDied(self): cleanse the Aegis buff from all allies (since it was
//                      sourced from this Patamon; canon: "buff dies with the
//                      Aegis owner").
//
// Required SP1 primitives:
//   * CombatEventKind::CombatStarted                       (likely already present;
//                                                          if not, SP1 should add it)
//   * CombatEventKind::UnitDied { unit, killer }           (SP1 §"Reactive signature bus")
//
// Required SP3 effects:
//   * Effect::ApplyBuff { id, target, mul, kind, dur }     (SP3 add-now §1)
//   * Effect::EmitCleanse { target, count, filter, priority } (SP3 add-now §3)
//   * BuffKind::DR                                         (SP3 add-now §1)
//   * BuffDuration::Permanent                              (SP3 add-now §1)
//   * TargetRef::AoE { side: TargetSide }                  (SP3 add-now §7)
//   * TargetSide::AllyTeamInclSelf                         (SP3 add-now §7)
//   * CleanseFilter::ById(BuffId)                          (SP3 add-now §3)
//
// Verdict: cleanly expressible. Zero commit-time signals, zero state, two
// distinct event-kind branches inside on_event. Trait defaults cover the rest.

use super::blueprint_trait::{Blueprint, BlueprintCtx, BlueprintId};

/// Patamon's Holy Aegis passive — listener-only blueprint.
///
/// Two reactive shapes hung off a single trait impl. The trait surface is
/// flexible enough to match arbitrary `CombatEventKind` patterns inside
/// one `on_event` body; no need for two separate trait impls.
pub struct HolyAegisBlueprint;

// Canonical buff id used both at apply-time (ApplyBuff.id) and at cleanse-time
// (EmitCleanse.filter = CleanseFilter::ById). Constant to keep them in sync.
const HOLY_AEGIS_BUFF_ID: &str = "holy_aegis";

impl Blueprint for HolyAegisBlueprint {
    fn id(&self) -> BlueprintId {
        BlueprintId("holy_aegis")
    }

    // commit_signals: default empty — Holy Aegis emits no commit-time signals.

    fn on_event(
        &self,
        event: &super::CombatEvent,
        ctx: &BlueprintCtx,
    ) -> Vec<super::Effect> {
        let self_id = match ctx.find_unit_id_for_blueprint(self.id()) {
            Some(id) => id,
            None => return Vec::new(),
        };

        match &event.kind {
            // Branch A: CombatStarted — apply the Aegis aura to all allies + self.
            // Single emission at combat boot; permanent until owner dies.
            super::CombatEventKind::CombatStarted => {
                vec![super::Effect::ApplyBuff {
                    id: HOLY_AEGIS_BUFF_ID.into(),
                    target: super::TargetRef::AoE {
                        side: super::TargetSide::AllyTeamInclSelf,
                    },
                    mul: Some(0.10), // 10% DR (canon)
                    kind: super::BuffKind::DR,
                    dur: super::BuffDuration::Permanent,
                }]
            }

            // Branch B: UnitDied where the dead unit is the Aegis owner.
            // Emit a targeted cleanse that strips ONLY the Aegis buff
            // (CleanseFilter::ById) from the entire ally team. count=u8::MAX
            // because every ally currently carries the buff and we want all
            // of them cleared; the filter narrows by buff id, so collateral
            // is zero. priority does not matter for an id-filtered cleanse
            // but we set OldestFirst for determinism.
            super::CombatEventKind::UnitDied { unit, .. } if *unit == self_id => {
                vec![super::Effect::EmitCleanse {
                    target: super::TargetRef::AoE {
                        side: super::TargetSide::AllyTeam,
                    },
                    count: u8::MAX,
                    filter: super::CleanseFilter::ById(HOLY_AEGIS_BUFF_ID.into()),
                    priority: super::CleansePriority::OldestFirst,
                }]
            }

            // All other events: silent no-op.
            _ => Vec::new(),
        }
    }

    // snapshot: default empty — buff state is owned by the status registry,
    // not by Holy Aegis itself. ValidationSnapshot reads buff state via the
    // unit-level status snapshot, not via this blueprint.
}

// Note on test coverage (M017 Slice G):
//   * CombatStarted → 1 ApplyBuff emitted, targeting AoE(AllyTeamInclSelf)
//   * UnitDied(self_id) → 1 EmitCleanse emitted, filter=ById("holy_aegis")
//   * UnitDied(other) → no Effect emitted
//   * other event kinds → no Effect emitted
//   * Patamon not in combat → no Effect emitted (defensive)
//
// Edge case to validate in M017: what happens if CombatStarted fires after
// a mid-combat revive of a dead Patamon? Canon §holy_aegis is silent.
// SP2 reads the canon literally — only the original CombatStarted triggers
// the buff. Mid-combat re-apply would need a separate canon clarification.
