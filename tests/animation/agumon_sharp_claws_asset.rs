use bevyrogue::animation::{
    AnimGraph, AnimationValidationCatalogs, AnimationValidationCheck, Clip, ParticleId,
    validate_anim_graph,
};
use bevyrogue::combat::{
    blueprints::register_all_blueprint_exts,
    runtime::{BeatKind, BeatPayload, ExtRegistries, register_kernel_builtins},
    types::{DamageTag, SkillId},
};
use bevyrogue::data::{
    aggregate_skill_book, aggregate_unit_roster,
    skill_timeline::compile_skill_book_timelines,
    skills_ron::{Effect, SkillBook, SkillDef, TargetShape},
    units_ron::{UnitDef, UnitRoster},
};

fn canonical_book() -> SkillBook {
    aggregate_skill_book()
}

fn canonical_roster() -> UnitRoster {
    aggregate_unit_roster()
}

fn canonical_regs() -> ExtRegistries {
    let mut regs = ExtRegistries::default();
    register_kernel_builtins(&mut regs);
    register_all_blueprint_exts(&mut regs);
    regs
}

fn agumon_def(roster: &UnitRoster) -> &UnitDef {
    roster
        .0
        .iter()
        .find(|unit| unit.name == "Agumon")
        .expect("Agumon should exist in the canonical roster")
}

fn skill_def<'a>(book: &'a SkillBook, id: &str) -> &'a SkillDef {
    book.0
        .iter()
        .find(|skill| skill.id == SkillId(id.into()))
        .unwrap_or_else(|| panic!("missing canonical skill {id}"))
}

fn parse_valid_clip() -> Clip {
    ron::from_str(include_str!(
        "../../assets/test/animation_validation/valid_clip.ron"
    ))
    .expect("valid animation clip fixture should parse")
}

#[test]
fn agumon_basic_routes_to_sharp_claws_timeline_barrier() {
    let roster = canonical_roster();
    let book = canonical_book();

    let agumon = agumon_def(&roster);
    assert_eq!(agumon.basic_skill, SkillId("sharp_claws".into()));
    assert!(
        agumon.skill_ids.contains(&SkillId("baby_flame".into())),
        "Baby Flame should remain a selectable Agumon skill entry"
    );

    let sharp_claws = skill_def(&book, "sharp_claws");
    assert_eq!(sharp_claws.sp_cost, 0, "Agumon Basic must be free");
    assert_eq!(
        sharp_claws.targeting.shape,
        TargetShape::Single,
        "Sharp Claws should stay single-target"
    );

    let base_damage = sharp_claws
        .legacy_ops
        .iter()
        .find_map(|op| match op {
            Effect::Damage { amount, .. } => Some(*amount),
            _ => None,
        })
        .expect("Sharp Claws should carry legacy damage for current resolution paths");
    let toughness_hit = sharp_claws
        .legacy_ops
        .iter()
        .find_map(|op| match op {
            Effect::ToughnessHit(amount) => Some(*amount),
            _ => None,
        })
        .expect("Sharp Claws should carry legacy break for current resolution paths");

    let compiled = compile_skill_book_timelines(&book, &canonical_regs())
        .expect("canonical skill book should compile");
    let timeline = compiled
        .iter()
        .find(|timeline| timeline.id == "sharp_claws")
        .expect("Sharp Claws timeline should compile into the library");

    assert_eq!(timeline.entry, "cast");
    assert_eq!(
        timeline.beats.len(),
        5,
        "Sharp Claws should compile as cast → windup → impact_damage → impact_break → recovery"
    );
    assert_eq!(timeline.edges.len(), 4);

    let impact_damage = timeline
        .beats
        .iter()
        .find(|beat| beat.id == "impact_damage")
        .expect("Sharp Claws impact beat should exist");
    assert!(matches!(impact_damage.kind, BeatKind::Impact));
    assert_eq!(impact_damage.hook.as_deref(), Some("core/deal_damage"));
    assert_eq!(impact_damage.selector.as_deref(), Some("core/primary"));
    assert_eq!(
        impact_damage
            .presentation
            .as_ref()
            .map(|p| p.cue_id.as_str()),
        Some("agumon/sharp_claws/impact")
    );
    assert_eq!(
        impact_damage
            .presentation
            .as_ref()
            .and_then(|p| p.anim.as_deref()),
        Some("sharp_claws_strike")
    );
    assert_eq!(
        impact_damage.payload,
        Some(BeatPayload::DealDamage {
            amount: base_damage,
            tag: DamageTag::Fire,
            target: TargetShape::Single,
        })
    );

    let impact_break = timeline
        .beats
        .iter()
        .find(|beat| beat.id == "impact_break")
        .expect("Sharp Claws break beat should exist");
    assert!(matches!(impact_break.kind, BeatKind::Aftermath));
    assert_eq!(impact_break.hook.as_deref(), Some("core/apply_effect"));
    assert_eq!(impact_break.selector.as_deref(), Some("core/primary"));
    assert_eq!(
        impact_break.payload,
        Some(BeatPayload::BreakToughness {
            amount: toughness_hit,
            tag: DamageTag::Fire,
            target: TargetShape::Single,
        })
    );

    let recovery = timeline
        .beats
        .iter()
        .find(|beat| beat.id == "recovery")
        .expect("Sharp Claws recovery beat should exist");
    assert!(matches!(recovery.kind, BeatKind::Aftermath));
    assert!(recovery.hook.is_none());
    assert!(recovery.payload.is_none());
}

