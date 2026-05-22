//! Out-of-turn ultimate burst (HSR-style).
//!
//! `burst_action_system` drains [`UltBurstRequest`]s, validates each through the
//! *existing* legality gate (with the active-unit check lifted via
//! [`mark_unit_active`]), and — if legal — sets [`OutOfTurnBurst`] and writes an
//! `ActionIntent::Ultimate`. It runs **before** `resolve_action_system`, which
//! reads `OutOfTurnBurst` to honor the same active-unit lift during resolution.
//!
//! The system never touches `TurnOrder.active_unit` or any `ActionValue`: a burst
//! is a free, out-of-turn cast. The transient `OutOfTurnBurst` flag is cleared at
//! the top of every run, so it is `Some` only across the single frame in which the
//! burst resolves.

use bevy::prelude::*;

use crate::combat::action_query::{
    ActionQueryKind, build_snapshot_from_ecs, mark_unit_active, query_intent_legality,
};
use crate::combat::counterplay::EnemyCounterplayKit;
use crate::combat::energy::Energy;
use crate::combat::kit::UnitSkills;
use crate::combat::sp::SpPool;
use crate::combat::state::{CombatPhase, CombatState};
use crate::combat::stun::Stunned;
use crate::combat::team::Team;
use crate::combat::toughness::Toughness;
use crate::combat::turn_order::TurnOrder;
use crate::combat::ult_gauge::UltGaugeMetadata;
use crate::combat::ultimate::UltimateCharge;
use crate::combat::unit::{Commander, Ko, Unit};
use crate::data::SkillBookHandle;
use crate::data::skills_ron::SkillBook;

use super::types::{ActionIntent, OutOfTurnBurst, UltBurstRequest};

/// Read-only mirror of the components `build_snapshot_from_ecs` consumes.
type BurstActorsQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Team,
        &'static Unit,
        Option<&'static UnitSkills>,
        Option<&'static UltimateCharge>,
        Option<&'static Toughness>,
        Option<&'static EnemyCounterplayKit>,
        Option<&'static Ko>,
        Option<&'static Stunned>,
        Option<&'static Commander>,
        Option<&'static Energy>,
        Option<&'static UltGaugeMetadata>,
    ),
>;

pub fn burst_action_system(
    mut requests: MessageReader<UltBurstRequest>,
    mut intent_writer: MessageWriter<ActionIntent>,
    mut out_of_turn: ResMut<OutOfTurnBurst>,
    state: Res<CombatState>,
    turn_order: Res<TurnOrder>,
    sp: Res<SpPool>,
    skill_books: Res<Assets<SkillBook>>,
    skill_book_handle: Option<Res<SkillBookHandle>>,
    actors: BurstActorsQuery,
) {
    // The flag is transient: clear it every frame so it is only `Some` for the
    // single resolution cycle following a legal burst.
    out_of_turn.0 = None;

    // Drain even when we cannot act, so stale requests never pile up.
    let pending: Vec<UltBurstRequest> = requests.read().cloned().collect();
    if pending.is_empty() {
        return;
    }

    // A burst is only honored during the action-selection phase (D3).
    if state.phase != CombatPhase::WaitingAction {
        return;
    }

    let Some(skill_book) = skill_book_handle
        .as_ref()
        .and_then(|handle| skill_books.get(&handle.0))
    else {
        return;
    };

    let units_data: Vec<_> = actors
        .iter()
        .map(
            |(team, unit, skills, ult, toughness, counterplay, ko, stunned, commander, energy, gauge_meta)| {
                (
                    unit.id,
                    *team,
                    unit,
                    skills,
                    ult,
                    toughness,
                    counterplay,
                    ko.is_some(),
                    stunned.is_some(),
                    commander.is_some(),
                    energy,
                    gauge_meta,
                )
            },
        )
        .collect();

    for req in pending {
        let mut snapshot = build_snapshot_from_ecs(
            &state,
            &turn_order,
            &sp,
            req.attacker,
            req.target,
            units_data.clone(),
        );
        // Lift only the active-unit gate for the burst attacker.
        mark_unit_active(&mut snapshot, req.attacker);

        match query_intent_legality(
            &snapshot,
            skill_book,
            req.attacker,
            &ActionQueryKind::Ultimate,
            req.target,
        ) {
            Ok(()) => {
                // Authorize the resolve cycle to treat this unit as active, then
                // emit the ult intent. AV / active_unit are deliberately untouched.
                out_of_turn.0 = Some(req.attacker);
                intent_writer.write(ActionIntent::Ultimate {
                    attacker: req.attacker,
                    target: req.target,
                });
                debug!(
                    target: "combat.burst",
                    attacker = ?req.attacker,
                    target = ?req.target,
                    "out-of-turn ult burst authorized"
                );
            }
            Err(reason) => {
                debug!(
                    target: "combat.burst",
                    attacker = ?req.attacker,
                    target = ?req.target,
                    ?reason,
                    "out-of-turn ult burst rejected"
                );
            }
        }
    }
}
