use bevyrogue::combat::action_query::{
    ChargedTelegraphAffordance, EnemyTraitAffordance, ImplementationStatus, ResourceKind,
    ResourceStatus, UnitQuerySnapshot, query_charged_telegraph_affordance,
    query_enemy_trait_affordances,
};
use bevyrogue::combat::counterplay::{
    ChargedAttackDeclaration, EnemyCounterplayKind, EnemyCounterplayStatus, EnemyTraitDeclaration,
};
use bevyrogue::combat::team::Team;
use bevyrogue::combat::toughness::{Toughness, ToughnessCategory};
use bevyrogue::combat::types::{DamageTag, SkillId, UnitId};
use bevyrogue::data::skills_ron::LegalityReasonCode;
use bevyrogue::data::units_ron::UnitRoster;

fn unit_snapshot(
    id: u32,
    toughness: Option<Toughness>,
    enemy_traits: Vec<EnemyTraitDeclaration>,
    charged_attack: Option<ChargedAttackDeclaration>,
) -> UnitQuerySnapshot {
    UnitQuerySnapshot {
        id: UnitId(id),
        team: Team::Enemy,
        is_active: false,
        is_ko: false,
        is_stunned: false,
        is_commander: false,
        hp_current: 100,
        hp_max: 100,
        sp: 0,
        ultimate_current: 0,
        ultimate_trigger: 100,
        ultimate_ready: false,
        energy: 0,
        energy_secondary_gained: 0,
        energy_external_gained: 0,
        skills: None,
        toughness,
        enemy_traits,
        charged_attack,
        ..Default::default()
    }
}

fn shielded_toughness() -> Toughness {
    Toughness::with_category(30, vec![DamageTag::Fire], ToughnessCategory::Shielded)
}

#[test]
fn canonical_devimon_projection_surfaces_implemented_and_deferred_states() {
    let roster: UnitRoster =
        ron::from_str(include_str!("../assets/data/units.ron")).expect("parse units.ron");
    let devimon = roster
        .0
        .iter()
        .find(|unit| unit.name == "Devimon")
        .expect("Devimon");

    let snapshot = unit_snapshot(
        devimon.id.0,
        Some(Toughness::with_category(
            devimon.toughness_max,
            devimon.weaknesses.clone(),
            devimon.toughness_category,
        )),
        devimon.enemy_traits.clone(),
        devimon.charged_attack.clone(),
    );

    let traits = query_enemy_trait_affordances(&snapshot);
    assert_eq!(traits.len(), 3);

    assert!(matches!(
        traits.as_slice(),
        [
            EnemyTraitAffordance {
                kind: EnemyCounterplayKind::TempoAnchor,
                implementation: ImplementationStatus::Implemented,
                resource: bevyrogue::combat::action_query::ResourceAffordanceDetail {
                    kind: ResourceKind::EnemyTrait,
                    status: ResourceStatus::Enabled,
                    ..
                },
            },
            EnemyTraitAffordance {
                kind: EnemyCounterplayKind::TypeTrap,
                implementation: ImplementationStatus::Deferred {
                    reason: LegalityReasonCode::EnemyTraitDeferred
                },
                resource: bevyrogue::combat::action_query::ResourceAffordanceDetail {
                    kind: ResourceKind::EnemyTrait,
                    status: ResourceStatus::Deferred {
                        reason: LegalityReasonCode::EnemyTraitDeferred
                    },
                    ..
                },
            },
            EnemyTraitAffordance {
                kind: EnemyCounterplayKind::ReactiveArmor,
                implementation: ImplementationStatus::Deferred {
                    reason: LegalityReasonCode::EnemyTraitDeferred
                },
                resource: bevyrogue::combat::action_query::ResourceAffordanceDetail {
                    kind: ResourceKind::EnemyTrait,
                    status: ResourceStatus::Deferred {
                        reason: LegalityReasonCode::EnemyTraitDeferred
                    },
                    ..
                },
            },
        ]
    ));

    let telegraph = query_charged_telegraph_affordance(&snapshot).expect("charged telegraph");
    assert!(matches!(
        telegraph,
        ChargedTelegraphAffordance {
            skill_id: SkillId(skill_id),
            lead_turns: 2,
            implementation: ImplementationStatus::Deferred { reason: LegalityReasonCode::ChargedTelegraphDeferred },
            resource: bevyrogue::combat::action_query::ResourceAffordanceDetail {
                kind: ResourceKind::ChargedTelegraph,
                status: ResourceStatus::Deferred { reason: LegalityReasonCode::ChargedTelegraphDeferred },
                ..
            },
        } if skill_id == "enemy_ult_fire"
    ));
}

