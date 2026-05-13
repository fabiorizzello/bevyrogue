/// Scenario test verifying that `OnDamageDealt` carries `tag_mod_pct` and
/// `triangle_mod_pct` fields with values derived from the v5.3 multiplicative
/// damage model.  Closes the S02 roadmap requirement: "log JSONL mostra
/// tag_mod, triangle_mod e final_dmg coerenti."
///
/// Scenario A — Greymon (Vaccine) attacks Devimon (Virus, resists Fire):
///   tag_mod_pct   = 75   (Devimon resists Fire → ×0.75)
///   triangle_mod_pct = 111 (Vaccine > Virus → attacker wins → ×1.11)
///   amount        = round(100 × 0.75 × 1.11) = round(83.25) = 83
///
/// Scenario B — Greymon (Vaccine) attacks Devimon (Virus, weak Fire):
///   tag_mod_pct   = 125  (Devimon is weak to Fire → ×1.25)
///   triangle_mod_pct = 111 (same triangle win)
///   amount        = round(100 × 1.25 × 1.11) = round(138.75) = 139
use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::{
    events::{CombatEvent, CombatEventKind},
    kit::UnitSkills,
    log::ActionLog,
    sp::SpPool,
    state::CombatState,
    team::Team,
    toughness::DamageKind,
    toughness::Toughness,
    turn_order::TurnOrder,
    turn_system::{ActionIntent, resolve_action_system},
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::Unit,
};
use bevyrogue::data::{
    SkillBookHandle,
    skills_ron::{
        Effect, SelfTargetRule, SkillBook, SkillDef, SkillImplementation,
        SkillTargeting, TargetLife, TargetShape, TargetSide,
    },
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn fire_skill() -> SkillDef {
    SkillDef {
        id: SkillId("fire_basic".into()),
        name: "Fire Basic".into(),
        damage_tag: DamageTag::Fire,
        sp_cost: 0,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Implemented,
        effects: vec![
            Effect::Damage {
                amount: 100,
                target: TargetShape::Single,
            },
            Effect::ToughnessHit(0),
        ],
        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
    }
}

/// Spawn a minimal headless Bevy app wired for `resolve_action_system`.
/// `defender_resists` and `defender_weaknesses` control the tag mod.
fn setup_app(
    attacker_attr: Attribute,
    defender_attr: Attribute,
    defender_resists: Vec<DamageTag>,
    defender_weaknesses: Vec<DamageTag>,
) -> App {
    let mut app = App::new();
    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(SkillBook(vec![fire_skill()]));
    app.insert_resource(assets)
        .insert_resource(SkillBookHandle(handle))
        .init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .insert_resource(SpPool {
            current: 100,
            max: 100,
        })
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_systems(Update, resolve_action_system);

    // Attacker (Greymon)
    app.world_mut().spawn((
        Unit {
            id: UnitId(1),
            name: "Greymon".into(),
            hp_max: 500,
            hp_current: 500,
            attribute: attacker_attr,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Ally,
        UnitSkills {
            basic: SkillId("fire_basic".into()),
            skills: vec![],
            ultimate: SkillId("fire_basic".into()),
            follow_up: None,
        },
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 10,
        },
        Toughness::new(100, vec![]),
    ));

    // Defender (Devimon): high HP so it survives the hit; weaknesses via Toughness.
    app.world_mut().spawn((
        Unit {
            id: UnitId(2),
            name: "Devimon".into(),
            hp_max: 10_000,
            hp_current: 10_000,
            attribute: defender_attr,
            resists: defender_resists,
            evo_stage: EvoStage::Adult,
        },
        Team::Enemy,
        Toughness::new(1_000, defender_weaknesses),
    ));

    app
}

fn drain_damage_dealt(app: &mut App) -> Vec<(i32, DamageKind, i32, i32)> {
    let mut cursor: MessageCursor<CombatEvent> = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor();
    cursor
        .read(app.world().resource::<Messages<CombatEvent>>())
        .filter_map(|e| {
            if let CombatEventKind::OnDamageDealt {
                amount,
                kind,
                tag_mod_pct,
                triangle_mod_pct,
                ..
            } = e.kind
            {
                Some((amount, kind, tag_mod_pct, triangle_mod_pct))
            } else {
                None
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Scenario A: Greymon (Vaccine) attacks Devimon (Virus, resists Fire)
//   Expected: amount=83, tag_mod_pct=75, triangle_mod_pct=111
// ---------------------------------------------------------------------------

#[test]
fn greymon_vs_devimon_resist_fire_breakdown() {
    let mut app = setup_app(
        Attribute::Vaccine,
        Attribute::Virus,
        vec![DamageTag::Fire], // Devimon resists Fire
        vec![],
    );

    app.world_mut().write_message(ActionIntent::Basic {
        attacker: UnitId(1),
        target: UnitId(2),
    });
    app.update();

    let hits = drain_damage_dealt(&mut app);
    assert_eq!(hits.len(), 1, "expected exactly one OnDamageDealt");
    let (amount, _kind, tag_mod_pct, triangle_mod_pct) = hits[0];

    assert_eq!(tag_mod_pct, 75, "Devimon resists Fire → tag_mod_pct=75");
    assert_eq!(
        triangle_mod_pct, 111,
        "Vaccine > Virus → attacker wins → triangle_mod_pct=111"
    );
    assert_eq!(amount, 83, "round(100 × 0.75 × 1.11) = 83");
}

// ---------------------------------------------------------------------------
// Scenario B: Greymon (Vaccine) attacks Devimon (Virus, weak Fire)
//   Expected: amount=139, tag_mod_pct=125, triangle_mod_pct=111
// ---------------------------------------------------------------------------

#[test]
fn greymon_vs_devimon_weak_fire_breakdown() {
    let mut app = setup_app(
        Attribute::Vaccine,
        Attribute::Virus,
        vec![],                // no resists
        vec![DamageTag::Fire], // Devimon is weak to Fire
    );

    app.world_mut().write_message(ActionIntent::Basic {
        attacker: UnitId(1),
        target: UnitId(2),
    });
    app.update();

    let hits = drain_damage_dealt(&mut app);
    assert_eq!(hits.len(), 1, "expected exactly one OnDamageDealt");
    let (amount, _kind, tag_mod_pct, triangle_mod_pct) = hits[0];

    assert_eq!(tag_mod_pct, 125, "Devimon weak to Fire → tag_mod_pct=125");
    assert_eq!(
        triangle_mod_pct, 111,
        "Vaccine > Virus → attacker wins → triangle_mod_pct=111"
    );
    assert_eq!(amount, 139, "round(100 × 1.25 × 1.11) = 139");
}
