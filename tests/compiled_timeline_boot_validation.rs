use std::collections::HashSet;

use bevy::prelude::*;
use bevyrogue::{
    combat::types::SkillId,
    combat::{
        api::{ExtRegistries, TimelineLibrary, register_kernel_builtins},
        blueprints::register_all_blueprint_exts,
        plugin::CombatPlugin,
    },
    data::{
        skill_timeline::compile_skill_book_timelines,
        skills_ron::{SkillBook, validate_skill_book},
    },
};

fn canonical_regs() -> ExtRegistries {
    let mut regs = ExtRegistries::default();
    register_kernel_builtins(&mut regs);
    register_all_blueprint_exts(&mut regs);
    regs
}

fn canonical_book() -> SkillBook {
    bevyrogue::data::aggregate_skill_book()
}

#[test]
fn canonical_asset_compiles_timeline_backed_skills_into_library_shape() {
    let book = canonical_book();
    validate_skill_book(&book).expect("canonical skills.ron must validate");

    let compiled = compile_skill_book_timelines(&book, &canonical_regs())
        .expect("timeline-backed skills must compile");

    let ids: HashSet<_> = compiled
        .iter()
        .map(|timeline| timeline.id.as_str())
        .collect();
    assert_eq!(
        ids.len(),
        15,
        "expected 15 timeline-backed canon skills after child-roster migration"
    );

    // child basic/active skills
    for required in [
        "baby_flame",
        "bubble_blast",
        "draconic_edge",
        "diamond_storm",
        "holy_breeze",
        "tentomon_basic",
        "patamon_revive",
    ] {
        assert!(
            ids.contains(required),
            "missing child-basic timeline: {required}"
        );
    }

    // child follow-up skills
    for required in [
        "agumon_follow_up",
        "gabumon_follow_up",
        "dorumon_follow_up",
        "renamon_follow_up",
        "patamon_follow_up",
        "tentomon_follow_up",
    ] {
        assert!(
            ids.contains(required),
            "missing child-follow-up timeline: {required}"
        );
    }

    // previously migrated
    assert!(ids.contains("petit_thunder"));
    assert!(ids.contains("renamon_ult"));

    let petit = compiled
        .iter()
        .find(|timeline| timeline.id == "petit_thunder")
        .expect("petit_thunder timeline present");
    assert_eq!(petit.entry, "cast");
    assert_eq!(petit.beats.len(), 5);
    assert_eq!(petit.edges.len(), 4);
}

#[test]
fn asset_typo_in_hook_id_fails_with_skill_and_beat_site() {
    let bad_ron = bevyrogue::data::aggregate_skill_book_ron_text().replacen(
        "core/deal_damage",
        "core/deal_damge",
        1,
    );
    let book: SkillBook = ron::from_str(&bad_ron).expect("parse tweaked skills.ron");

    let err = compile_skill_book_timelines(&book, &canonical_regs())
        .expect_err("hook typo must fail before runtime");

    // baby_flame is now the first skill with a core/deal_damage beat (child-roster migration)
    assert_eq!(err.skill_id, SkillId("baby_flame".into()));
    assert_eq!(err.site, "beat impact_damage");
    assert!(err.detail.contains("core/deal_damge"));
}

#[test]
fn invalid_timeline_refs_report_hook_and_predicate_sites() {
    let timeline: bevyrogue::combat::api::timeline::CompiledTimeline<String> =
        bevyrogue::combat::api::timeline::CompiledTimeline {
        id: "bad_boot_timeline".into(),
        entry: "cast".into(),
        beats: vec![bevyrogue::combat::api::timeline::Beat {
            id: "cast".into(),
            kind: bevyrogue::combat::api::timeline::BeatKind::Cast,
            hook: Some("missing_boot_hook".into()),
            selector: None,
            presentation: None,
            payload: None,
        }],
        edges: vec![bevyrogue::combat::api::timeline::BeatEdge {
            from: "cast".into(),
            to: "impact".into(),
            gate: Some("missing_boot_pred".into()),
        }],
    };

    let errs = bevyrogue::combat::api::timeline::validate_timeline_refs(&timeline, &canonical_regs())
        .expect_err("invalid timeline refs must fail validation");

    assert!(errs.iter().any(|err| {
        err.axis == "hook" && err.missing_id == "missing_boot_hook" && err.site == "beat cast"
    }));
    assert!(errs.iter().any(|err| {
        err.axis == "predicate"
            && err.missing_id == "missing_boot_pred"
            && err.site == "edge cast->impact"
    }));
}

#[test]
#[should_panic(expected = "CombatPlugin::finish — dangling timeline references")]
fn invalid_timeline_ids_fail_during_app_finish() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(CombatPlugin);

    app.world_mut().resource_mut::<TimelineLibrary<String>>().timelines.push(
        bevyrogue::combat::api::timeline::CompiledTimeline {
            id: "bad_boot_timeline".into(),
            entry: "cast".into(),
            beats: vec![bevyrogue::combat::api::timeline::Beat {
                id: "cast".into(),
                kind: bevyrogue::combat::api::timeline::BeatKind::Cast,
                hook: Some("missing_boot_hook".into()),
                selector: None,
                presentation: None,
                payload: None,
            }],
            edges: vec![bevyrogue::combat::api::timeline::BeatEdge {
                from: "cast".into(),
                to: "impact".into(),
                gate: Some("missing_boot_pred".into()),
            }],
        },
    );

    app.finish();
}
