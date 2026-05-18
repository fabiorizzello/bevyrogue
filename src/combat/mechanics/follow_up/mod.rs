mod form_identity;
mod resolve;
mod triggers;
mod types;

pub use form_identity::form_identity_listener_system;
pub use resolve::resolve_follow_up_action_system;
pub use triggers::follow_up_listener_system;
pub use types::{
    FollowUpDecision, FollowUpIntent, FollowUpOriginKind, FollowUpSkipReason, FollowUpTrace,
};

#[cfg(test)]
mod tests {
    use super::triggers::{FollowerSnapshot, evaluate_follow_up};
    use super::types::*;
    use bevy::{
        ecs::message::MessageCursor,
        prelude::{App, Entity, IntoScheduleConfigs, Messages, Update},
    };

    use crate::combat::runtime::intent::CastId;
    use crate::combat::runtime::timeline::{Beat, BeatEdge, BeatKind, BeatPayload, TimelineLibrary};
    use crate::combat::runtime::{ExtRegistries, SignalBus, SignalTaxonomy, register_kernel_builtins};
    use crate::combat::rng::CombatRng;
    use crate::combat::{
        events::{CombatEvent, CombatEventKind},
        kit::{FollowUpConfig, FollowUpTrigger, UnitSkills},
        log::{ActionLog, LogEntry},
        sp::SpPool,
        state::CombatState,
        team::Team,
        toughness::Toughness,
        turn_order::TurnOrder,
        turn_system::{resolve_action_system, ActionIntent},
        types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
        ultimate::{UltAccumulationTrigger, UltimateCharge},
        unit::Unit,
    };
    use crate::data::skill_timeline::SkillTimeline;
    use crate::data::{
        SkillBookHandle,
        skill_timeline::compile_skill_book_timelines,
        skills_ron::{
            Effect, SelfTargetRule, SkillBook, SkillDef, SkillImplementation, SkillTargeting,
            TargetLife, TargetShape, TargetSide,
        },
    };

    fn unit(id: u32, attribute: Attribute, hp_max: i32, hp_current: i32) -> Unit {
        Unit {
            id: UnitId(id),
            name: format!("Unit{id}"),
            hp_max,
            hp_current,
            attribute,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        }
    }

    fn skill(id: &str, damage_tag: DamageTag, damage: i32, toughness_damage: i32) -> SkillDef {
        SkillDef {
            id: SkillId(id.into()),
            name: id.into(),
            damage_tag,
            sp_cost: 0,
            targeting: SkillTargeting {
                shape: TargetShape::Single,
                side: TargetSide::Enemy,
                life: TargetLife::Alive,
                self_rule: SelfTargetRule::Forbid,
                ..Default::default()
            },
            implementation: SkillImplementation::Implemented,
            legacy_ops: vec![
                Effect::Damage {
                    amount: damage,
                    target: TargetShape::Single,
                    per_hop: Default::default(),
                },
                Effect::ToughnessHit(toughness_damage),
            ],
            custom_signals: vec![],
            animation_sequence: None,
            qte: None,
            timeline: Some(SkillTimeline {
                entry: "cast".into(),
                beats: vec![
                    Beat {
                        id: "cast".into(),
                        kind: BeatKind::Cast,
                        hook: None,
                        selector: None,
                        presentation: None,
                        payload: None,
                    },
                    Beat {
                        id: "impact_damage".into(),
                        kind: BeatKind::Impact,
                        hook: Some("core/deal_damage".into()),
                        selector: Some("core/primary".into()),
                        presentation: None,
                        payload: Some(BeatPayload::DealDamage {
                            amount: damage,
                            tag: damage_tag,
                            target: TargetShape::Single,
                        }),
                    },
                    Beat {
                        id: "impact_break".into(),
                        kind: BeatKind::Impact,
                        hook: Some("core/apply_effect".into()),
                        selector: Some("core/primary".into()),
                        presentation: None,
                        payload: Some(BeatPayload::BreakToughness {
                            amount: toughness_damage,
                            tag: damage_tag,
                            target: TargetShape::Single,
                        }),
                    },
                ],
                edges: vec![
                    BeatEdge {
                        from: "cast".into(),
                        to: "impact_damage".into(),
                        gate: Some("core/always".into()),
                    },
                    BeatEdge {
                        from: "impact_damage".into(),
                        to: "impact_break".into(),
                        gate: Some("core/always".into()),
                    },
                ],
            }),
        }
    }

