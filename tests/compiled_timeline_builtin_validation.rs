use bevy::prelude::App;
use bevyrogue::combat::{
    api::{
        Beat, BeatEdge, BeatKind, BeatPayload, CompiledTimeline, ExtRegistries,
        Presentation, SelectorCtx, SkillCtx, SkillCtxMode, validate_timeline_refs,
    },
    plugin::CombatPlugin,
    types::{DamageTag, UnitId},
    api::{intent::{CastId, Intent}, builtins::register_kernel_builtins},
};
use std::{collections::{HashSet, VecDeque}, num::NonZeroU32};

fn owned_builtin_timeline(
    hook_id: &str,
    selector_id: &str,
    gate_id: &str,
) -> CompiledTimeline<String> {
    CompiledTimeline {
        id: "owned_builtin_timeline".to_string(),
        entry: "cast".to_string(),
        beats: vec![
            Beat {
                id: "cast".to_string(),
                kind: BeatKind::Cast,
                hook: None,
                selector: None,
                presentation: None,
                payload: None,
            },
            Beat {
                id: "impact".to_string(),
                kind: BeatKind::Impact,
                hook: Some(hook_id.to_string()),
                selector: Some(selector_id.to_string()),
                presentation: Some(Presentation {
                    cue_id: "impact_cue".to_string(),
                    anim: None,
                    vfx: None,
                    sfx: None,
                }),
                payload: Some(BeatPayload::DealDamage {
                    amount: 37,
                    tag: DamageTag::Physical,
                }),
            },
        ],
        edges: vec![BeatEdge {
            from: "cast".to_string(),
            to: "impact".to_string(),
            gate: Some(gate_id.to_string()),
        }],
    }
}

#[test]
fn combat_plugin_registers_builtins_and_owned_timeline_validates() {
    let mut app = App::new();
    app.add_plugins(CombatPlugin);
    app.finish();

    let regs = app.world().resource::<ExtRegistries>();
    assert!(regs.hooks.get("core/deal_damage").is_some());
    assert!(regs.selectors.get("core/primary").is_some());
    assert!(regs.predicates.get("core/always").is_some());
    assert!(regs.predicates.get("core/never").is_some());

    let timeline = owned_builtin_timeline("core/deal_damage", "core/primary", "core/always");
    assert!(validate_timeline_refs(&timeline, regs).is_ok());
}

#[test]
fn builtins_hook_selector_and_predicates_work_through_registry() {
    let mut regs = ExtRegistries::default();
    register_kernel_builtins(&mut regs);

    let selector = regs.selectors.get("core/primary").expect("builtin selector");
    let selected = selector(&SelectorCtx {
        caster: UnitId(11),
        primary_target: UnitId(22),
        state: &(),
    });
    assert_eq!(selected, vec![UnitId(22)]);

    let always = regs.predicates.get("core/always").expect("builtin predicate");
    let never = regs.predicates.get("core/never").expect("builtin predicate");
    let evt = bevyrogue::combat::api::BeatEvent {
        cast_id: CastId(NonZeroU32::new(8).unwrap()),
        beat_id: "impact",
        hop_index: 0,
        beat_targets: vec![UnitId(33)],
    };
    let world = bevy::prelude::World::new();
    let mut cast_hit_set = HashSet::new();
    let mut pending = VecDeque::new();
    let payload = BeatPayload::DealDamage {
        amount: 19,
        tag: DamageTag::Physical,
    };
    let mut ctx = SkillCtx::new(
        UnitId(1),
        UnitId(2),
        evt.cast_id,
        SkillCtxMode::Execute,
        &regs,
        &world,
        &mut cast_hit_set,
        &mut pending,
        Some(&payload),
    );

    assert!(always(&evt, &ctx));
    assert!(!never(&evt, &ctx));

    let hook = regs.hooks.get("core/deal_damage").expect("builtin hook");
    hook(&evt, &mut ctx);

    match pending.pop_front().expect("hook should enqueue an intent") {
        Intent::DealDamage { source, target, amount, .. } => {
            assert_eq!(source, UnitId(1));
            assert_eq!(target, UnitId(33));
            assert_eq!(amount, 19);
        }
        other => panic!("unexpected intent: {other:?}"),
    }
}

#[test]
fn builtin_hook_typos_report_precise_axis_and_site() {
    let regs = {
        let mut regs = ExtRegistries::default();
        register_kernel_builtins(&mut regs);
        regs
    };

    let timeline = owned_builtin_timeline("core/deal_damge", "core/primary", "core/always");
    let err = validate_timeline_refs(&timeline, &regs).unwrap_err();
    assert_eq!(err.len(), 1);
    assert_eq!(err[0].axis, "hook");
    assert_eq!(err[0].missing_id, "core/deal_damge");
    assert_eq!(err[0].site, "beat impact");
}

#[test]
fn builtin_selector_typos_report_precise_axis_and_site() {
    let regs = {
        let mut regs = ExtRegistries::default();
        register_kernel_builtins(&mut regs);
        regs
    };

    let timeline = owned_builtin_timeline("core/deal_damage", "core/priary", "core/always");
    let err = validate_timeline_refs(&timeline, &regs).unwrap_err();
    assert_eq!(err.len(), 1);
    assert_eq!(err[0].axis, "selector");
    assert_eq!(err[0].missing_id, "core/priary");
    assert_eq!(err[0].site, "beat impact");
}

#[test]
fn builtin_predicate_typos_report_precise_axis_and_site() {
    let regs = {
        let mut regs = ExtRegistries::default();
        register_kernel_builtins(&mut regs);
        regs
    };

    let timeline = owned_builtin_timeline("core/deal_damage", "core/primary", "core/alwys");
    let err = validate_timeline_refs(&timeline, &regs).unwrap_err();
    assert_eq!(err.len(), 1);
    assert_eq!(err[0].axis, "predicate");
    assert_eq!(err[0].missing_id, "core/alwys");
    assert_eq!(err[0].site, "edge cast->impact");
}
