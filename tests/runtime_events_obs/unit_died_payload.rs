/// Integration test for M020/S01/T02: UnitDied carries the defender's status snapshot at KO.
///
/// Uses `apply_legacy_ops` directly (no full Bevy world needed) to isolate the
/// UnitDied payload construction in resolution.rs → ko_payload.
use bevyrogue::combat::{
    StatusBag, StatusEffectKind,
    events::CombatEventKind,
    resolution::apply_legacy_ops,
    sp::{RoundSpTracker, SpPool},
    state::{ResolvedAction, UltEffect},
    team::Team,
    toughness::Toughness,
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::{BasicStreak, Unit},
};

fn make_unit(id: u32, hp: i32) -> Unit {
    Unit {
        id: UnitId(id),
        name: format!("u{id}"),
        hp_max: hp,
        hp_current: hp,
        attribute: Attribute::Data,
        resists: vec![],
        evo_stage: EvoStage::Adult,
    }
}

fn default_ult() -> UltimateCharge {
    UltimateCharge {
        current: 0,
        trigger: 100,
        cap: 150,
        trigger_type: UltAccumulationTrigger::OnBasicAttack,
        charge_per_event: 25,
    }
}

/// Lethal basic attack: base_damage >> defender HP.
fn lethal_action() -> ResolvedAction {
    ResolvedAction {
        source: UnitId(1),
        target: UnitId(2),
        skill_id: SkillId("basic".into()),
        damage_tag: DamageTag::Physical,
        base_damage: 9_999,
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
        target_shape: bevyrogue::data::skills_ron::TargetShape::Single,
        custom_signals: vec![],
        damage_curve: Default::default(),
        cleanse_count: None,
    }
}

/// Defender has Heated(dur 2) + Slowed(dur 1); a fatal hit must produce UnitDied
/// with both kinds in status_remaining and heated_remaining == 2.
#[test]
fn unit_died_carries_defender_status_snapshot() {
    let attacker = make_unit(1, 1_000);
    let mut defender = make_unit(2, 1); // 1 HP → lethal hit
    let mut tough = Toughness::new(1_000, vec![]);
    let mut ult = default_ult();
    let mut sp = SpPool { current: 5, max: 5 };
    let action = lethal_action();

    let mut defender_bag = StatusBag::default();
    defender_bag.apply(StatusEffectKind::Heated, 2);
    defender_bag.apply(StatusEffectKind::Slowed, 1);

    let (_, events) = apply_legacy_ops(
        &action,
        &attacker,
        &mut defender,
        Team::Enemy,
        Some(&mut tough),
        &mut ult,
        &mut sp,
        &mut RoundSpTracker::default(),
        &mut BasicStreak::default(),
        false,
        false,
        Some(&defender_bag),
        None,
        None,
            None,
            None,
        );

    let died = events
        .iter()
        .find_map(|e| match e {
            CombatEventKind::UnitDied {
                status_remaining,
                heated_remaining,
            } => Some((status_remaining.clone(), *heated_remaining)),
            _ => None,
        })
        .expect("UnitDied must be emitted on lethal hit");

    let (status_remaining, heated_remaining) = died;
    assert!(
        status_remaining.contains(&StatusEffectKind::Heated),
        "status_remaining must contain Heated; got {status_remaining:?}"
    );
    assert!(
        status_remaining.contains(&StatusEffectKind::Slowed),
        "status_remaining must contain Slowed; got {status_remaining:?}"
    );
    assert_eq!(
        heated_remaining, 2,
        "heated_remaining must equal Heated duration (2); got {heated_remaining}"
    );
}

/// Non-lethal hit must NOT produce UnitDied.
#[test]
fn unit_died_not_emitted_on_survival() {
    let attacker = make_unit(1, 1_000);
    let mut defender = make_unit(2, 1_000); // high HP → survives
    let mut tough = Toughness::new(1_000, vec![]);
    let mut ult = default_ult();
    let mut sp = SpPool { current: 5, max: 5 };
    let action = lethal_action(); // large damage but defender HP is 1_000

    // Override defender HP so it survives (base_damage = 9999 vs 1000 HP)
    // This will still KO, so let's use a non-lethal action instead.
    drop(action);
    let action = ResolvedAction {
        source: UnitId(1),
        target: UnitId(2),
        skill_id: SkillId("basic".into()),
        damage_tag: DamageTag::Physical,
        base_damage: 1, // 1 damage vs 1000 HP → survives
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
        target_shape: bevyrogue::data::skills_ron::TargetShape::Single,
        custom_signals: vec![],
        damage_curve: Default::default(),
        cleanse_count: None,
    };

    let (_, events) = apply_legacy_ops(
        &action,
        &attacker,
        &mut defender,
        Team::Enemy,
        Some(&mut tough),
        &mut ult,
        &mut sp,
        &mut RoundSpTracker::default(),
        &mut BasicStreak::default(),
        false,
        false,
        None,
        None,
        None,
            None,
            None,
        );

    assert!(
        !events
            .iter()
            .any(|e| matches!(e, CombatEventKind::UnitDied { .. })),
        "UnitDied must not be emitted when defender survives"
    );
}
