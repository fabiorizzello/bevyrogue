use bevy::{ecs::message::{Message, MessageCursor, Messages}, prelude::*};
use bevyrogue::combat::{
    energy::Energy,
    events::{CombatEvent, CombatEventKind},
    kit::UnitSkills,
    team::Team,
    toughness::{Toughness, ToughnessCategory},
    turn_system::ActionIntent,
    types::{Attribute, SkillId, UnitId},
    ult_gauge::UltGaugeMetadata,
    ultimate::UltimateCharge,
    unit::{BasicStreak, Unit},
};
use bevyrogue::data::units_ron::{UnitDef, UnitRoster};

use crate::common::{app::skill_book_runtime_app, load_roster, load_skill_book};

const DUMMY_ID: UnitId = UnitId(99);
const SHARP_CLAWS_ENERGY_GAIN: i32 = 6;

fn pilot(roster: &UnitRoster, name: &str) -> UnitDef {
    roster
        .0
        .iter()
        .find(|unit| unit.name == name)
        .cloned()
        .unwrap_or_else(|| panic!("missing pilot {name}"))
}

fn spawn_from_def(
    app: &mut App,
    def: &UnitDef,
    hp_current: i32,
    toughness_current: i32,
    ultimate_current: i32,
    energy_current: i32,
) {
    app.world_mut().spawn((
        Unit {
            id: def.id,
            name: def.name.clone(),
            hp_max: def.hp_max,
            hp_current,
            attribute: def.attribute,
            resists: def.resists.clone(),
            evo_stage: def.evo_stage,
        },
        def.team,
        Toughness {
            max: def.toughness_max,
            current: toughness_current,
            weaknesses: def.weaknesses.clone(),
            broken: false,
            category: def.toughness_category,
        },
        UltimateCharge {
            current: ultimate_current,
            trigger: def.ultimate_trigger,
            cap: def.ultimate_cap,
            trigger_type: def.ultimate_accumulation_trigger,
            charge_per_event: def.ultimate_charge_per_event,
        },
        UnitSkills {
            basic: def.basic_skill.clone(),
            skills: def.skill_ids.clone(),
            ultimate: def.ultimate_skill.clone(),
            follow_up: def.follow_up.clone(),
        },
        Energy {
            current: energy_current,
            max: 100,
        },
        BasicStreak::default(),
        UltGaugeMetadata(def.blueprint_metadata.clone()),
    ));
}

fn spawn_training_dummy(app: &mut App) {
    app.world_mut().spawn((
        Unit {
            id: DUMMY_ID,
            name: "Training Dummy".into(),
            hp_max: 5_000,
            hp_current: 5_000,
            attribute: Attribute::Virus,
            resists: vec![],
            evo_stage: bevyrogue::combat::types::EvoStage::Adult,
        },
        Team::Enemy,
        Toughness {
            max: 5_000,
            current: 5_000,
            weaknesses: vec![],
            broken: false,
            category: ToughnessCategory::Standard,
        },
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: bevyrogue::combat::ultimate::UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
        UnitSkills {
            basic: SkillId("dummy_basic".into()),
            skills: vec![SkillId("dummy_basic".into())],
            ultimate: SkillId("dummy_ult".into()),
            follow_up: None,
        },
    ));
}

fn message_cursor<T: Message>(app: &mut App) -> MessageCursor<T> {
    app.world_mut().resource_mut::<Messages<T>>().get_cursor()
}

fn drain_messages<T: Message + Clone>(cursor: &mut MessageCursor<T>, app: &App) -> Vec<T> {
    cursor
        .read(app.world().resource::<Messages<T>>())
        .cloned()
        .collect()
}

fn unit_energy(app: &mut App, id: UnitId) -> Energy {
    let mut q = app.world_mut().query::<(&Unit, &Energy)>();
    q.iter(app.world())
        .find(|(unit, _)| unit.id == id)
        .map(|(_, energy)| *energy)
        .unwrap_or_else(|| panic!("unit {id:?} missing Energy"))
}

