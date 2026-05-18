use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevyrogue::combat::bootstrap::{
    EncounterPreset, SelectionRequest, apply_composition, bootstrap_encounter,
};
use bevyrogue::combat::team::Team;
use bevyrogue::combat::toughness::{Toughness, exposes_toughness_affordance};
use bevyrogue::combat::turn_order::TurnOrder;
use bevyrogue::combat::types::{EvoLineId, EvoStage, SkillId, UnitId};
use bevyrogue::combat::ultimate::UltAccumulationTrigger;
use bevyrogue::combat::unit::Unit;
use bevyrogue::data::units_ron::{UnitDef, UnitRoster};

fn setup_minimal_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<TurnOrder>();
    app.init_resource::<Assets<UnitRoster>>();
    app
}

#[test]
fn test_bootstrap_spawn_composition() {
    let mut app = setup_minimal_app();

    // 1. Prepare roster asset
    let roster_handle = {
        let mut assets = app.world_mut().resource_mut::<Assets<UnitRoster>>();
        assets.add(UnitRoster(vec![
            // Agumon
            UnitDef {
                id: UnitId(1),
                name: "Agumon".into(),
                role_tags: vec![],
                signature_traits: vec![],
                hp_max: 100,
                attribute: bevyrogue::combat::types::Attribute::Vaccine,
                team: Team::Ally,
                basic_damage_tag: bevyrogue::combat::types::DamageTag::Fire,
                basic_skill: SkillId("baby_flame".into()),
                skill_ids: vec![SkillId("baby_flame".into())],
                ultimate_skill: SkillId("agumon_ult".into()),
                follow_up: None,
                enemy_traits: vec![],
                charged_attack: None,
                form_identity: None,
                blueprint_metadata: Default::default(),
                resists: vec![],
                toughness_max: 50,
                weaknesses: vec![],
                ultimate_trigger: 100,
                ultimate_cap: 100,
                ultimate_accumulation_trigger: UltAccumulationTrigger::OnBasicAttack,
                ultimate_charge_per_event: 25,
                speed: 100,
                evo_stage: EvoStage::Child,
                evo_line: EvoLineId("test".into()),
                evolves_to: vec![],
                tempo_resistant: false,
                toughness_category: Default::default(),
            },
            // Gabumon
            UnitDef {
                id: UnitId(2),
                name: "Gabumon".into(),
                role_tags: vec![],
                signature_traits: vec![],
                hp_max: 100,
                attribute: bevyrogue::combat::types::Attribute::Data,
                team: Team::Ally,
                basic_damage_tag: bevyrogue::combat::types::DamageTag::Ice,
                basic_skill: SkillId("bubble_blast".into()),
                skill_ids: vec![SkillId("bubble_blast".into())],
                ultimate_skill: SkillId("gabumon_ult".into()),
                follow_up: None,
                enemy_traits: vec![],
                charged_attack: None,
                form_identity: None,
                blueprint_metadata: Default::default(),
                resists: vec![],
                toughness_max: 50,
                weaknesses: vec![],
                ultimate_trigger: 100,
                ultimate_cap: 100,
                ultimate_accumulation_trigger: UltAccumulationTrigger::OnBasicAttack,
                ultimate_charge_per_event: 25,
                speed: 90,
                evo_stage: EvoStage::Child,
                evo_line: EvoLineId("test".into()),
                evolves_to: vec![],
                tempo_resistant: false,
                toughness_category: Default::default(),
            },
            // V-mon
            UnitDef {
                id: UnitId(3),
                name: "V-mon".into(),
                role_tags: vec![],
                signature_traits: vec![],
                hp_max: 100,
                attribute: bevyrogue::combat::types::Attribute::Free,
                team: Team::Ally,
                basic_damage_tag: bevyrogue::combat::types::DamageTag::Electric,
                basic_skill: SkillId("v_header".into()),
                skill_ids: vec![SkillId("v_header".into())],
                ultimate_skill: SkillId("vmon_ult".into()),
                follow_up: None,
                enemy_traits: vec![],
                charged_attack: None,
                form_identity: None,
                blueprint_metadata: Default::default(),
                resists: vec![],
                toughness_max: 50,
                weaknesses: vec![],
                ultimate_trigger: 100,
                ultimate_cap: 100,
                ultimate_accumulation_trigger: UltAccumulationTrigger::OnBasicAttack,
                ultimate_charge_per_event: 25,
                speed: 110,
                evo_stage: EvoStage::Child,
                evo_line: EvoLineId("test".into()),
                evolves_to: vec![],
                tempo_resistant: false,
                toughness_category: Default::default(),
            },
            // Hackmon
            UnitDef {
                id: UnitId(4),
                name: "Hackmon".into(),
                role_tags: vec![],
                signature_traits: vec![],
                hp_max: 100,
                attribute: bevyrogue::combat::types::Attribute::Vaccine,
                team: Team::Ally,
                basic_damage_tag: bevyrogue::combat::types::DamageTag::Physical,
                basic_skill: SkillId("blade_code".into()),
                skill_ids: vec![SkillId("blade_code".into())],
                ultimate_skill: SkillId("hackmon_ult".into()),
                follow_up: None,
                enemy_traits: vec![],
                charged_attack: None,
                form_identity: None,
                blueprint_metadata: Default::default(),
                resists: vec![],
                toughness_max: 50,
                weaknesses: vec![],
                ultimate_trigger: 100,
                ultimate_cap: 100,
                ultimate_accumulation_trigger: UltAccumulationTrigger::OnBasicAttack,
                ultimate_charge_per_event: 25,
                speed: 95,
                evo_stage: EvoStage::Child,
                evo_line: EvoLineId("test".into()),
                evolves_to: vec![],
                tempo_resistant: false,
                toughness_category: Default::default(),
            },
            // Dorumon (not selected)
            UnitDef {
                id: UnitId(5),
                name: "Dorumon".into(),
                role_tags: vec![],
                signature_traits: vec![],
                hp_max: 100,
                attribute: bevyrogue::combat::types::Attribute::Virus,
                team: Team::Ally,
                basic_damage_tag: bevyrogue::combat::types::DamageTag::Dark,
                basic_skill: SkillId("draconic_edge".into()),
                skill_ids: vec![SkillId("draconic_edge".into())],
                ultimate_skill: SkillId("dorumon_ult".into()),
                follow_up: None,
                enemy_traits: vec![],
                charged_attack: None,
                form_identity: None,
                blueprint_metadata: Default::default(),
                resists: vec![],
                toughness_max: 50,
                weaknesses: vec![],
                ultimate_trigger: 100,
                ultimate_cap: 100,
                ultimate_accumulation_trigger: UltAccumulationTrigger::OnBasicAttack,
                ultimate_charge_per_event: 25,
                speed: 92,
                evo_stage: EvoStage::Child,
                evo_line: EvoLineId("test".into()),
                evolves_to: vec![],
                tempo_resistant: false,
                toughness_category: Default::default(),
            },
            // Devimon (boss enemy, required for BossEncounter preset)
            UnitDef {
                id: UnitId(101),
                name: "Devimon".into(),
                role_tags: vec!["boss".into()],
                signature_traits: vec!["evil".into()],
                hp_max: 500,
                attribute: bevyrogue::combat::types::Attribute::Virus,
                team: Team::Enemy,
                basic_damage_tag: bevyrogue::combat::types::DamageTag::Dark,
                basic_skill: SkillId("enemy_skill_fire".into()),
                skill_ids: vec![SkillId("enemy_skill_fire".into())],
                ultimate_skill: SkillId("enemy_ult_fire".into()),
                follow_up: None,
                enemy_traits: vec![],
                charged_attack: None,
                form_identity: None,
                blueprint_metadata: Default::default(),
                resists: vec![],
                toughness_max: 100,
                weaknesses: vec![],
                ultimate_trigger: 100,
                ultimate_cap: 150,
                ultimate_accumulation_trigger: UltAccumulationTrigger::OnBasicAttack,
                ultimate_charge_per_event: 25,
                speed: 80,
                evo_stage: EvoStage::Child,
                evo_line: EvoLineId("devimon_line".into()),
                evolves_to: vec![],
                tempo_resistant: true,
                toughness_category: Default::default(),
            },
        ]))
    };

    // 2. Perform bootstrap
    let request = SelectionRequest {
        rookie_ids: vec![UnitId(1), UnitId(2), UnitId(3), UnitId(4)],
    };

    let composition = {
        let rosters = app.world().resource::<Assets<UnitRoster>>();
        let roster_asset = rosters.get(&roster_handle).expect("asset not found");
        bootstrap_encounter(roster_asset, &request, EncounterPreset::BossEncounter)
            .expect("bootstrap failed")
    };

    // 3. Apply composition
    {
        let mut system_state: SystemState<Commands> = SystemState::new(app.world_mut());
        let mut commands = system_state.get_mut(app.world_mut());
        apply_composition(&mut commands, &composition);
        system_state.apply(app.world_mut());
    }

    // Flush commands
    app.update();

    // 4. Assertions
    let mut units_query = app.world_mut().query::<&Unit>();
    let spawned_ids: Vec<_> = units_query.iter(app.world()).map(|u| u.id).collect();

    // Should have 1, 2, 3, 4 + 0 (Taichi) + 101 (Devimon from BossEncounter)
    assert_eq!(spawned_ids.len(), 6);
    assert!(spawned_ids.contains(&UnitId(1)));
    assert!(spawned_ids.contains(&UnitId(2)));
    assert!(spawned_ids.contains(&UnitId(3)));
    assert!(spawned_ids.contains(&UnitId(4)));
    assert!(spawned_ids.contains(&UnitId(0))); // Taichi
    assert!(spawned_ids.contains(&UnitId(101))); // Devimon

    // Dorumon (5) should NOT be spawned
    assert!(!spawned_ids.contains(&UnitId(5)));

    let mut unit_state_query = app
        .world_mut()
        .query::<(&Unit, &Team, Option<&Toughness>)>();
    for (_, team, toughness) in unit_state_query.iter(app.world()) {
        match team {
            Team::Ally => assert!(
                !exposes_toughness_affordance(*team, toughness.as_deref()),
                "ally units must not expose toughness affordances"
            ),
            Team::Enemy => assert!(
                exposes_toughness_affordance(*team, toughness.as_deref()),
                "positive-bar enemies must expose toughness affordances"
            ),
        }
    }

    // In the AV system, turn order is determined by ActionValue components — check 6 units spawned with AV
    let mut av_query = app
        .world_mut()
        .query::<&bevyrogue::combat::av::ActionValue>();
    assert_eq!(av_query.iter(app.world()).count(), 6);
}
