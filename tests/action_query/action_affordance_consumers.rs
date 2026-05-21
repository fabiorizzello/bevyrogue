use bevyrogue::combat::action_query::{
    ActionQueryKind, ActionStatus, CombatQuerySnapshot, ResourceKind, ResourceStatus, TargetStatus,
    build_snapshot_from_ecs, build_snapshot_from_ecs_with_sp, enabled_target_ids,
    first_enabled_target_id, query_action_affordance,
};
use bevyrogue::combat::counterplay::EnemyCounterplayKit;
use bevyrogue::combat::energy::{Energy, RoundEnergyTracker};
use bevyrogue::combat::kit::UnitSkills;
use bevyrogue::combat::sp::SpPool;
use bevyrogue::combat::state::{CombatPhase, CombatState};
use bevyrogue::combat::team::Team;
use bevyrogue::combat::toughness::Toughness;
use bevyrogue::combat::types::{Attribute, DamageTag, EvoStage, SkillId, UnitId};
use bevyrogue::combat::ultimate::UltimateCharge;
use bevyrogue::combat::unit::Unit;
use bevyrogue::data::skills_ron::{
    Effect, LegalityReasonCode, SelfTargetRule, SkillBook, SkillDef, SkillImplementation,
    SkillTargeting, TargetHpRule, TargetLife, TargetShape, TargetSide,
};

#[derive(Clone)]
struct Fixture {
    id: UnitId,
    team: Team,
    hp_current: i32,
    hp_max: i32,
    is_ko: bool,
    is_stunned: bool,
    is_commander: bool,
    skills: Option<UnitSkills>,
    ultimate: Option<UltimateCharge>,
    toughness: Option<Toughness>,
    energy: Option<Energy>,
    tracker: Option<RoundEnergyTracker>,
    counterplay: Option<EnemyCounterplayKit>,
}

fn unit(
    id: u32,
    team: Team,
    hp_current: i32,
    hp_max: i32,
    _is_ko: bool,
    _is_commander: bool,
) -> Unit {
    Unit {
        id: UnitId(id),
        name: format!("Unit{id}"),
        hp_max,
        hp_current,
        attribute: match team {
            Team::Ally => Attribute::Vaccine,
            Team::Enemy => Attribute::Virus,
        },
        resists: vec![],
        evo_stage: EvoStage::Adult,
    }
}

fn unit_skills(basic: &str, skills: Vec<&str>, ultimate: &str) -> UnitSkills {
    UnitSkills {
        basic: SkillId(basic.into()),
        skills: skills
            .into_iter()
            .map(|skill| SkillId(skill.into()))
            .collect(),
        ultimate: SkillId(ultimate.into()),
        follow_up: None,
    }
}

fn ecs_units(fixtures: &[Fixture]) -> Vec<Unit> {
    fixtures
        .iter()
        .map(|fixture| {
            unit(
                fixture.id.0,
                fixture.team,
                fixture.hp_current,
                fixture.hp_max,
                fixture.is_ko,
                fixture.is_commander,
            )
        })
        .collect()
}

fn basic_skill(id: &str) -> SkillDef {
    SkillDef {
        id: SkillId(id.into()),
        name: "Basic Attack".into(),
        damage_tag: DamageTag::Fire,
        sp_cost: 0,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            target_hp_rule: TargetHpRule::Any,
        },
        implementation: SkillImplementation::Implemented,
        legacy_ops: vec![Effect::Damage {
            amount: 10,
            target: TargetShape::Single,
            per_hop: Default::default(),
        }],
        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
        timeline: None,
    }
}

fn revive_skill(id: &str, sp_cost: i32) -> SkillDef {
    SkillDef {
        id: SkillId(id.into()),
        name: "Revive Skill".into(),
        damage_tag: DamageTag::Light,
        sp_cost,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Ally,
            life: TargetLife::Ko,
            self_rule: SelfTargetRule::Forbid,
            target_hp_rule: TargetHpRule::Any,
        },
        implementation: SkillImplementation::Implemented,
        legacy_ops: vec![Effect::Revive(25)],
        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
        timeline: None,
    }
}

fn deferred_skill(id: &str) -> SkillDef {
    SkillDef {
        id: SkillId(id.into()),
        name: "Deferred Skill".into(),
        damage_tag: DamageTag::Fire,
        sp_cost: 0,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            target_hp_rule: TargetHpRule::Any,
        },
        implementation: SkillImplementation::Deferred {
            reason: LegalityReasonCode::UnimplementedTargetShape,
        },
        legacy_ops: vec![Effect::Damage {
            amount: 10,
            target: TargetShape::Single,
            per_hop: Default::default(),
        }],
        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
        timeline: None,
    }
}

