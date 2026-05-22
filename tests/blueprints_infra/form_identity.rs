use bevy::{ecs::message::MessageCursor, prelude::*};
use bevyrogue::combat::{
    energy::Energy,
    events::{CombatEvent, CombatEventKind},
    follow_up::FollowUpTrace,
    kit::{FormIdentityKit, UnitSkills},
    round_flags::RoundFlags,
    team::Team,
    toughness::Toughness,
    turn_system::ActionIntent,
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::Unit,
};
use bevyrogue::data::units_ron::{UnitDef, UnitRoster};

use crate::common::{app::form_identity_runtime_app as setup_app, load_roster, load_skill_book};

fn pilot(roster: &UnitRoster, name: &str) -> UnitDef {
    roster
        .0
        .iter()
        .find(|unit| unit.name == name)
        .cloned()
        .unwrap_or_else(|| panic!("missing pilot {name}"))
}

fn spawn_greymon(app: &mut App, def: &UnitDef) -> Entity {
    let fi_config = def
        .form_identity
        .clone()
        .expect("Greymon must have form_identity");

    app.world_mut()
        .spawn((
            Unit {
                id: def.id,
                name: def.name.clone(),
                hp_max: def.hp_max,
                hp_current: def.hp_max,
                attribute: def.attribute,
                resists: def.resists.clone(),
                evo_stage: EvoStage::Adult,
            },
            def.team,
            UnitSkills {
                basic: def.basic_skill.clone(),
                skills: def.skill_ids.clone(),
                ultimate: def.ultimate_skill.clone(),
                follow_up: def.follow_up.clone(),
            },
            FormIdentityKit { config: fi_config },
            Toughness {
                max: def.toughness_max,
                current: def.toughness_max,
                weaknesses: def.weaknesses.clone(),
                broken: false,
                category: Default::default(),
            },
            UltimateCharge {
                current: 0,
                trigger: def.ultimate_trigger,
                cap: def.ultimate_cap,
                trigger_type: def.ultimate_accumulation_trigger,
                charge_per_event: def.ultimate_charge_per_event,
            },
            Energy::default(),
            RoundFlags::default(),
        ))
        .id()
}

fn spawn_tough_enemy(app: &mut App, id: UnitId) -> Entity {
    app.world_mut()
        .spawn((
            Unit {
                id,
                name: "Boss".into(),
                hp_max: 1000,
                hp_current: 1000,
                attribute: Attribute::Virus,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            Team::Enemy,
            Toughness {
                max: 200,
                current: 200,
                weaknesses: vec![DamageTag::Fire],
                broken: false,
                category: Default::default(),
            },
            UltimateCharge {
                current: 0,
                trigger: 100,
                cap: 150,
                trigger_type: UltAccumulationTrigger::OnBasicAttack,
                charge_per_event: 25,
            },
            UnitSkills {
                basic: SkillId("dummy".into()),
                skills: vec![SkillId("dummy".into())],
                ultimate: SkillId("dummy_ult".into()),
                follow_up: None,
            },
        ))
        .id()
}

fn unit_energy(app: &mut App, unit_id: UnitId) -> i32 {
    let mut q = app.world_mut().query::<(&Unit, &Energy)>();
    q.iter(app.world())
        .find(|(u, _)| u.id == unit_id)
        .map(|(_, e)| e.current)
        .unwrap_or_else(|| panic!("missing unit {:?}", unit_id))
}

fn form_identity_used(app: &mut App, unit_id: UnitId) -> bool {
    let mut q = app.world_mut().query::<(&Unit, &RoundFlags)>();
    q.iter(app.world())
        .find(|(u, _)| u.id == unit_id)
        .map(|(_, f)| f.form_identity_used)
        .unwrap_or(false)
}

#[test]
fn greymon_first_fire_hit_grants_energy() {
    let roster = load_roster();
    let mut app = setup_app(load_skill_book());

    let greymon = pilot(&roster, "Greymon");
    let enemy_id = UnitId(99);

    let greymon_entity = spawn_greymon(&mut app, &greymon);
    spawn_tough_enemy(&mut app, enemy_id);

    let fire_skill = greymon.skill_ids[0].clone(); // mega_flame
    let mut trace_cursor = app
        .world_mut()
        .resource_mut::<Messages<FollowUpTrace>>()
        .get_cursor();
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: greymon.id,
        skill_id: fire_skill,
        target: enemy_id,
    });
    app.update();

    let traces = trace_cursor
        .read(app.world().resource::<Messages<FollowUpTrace>>())
        .cloned()
        .collect::<Vec<_>>();
    assert!(
        traces.iter().any(|trace| trace.follower == greymon.id),
        "Greymon Form Identity should emit a follow-up trace"
    );

    assert_eq!(
        unit_energy(&mut app, greymon.id),
        5,
        "Greymon should have gained 5 Energy from Form Identity"
    );
    assert!(
        form_identity_used(&mut app, greymon.id),
        "form_identity_used should be true after first hit"
    );
    let _ = greymon_entity; // keep alive
}

