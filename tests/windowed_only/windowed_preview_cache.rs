#![cfg(feature = "windowed")]

use bevy::prelude::*;
use bevyrogue::combat::{
    events::{CombatEvent, CombatEventKind, CombatKernelTransition},
    kit::UnitSkills,
    preview::{query_skill_preview, summarize_preview_damage},
    runtime::{
        CastId, CastIdGen, CueBarrierStatus, ExtRegistries, SignalPayload, register_kernel_builtins,
    },
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
    BABY_BURNER_FLASH_LIFETIME_FRAMES, BabyBurnerFlashState, PendingAction, PendingKind,
    PreviewDamageCache, advance_baby_burner_flash_state, baby_burner_flash_chip, cue_barrier_chip,
    observe_baby_burner_flash, refresh_preview_damage_cache, telegraph_chip_text,
    telegraph_chip_tooltip,
};

const CASTER_ID: UnitId = UnitId(11);
const TARGET_ID: UnitId = UnitId(22);

fn preview_skill_id() -> SkillId {
    SkillId("preview_bridge_strike".into())
}

fn sharp_claws_skill_id() -> SkillId {
    SkillId("sharp_claws".into())
}

fn build_app(pending_kind: PendingKind, basic_skill: SkillId) -> App {
    let mut app = App::new();
    app.init_resource::<CastIdGen>()
        .init_resource::<ExtRegistries>()
        .init_resource::<Assets<SkillBook>>()
        .init_resource::<TurnOrder>()
        .init_resource::<CombatState>()
        .init_resource::<SpPool>()
        .insert_resource(PendingAction {
            kind: Some(pending_kind),
        })
        .insert_resource(PreviewDamageCache::default())
        .init_resource::<BabyBurnerFlashState>()
        .add_message::<CombatEvent>();

    let mut regs = ExtRegistries::default();
    register_kernel_builtins(&mut regs);
    app.insert_resource(regs);

    let book = SkillBook(vec![
        SkillDef {
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
        },
        SkillDef {
            id: sharp_claws_skill_id(),
            name: "Sharp Claws".into(),
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
                        id: "impact_damage".into(),
                        kind: bevyrogue::combat::runtime::timeline::BeatKind::Impact,
                        hook: Some("core/deal_damage".into()),
                        selector: Some("core/primary".into()),
                        presentation: None,
                        payload: Some(
                            bevyrogue::combat::runtime::timeline::BeatPayload::DealDamage {
                                amount: 18,
                                tag: DamageTag::Fire,
                                target: TargetShape::Single,
                            },
                        ),
                    },
                ],
                edges: vec![bevyrogue::combat::runtime::timeline::BeatEdge {
                    from: "cast".into(),
                    to: "impact_damage".into(),
                    gate: Some("core/always".into()),
                }],
            }),
        },
    ]);
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
            basic: basic_skill,
            skills: vec![preview_skill_id(), sharp_claws_skill_id()],
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

    app.add_systems(
        Update,
        (
            advance_baby_burner_flash_state,
            refresh_preview_damage_cache,
            observe_baby_burner_flash,
        )
            .chain(),
    );
    app
}

fn unit_hp(app: &mut App, id: UnitId) -> i32 {
    let mut query = app.world_mut().query::<&Unit>();
    query
        .iter(app.world())
        .find(|unit| unit.id == id)
        .map(|unit| unit.hp_current)
        .unwrap_or_else(|| panic!("unit {id:?} missing"))
}