fn snapshot_from_fixtures(
    fixtures: &[Fixture],
    actor_id: UnitId,
    target_id: UnitId,
    phase: CombatPhase,
    active_unit: Option<UnitId>,
    sp_current: i32,
) -> CombatQuerySnapshot {
    let units = ecs_units(fixtures);
    let units_data: Vec<_> = fixtures
        .iter()
        .zip(units.iter())
        .map(|(fixture, unit)| {
            (
                fixture.id,
                fixture.team,
                unit,
                fixture.skills.as_ref(),
                fixture.ultimate.as_ref(),
                fixture.toughness.as_ref(),
                fixture.counterplay.as_ref(),
                fixture.is_ko,
                fixture.is_stunned,
                fixture.is_commander,
                fixture.energy.as_ref(),
                fixture.tracker.as_ref(),
                None,
            )
        })
        .collect();

    let state = CombatState {
        phase,
        winner: None,
    };
    let mut turn_order = bevyrogue::combat::turn_order::TurnOrder::default();
    turn_order.active_unit = active_unit;

    build_snapshot_from_ecs_with_sp(
        &state,
        &turn_order,
        sp_current,
        actor_id,
        target_id,
        units_data,
    )
}

fn action_from_snapshot<'a>(
    snapshot: &'a CombatQuerySnapshot,
    skill_book: &'a SkillBook,
    actor_id: UnitId,
    kind: ActionQueryKind<'a>,
) -> bevyrogue::combat::action_query::ActionAffordance<'a> {
    query_action_affordance(snapshot, skill_book, actor_id, kind)
}

#[test]
fn explicit_sp_snapshot_blocks_revive_but_bypass_snapshot_remains_separate() {
    let revive = revive_skill("revive_skill", 3);
    let basic = basic_skill("basic_attack");
    let skill_book = SkillBook(vec![basic.clone(), revive.clone()]);

    let fixtures = vec![
        Fixture {
            id: UnitId(1),
            team: Team::Ally,
            hp_current: 40,
            hp_max: 40,
            is_ko: false,
            is_stunned: false,
            is_commander: false,
            skills: Some(unit_skills(
                "basic_attack",
                vec!["revive_skill"],
                "basic_attack",
            )),
            ultimate: Some(UltimateCharge {
                current: 120,
                trigger: 100,
                cap: 150,
                trigger_type: bevyrogue::combat::ultimate::UltAccumulationTrigger::OnBasicAttack,
                charge_per_event: 25,
            }),
            toughness: None,
            energy: Some(Energy {
                current: 0,
                max: 100,
            }),
            tracker: Some(RoundEnergyTracker::default()),
            counterplay: None,
        },
        Fixture {
            id: UnitId(2),
            team: Team::Ally,
            hp_current: 0,
            hp_max: 40,
            is_ko: true,
            is_stunned: false,
            is_commander: false,
            skills: None,
            ultimate: None,
            toughness: None,
            energy: None,
            tracker: None,
            counterplay: None,
        },
    ];

    let explicit_snapshot = snapshot_from_fixtures(
        &fixtures,
        UnitId(1),
        UnitId(2),
        CombatPhase::WaitingAction,
        Some(UnitId(1)),
        2,
    );
    let explicit_affordance = action_from_snapshot(
        &explicit_snapshot,
        &skill_book,
        UnitId(1),
        ActionQueryKind::Skill(&revive.id),
    );

    assert!(matches!(
        explicit_affordance.action,
        ActionStatus::Disabled {
            reason: LegalityReasonCode::SpShortfall
        }
    ));
    assert!(matches!(
        explicit_affordance.resource,
        ResourceStatus::Disabled {
            reason: LegalityReasonCode::SpShortfall
        }
    ));
    assert_eq!(
        explicit_affordance
            .resource_details
            .iter()
            .find(|detail| detail.kind == ResourceKind::Sp)
            .expect("sp detail")
            .current,
        Some(2)
    );
    assert_eq!(
        explicit_affordance
            .resource_details
            .iter()
            .find(|detail| detail.kind == ResourceKind::Sp)
            .expect("sp detail")
            .required,
        Some(3)
    );

    let units = ecs_units(&fixtures);
    let units_data: Vec<_> = fixtures
        .iter()
        .zip(units.iter())
        .map(|(fixture, unit)| {
            (
                fixture.id,
                fixture.team,
                unit,
                fixture.skills.as_ref(),
                fixture.ultimate.as_ref(),
                fixture.toughness.as_ref(),
                fixture.counterplay.as_ref(),
                fixture.is_ko,
                fixture.is_stunned,
                fixture.is_commander,
                fixture.energy.as_ref(),
                fixture.tracker.as_ref(),
                None,
            )
        })
        .collect();

    let bypass_snapshot = build_snapshot_from_ecs(
        &CombatState {
            phase: CombatPhase::WaitingAction,
            winner: None,
        },
        &bevyrogue::combat::turn_order::TurnOrder {
            active_unit: Some(UnitId(1)),
        },
        &SpPool::default(),
        UnitId(1),
        UnitId(2),
        units_data,
    );
    let bypass_affordance = action_from_snapshot(
        &bypass_snapshot,
        &skill_book,
        UnitId(1),
        ActionQueryKind::Skill(&revive.id),
    );

    assert!(matches!(bypass_affordance.action, ActionStatus::Enabled));
    assert!(matches!(bypass_affordance.target, TargetStatus::Enabled));
    assert_eq!(enabled_target_ids(&bypass_affordance), vec![UnitId(2)]);
}

