use std::collections::VecDeque;
use std::sync::Arc;

use bevy::prelude::*;
use bevyrogue::combat::{
    runtime::{
        CastIdGen, ExtRegistries, Intent, IntentQueue, SkillCtxMode, StepOutcome,
        register_kernel_builtins,
        runner::BeatRunner,
        timeline::{Beat, BeatEdge, BeatKind, BeatPayload, CompiledTimeline, TimelineLibrary},
    },
    preview::query_skill_preview,
    team::Team,
    types::{Attribute, DamageTag, EvoStage, SkillId, UnitId},
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

const CASTER_ID: UnitId = UnitId(11);
const TARGET_ID: UnitId = UnitId(22);
fn skill_id() -> SkillId {
    SkillId("preview_strike".into())
}
const DAMAGE: i32 = 37;

fn build_app() -> App {
    let mut app = App::new();
    app.init_resource::<CastIdGen>()
        .init_resource::<IntentQueue>()
        .init_resource::<TimelineLibrary<String>>()
        .init_resource::<Assets<SkillBook>>()
        .add_message::<bevyrogue::combat::events::CombatEvent>();

    let mut regs = ExtRegistries::default();
    register_kernel_builtins(&mut regs);
    app.insert_resource(regs);

    let book = SkillBook(vec![SkillDef {
        id: skill_id(),
        name: "Preview Strike".into(),
        damage_tag: DamageTag::Fire,
        sp_cost: 0,
        targeting: SkillTargeting {
            shape: TargetShape::Single,
            side: TargetSide::Enemy,
            life: TargetLife::Alive,
            self_rule: SelfTargetRule::Forbid,
            ..Default::default()
        },
        implementation: SkillImplementation::Implemented,
        legacy_ops: vec![],
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
                    id: "impact".into(),
                    kind: BeatKind::Impact,
                    hook: Some("core/deal_damage".into()),
                    selector: Some("core/primary".into()),
                    presentation: None,
                    payload: Some(BeatPayload::DealDamage {
                        amount: DAMAGE,
                        tag: DamageTag::Fire,
                        target: TargetShape::Single,
                    }),
                },
            ],
            edges: vec![BeatEdge {
                from: "cast".into(),
                to: "impact".into(),
                gate: Some("core/always".into()),
            }],
        }),
    }]);

    let mut assets = Assets::<SkillBook>::default();
    let handle = assets.add(book.clone());
    app.insert_resource(assets);
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
    ));

    app
}

fn preview_timeline() -> Arc<CompiledTimeline> {
    Arc::new(CompiledTimeline {
        id: "preview_strike",
        entry: "cast",
        beats: vec![
            Beat {
                id: "cast",
                kind: BeatKind::Cast,
                hook: None,
                selector: None,
                presentation: None,
                payload: None,
            },
            Beat {
                id: "impact",
                kind: BeatKind::Impact,
                hook: Some("core/deal_damage"),
                selector: Some("core/primary"),
                presentation: None,
                payload: Some(BeatPayload::DealDamage {
                    amount: DAMAGE,
                    tag: DamageTag::Fire,
                    target: TargetShape::Single,
                }),
            },
        ],
        edges: vec![BeatEdge {
            from: "cast",
            to: "impact",
            gate: Some("core/always"),
        }],
    })
}

fn normalize(pending: &VecDeque<Intent>) -> Vec<String> {
    pending
        .iter()
        .map(|intent| match intent {
            Intent::DealDamage {
                source,
                target,
                amount,
                tag,
                ..
            } => {
                format!(
                    "DealDamage(src={:?},tgt={:?},amt={},tag={:?})",
                    source, target, amount, tag
                )
            }
            other => format!("{other:?}"),
        })
        .collect()
}

fn entity_for_unit(world: &mut World, unit_id: UnitId) -> Entity {
    let mut query = world.query::<(Entity, &Unit)>();
    query
        .iter(world)
        .find_map(|(entity, unit)| (unit.id == unit_id).then_some(entity))
        .expect("unit entity must exist")
}

#[test]
fn query_skill_preview_matches_execute_stream_and_leaves_world_unchanged() {
    let mut app = build_app();
    let cast_id = app.world_mut().resource_mut::<CastIdGen>().next();
    let target_entity = entity_for_unit(app.world_mut(), TARGET_ID);
    let before_hp = app
        .world()
        .get::<Unit>(target_entity)
        .expect("target unit")
        .hp_current;
    let before_queue_len = app.world().resource::<IntentQueue>().0.len();

    let preview_pending =
        query_skill_preview(app.world_mut(), &skill_id(), cast_id, CASTER_ID, TARGET_ID);

    assert_eq!(
        app.world()
            .get::<Unit>(target_entity)
            .expect("target unit")
            .hp_current,
        before_hp,
        "preview must not mutate target HP"
    );
    assert_eq!(
        app.world().resource::<IntentQueue>().0.len(),
        before_queue_len,
        "preview must not drain or populate the shared intent queue"
    );

    let regs_ptr: *const ExtRegistries = {
        let regs = app.world().resource::<ExtRegistries>();
        regs as *const _
    };
    let regs = unsafe { &*regs_ptr };
    let mut execute_pending = VecDeque::new();
    let mut runner = BeatRunner::new(preview_timeline(), cast_id, CASTER_ID, TARGET_ID);
    let outcome = runner.run_to_completion(
        app.world_mut(),
        regs,
        SkillCtxMode::Execute,
        &mut execute_pending,
        32,
    );
    assert_eq!(
        outcome,
        StepOutcome::Done,
        "execute-mode timeline must complete"
    );

    let preview_shape = normalize(&preview_pending);
    let execute_shape = normalize(&execute_pending);
    assert_eq!(
        preview_shape, execute_shape,
        "preview stream must match execute-mode intent shape"
    );
    assert_eq!(
        preview_shape,
        vec![format!(
            "DealDamage(src={:?},tgt={:?},amt={},tag={:?})",
            CASTER_ID,
            TARGET_ID,
            DAMAGE,
            DamageTag::Fire
        )],
        "preview stream must contain the authored damage intent"
    );
}