fn unit_ult(app: &mut App, id: UnitId) -> UltimateCharge {
    let mut q = app.world_mut().query::<(&Unit, &UltimateCharge)>();
    q.iter(app.world())
        .find(|(unit, _)| unit.id == id)
        .map(|(_, ult)| ult.clone())
        .unwrap_or_else(|| panic!("unit {id:?} missing UltimateCharge"))
}

#[test]
fn agumon_energy_gauge_fills_locks_and_drains_end_to_end() {
    let roster = load_roster();
    let agumon = pilot(&roster, "Agumon");
    let mut app = skill_book_runtime_app(load_skill_book());
    spawn_from_def(
        &mut app,
        &agumon,
        agumon.hp_max,
        agumon.toughness_max,
        0,
        0,
    );
    spawn_training_dummy(&mut app);

    let mut cursor = message_cursor::<CombatEvent>(&mut app);

    for cast in 1..=4 {
        app.world_mut().write_message(ActionIntent::Basic {
            attacker: agumon.id,
            target: DUMMY_ID,
        });
        app.update();
        let _ = drain_messages(&mut cursor, &app);

        let energy = unit_energy(&mut app, agumon.id);
        assert_eq!(
            energy.current,
            cast * SHARP_CLAWS_ENERGY_GAIN,
            "sharp_claws should grant +6 energy per hit"
        );
    }

    let prelock_energy = unit_energy(&mut app, agumon.id);
    let prelock_ult = unit_ult(&mut app, agumon.id);
    assert_eq!(prelock_energy.current, 24);
    assert_eq!(prelock_ult.current, 100, "legacy charge is already primed after 4 basics");

    app.world_mut().write_message(ActionIntent::Ultimate {
        attacker: agumon.id,
        target: DUMMY_ID,
    });
    app.update();
    let failed_ult_events = drain_messages(&mut cursor, &app);

    assert!(failed_ult_events.iter().any(|event| {
        matches!(
            &event.kind,
            CombatEventKind::OnActionFailed { reason } if reason == "UltimateNotReady"
        )
    }));
    assert_eq!(
        unit_energy(&mut app, agumon.id).current,
        24,
        "failed ult must not spend Agumon's energy-backed gauge"
    );
    assert_eq!(
        unit_ult(&mut app, agumon.id).current,
        100,
        "failed ult must not spend legacy charge either"
    );

    for _ in 0..13 {
        app.world_mut().write_message(ActionIntent::Basic {
            attacker: agumon.id,
            target: DUMMY_ID,
        });
        app.update();
        let _ = drain_messages(&mut cursor, &app);
    }

    let ready_energy = unit_energy(&mut app, agumon.id);
    let ready_ult = unit_ult(&mut app, agumon.id);
    assert_eq!(ready_energy.current, ready_energy.max, "energy gauge should cap at full");
    assert!(
        ready_ult.current >= ready_ult.trigger,
        "legacy charge can stay primed; readiness is now gated by energy"
    );

    app.world_mut().write_message(ActionIntent::Ultimate {
        attacker: agumon.id,
        target: DUMMY_ID,
    });
    app.update();
    let ult_events = drain_messages(&mut cursor, &app);

    assert!(
        !ult_events.iter().any(|event| matches!(&event.kind, CombatEventKind::OnActionFailed { .. })),
        "full-energy Agumon ult should succeed: {ult_events:?}"
    );
    assert!(ult_events.iter().any(|event| {
        matches!(
            &event.kind,
            CombatEventKind::OnSkillCast { skill_id } if *skill_id == SkillId("agumon_ult".into())
        )
    }));
    assert!(ult_events.iter().any(|event| {
        matches!(&event.kind, CombatEventKind::UltimateUsed { unit_id } if *unit_id == agumon.id)
    }));

    let spent_energy = unit_energy(&mut app, agumon.id);
    let spent_ult = unit_ult(&mut app, agumon.id);
    assert_eq!(spent_energy.current, 0, "successful ult must drain Agumon's Energy.current");
    assert_eq!(spent_ult.current, 0, "successful ult must also zero legacy UltimateCharge.current");
}