#[test]
fn greymon_second_fire_hit_blocked() {
    let roster = load_roster();
    let mut app = setup_app(load_skill_book());

    let greymon = pilot(&roster, "Greymon");
    let enemy_id = UnitId(99);

    spawn_greymon(&mut app, &greymon);
    spawn_tough_enemy(&mut app, enemy_id);

    let fire_skill = greymon.skill_ids[0].clone();

    // First hit: form identity fires, energy +5
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: greymon.id,
        skill_id: fire_skill.clone(),
        target: enemy_id,
    });
    app.update();
    assert_eq!(
        unit_energy(&mut app, greymon.id),
        5,
        "expected 5 after first hit"
    );

    // Second hit: form_identity_used=true blocks the listener
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: greymon.id,
        skill_id: fire_skill,
        target: enemy_id,
    });
    app.update();
    assert_eq!(
        unit_energy(&mut app, greymon.id),
        5,
        "energy must not increase on second hit same round"
    );
}

#[test]
fn greymon_resets_next_turn() {
    let roster = load_roster();
    let mut app = setup_app(load_skill_book());

    let greymon = pilot(&roster, "Greymon");
    let enemy_id = UnitId(99);

    let greymon_entity = spawn_greymon(&mut app, &greymon);
    spawn_tough_enemy(&mut app, enemy_id);

    let fire_skill = greymon.skill_ids[0].clone();

    // Turn 1: form identity fires
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: greymon.id,
        skill_id: fire_skill.clone(),
        target: enemy_id,
    });
    app.update();
    assert_eq!(
        unit_energy(&mut app, greymon.id),
        5,
        "expected 5 after first hit"
    );

    // Simulate advance_turn_system resetting form_identity_used
    app.world_mut()
        .get_mut::<RoundFlags>(greymon_entity)
        .expect("RoundFlags missing")
        .form_identity_used = false;

    // Turn 2: form identity fires again
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: greymon.id,
        skill_id: fire_skill,
        target: enemy_id,
    });
    app.update();
    assert_eq!(
        unit_energy(&mut app, greymon.id),
        10,
        "Greymon should gain another 5 Energy after round reset"
    );
}

fn spawn_unit_with_fi(app: &mut App, def: &UnitDef) -> Entity {
    let fi_config = def
        .form_identity
        .clone()
        .unwrap_or_else(|| panic!("{} must have form_identity", def.name));

    app.world_mut()
        .spawn((
            Unit {
                id: def.id,
                name: def.name.clone(),
                hp_max: def.hp_max,
                hp_current: def.hp_max,
                attribute: def.attribute,
                resists: def.resists.clone(),
                evo_stage: EvoStage::Adult,
            },
            def.team,
            UnitSkills {
                basic: def.basic_skill.clone(),
                skills: def.skill_ids.clone(),
                ultimate: def.ultimate_skill.clone(),
                follow_up: def.follow_up.clone(),
            },
            FormIdentityKit { config: fi_config },
            Toughness {
                max: def.toughness_max,
                current: def.toughness_max,
                weaknesses: def.weaknesses.clone(),
                broken: false,
                category: Default::default(),
            },
            UltimateCharge {
                current: 0,
                trigger: def.ultimate_trigger,
                cap: def.ultimate_cap,
                trigger_type: def.ultimate_accumulation_trigger,
                charge_per_event: def.ultimate_charge_per_event,
            },
            Energy::default(),
            RoundFlags::default(),
        ))
        .id()
}

fn drain_combat_events(cursor: &mut MessageCursor<CombatEvent>, app: &App) -> Vec<CombatEvent> {
    cursor
        .read(app.world().resource::<Messages<CombatEvent>>())
        .cloned()
        .collect()
}

#[test]
fn garurumon_first_ice_hit_grants_energy() {
    let roster = load_roster();
    let mut app = setup_app(load_skill_book());

    let garurumon = pilot(&roster, "Garurumon");
    let enemy_id = UnitId(99);

    let garurumon_entity = spawn_unit_with_fi(&mut app, &garurumon);
    spawn_tough_enemy(&mut app, enemy_id);

    let ice_skill = garurumon.skill_ids[0].clone(); // foxfire
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: garurumon.id,
        skill_id: ice_skill,
        target: enemy_id,
    });
    app.update();

    assert_eq!(
        unit_energy(&mut app, garurumon.id),
        5,
        "Garurumon should gain 5 Energy from Form Identity on first Ice hit"
    );
    assert!(
        form_identity_used(&mut app, garurumon.id),
        "form_identity_used should be true after first ice hit"
    );
    let _ = garurumon_entity;
}

