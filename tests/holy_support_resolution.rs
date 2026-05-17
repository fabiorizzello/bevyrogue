use bevy::prelude::*;

use bevyrogue::combat::blueprints;
use bevyrogue::combat::api::intent::CastId;
use bevyrogue::combat::events::{CombatEvent, CombatEventKind};
use bevyrogue::combat::blueprints::patamon::{HolySupportState, HolySupportTransition};
use bevyrogue::combat::api::SignalPayload;
use bevyrogue::combat::kernel::{CombatKernelRegistry, CombatKernelTransition};
use bevyrogue::combat::kit::UnitSkills;
use bevyrogue::combat::log::ActionLog;
use bevyrogue::combat::observability::{capture_validation_snapshot, format_validation_snapshot};
use bevyrogue::combat::resolution::{apply_legacy_ops, resolve_action};
use bevyrogue::combat::sp::{RoundSpTracker, SpPool};
use bevyrogue::combat::state::CombatState;
use bevyrogue::combat::team::Team;
use bevyrogue::combat::toughness::Toughness;
use bevyrogue::combat::turn_system::ActionIntent;
use bevyrogue::combat::types::{Attribute, DamageTag, EvoStage, SkillId, UnitId};
use bevyrogue::combat::ultimate::{UltAccumulationTrigger, UltimateCharge};
use bevyrogue::combat::unit::{BasicStreak, Unit};
use bevyrogue::data::skills_ron::SkillBook;

fn load_skill_book() -> SkillBook {
    ron::from_str(include_str!("../assets/data/skills.ron")).expect("parse skills.ron")
}

fn unit(id: u32, hp_current: i32) -> Unit {
    Unit {
        id: UnitId(id),
        name: format!("Unit{id}"),
        hp_max: 100,
        hp_current,
        attribute: Attribute::Vaccine,
        resists: vec![],
        evo_stage: EvoStage::Adult,
    }
}

fn resolved_skill(
    book: &SkillBook,
    skill_id: &str,
    attacker: UnitId,
    target: UnitId,
) -> bevyrogue::combat::state::ResolvedAction {
    let skill = book
        .0
        .iter()
        .find(|skill| skill.id == SkillId(skill_id.into()))
        .expect("skill exists in book")
        .clone();

    let kit = UnitSkills {
        basic: skill.id.clone(),
        skills: vec![skill.id.clone()],
        ultimate: skill.id.clone(),
        follow_up: None,
    };
    let intent = ActionIntent::Skill {
        attacker,
        skill_id: skill.id,
        target,
    };
    resolve_action(&intent, &kit, Some(book)).expect("skill should resolve")
}

fn app_with_holy_support() -> App {
    let mut app = App::new();
    app.add_message::<CombatEvent>();
    bevyrogue::combat::kernel::register_combat_kernel_runtime(&mut app);
    app.insert_resource(CombatState::default())
        .insert_resource(SpPool::default())
        .insert_resource(ActionLog::default());
    app
}

fn emit_transitions(
    app: &mut App,
    transitions: Vec<CombatKernelTransition>,
    source: UnitId,
    target: UnitId,
) -> Vec<CombatKernelTransition> {
    let mut emitted = Vec::new();

    for transition in transitions {
        let dispatched = {
            let registry = app.world().resource::<CombatKernelRegistry>();
            registry.dispatch(transition)
        };

        for transition in dispatched {
            emitted.push(transition.clone());
            app.world_mut().write_message(CombatEvent {
                kind: CombatEventKind::OnKernelTransition { transition },
                source,
                target,
                follow_up_depth: 0,
                cast_id: CastId::ROOT,
            });
        }
    }

    app.update();
    emitted
}

#[test]
fn patamon_ult_builds_grace_through_the_blueprint_kernel_path() {
    let book = load_skill_book();
    let attacker = unit(9, 88);
    let mut defender = unit(2, 100);
    let mut tough = Toughness::new(50, vec![DamageTag::Fire]);
    let mut ult = UltimateCharge {
        current: 100,
        trigger: 100,
        cap: 150,
        trigger_type: UltAccumulationTrigger::OnBasicAttack,
        charge_per_event: 25,
    };
    let mut sp = SpPool::default();
    let mut streak = BasicStreak::default();
    let resolved = resolved_skill(&book, "patamon_ult", attacker.id, defender.id);

    let (outcome, events) = apply_legacy_ops(
        &resolved,
        &attacker,
        &mut defender,
        Team::Enemy,
        Some(&mut tough),
        &mut ult,
        &mut sp,
        &mut RoundSpTracker::default(),
        &mut streak,
        false,
        false,
        None,
        None,
        None,
    );

    assert!(outcome.succeeded);
    assert!(events.iter().all(|event| !matches!(
        event,
        CombatEventKind::OnKernelTransition {
            transition: CombatKernelTransition::HolySupport(_)
        }
    )));

    let transitions = blueprints::transitions_for_action(&resolved);
    assert_eq!(
        transitions,
        vec![CombatKernelTransition::Blueprint {
            owner: "patamon".to_owned(),
            name: "build_holy_support_grace".to_owned(),
            payload: SignalPayload::Amount(1),
        }]
    );

    let mut app = app_with_holy_support();
    let emitted = emit_transitions(&mut app, transitions, attacker.id, defender.id);

    assert_eq!(
        emitted,
        vec![CombatKernelTransition::Blueprint {
            owner: "patamon".to_owned(),
            name: "build_holy_support_grace".to_owned(),
            payload: SignalPayload::Amount(1),
        }]
    );

    let state = app.world().resource::<HolySupportState>();
    assert_eq!(state.grace, 1);
    assert_eq!(
        state.last_signal,
        Some(HolySupportTransition::build_grace(1))
    );

    let snapshot = capture_validation_snapshot(app.world_mut()).expect("snapshot");
    let formatted = format_validation_snapshot(&snapshot);
    assert!(formatted.contains("holy_support=grace=1/3"));
    assert!(formatted.contains("last=build(1)"));
}

#[test]
fn plain_patamon_skill_has_no_holy_support_blueprint_dispatch() {
    let book = load_skill_book();
    let attacker = unit(9, 88);
    let defender = unit(2, 100);
    let resolved = resolved_skill(&book, "holy_breeze", attacker.id, defender.id);

    assert!(blueprints::transitions_for_action(&resolved).is_empty());
}