#[test]
fn windowed_preview_cache_tracks_shared_preview_summary_and_stays_put_without_preview() {
    let mut app = build_app(
        PendingKind::Skill(preview_skill_id()),
        SkillId("caster_basic".into()),
    );

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

#[test]
fn windowed_basic_preview_uses_sharp_claws_damage_data() {
    let mut app = build_app(PendingKind::Basic, sharp_claws_skill_id());

    app.update();

    let cached = app.world().resource::<PreviewDamageCache>().clone();
    assert_eq!(cached.pending_kind, Some(PendingKind::Basic));
    assert_eq!(cached.skill_id, Some(sharp_claws_skill_id()));
    assert_eq!(cached.target_id, Some(TARGET_ID));

    let summary = cached.summary.expect("basic preview summary");
    assert_eq!(summary.total_damage, 18);
    assert_eq!(summary.deal_damage_intents, 1);
    assert_eq!(
        cached.summary.as_ref().map(|summary| format!(
            "preview: {} dmg across {} hit(s)",
            summary.total_damage, summary.deal_damage_intents
        )),
        Some("preview: 18 dmg across 1 hit(s)".to_string())
    );
}

#[test]
fn baby_burner_flash_state_projects_detonate_transitions_for_fixed_frames_without_touching_hp() {
    let mut app = build_app(
        PendingKind::Skill(preview_skill_id()),
        SkillId("caster_basic".into()),
    );
    let combat_before = app.world().resource::<CombatState>().clone();
    let target_hp_before = unit_hp(&mut app, TARGET_ID);
    let cast_id = {
        let mut cast_id_gen = app.world_mut().resource_mut::<CastIdGen>();
        cast_id_gen.next()
    };

    app.world_mut().write_message(CombatEvent {
        kind: CombatEventKind::OnKernelTransition {
            transition: CombatKernelTransition::Blueprint {
                owner: "agumon".into(),
                name: "baby_burner_detonate".into(),
                payload: SignalPayload::UnitTarget(TARGET_ID),
            },
        },
        source: CASTER_ID,
        target: TARGET_ID,
        follow_up_depth: 1,
        cast_id,
    });
    app.world_mut().write_message(CombatEvent {
        kind: CombatEventKind::OnKernelTransition {
            transition: CombatKernelTransition::Blueprint {
                owner: "agumon".into(),
                name: "baby_burner_detonate".into(),
                payload: SignalPayload::UnitTarget(UnitId(23)),
            },
        },
        source: CASTER_ID,
        target: UnitId(23),
        follow_up_depth: 1,
        cast_id,
    });

    app.update();

    let flash = app.world().resource::<BabyBurnerFlashState>().clone();
    let active = flash
        .active
        .expect("flash should be visible after detonate transition");
    assert_eq!(active.source, CASTER_ID);
    assert_eq!(active.cast_id, cast_id);
    assert_eq!(active.targets, vec![TARGET_ID, UnitId(23)]);
    assert_eq!(active.remaining_frames, BABY_BURNER_FLASH_LIFETIME_FRAMES);
    assert_eq!(active.total_frames, BABY_BURNER_FLASH_LIFETIME_FRAMES);

    let chip = baby_burner_flash_chip(Some(&active)).expect("flash chip");
    assert!(chip.label.contains("2 target(s)"));
    assert!(chip.tooltip.contains("signal=agumon/baby_burner_detonate"));
    assert!(chip.tooltip.contains("targets=[UnitId(22), UnitId(23)]"));

    assert_eq!(
        *app.world().resource::<CombatState>(),
        combat_before,
        "windowed flash projection must not mutate combat state"
    );
    let target_hp_after_first_update = unit_hp(&mut app, TARGET_ID);
    assert_eq!(target_hp_after_first_update, target_hp_before);

    for expected_remaining in (1..BABY_BURNER_FLASH_LIFETIME_FRAMES).rev() {
        app.update();
        let active = app
            .world()
            .resource::<BabyBurnerFlashState>()
            .active
            .clone()
            .expect("flash should stay visible until the final frame expires");
        assert_eq!(active.remaining_frames, expected_remaining);
    }

    app.update();
    assert!(
        app.world()
            .resource::<BabyBurnerFlashState>()
            .active()
            .is_none(),
        "flash must hide once the deterministic frame budget expires"
    );
    let target_hp_after_expiry = unit_hp(&mut app, TARGET_ID);
    assert_eq!(target_hp_after_expiry, target_hp_before);
}

#[test]
fn telegraph_chip_helpers_surface_and_hide_sharp_claws_barriers_without_egui() {
    let book = SkillBook(vec![SkillDef {
        id: sharp_claws_skill_id(),
        name: "Sharp Claws".into(),
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
        timeline: None,
    }]);

    let awaiting = CueBarrierStatus {
        cast_id: CastId::ROOT,
        skill_id: sharp_claws_skill_id(),
        timeline_id: "sharp_claws",
        beat_id: "impact_damage",
        cue_id: "agumon/sharp_claws/impact",
        awaiting_release: true,
        released: false,
        timed_out: false,
        waited_frames: 0,
        timeout_frames: 180,
        animation_node: Some("sharp_claws_strike".into()),
        animation_frame: Some(4),
        hop_index: None,
    };

    assert_eq!(
        telegraph_chip_text("Sharp Claws"),
        "Telegraph: Sharp Claws · impact pending"
    );
    assert!(
        telegraph_chip_tooltip(&awaiting).contains("cue=agumon/sharp_claws/impact"),
        "tooltip should expose the cue id for stuck-barrier diagnosis"
    );

    let chip = cue_barrier_chip(Some(&awaiting), Some(&book)).expect("awaiting chip");
    assert_eq!(chip.label, "Telegraph: Sharp Claws · impact pending");
    assert!(chip.tooltip.contains("node=sharp_claws_strike"));
    assert!(chip.tooltip.contains("frame=4"));

    let released = CueBarrierStatus {
        awaiting_release: false,
        released: true,
        ..awaiting.clone()
    };
    assert!(
        cue_barrier_chip(Some(&released), Some(&book)).is_none(),
        "chip must hide after release/resolution"
    );
    assert!(cue_barrier_chip(None, Some(&book)).is_none());
}