#[test]
fn snapshot_carries_commander_energy_tracker_and_real_sp() {
    let snapshot = snapshot_from_fixtures(
        &[Fixture {
            id: UnitId(1),
            team: Team::Ally,
            hp_current: 40,
            hp_max: 40,
            is_ko: false,
            is_stunned: false,
            is_commander: true,
            skills: Some(unit_skills("basic_attack", vec![], "basic_attack")),
            ultimate: Some(UltimateCharge {
                current: 120,
                trigger: 100,
                cap: 150,
                trigger_type: bevyrogue::combat::ultimate::UltAccumulationTrigger::OnBasicAttack,
                charge_per_event: 25,
            }),
            toughness: None,
            energy: Some(Energy {
                current: 37,
                max: 100,
            }),
            tracker: Some(RoundEnergyTracker {
                secondary_gained: 8,
                external_gained: 12,
            }),
            counterplay: None,
        }],
        UnitId(1),
        UnitId(1),
        CombatPhase::WaitingAction,
        Some(UnitId(1)),
        4,
    );

    assert_eq!(snapshot.acting_unit.sp, 4);
    assert!(snapshot.acting_unit.is_commander);
    assert_eq!(snapshot.acting_unit.energy, 37);
    assert_eq!(snapshot.acting_unit.energy_secondary_gained, 8);
    assert_eq!(snapshot.acting_unit.energy_external_gained, 12);
}

#[test]
fn disabled_resource_keeps_target_reason_codes_for_ko_live_and_enemy_targets() {
    let revive = revive_skill("revive_skill", 3);
    let skill_book = SkillBook(vec![revive.clone()]);
    let snapshot = snapshot_from_fixtures(
        &[
            Fixture {
                id: UnitId(1),
                team: Team::Ally,
                hp_current: 40,
                hp_max: 40,
                is_ko: false,
                is_stunned: false,
                is_commander: false,
                skills: Some(unit_skills(
                    "basic_attack",
                    vec!["revive_skill"],
                    "basic_attack",
                )),
                ultimate: Some(UltimateCharge {
                    current: 120,
                    trigger: 100,
                    cap: 150,
                    trigger_type:
                        bevyrogue::combat::ultimate::UltAccumulationTrigger::OnBasicAttack,
                    charge_per_event: 25,
                }),
                toughness: None,
                energy: None,
                tracker: None,
                counterplay: None,
            },
            Fixture {
                id: UnitId(2),
                team: Team::Ally,
                hp_current: 0,
                hp_max: 40,
                is_ko: true,
                is_stunned: false,
                is_commander: false,
                skills: None,
                ultimate: None,
                toughness: None,
                energy: None,
                tracker: None,
                counterplay: None,
            },
            Fixture {
                id: UnitId(3),
                team: Team::Ally,
                hp_current: 30,
                hp_max: 40,
                is_ko: false,
                is_stunned: false,
                is_commander: false,
                skills: None,
                ultimate: None,
                toughness: None,
                energy: None,
                tracker: None,
                counterplay: None,
            },
            Fixture {
                id: UnitId(4),
                team: Team::Enemy,
                hp_current: 25,
                hp_max: 40,
                is_ko: false,
                is_stunned: false,
                is_commander: false,
                skills: None,
                ultimate: None,
                toughness: None,
                energy: None,
                tracker: None,
                counterplay: None,
            },
        ],
        UnitId(1),
        UnitId(2),
        CombatPhase::WaitingAction,
        Some(UnitId(1)),
        2,
    );

    let affordance = action_from_snapshot(
        &snapshot,
        &skill_book,
        UnitId(1),
        ActionQueryKind::Skill(&revive.id),
    );

    assert!(matches!(
        affordance.action,
        ActionStatus::Disabled {
            reason: LegalityReasonCode::SpShortfall
        }
    ));
    assert!(matches!(
        affordance.resource,
        ResourceStatus::Disabled {
            reason: LegalityReasonCode::SpShortfall
        }
    ));
    assert!(matches!(affordance.target, TargetStatus::Enabled));
    assert!(
        affordance
            .targets
            .iter()
            .any(|(id, target)| *id == UnitId(2) && matches!(target.status, TargetStatus::Enabled))
    );
    assert!(
        affordance
            .targets
            .iter()
            .any(|(id, target)| *id == UnitId(3)
                && matches!(
                    target.status,
                    TargetStatus::Disabled {
                        reason: LegalityReasonCode::TargetNotKo
                    }
                ))
    );
    assert!(
        affordance
            .targets
            .iter()
            .any(|(id, target)| *id == UnitId(4)
                && matches!(
                    target.status,
                    TargetStatus::Disabled {
                        reason: LegalityReasonCode::WrongSide
                    }
                ))
    );
}