#[test]
fn shielded_break_seal_is_implemented_while_armored_reactive_armor_stays_deferred() {
    let break_seal_snapshot = unit_snapshot(
        777,
        Some(shielded_toughness()),
        vec![EnemyTraitDeclaration {
            kind: EnemyCounterplayKind::BreakSeal,
            status: EnemyCounterplayStatus::Implemented,
        }],
        None,
    );

    let traits = query_enemy_trait_affordances(&break_seal_snapshot);
    assert!(matches!(
        traits.as_slice(),
        [EnemyTraitAffordance {
            kind: EnemyCounterplayKind::BreakSeal,
            implementation: ImplementationStatus::Implemented,
            resource: bevyrogue::combat::action_query::ResourceAffordanceDetail {
                kind: ResourceKind::EnemyTrait,
                status: ResourceStatus::Enabled,
                ..
            },
        }]
    ));

    let armored_snapshot = unit_snapshot(
        778,
        Some(Toughness::with_category(
            30,
            vec![DamageTag::Fire],
            ToughnessCategory::Armored,
        )),
        vec![EnemyTraitDeclaration {
            kind: EnemyCounterplayKind::ReactiveArmor,
            status: EnemyCounterplayStatus::Deferred {
                reason: LegalityReasonCode::EnemyTraitDeferred,
            },
        }],
        None,
    );
    let armored_traits = query_enemy_trait_affordances(&armored_snapshot);
    assert!(matches!(
        armored_traits.as_slice(),
        [EnemyTraitAffordance {
            kind: EnemyCounterplayKind::ReactiveArmor,
            implementation: ImplementationStatus::Deferred {
                reason: LegalityReasonCode::EnemyTraitDeferred
            },
            resource: bevyrogue::combat::action_query::ResourceAffordanceDetail {
                kind: ResourceKind::EnemyTrait,
                status: ResourceStatus::Deferred {
                    reason: LegalityReasonCode::EnemyTraitDeferred
                },
                ..
            },
        }]
    ));
}

#[test]
fn empty_minion_declarations_stay_empty_and_hidden_telegraphs_stay_hidden() {
    let minion_snapshot = unit_snapshot(102, None, vec![], None);
    assert!(query_enemy_trait_affordances(&minion_snapshot).is_empty());
    assert!(query_charged_telegraph_affordance(&minion_snapshot).is_none());

    let hidden_snapshot = unit_snapshot(
        103,
        Some(Toughness::with_category(
            20,
            vec![DamageTag::Fire],
            ToughnessCategory::Standard,
        )),
        vec![],
        Some(ChargedAttackDeclaration {
            skill_id: SkillId("ogremon_ult".into()),
            lead_turns: 1,
            status: EnemyCounterplayStatus::Hidden {
                reason: LegalityReasonCode::ChargedTelegraphDeferred,
            },
        }),
    );
    let hidden = query_charged_telegraph_affordance(&hidden_snapshot).expect("hidden telegraph");
    assert!(matches!(
        hidden.resource.status,
        ResourceStatus::Hidden {
            reason: LegalityReasonCode::ChargedTelegraphDeferred
        }
    ));
}
