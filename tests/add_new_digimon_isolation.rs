use bevyrogue::combat::blueprints::{self, CustomSignalDispatchError};
use bevyrogue::combat::resolution::resolve_action;
use bevyrogue::combat::turn_system::ActionIntent;
use bevyrogue::combat::types::{DamageTag, SkillId, UnitId};
use bevyrogue::combat::kit::UnitSkills;
use bevyrogue::data::skills_ron::{CustomSignalPayload, Effect, SelfTargetRule, SkillBook, SkillCustomSignal, SkillDef, SkillImplementation, SkillTargeting, TargetLife, TargetShape, TargetSide};
use bevyrogue::data::units_ron::UnitRoster;

fn canonical_roster() -> UnitRoster {
    bevyrogue::data::aggregate_unit_roster()
}

fn canonical_skill_book() -> SkillBook {
    bevyrogue::data::aggregate_skill_book()
}

fn skill_with_signal(id: &str, owner: &str, signal: &str, payload: CustomSignalPayload) -> SkillDef {
    SkillDef {
        id: SkillId(id.into()),
        name: id.to_owned(),
        damage_tag: DamageTag::Light,
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

fn resolve_skill(book: &SkillBook, skill_id: &str) -> bevyrogue::combat::state::ResolvedAction {
    let skill = book
        .0
        .iter()
        .find(|skill| skill.id == SkillId(skill_id.into()))
        .unwrap_or_else(|| panic!("{skill_id} in canonical skill book"));
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

#[test]
fn add_new_digimon_roster_metadata_stays_optional_for_existing_units() {
    let roster = canonical_roster();
    let agumon = roster
        .0
        .iter()
        .find(|unit| unit.name == "Agumon")
        .expect("Agumon in roster");

    assert!(
        agumon.blueprint_metadata.0.is_empty(),
        "adding a new digimon must not require shared roster edits for existing units"
    );
}

#[test]
fn add_new_digimon_requires_registry_owner_not_shared_kernel_names() {
    let book = canonical_skill_book();
    let skill = skill_with_signal(
        "unknown_owner_signal",
        "unknown-owner",
        "build_exploit",
        CustomSignalPayload::Amount { amount: 1 },
    );
    let book = SkillBook(vec![skill.clone(), book.0[0].clone()]);
    let resolved = resolve_skill(&book, &skill.id.0);

    let err = blueprints::transitions_for_action_checked(&resolved)
        .expect_err("unknown owner must be rejected");
    assert!(matches!(err, CustomSignalDispatchError::UnknownOwner { .. }));
}

#[test]
fn add_new_digimon_defaults_unknown_blueprint_metadata_and_dispatch_isolation() {
    let roster = canonical_roster();
    let skill_book = canonical_skill_book();
    let patamon_ult = skill_book
        .0
        .iter()
        .find(|skill| skill.id == SkillId("patamon_ult".into()))
        .expect("Patamon ult in skill book");

    assert_eq!(
        patamon_ult.custom_signals,
        vec![SkillCustomSignal::blueprint(
            "patamon",
            "build_holy_support_grace",
            CustomSignalPayload::Amount { amount: 1 },
        )],
        "the existing blueprint path remains owner-keyed and isolated"
    );

    let agumon = roster
        .0
        .iter()
        .find(|unit| unit.name == "Agumon")
        .expect("Agumon in roster");
    assert!(agumon.blueprint_metadata.0.is_empty());
}