#[test]
fn kabuterimon_first_electric_hit_grants_energy() {
    let roster = load_roster();
    let mut app = setup_app(load_skill_book());

    let kabuterimon = pilot(&roster, "Kabuterimon");
    let enemy_id = UnitId(99);

    let kabuterimon_entity = spawn_unit_with_fi(&mut app, &kabuterimon);
    spawn_tough_enemy(&mut app, enemy_id);

    let electric_skill = kabuterimon.skill_ids[0].clone(); // mega_blaster
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: kabuterimon.id,
        skill_id: electric_skill,
        target: enemy_id,
    });
    app.update();

    assert_eq!(
        unit_energy(&mut app, kabuterimon.id),
        5,
        "Kabuterimon should gain 5 Energy from Form Identity on first Electric hit"
    );
    assert!(
        form_identity_used(&mut app, kabuterimon.id),
        "form_identity_used should be true after first electric hit"
    );
    let _ = kabuterimon_entity;
}

#[test]
fn kyubimon_freeze_application_self_advances() {
    let roster = load_roster();
    let mut app = setup_app(load_skill_book());

    let kyubimon = pilot(&roster, "Kyubimon");
    let enemy_id = UnitId(99);

    let kyubimon_entity = spawn_unit_with_fi(&mut app, &kyubimon);
    spawn_tough_enemy(&mut app, enemy_id);

    // Get cursor before update so we can read events produced during this update
    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor();

    let freeze_skill = kyubimon.skill_ids[0].clone(); // onibidama — applies Chilled
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: kyubimon.id,
        skill_id: freeze_skill.clone(),
        target: enemy_id,
    });
    app.update();

    let events = drain_combat_events(&mut cursor, &app);
    let has_self_advance = events.iter().any(|e| {
        matches!(&e.kind, CombatEventKind::AdvanceTurn { target, amount_pct }
            if *target == kyubimon.id && *amount_pct == 20)
    });
    assert!(
        has_self_advance,
        "Kyubimon should emit AdvanceTurn(self, 20%) from Form Identity after Chilled application"
    );
    assert!(
        form_identity_used(&mut app, kyubimon.id),
        "form_identity_used should be true after chilled application"
    );

    // Second freeze application in same round must NOT re-trigger
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: kyubimon.id,
        skill_id: freeze_skill,
        target: enemy_id,
    });
    app.update();

    assert!(
        form_identity_used(&mut app, kyubimon.id),
        "form_identity_used should remain true — second freeze must not re-trigger"
    );
    let _ = kyubimon_entity;
}

fn toughness_current(app: &mut App, unit_id: UnitId) -> i32 {
    let mut q = app.world_mut().query::<(&Unit, &Toughness)>();
    q.iter(app.world())
        .find(|(u, _)| u.id == unit_id)
        .map(|(_, t)| t.current)
        .unwrap_or_else(|| panic!("missing unit {:?}", unit_id))
}

fn spawn_enemy_with_attribute(app: &mut App, id: UnitId, attribute: Attribute) -> Entity {
    app.world_mut()
        .spawn((
            Unit {
                id,
                name: "TestEnemy".into(),
                hp_max: 1000,
                hp_current: 1000,
                attribute,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            Team::Enemy,
            Toughness {
                max: 200,
                current: 200,
                weaknesses: vec![DamageTag::Fire],
                broken: false,
                category: Default::default(),
            },
            UltimateCharge {
                current: 0,
                trigger: 100,
                cap: 150,
                trigger_type: UltAccumulationTrigger::OnBasicAttack,
                charge_per_event: 25,
            },
            UnitSkills {
                basic: SkillId("dummy".into()),
                skills: vec![SkillId("dummy".into())],
                ultimate: SkillId("dummy_ult".into()),
                follow_up: None,
            },
        ))
        .id()
}

/// DORUgamon's OnFirstSkillCastWithTag(Dark) Form Identity fires a ToughnessHit(10) follow-up
/// on the first Dark attack each round, draining toughness further than the base skill alone.
#[test]
fn dorugamon_first_dark_skill_grants_bonus_toughness() {
    let roster = load_roster();
    let mut app = setup_app(load_skill_book());

    let dorugamon = pilot(&roster, "DORUgamon");
    let enemy_id = UnitId(99);

    spawn_unit_with_fi(&mut app, &dorugamon);
    let enemy_entity = spawn_enemy_with_attribute(&mut app, enemy_id, Attribute::Vaccine);

    // power_metal: Dark, ToughnessHit(18). After form_identity fires (ToughnessHit(10)), total = 28.
    let dark_skill = dorugamon.skill_ids[0].clone(); // power_metal
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: dorugamon.id,
        skill_id: dark_skill,
        target: enemy_id,
    });
    app.update();

    assert!(
        form_identity_used(&mut app, dorugamon.id),
        "DORUgamon form_identity_used should be true after first Dark skill"
    );

    // Toughness after: 200 - 18 (power_metal) - 10 (form_identity ToughnessHit) = 172.
    // Enemy has no Dark weakness so no 2x multiplier; Armored/Shielded not set (Standard).
    assert_eq!(
        toughness_current(&mut app, enemy_id),
        172,
        "toughness should be reduced by power_metal(18) + form_identity(10)"
    );

    // Once-per-round guard: second cast must not re-trigger.
    let cannonball = dorugamon.skill_ids[1].clone();
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: dorugamon.id,
        skill_id: cannonball,
        target: enemy_id,
    });
    app.update();

    // cannonball ToughnessHit(20) applied, but form_identity does NOT fire again.
    assert_eq!(
        toughness_current(&mut app, enemy_id),
        172 - 20,
        "second cast should reduce toughness by cannonball(20) only — form identity suppressed"
    );
    let _ = enemy_entity;
}

