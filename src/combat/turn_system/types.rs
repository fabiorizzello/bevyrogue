use bevy::prelude::*;

use crate::combat::StatusBag;
use crate::combat::buffs::DrBag;
use crate::combat::counterplay::EnemyCounterplayKit;
use crate::combat::kit::UnitSkills;
use crate::combat::round_flags::RoundFlags;
use crate::combat::stun::Stunned;
use crate::combat::team::Team;
use crate::combat::toughness::Toughness;
use crate::combat::types::{SkillId, UnitId};
use crate::combat::ultimate::UltimateCharge;
use crate::combat::unit::{Commander, Ko, SlotIndex, Unit};

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub enum ActionIntent {
    Basic {
        attacker: UnitId,
        target: UnitId,
    },
    Skill {
        attacker: UnitId,
        skill_id: SkillId,
        target: UnitId,
    },
    Ultimate {
        attacker: UnitId,
        target: UnitId,
    },
}

#[derive(Resource, Debug, Default, Clone)]
pub struct EnemyTurnRequestQueue(pub Vec<UnitId>);

/// Request to fire a unit's Ultimate out of turn (HSR-style burst).
///
/// Written by UI/CLI/AI when a *non-active* unit has a ready ult gauge. Consumed
/// by `burst_action_system`, which validates it through the existing legality
/// gate and, if legal, writes an `ActionIntent::Ultimate` while never touching
/// `TurnOrder.active_unit` or any `ActionValue`.
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct UltBurstRequest {
    pub attacker: UnitId,
    pub target: UnitId,
}

/// Transient marker: the unit currently authorized to act out of turn.
///
/// Set by `burst_action_system` for the burst attacker and consumed by
/// `resolve_action_system`, which forces that unit's `is_active` in the legality
/// snapshot for exactly one resolution cycle, then clears this back to `None`.
/// SP-cost, gauge-ready, KO, stun, and targeting checks all still apply — only
/// the active-unit gate is bypassed.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct OutOfTurnBurst(pub Option<UnitId>);

/// FIFO queue of out-of-turn ult bursts awaiting a launchable window.
///
/// A burst is launchable whenever the enemy is *not* taking its turn (i.e. the
/// active unit is not an enemy and combat is not mid-resolution). When the
/// player requests a burst while an enemy is acting, `burst_action_system`
/// parks the request here and fires it on the first launchable frame — the
/// moment the enemy's turn ends. Requests pressed during a launchable window
/// fire immediately and never linger in this queue.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct PendingBurstQueue(pub Vec<UltBurstRequest>);

pub(crate) type ResolveActorsQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Team,
        &'static mut Unit,
        Option<&'static UnitSkills>,
        Option<&'static mut UltimateCharge>,
        Option<&'static mut Toughness>,
        Option<&'static EnemyCounterplayKit>,
        Option<&'static Ko>,
        Option<&'static Stunned>,
        Option<&'static Commander>,
        Option<&'static mut StatusBag>,
        Option<&'static mut RoundFlags>,
        Option<&'static SlotIndex>,
        Option<&'static mut DrBag>,
    ),
>;