#[test]
fn enabled_basic_target_is_chosen_from_query_output_not_local_team_assumptions() {
    let basic = basic_skill("basic_attack");
    let skill_book = SkillBook(vec![basic.clone()]);
    let snapshot = snapshot_from_fixtures(
        &[
            Fixture {
                id: UnitId(1),
                team: Team::Ally,
                hp_current: 40,
                hp_max: 40,
                is_ko: false,
                is_stunned: false,
                is_commander: false,
                skills: Some(unit_skills("basic_attack", vec![], "basic_attack")),
                ultimate: Some(UltimateCharge {
                    current: 120,
                    trigger: 100,
                    cap: 150,
                    trigger_type:
                        bevyrogue::combat::ultimate::UltAccumulationTrigger::OnBasicAttack,
                    charge_per_event: 25,
                }),
                toughness: None,
                energy: None,
                tracker: None,
                counterplay: None,
            },
            Fixture {
                id: UnitId(2),
                team: Team::Ally,
                hp_current: 30,
                hp_max: 40,
                is_ko: false,
                is_stunned: false,
                is_commander: false,
                skills: None,
                ultimate: None,
                toughness: None,
                energy: None,
                tracker: None,
                counterplay: None,
            },
            Fixture {
                id: UnitId(3),
                team: Team::Enemy,
                hp_current: 25,
                hp_max: 40,
                is_ko: false,
                is_stunned: false,
                is_commander: false,
                skills: None,
                ultimate: None,
                toughness: None,
                energy: None,
                tracker: None,
                counterplay: None,
            },
        ],
        UnitId(1),
        UnitId(3),
        CombatPhase::WaitingAction,
        Some(UnitId(1)),
        5,
    );

    let affordance =
        action_from_snapshot(&snapshot, &skill_book, UnitId(1), ActionQueryKind::Basic);

    assert!(matches!(affordance.action, ActionStatus::Enabled));
    assert_eq!(enabled_target_ids(&affordance), vec![UnitId(3)]);
    assert_eq!(first_enabled_target_id(&affordance), Some(UnitId(3)));
    assert!(
        affordance
            .targets
            .iter()
            .any(|(id, target)| *id == UnitId(2)
                && matches!(
                    target.status,
                    TargetStatus::Disabled {
                        reason: LegalityReasonCode::WrongSide
                    }
                ))
    );
}

