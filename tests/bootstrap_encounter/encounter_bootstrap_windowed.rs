use bevyrogue::combat::bootstrap::{
    AGUMON_DUMMY_ID, EncounterPreset, SelectionRequest, bootstrap_encounter,
};
use bevyrogue::combat::team::Team;
use bevyrogue::combat::toughness::ToughnessCategory;
use bevyrogue::combat::types::{Attribute, DamageTag, EvoLineId, EvoStage, SkillId, UnitId};
use bevyrogue::combat::ultimate::UltAccumulationTrigger;
use bevyrogue::data::units_ron::{UnitDef, UnitRoster};

fn agumon_unit_def() -> UnitDef {
    UnitDef {
        id: UnitId(1),
        name: "Agumon".into(),
        role_tags: vec!["vanguard".into(), "breaker".into()],
        signature_traits: vec!["courage".into(), "fire".into()],
        hp_max: 100,
        attribute: Attribute::Vaccine,
        team: Team::Ally,
        basic_damage_tag: DamageTag::Fire,
        basic_skill: SkillId("sharp_claws".into()),
        skill_ids: vec![SkillId("baby_flame".into())],
        ultimate_skill: SkillId("agumon_ult".into()),
        follow_up: None,
        enemy_traits: vec![],
        charged_attack: None,
        form_identity: None,
        blueprint_metadata: Default::default(),
        resists: vec![],
        toughness_max: 50,
        weaknesses: vec![DamageTag::Ice],
        ultimate_trigger: 100,
        ultimate_cap: 150,
        ultimate_accumulation_trigger: UltAccumulationTrigger::OnBasicAttack,
        ultimate_charge_per_event: 25,
        speed: 100,
        evo_stage: EvoStage::Child,
        evo_line: EvoLineId("agumon_line".into()),
        evolves_to: vec![UnitId(12)],
        tempo_resistant: false,
        toughness_category: ToughnessCategory::Standard,
    }
}

#[test]
fn agumon_training_dummy_has_one_ally_and_one_enemy() {
    let roster = UnitRoster(vec![agumon_unit_def()]);
    let request = SelectionRequest { rookie_ids: vec![] };

    let composition = bootstrap_encounter(&roster, &request, EncounterPreset::AgumonTrainingDummy)
        .expect("AgumonTrainingDummy should never fail given a valid Agumon in the roster");

    assert_eq!(composition.allies.len(), 1, "expected exactly one ally");
    assert_eq!(composition.enemies.len(), 1, "expected exactly one enemy");

    let ally = &composition.allies[0];
    assert_eq!(ally.id, UnitId(1), "ally should be Agumon (UnitId 1)");
    assert_eq!(ally.team, Team::Ally, "ally should have Team::Ally");

    let dummy = &composition.enemies[0];
    assert_ne!(dummy.id, ally.id, "dummy must have a distinct UnitId from the ally");
    assert_eq!(dummy.id, AGUMON_DUMMY_ID, "dummy should use the stable AGUMON_DUMMY_ID");
    assert_eq!(dummy.team, Team::Enemy, "dummy must be on Team::Enemy");
}

#[test]
fn agumon_training_dummy_fails_gracefully_when_agumon_missing_from_roster() {
    let empty_roster = UnitRoster(vec![]);
    let request = SelectionRequest { rookie_ids: vec![] };

    let result =
        bootstrap_encounter(&empty_roster, &request, EncounterPreset::AgumonTrainingDummy);

    assert!(result.is_err(), "should fail when Agumon is not in the roster");
}