#[test]
fn baby_flame_still_parses_as_agumon_skill_entry() {
    let roster = canonical_roster();
    let book = canonical_book();

    let agumon = agumon_def(&roster);
    assert_eq!(agumon.skill_ids, vec![SkillId("baby_flame".into())]);

    let baby_flame = skill_def(&book, "baby_flame");
    assert!(
        baby_flame.timeline.is_some(),
        "Baby Flame should remain timeline-backed while Agumon Basic moves to Sharp Claws"
    );
}

#[test]
fn sharp_claws_fails_compile_without_kernel_builtins() {
    let sharp_claws = skill_def(&canonical_book(), "sharp_claws").clone();
    let err =
        compile_skill_book_timelines(&SkillBook(vec![sharp_claws]), &ExtRegistries::default())
            .expect_err("Sharp Claws should fail before runtime when builtins are not registered");

    assert_eq!(err.skill_id, SkillId("sharp_claws".into()));
    assert_eq!(err.site, "beat impact_damage");
    assert!(
        err.detail.contains("core/deal_damage"),
        "error detail should point at the missing builtin hook: {}",
        err.detail
    );
}

#[test]
fn sharp_claws_reports_bad_selector_with_existing_compile_error_path() {
    let mut sharp_claws = skill_def(&canonical_book(), "sharp_claws").clone();
    let timeline = sharp_claws
        .timeline
        .as_mut()
        .expect("Sharp Claws timeline should exist");
    let impact_damage = timeline
        .beats
        .iter_mut()
        .find(|beat| beat.id == "impact_damage")
        .expect("impact beat should exist");
    impact_damage.selector = Some("core/missing_selector".into());

    let err = compile_skill_book_timelines(&SkillBook(vec![sharp_claws]), &canonical_regs())
        .expect_err("unknown selector ids should fail through the existing compile path");

    assert_eq!(err.skill_id, SkillId("sharp_claws".into()));
    assert_eq!(err.site, "beat impact_damage");
    assert!(
        err.detail.contains("core/missing_selector"),
        "error detail should point at the missing selector: {}",
        err.detail
    );
}

#[test]
fn agumon_anim_graph_keeps_kernel_release_separate_from_gameplay_commands() {
    let graph: AnimGraph =
        ron::from_str(include_str!("../../assets/digimon/agumon/anim_graph.ron"))
            .expect("agumon anim graph should parse");
    let mut catalogs = AnimationValidationCatalogs::default();
    catalogs.particles.insert(ParticleId("baby_flame".into()));
    let report = validate_anim_graph(&graph, &parse_valid_clip(), &catalogs);

    let forbidden: Vec<_> = report
        .diagnostics
        .iter()
        .filter(|diag| diag.check == AnimationValidationCheck::GameplayCommandForbidden)
        .collect();
    assert!(
        forbidden.is_empty(),
        "Agumon animation graph should keep gameplay in the timeline/release boundary, not animation commands: {:?}",
        forbidden
    );
}
