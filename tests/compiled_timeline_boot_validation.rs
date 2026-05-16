use std::collections::HashSet;

use bevyrogue::{
    combat::{
        api::{ExtRegistries, register_kernel_builtins},
        blueprints::register_all_blueprint_exts,
    },
    combat::types::SkillId,
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
    ron::from_str(include_str!("../assets/data/skills.ron")).expect("parse canonical skills.ron")
}

#[test]
fn canonical_asset_compiles_timeline_backed_skills_into_library_shape() {
    let book = canonical_book();
    validate_skill_book(&book).expect("canonical skills.ron must validate");

    let compiled = compile_skill_book_timelines(&book, &canonical_regs())
        .expect("timeline-backed skills must compile");

    let ids: HashSet<_> = compiled.iter().map(|timeline| timeline.id.as_str()).collect();
    assert_eq!(ids.len(), 15, "expected 15 timeline-backed canon skills after child-roster migration");

    // child basic/active skills
    for required in [
        "baby_flame", "bubble_blast", "draconic_edge", "diamond_storm", "holy_breeze",
        "tentomon_basic", "patamon_revive",
    ] {
        assert!(ids.contains(required), "missing child-basic timeline: {required}");
    }

    // child follow-up skills
    for required in [
        "agumon_follow_up", "gabumon_follow_up", "dorumon_follow_up",
        "renamon_follow_up", "patamon_follow_up", "tentomon_follow_up",
    ] {
        assert!(ids.contains(required), "missing child-follow-up timeline: {required}");
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
    let bad_ron = include_str!("../assets/data/skills.ron").replacen("core/deal_damage", "core/deal_damge", 1);
    let book: SkillBook = ron::from_str(&bad_ron).expect("parse tweaked skills.ron");

    let err = compile_skill_book_timelines(&book, &canonical_regs())
        .expect_err("hook typo must fail before runtime");

    // baby_flame is now the first skill with a core/deal_damage beat (child-roster migration)
    assert_eq!(err.skill_id, SkillId("baby_flame".into()));
    assert_eq!(err.site, "beat impact_damage");
    assert!(err.detail.contains("core/deal_damge"));
}