#[test]
fn deferred_actions_are_excluded_from_enabled_selection() {
    let basic = basic_skill("basic_attack");
    let deferred = deferred_skill("deferred_skill");
    let skill_book = SkillBook(vec![basic.clone(), deferred.clone()]);
    let snapshot = snapshot_from_fixtures(
        &[
            Fixture {
                id: UnitId(1),
                team: Team::Ally,
                hp_current: 40,
                hp_max: 40,
                is_ko: false,
                is_stunned: false,
                is_commander: false,
                skills: Some(unit_skills(
                    "basic_attack",
                    vec!["deferred_skill"],
                    "basic_attack",
                )),
                ultimate: Some(UltimateCharge {
                    current: 120,
                    trigger: 100,
                    cap: 150,
                    trigger_type:
                        bevyrogue::combat::ultimate::UltAccumulationTrigger::OnBasicAttack,
                    charge_per_event: 25,
                }),
                toughness: None,
                energy: None,
                tracker: None,
                counterplay: None,
            },
            Fixture {
                id: UnitId(2),
                team: Team::Enemy,
                hp_current: 25,
                hp_max: 40,
                is_ko: false,
                is_stunned: false,
                is_commander: false,
                skills: None,
                ultimate: None,
                toughness: None,
                energy: None,
                tracker: None,
                counterplay: None,
            },
        ],
        UnitId(1),
        UnitId(2),
        CombatPhase::WaitingAction,
        Some(UnitId(1)),
        5,
    );

    let basic_affordance =
        action_from_snapshot(&snapshot, &skill_book, UnitId(1), ActionQueryKind::Basic);
    let deferred_affordance = action_from_snapshot(
        &snapshot,
        &skill_book,
        UnitId(1),
        ActionQueryKind::Skill(&deferred.id),
    );

    let enabled_labels: Vec<&str> = [
        ("Basic", &basic_affordance),
        ("Deferred", &deferred_affordance),
    ]
    .into_iter()
    .filter_map(|(label, affordance)| {
        first_enabled_target_id(affordance)
            .is_some()
            .then_some(label)
    })
    .collect();

    assert_eq!(enabled_labels, vec!["Basic"]);
    assert!(matches!(
        deferred_affordance.action,
        ActionStatus::Disabled {
            reason: LegalityReasonCode::NoValidTargets
        }
    ));
    assert!(enabled_target_ids(&deferred_affordance).is_empty());
    assert!(first_enabled_target_id(&deferred_affordance).is_none());
    assert!(matches!(
        deferred_affordance.target,
        TargetStatus::Disabled {
            reason: LegalityReasonCode::NoValidTargets
        }
    ));
    assert!(matches!(
        deferred_affordance.implementation,
        bevyrogue::combat::action_query::ImplementationStatus::Deferred {
            reason: LegalityReasonCode::UnimplementedTargetShape
        }
    ));
}

/// Reads and concatenates every `.rs` file at `path`, recursing if it is a
/// directory. Used by the source-hardcoding guards so they cover the whole
/// module tree (the modules were split into directory modules), not just one
/// file — splitting a file must never weaken these guards.
fn read_module_source(path: &str) -> String {
    fn collect(path: &std::path::Path, out: &mut String) {
        if path.is_dir() {
            let mut entries: Vec<_> = std::fs::read_dir(path)
                .unwrap_or_else(|e| panic!("cannot read dir {}: {e}", path.display()))
                .map(|e| e.expect("dir entry").path())
                .collect();
            entries.sort();
            for entry in entries {
                collect(&entry, out);
            }
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push_str(
                &std::fs::read_to_string(path)
                    .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display())),
            );
            out.push('\n');
        }
    }

    let root = std::path::Path::new(path);
    let mut out = String::new();
    // A module may be either `foo.rs` or the directory `foo/`; scan whichever exists.
    let file = root.with_extension("rs");
    if file.is_file() {
        collect(&file, &mut out);
    }
    if root.is_dir() {
        collect(root, &mut out);
    }
    assert!(
        !out.is_empty(),
        "no source found for module at {path} (.rs or directory)"
    );
    out
}

#[test]
fn combat_cli_source_does_not_reintroduce_ko_or_skill_id_hardcoding() {
    let source = read_module_source("src/bin/combat_cli");

    assert!(!source.contains("patamon_revive"));
    assert!(!source.contains("ko.is_none()"));
}

#[test]
fn combat_windowed_source_does_not_reintroduce_ko_or_skill_id_hardcoding() {
    let source = read_module_source("src/ui/combat_panel");

    assert!(!source.contains("patamon_revive"));
    assert!(!source.contains("ko.is_none()"));
    // counterplay declarations must not be decided by name or free-text traits
    assert!(
        !source.contains("\"devimon\""),
        "combat_panel must not hardcode enemy name 'devimon'"
    );
    assert!(
        !source.contains("\"ogremon\""),
        "combat_panel must not hardcode enemy name 'ogremon'"
    );
    assert!(
        !source.contains("signature_traits"),
        "combat_panel must not branch on signature_traits"
    );
}

#[test]
fn combat_cli_source_does_not_hardcode_counterplay_names() {
    let source = read_module_source("src/bin/combat_cli");

    assert!(
        !source.contains("\"devimon\""),
        "combat_cli must not hardcode enemy name 'devimon'"
    );
    assert!(
        !source.contains("\"ogremon\""),
        "combat_cli must not hardcode enemy name 'ogremon'"
    );
    assert!(
        !source.contains("signature_traits"),
        "combat_cli must not branch on signature_traits"
    );
}