    fn cursor(app: &mut App) -> MessageCursor<CombatEvent> {
        app.world_mut()
            .resource_mut::<Messages<CombatEvent>>()
            .get_cursor()
    }

    fn drain(cursor: &mut MessageCursor<CombatEvent>, app: &App) -> Vec<CombatEvent> {
        cursor
            .read(app.world().resource::<Messages<CombatEvent>>())
            .cloned()
            .collect()
    }

    fn setup_app(book: SkillBook) -> App {
        let mut app = App::new();
        app.init_resource::<CombatState>()
            .init_resource::<TurnOrder>()
            .init_resource::<SpPool>()
            .init_resource::<ActionLog>()
            .init_resource::<bevy::prelude::Time>()
            .insert_resource(CombatRng::from_seed(42))
            .insert_resource(TimelineLibrary::<String>::default())
            .init_resource::<SignalBus>()
            .init_resource::<ExtRegistries>()
            .init_resource::<SignalTaxonomy>()
            .add_message::<ActionIntent>()
            .add_message::<FollowUpIntent>()
            .add_message::<FollowUpTrace>()
            .add_message::<CombatEvent>()
            .add_systems(
                Update,
                (
                    resolve_action_system,
                    super::follow_up_listener_system,
                    super::resolve_follow_up_action_system,
                )
                    .chain(),
            );

        let mut assets = bevy::prelude::Assets::<SkillBook>::default();
        let handle = assets.add(book.clone());
        app.insert_resource(assets);
        app.insert_resource(SkillBookHandle(handle));

        {
            let mut regs = app.world_mut().resource_mut::<ExtRegistries>();
            register_kernel_builtins(&mut regs);
            let compiled = compile_skill_book_timelines(&book, &regs)
                .expect("test timeline book must compile");
            app.world_mut()
                .resource_mut::<TimelineLibrary<String>>()
                .timelines = compiled;
        }

        app
    }

    fn spawn_combatant(
        app: &mut App,
        unit: Unit,
        team: Team,
        toughness_max: i32,
        weaknesses: Vec<DamageTag>,
        skills: UnitSkills,
    ) -> Entity {
        app.world_mut()
            .spawn((
                unit,
                team,
                Toughness::new(toughness_max, weaknesses),
                UltimateCharge {
                    current: 0,
                    trigger: 100,
                    cap: 150,
                    trigger_type: UltAccumulationTrigger::OnBasicAttack,
                    charge_per_event: 25,
                },
                skills,
            ))
            .id()
    }

    #[test]
    fn follow_up_break_event_resolves_same_update() {
        let mut app = setup_app(SkillBook(vec![
            skill("breaker", DamageTag::Fire, 8, 10),
            skill("ally_follow_up", DamageTag::Light, 6, 3),
            skill("enemy_basic", DamageTag::Ice, 4, 0),
        ]));

        spawn_combatant(
            &mut app,
            unit(1, Attribute::Vaccine, 100, 100),
            Team::Ally,
            40,
            vec![],
            UnitSkills {
                basic: SkillId("breaker".into()),
                skills: vec![SkillId("breaker".into())],
                ultimate: SkillId("breaker".into()),
                follow_up: None,
            },
        );
        spawn_combatant(
            &mut app,
            unit(2, Attribute::Data, 90, 90),
            Team::Ally,
            35,
            vec![],
            UnitSkills {
                basic: SkillId("ally_follow_up".into()),
                skills: vec![SkillId("ally_follow_up".into())],
                ultimate: SkillId("ally_follow_up".into()),
                follow_up: Some(FollowUpConfig {
                    trigger: FollowUpTrigger::OnEnemyBreak,
                    action: SkillId("ally_follow_up".into()),
                }),
            },
        );
        spawn_combatant(
            &mut app,
            unit(4, Attribute::Virus, 100, 100),
            Team::Enemy,
            5,
            vec![DamageTag::Fire],
            UnitSkills {
                basic: SkillId("enemy_basic".into()),
                skills: vec![SkillId("enemy_basic".into())],
                ultimate: SkillId("enemy_basic".into()),
                follow_up: None,
            },
        );

        let mut event_cursor = cursor(&mut app);
        app.world_mut().write_message(ActionIntent::Skill {
            attacker: UnitId(1),
            skill_id: SkillId("breaker".into()),
            target: UnitId(4),
        });

        app.update();

        let events = drain(&mut event_cursor, &app);
        assert!(events.iter().any(|event| {
            event.follow_up_depth == 1 && event.source == UnitId(2) && event.target == UnitId(4)
        }));

        let hits: Vec<(UnitId, UnitId)> = app
            .world()
            .resource::<ActionLog>()
            .events
            .iter()
            .filter_map(|entry| match entry {
                LogEntry::BasicHit {
                    attacker, target, ..
                } => Some((*attacker, *target)),
                _ => None,
            })
            .collect();
        assert!(hits.contains(&(UnitId(1), UnitId(4))));
        assert!(hits.contains(&(UnitId(2), UnitId(4))));
    }

