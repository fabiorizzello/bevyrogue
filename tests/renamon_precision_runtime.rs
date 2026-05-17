use bevy::prelude::*;

use bevyrogue::combat::api::intent::CastId;
use bevyrogue::combat::blueprints;
use bevyrogue::combat::events::{CombatEvent, CombatEventKind};
use bevyrogue::combat::kernel::{
    PrecisionCommitment, PrecisionOutcome, PrecisionReveal, PrecisionWindowKind,
    register_combat_kernel_runtime,
};
use bevyrogue::combat::kit::UnitSkills;
use bevyrogue::combat::log::ActionLog;
use bevyrogue::combat::observability::{capture_validation_snapshot, format_validation_snapshot};
use bevyrogue::combat::precision_mind_game::PrecisionMindGameState;
use bevyrogue::combat::sp::SpPool;
use bevyrogue::combat::state::CombatState;
use bevyrogue::combat::team::Team;
use bevyrogue::combat::toughness::Toughness;
use bevyrogue::combat::turn_system::ActionIntent;
use bevyrogue::combat::types::{Attribute, EvoStage, SkillId, UnitId};
use bevyrogue::combat::ultimate::{UltAccumulationTrigger, UltimateCharge};
use bevyrogue::combat::unit::Unit;
use bevyrogue::data::skills_ron::{SkillBook, SkillDef};

fn canonical_skill_book() -> SkillBook {
    ron::from_str(include_str!("../assets/data/skills.ron")).expect("parse skills.ron")
}

fn find_skill<'a>(book: &'a SkillBook, id: &str) -> &'a SkillDef {
    book.0
        .iter()
        .find(|skill| skill.id == SkillId(id.into()))
        .unwrap_or_else(|| panic!("missing canonical skill {id}"))
}

fn resolve_skill(
    book: &SkillBook,
    skill_id: &str,
    attacker: UnitId,
    target: UnitId,
) -> bevyrogue::combat::state::ResolvedAction {
    let skill = find_skill(book, skill_id);
    let kit = UnitSkills {
        basic: skill.id.clone(),
        skills: vec![skill.id.clone()],
        ultimate: skill.id.clone(),
        follow_up: None,
    };
    let intent = ActionIntent::Skill {
        attacker,
        skill_id: skill.id.clone(),
        target,
    };
    bevyrogue::combat::resolution::resolve_action(&intent, &kit, Some(book))
        .expect("skill resolves")
}

fn runtime_unit(id: UnitId, name: &str, attribute: Attribute) -> Unit {
    Unit {
        id,
        name: name.to_owned(),
        hp_max: 90,
        hp_current: 90,
        attribute,
        resists: vec![],
        evo_stage: EvoStage::Adult,
    }
}

#[test]
fn renamon_precision_loop_runtime_proof() {
    let mut app = App::new();
    register_combat_kernel_runtime(&mut app);
    app.add_message::<CombatEvent>();
    app.insert_resource(CombatState::default());
    app.insert_resource(SpPool::default());
    app.insert_resource(ActionLog::default());

    let renamon_id = UnitId(7);
    let target_id = UnitId(103);

    app.world_mut().spawn((
        runtime_unit(renamon_id, "Renamon", Attribute::Data),
        Team::Ally,
        Toughness::new(10, vec![]),
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
    ));
    app.world_mut().spawn((
        runtime_unit(target_id, "Ogremon", Attribute::Data),
        Team::Enemy,
        Toughness::new(20, vec![]),
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
    ));

    let book = canonical_skill_book();

    // 1. Open Momentum Window
    let resolved = resolve_skill(&book, "diamond_storm", renamon_id, target_id);
    let transitions = blueprints::transitions_for_action_checked(&resolved)
        .expect("renamon blueprint dispatch succeeds");
    assert_eq!(transitions.len(), 1);

    for transition in transitions {
        app.world_mut().write_message(CombatEvent {
            kind: CombatEventKind::OnKernelTransition { transition },
            source: renamon_id,
            target: target_id,
            follow_up_depth: 0,
            cast_id: CastId::ROOT,
        });
    }

    app.update();

    let precision = app.world().resource::<PrecisionMindGameState>();
    assert!(precision.is_window_open());
    assert_eq!(
        precision.current_window,
        Some(PrecisionWindowKind::Momentum)
    );

    // 2. Commit Press (Renamon Ult)
    let resolved = resolve_skill(&book, "renamon_ult", renamon_id, target_id);
    let transitions = blueprints::transitions_for_action_checked(&resolved)
        .expect("renamon blueprint dispatch succeeds");

    for transition in transitions {
        app.world_mut().write_message(CombatEvent {
            kind: CombatEventKind::OnKernelTransition { transition },
            source: renamon_id,
            target: target_id,
            follow_up_depth: 0,
            cast_id: CastId::ROOT,
        });
    }

    app.update();

    let precision = app.world().resource::<PrecisionMindGameState>();
    assert_eq!(precision.commitment, Some(PrecisionCommitment::Press));

    // 3. Reveal Bait (Kyubimon Onibidama)
    let resolved = resolve_skill(&book, "onibidama", renamon_id, target_id);
    let transitions = blueprints::transitions_for_action_checked(&resolved)
        .expect("renamon blueprint dispatch succeeds");

    for transition in transitions {
        app.world_mut().write_message(CombatEvent {
            kind: CombatEventKind::OnKernelTransition { transition },
            source: renamon_id,
            target: target_id,
            follow_up_depth: 0,
            cast_id: CastId::ROOT,
        });
    }

    app.update();

    let precision = app.world().resource::<PrecisionMindGameState>();
    assert_eq!(precision.reveal, Some(PrecisionReveal::Baited));

    // 4. Resolve Success (Kyubimon Koenryu)
    let resolved = resolve_skill(&book, "koenryu", renamon_id, target_id);
    let transitions = blueprints::transitions_for_action_checked(&resolved)
        .expect("renamon blueprint dispatch succeeds");

    for transition in transitions {
        app.world_mut().write_message(CombatEvent {
            kind: CombatEventKind::OnKernelTransition { transition },
            source: renamon_id,
            target: target_id,
            follow_up_depth: 0,
            cast_id: CastId::ROOT,
        });
    }

    app.update();

    let precision = app.world().resource::<PrecisionMindGameState>();
    assert_eq!(precision.outcome, Some(PrecisionOutcome::Success));

    // Final Validation Snapshot check
    let snapshot = capture_validation_snapshot(app.world_mut()).expect("snapshot should build");
    let formatted = format_validation_snapshot(&snapshot);

    assert!(formatted.contains("mind_game=phase=Resolved,window_index=1,window=Momentum,commitment=Press,reveal=Baited,outcome=Success"), "Snapshot format mismatch: {formatted}");
}