/// Angemon's OnAttackVsAttribute(Virus) Form Identity fires a bonus Damage(15) follow-up
/// when attacking a Virus-attribute enemy.
#[test]
fn angemon_attack_vs_virus_grants_bonus() {
    let roster = load_roster();
    let mut app = setup_app(load_skill_book());

    let angemon = pilot(&roster, "Angemon");
    let virus_id = UnitId(99);

    spawn_unit_with_fi(&mut app, &angemon);
    spawn_enemy_with_attribute(&mut app, virus_id, Attribute::Virus);

    let mut cursor = app
        .world_mut()
        .resource_mut::<Messages<CombatEvent>>()
        .get_cursor();

    let light_skill = angemon.skill_ids[0].clone(); // heavens_knuckle
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: angemon.id,
        skill_id: light_skill,
        target: virus_id,
    });
    app.update();

    assert!(
        form_identity_used(&mut app, angemon.id),
        "form_identity_used should be true after attacking Virus enemy"
    );

    // Expect at least 2 OnDamageDealt from Angemon (heavens_knuckle + form_identity follow-up).
    let events = drain_combat_events(&mut cursor, &app);
    let angemon_damage_count = events
        .iter()
        .filter(|e| {
            matches!(&e.kind, CombatEventKind::OnDamageDealt { amount, .. } if *amount > 0)
                && e.source == angemon.id
        })
        .count();
    assert!(
        angemon_damage_count >= 2,
        "expected at least 2 OnDamageDealt from Angemon (base + form_identity follow-up), got {angemon_damage_count}"
    );
}

/// Negative: Angemon's OnAttackVsAttribute(Virus) must NOT fire against a Data-attribute enemy.
#[test]
fn angemon_attack_vs_data_no_bonus() {
    let roster = load_roster();
    let mut app = setup_app(load_skill_book());

    let angemon = pilot(&roster, "Angemon");
    let data_id = UnitId(99);

    spawn_unit_with_fi(&mut app, &angemon);
    spawn_enemy_with_attribute(&mut app, data_id, Attribute::Data);

    let light_skill = angemon.skill_ids[0].clone(); // heavens_knuckle
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: angemon.id,
        skill_id: light_skill,
        target: data_id,
    });
    app.update();

    assert!(
        !form_identity_used(&mut app, angemon.id),
        "form_identity must NOT fire when attacking a Data-attribute enemy"
    );
}

/// Negative test: Greymon's form_identity (Fire trigger) must NOT fire when Garurumon
/// uses an Ice basic — tag specificity is enforced per unit.
#[test]
fn greymon_fire_trigger_does_not_fire_on_garurumon_ice_hit() {
    let roster = load_roster();
    let mut app = setup_app(load_skill_book());

    let greymon = pilot(&roster, "Greymon");
    let garurumon = pilot(&roster, "Garurumon");
    let enemy_id = UnitId(99);

    spawn_greymon(&mut app, &greymon);
    spawn_unit_with_fi(&mut app, &garurumon);
    spawn_tough_enemy(&mut app, enemy_id);

    // Garurumon fires an Ice skill — should trigger Garurumon's FI but NOT Greymon's
    let ice_skill = garurumon.skill_ids[0].clone(); // foxfire
    app.world_mut().write_message(ActionIntent::Skill {
        attacker: garurumon.id,
        skill_id: ice_skill,
        target: enemy_id,
    });
    app.update();

    assert_eq!(
        unit_energy(&mut app, garurumon.id),
        5,
        "Garurumon should gain Energy from its own Ice Form Identity"
    );
    assert_eq!(
        unit_energy(&mut app, greymon.id),
        0,
        "Greymon's Fire trigger must NOT fire on an Ice hit"
    );
    assert!(
        !form_identity_used(&mut app, greymon.id),
        "Greymon form_identity_used must remain false"
    );
}