    #[test]
    fn follow_up_reports_ineligible_reasons() {
        let roster = vec![
            FollowerSnapshot {
                id: UnitId(1),
                team: Team::Ally,
                hp_current: 100,
                follow_up: Some(FollowUpConfig {
                    trigger: FollowUpTrigger::OnEnemyBreak,
                    action: SkillId("follow_up".into()),
                }),
                is_ko: false,
                is_stunned: false,
            },
            FollowerSnapshot {
                id: UnitId(2),
                team: Team::Enemy,
                hp_current: 100,
                follow_up: Some(FollowUpConfig {
                    trigger: FollowUpTrigger::OnEnemyBreak,
                    action: SkillId("follow_up".into()),
                }),
                is_ko: false,
                is_stunned: false,
            },
            FollowerSnapshot {
                id: UnitId(3),
                team: Team::Ally,
                hp_current: 0,
                follow_up: Some(FollowUpConfig {
                    trigger: FollowUpTrigger::OnEnemyBreak,
                    action: SkillId("follow_up".into()),
                }),
                is_ko: true,
                is_stunned: false,
            },
            FollowerSnapshot {
                id: UnitId(4),
                team: Team::Ally,
                hp_current: 100,
                follow_up: Some(FollowUpConfig {
                    trigger: FollowUpTrigger::OnEnemyBreak,
                    action: SkillId("follow_up".into()),
                }),
                is_ko: false,
                is_stunned: true,
            },
            FollowerSnapshot {
                id: UnitId(5),
                team: Team::Ally,
                hp_current: 100,
                follow_up: Some(FollowUpConfig {
                    trigger: FollowUpTrigger::OnEnemyKill,
                    action: SkillId("follow_up".into()),
                }),
                is_ko: false,
                is_stunned: false,
            },
            FollowerSnapshot {
                id: UnitId(6),
                team: Team::Enemy,
                hp_current: 100,
                follow_up: None,
                is_ko: false,
                is_stunned: false,
            },
        ];

        let root_break = CombatEvent {
            kind: CombatEventKind::OnBreak {
                damage_tag: DamageTag::Fire,
            },
            source: UnitId(1),
            target: UnitId(6),
            follow_up_depth: 0,
            cast_id: CastId::ROOT,
        };

        assert_eq!(
            evaluate_follow_up(&roster[1], &root_break, &roster),
            Err(FollowUpSkipReason::WrongTeam)
        );
        assert_eq!(
            evaluate_follow_up(&roster[2], &root_break, &roster),
            Err(FollowUpSkipReason::FollowerKo)
        );
        assert_eq!(
            evaluate_follow_up(&roster[3], &root_break, &roster),
            Err(FollowUpSkipReason::FollowerStunned)
        );
        assert_eq!(
            evaluate_follow_up(&roster[4], &root_break, &roster),
            Err(FollowUpSkipReason::TriggerMismatch)
        );
    }
}
