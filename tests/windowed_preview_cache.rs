#![cfg(feature = "windowed")]

use bevy::prelude::*;
use bevyrogue::combat::{
    kit::UnitSkills,
    preview::{query_skill_preview, summarize_preview_damage},
    runtime::{CastIdGen, ExtRegistries, register_kernel_builtins},
    sp::SpPool,
    state::CombatState,
    team::Team,
    turn_order::TurnOrder,
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
    ultimate::{UltAccumulationTrigger, UltimateCharge},
    unit::Unit,
};
use bevyrogue::data::{
    SkillBookHandle,
    skill_timeline::SkillTimeline,
    skills_ron::{
        SelfTargetRule, SkillBook, SkillDef, SkillImplementation, SkillTargeting, TargetLife,
        TargetShape, TargetSide,
    },
};
use bevyrogue::ui::combat_panel::{
    PendingAction, PendingKind, PreviewDamageCache, refresh_preview_damage_cache,
};

const CASTER_ID: UnitId = UnitId(11);
const TARGET_ID: UnitId = UnitId(22);

fn preview_skill_id() -> SkillId {
    SkillId("preview_bridge_strike".into())
}

fn build_app() -> App {
    let mut app = App::new();
    app.init_resource::<CastIdGen>()
        .init_resource::<ExtRegistries>()
        .init_resource::<Assets<SkillBook>>()
        .init_resource::<TurnOrder>()
        .init_resource::<CombatState>()
        .init_resource::<SpPool>()
        .insert_resource(PendingAction {
            kind: Some(PendingKind::Skill(preview_skill_id())),
        })
        .insert_resource(PreviewDamageCache::default());

    let mut regs = ExtRegistries::default();
    register_kernel_builtins(&mut regs);
    app.insert_resource(regs);

    let book = SkillBook(vec![SkillDef {
        id: preview_skill_id(),
        name: "Preview Bridge Strike".into(),
        damage_tag: DamageTag::Fire,
        sp_cost: 0,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            target_hp_rule: Default::default(),
        },
        implementation: SkillImplementation::Implemented,
        legacy_ops: vec![],
        custom_signals: vec![],
        animation_sequence: None,
        qte: None,
        timeline: Some(SkillTimeline {
            entry: "cast".into(),
            beats: vec![
                bevyrogue::combat::runtime::timeline::Beat {
                    id: "cast".into(),
                    kind: bevyrogue::combat::runtime::timeline::BeatKind::Cast,
                    hook: None,
                    selector: None,
                    presentation: None,
                    payload: None,
                },
                bevyrogue::combat::runtime::timeline::Beat {
                    id: "impact_1".into(),
                    kind: bevyrogue::combat::runtime::timeline::BeatKind::Impact,
                    hook: Some("core/deal_damage".into()),
                    selector: Some("core/primary".into()),
                    presentation: None,
                    payload: Some(
                        bevyrogue::combat::runtime::timeline::BeatPayload::DealDamage {
                            amount: 11,
                            tag: DamageTag::Fire,
                            target: TargetShape::Single,
                        },
                    ),
                },
                bevyrogue::combat::runtime::timeline::Beat {
                    id: "impact_2".into(),
                    kind: bevyrogue::combat::runtime::timeline::BeatKind::Impact,
                    hook: Some("core/deal_damage".into()),
                    selector: Some("core/primary".into()),
                    presentation: None,
                    payload: Some(
                        bevyrogue::combat::runtime::timeline::BeatPayload::DealDamage {
                            amount: 13,
                            tag: DamageTag::Fire,
                            target: TargetShape::Single,
                        },
                    ),
                },
            ],
            edges: vec![
                bevyrogue::combat::runtime::timeline::BeatEdge {
                    from: "cast".into(),
                    to: "impact_1".into(),
                    gate: Some("core/always".into()),
                },
                bevyrogue::combat::runtime::timeline::BeatEdge {
                    from: "impact_1".into(),
                    to: "impact_2".into(),
                    gate: Some("core/always".into()),
                },
            ],
        }),
    }]);
    let handle = {
        let mut assets = app.world_mut().resource_mut::<Assets<SkillBook>>();
        assets.add(book)
    };
    app.insert_resource(SkillBookHandle(handle));

    app.world_mut().spawn((
        Unit {
            id: CASTER_ID,
            name: "caster".into(),
            hp_max: 100,
            hp_current: 100,
            attribute: Attribute::Vaccine,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Ally,
        UnitSkills {
            basic: SkillId("caster_basic".into()),
            skills: vec![preview_skill_id()],
            ultimate: SkillId("caster_ult".into()),
            follow_up: None,
        },
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
    ));
    app.world_mut().spawn((
        Unit {
            id: TARGET_ID,
            name: "target".into(),
            hp_max: 120,
            hp_current: 88,
            attribute: Attribute::Data,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        Team::Enemy,
        UnitSkills {
            basic: SkillId("target_basic".into()),
            skills: vec![SkillId("target_skill".into())],
            ultimate: SkillId("target_ult".into()),
            follow_up: None,
        },
        UltimateCharge {
            current: 0,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        },
    ));

    {
        let mut order = app.world_mut().resource_mut::<TurnOrder>();
        order.active_unit = Some(CASTER_ID);
    }

    app.add_systems(Update, refresh_preview_damage_cache);
    app
}

#[test]
fn windowed_preview_cache_tracks_shared_preview_summary_and_stays_put_without_preview() {
    let mut app = build_app();

    app.update();

    let cached = app.world().resource::<PreviewDamageCache>().clone();
    assert_eq!(cached.actor_id, Some(CASTER_ID));
    assert_eq!(
        cached.pending_kind,
        Some(PendingKind::Skill(preview_skill_id()))
    );
    assert_eq!(cached.target_id, Some(TARGET_ID));
    let summary = cached.summary.expect("preview summary");
    assert_eq!(summary.total_damage, 24);
    assert_eq!(summary.deal_damage_intents, 2);

    let cast_id = {
        let mut cast_id_gen = app.world_mut().resource_mut::<CastIdGen>();
        cast_id_gen.next()
    };
    let direct_preview = query_skill_preview(
        app.world_mut(),
        &preview_skill_id(),
        cast_id,
        CASTER_ID,
        TARGET_ID,
    );
    assert_eq!(
        summarize_preview_damage(&direct_preview),
        summary,
        "windowed cache must mirror the shared preview stream"
    );

    app.world_mut().resource_mut::<PendingAction>().kind = None;
    app.update();

    assert_eq!(
        app.world().resource::<PreviewDamageCache>().clone(),
        cached,
        "cache must stay stable when no preview can be refreshed"
    );
}
