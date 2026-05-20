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
use crate::combat::unit::{BasicStreak, Commander, Ko, SlotIndex, Unit};

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
        Option<&'static mut BasicStreak>,
        Option<&'static mut RoundFlags>,
        Option<&'static SlotIndex>,
        Option<&'static mut DrBag>,
    ),
>;
