#[cfg(feature = "windowed")]
use crate::combat::{
    team::Team,
    toughness::DamageKind,
    types::{Attribute, SkillId, UnitId},
};

#[cfg(feature = "windowed")]
#[derive(Clone)]
pub(super) struct SkillDisplay {
    pub(super) id: SkillId,
    pub(super) label: String,
}

#[cfg(feature = "windowed")]
#[derive(Clone)]
pub(super) struct UnitDisplay {
    pub(super) id: UnitId,
    pub(super) team: Team,
    pub(super) name: String,
    pub(super) attribute: Attribute,
    pub(super) hp_cur: i32,
    pub(super) hp_max: i32,
    pub(super) ult_cur: i32,
    pub(super) ult_trigger: i32,
    pub(super) ult_cap: i32,
    pub(super) ult_backing: &'static str,
    /// SP cost of this unit's Ultimate, resolved from the skill book. Used to
    /// gate the out-of-turn Burst button on affordability — mirrors the
    /// `SpShortfall` legality check so the button never looks live when the
    /// shared SP pool can't fund the cast.
    pub(super) ult_sp_cost: i32,
    pub(super) skills: Vec<SkillDisplay>,
    pub(super) is_ko: bool,
    pub(super) is_stunned: bool,
    pub(super) is_commander: bool,
    pub(super) toughness: Option<crate::combat::toughness::ToughnessView>,
    pub(super) energy_cur: Option<i32>,
    pub(super) energy_max: Option<i32>,
}

#[cfg(feature = "windowed")]
pub(super) struct FdDisplay {
    pub(super) target_idx: u32,
    pub(super) amount: i32,
    pub(super) kind: DamageKind,
    pub(super) alpha: u8,
}
