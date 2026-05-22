//! Out-of-turn ultimate burst (HSR-style).
//!
//! `burst_action_system` enqueues every [`UltBurstRequest`] into the persistent
//! [`PendingBurstQueue`], then — on each frame the burst is *launchable* — pops
//! the front request, validates it through the *existing* legality gate (with the
//! active-unit check lifted via [`mark_unit_active`]), and, if legal, sets
//! [`OutOfTurnBurst`] and writes an `ActionIntent::Ultimate`.
//!
//! **Launchable window.** Actions resolve only in `WaitingAction` (the legality
//! gate enforces this) and never while an enemy holds the turn. A burst pressed
//! during the player's own action window therefore fires immediately; one pressed
//! while an enemy is acting — or during the AV-ticking gap between turns — is
//! parked in the queue and fires on the first `WaitingAction` frame with a
//! non-enemy active unit, i.e. the moment control returns to the player. So a
//! burst is never silently dropped for timing.
//!
//! It runs **before** `resolve_action_system`, which reads `OutOfTurnBurst` to
//! honor the same active-unit lift during resolution. At most one burst is fired
//! per frame because `resolve_action_system` consumes only one `ActionIntent` per
//! frame; the rest wait for subsequent launchable frames.
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

use super::types::{ActionIntent, OutOfTurnBurst, PendingBurstQueue, UltBurstRequest};

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

#[allow(clippy::too_many_arguments)]
pub fn burst_action_system(
    mut requests: MessageReader<UltBurstRequest>,
    mut intent_writer: MessageWriter<ActionIntent>,
    mut out_of_turn: ResMut<OutOfTurnBurst>,
    mut queue: ResMut<PendingBurstQueue>,
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

    // Park every new request. Holding them in a persistent queue (rather than
    // draining the per-frame message buffer) is what lets a burst pressed during
    // an enemy turn survive until the turn ends.
    for req in requests.read() {
        queue.0.push(req.clone());
    }
    if queue.0.is_empty() {
        return;
    }

    let units_data: Vec<_> = actors
        .iter()
        .map(
            |(
                team,
                unit,
                skills,
                ult,
                toughness,
                counterplay,
                ko,
                stunned,
                commander,
                energy,
                gauge_meta,
            )| {
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

    // Launchable window. Actions resolve only in `WaitingAction` (the legality
    // gate is the single source of truth for this), and never while an enemy
    // holds the turn. Every other moment — an enemy's action window, the
    // AV-ticking gap (`WaitingForTurn`), or mid-`Resolving` — parks the burst in
    // the queue until the player's next action window opens.
    let active_is_enemy = turn_order.active_unit.is_some_and(|active| {
        units_data
            .iter()
            .any(|data| data.0 == active && matches!(data.1, Team::Enemy))
    });
    if state.phase != CombatPhase::WaitingAction || active_is_enemy {
        return;
    }

    let Some(skill_book) = skill_book_handle
        .as_ref()
        .and_then(|handle| skill_books.get(&handle.0))
    else {
        return;
    };

    // Fire at most one burst per frame: `resolve_action_system` consumes only one
    // `ActionIntent` per frame, so the rest stay queued for the next launchable
    // frame. The front request is always removed — legal ones fire, and ones now
    // illegal for a real reason (KO, stun, drained gauge) are genuinely rejected.
    let req = queue.0.remove(0);
    let mut snapshot = build_snapshot_from_ecs(
        &state,
        &turn_order,
        &sp,
        req.attacker,
        req.target,
        units_data,
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
