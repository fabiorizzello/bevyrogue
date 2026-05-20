//! Standard `ResolvedAction` / `UltimateCharge` fixtures.

#![allow(dead_code)]

use bevyrogue::combat::state::{ResolvedAction, UltEffect};
use bevyrogue::combat::types::{DamageTag, SkillId};
use bevyrogue::combat::ultimate::{UltAccumulationTrigger, UltimateCharge};
use bevyrogue::data::skills_ron::TargetShape;

use super::units::{ATTACKER_ID, DEFENDER_ID};

/// Fresh charge meter: 0/100, +25 per basic, cap 150.
pub fn default_ult() -> UltimateCharge {
    UltimateCharge {
        current: 0,
        trigger: 100,
        cap: 150,
        trigger_type: UltAccumulationTrigger::OnBasicAttack,
        charge_per_event: 25,
    }
}

/// Charge meter primed at the trigger threshold (ready to fire).
pub fn ready_ult() -> UltimateCharge {
    UltimateCharge {
        current: 100,
        ..default_ult()
    }
}

/// Minimal `ResolvedAction` for a single-target basic of the given base damage,
/// from `ATTACKER_ID` to `DEFENDER_ID`, with `UltEffect::GainFromBasic`.
pub fn basic_resolved(base_damage: i32) -> ResolvedAction {
    ResolvedAction {
        source: ATTACKER_ID,
        target: DEFENDER_ID,
        skill_id: SkillId("basic".into()),
        damage_tag: DamageTag::Physical,
        base_damage,
        toughness_damage: 0,
        revive_pct: 0,
        heal_pct: 0,
        sp_cost: 0,
        ult_effect: UltEffect::GainFromBasic,
        grant_free_skill_count: 0,
        status_to_apply: None,
        advance_pct: 0,
        delay_pct: 0,
        energy_grant: 0,
        self_advance_pct: 0,
        target_shape: TargetShape::Single,
        custom_signals: vec![],
        damage_curve: Default::default(),
        cleanse_count: None,
    }
}

/// `ResolvedAction` representing an Ultimate cast (`UltEffect::Reset`).
pub fn ult_resolved(base_damage: i32) -> ResolvedAction {
    ResolvedAction {
        ult_effect: UltEffect::Reset,
        ..basic_resolved(base_damage)
    }
}
