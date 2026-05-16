use bevyrogue::combat::blueprints::{self, CustomSignalDispatchError};
use bevyrogue::combat::kernel::{CombatKernelTransition, PredatorLoopTransition};
use bevyrogue::combat::kit::UnitSkills;
use bevyrogue::combat::resolution::resolve_action;
use bevyrogue::combat::turn_system::ActionIntent;
use bevyrogue::combat::types::{DamageTag, SkillId, UnitId};
use bevyrogue::data::skills_ron::{
    CustomSignalPayload, Effect, SelfTargetRule, SkillBook, SkillCustomSignal, SkillDef,
    SkillImplementation, SkillTargeting, TargetLife, TargetShape, TargetSide,
};

fn canonical_skill_book() -> SkillBook {
    ron::from_str(include_str!("../assets/data/skills.ron")).expect("parse skills.ron")
}

fn find_skill<'a>(book: &'a SkillBook, id: &str) -> &'a SkillDef {
    book.0
        .iter()
        .find(|skill| skill.id == SkillId(id.into()))
        .unwrap_or_else(|| panic!("{id} in canonical skill book"))
}

fn resolve_skill(book: &SkillBook, skill_id: &str) -> bevyrogue::combat::state::ResolvedAction {
    let skill = find_skill(book, skill_id);
    let kit = UnitSkills {
        basic: skill.id.clone(),
        skills: vec![skill.id.clone()],
        ultimate: skill.id.clone(),
        follow_up: None,
    };
    let intent = ActionIntent::Skill {
        attacker: UnitId(5),
        skill_id: skill.id.clone(),
        target: UnitId(7),
    };
    resolve_action(&intent, &kit, Some(book)).expect("skill resolves")
}

fn blueprint_skill(
    id: &str,
    owner: &str,
    signal: &str,
    payload: CustomSignalPayload,
) -> SkillDef {
    SkillDef {
        id: SkillId(id.into()),
        name: id.to_owned(),
        damage_tag: DamageTag::Dark,
        sp_cost: 0,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Implemented,
        legacy_ops: vec![Effect::Damage {
            amount: 1,
            target: TargetShape::Single,
        per_hop: Default::default(),
        }],
        custom_signals: vec![SkillCustomSignal::blueprint(owner, signal, payload)],
        ..Default::default()
    }
}

#[test]
fn dorumon_assets_use_generic_blueprint_envelopes() {
    let book = canonical_skill_book();
    let dorumon = find_skill(&book, "dorumon_ult");

    assert_eq!(
        dorumon.custom_signals,
        vec![
            SkillCustomSignal::blueprint(
                "dorumon",
                "build_exploit",
                CustomSignalPayload::Amount { amount: 2 },
            ),
            SkillCustomSignal::blueprint(
                "dorumon",
                "apply_prey_lock",
                CustomSignalPayload::Amount { amount: 2 },
            ),
        ]
    );
}

#[test]
fn registry_routes_dorumon_signals_to_predator_loop_transitions() {
    let book = canonical_skill_book();
    let resolved = resolve_skill(&book, "dorumon_ult");

    let transitions = blueprints::transitions_for_action_checked(&resolved)
        .expect("dorumon blueprint dispatch succeeds");

    assert_eq!(
        transitions,
        vec![
            CombatKernelTransition::PredatorLoop(PredatorLoopTransition::build_exploit(
                UnitId(7),
                2,
            )),
            CombatKernelTransition::PredatorLoop(PredatorLoopTransition::apply_prey_lock(
                UnitId(7),
                2,
            )),
        ]
    );
}

#[test]
fn renamon_precision_signals_route_to_precision_mind_game_transitions() {
    let book = canonical_skill_book();
    
    // Test open_momentum_window (Diamond Storm)
    let resolved = resolve_skill(&book, "diamond_storm");
    let transitions = blueprints::transitions_for_action_checked(&resolved).expect("dispatch");
    assert_eq!(transitions.len(), 1);
    assert!(matches!(transitions[0], CombatKernelTransition::PrecisionMindGame(_)));

    // Test commit_precision_press (Renamon Ult)
    let resolved = resolve_skill(&book, "renamon_ult");
    let transitions = blueprints::transitions_for_action_checked(&resolved).expect("dispatch");
    assert_eq!(transitions.len(), 1);
    assert!(matches!(transitions[0], CombatKernelTransition::PrecisionMindGame(_)));
}

#[test]
fn registry_rejects_unknown_blueprint_owner() {
    let skill = blueprint_skill(
        "unknown_owner_signal",
        "unknown-owner",
        "build_exploit",
        CustomSignalPayload::Amount { amount: 1 },
    );
    let book = SkillBook(vec![skill.clone()]);
    let resolved = resolve_skill(&book, &skill.id.0);

    let error = blueprints::transitions_for_action_checked(&resolved)
        .expect_err("unknown owner rejected");

    assert!(matches!(
        error,
        CustomSignalDispatchError::UnknownOwner { .. }
    ));
}

#[test]
fn registry_rejects_malformed_payload() {
    let skill = blueprint_skill(
        "malformed_dorumon_signal",
        "dorumon",
        "build_exploit",
        CustomSignalPayload::Empty,
    );
    let book = SkillBook(vec![skill.clone()]);
    let resolved = resolve_skill(&book, &skill.id.0);

    let error = blueprints::transitions_for_action_checked(&resolved)
        .expect_err("malformed payload rejected");

    assert!(matches!(
        error,
        CustomSignalDispatchError::MalformedPayload { .. }
    ));
}
