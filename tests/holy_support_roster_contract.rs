use bevyrogue::combat::types::SkillId;
use bevyrogue::data::skills_ron::{CustomSignalPayload, SkillBook, SkillCustomSignal, TargetShape};
use bevyrogue::data::units_ron::UnitRoster;

fn canonical_roster() -> UnitRoster {
    ron::from_str(include_str!("../assets/data/units.ron")).expect("parse units.ron")
}

fn canonical_skill_book() -> SkillBook {
    ron::from_str(include_str!("../assets/data/skills.ron")).expect("parse skills.ron")
}

fn patamon_signal() -> SkillCustomSignal {
    SkillCustomSignal::blueprint(
        "patamon",
        "build_holy_support_grace",
        CustomSignalPayload::Amount { amount: 1 },
    )
}

#[test]
fn patamon_angemon_keep_hope_line_roster_relationship() {
    let roster = canonical_roster();

    let patamon = roster
        .0
        .iter()
        .find(|u| u.name == "Patamon")
        .expect("Patamon in roster");
    assert_eq!(patamon.evo_line.0, "patamon_line");
    assert_eq!(patamon.role_tags, vec!["support", "healer"]);
    assert_eq!(patamon.ultimate_skill, SkillId("patamon_ult".into()));

    let angemon = roster
        .0
        .iter()
        .find(|u| u.name == "Angemon")
        .expect("Angemon in roster");
    assert_eq!(angemon.evo_line.0, "patamon_line");
    assert_eq!(angemon.role_tags, vec!["support", "healer"]);
    assert_eq!(angemon.ultimate_skill, SkillId("angemon_ult".into()));
}

#[test]
fn patamon_seeded_skill_declares_custom_signal_not_removed_direct_effects() {
    let book = canonical_skill_book();

    let patamon_ult = book
        .0
        .iter()
        .find(|s| s.id == SkillId("patamon_ult".into()))
        .expect("patamon_ult in book");
    assert_eq!(patamon_ult.targeting.shape, TargetShape::Single);
    assert_eq!(patamon_ult.custom_signals, vec![patamon_signal()]);

    let holy_breeze = book
        .0
        .iter()
        .find(|s| s.id == SkillId("holy_breeze".into()))
        .expect("holy_breeze in book");
    assert!(
        holy_breeze.custom_signals.is_empty(),
        "ordinary Patamon skill should not accidentally emit the blueprint signal"
    );
}

#[test]
fn patamon_roster_references_the_seeded_blueprint_skill() {
    let roster = canonical_roster();
    let book = canonical_skill_book();

    let patamon = roster
        .0
        .iter()
        .find(|u| u.name == "Patamon")
        .expect("Patamon in roster");
    assert_eq!(patamon.ultimate_skill, SkillId("patamon_ult".into()));

    let patamon_ult = book
        .0
        .iter()
        .find(|s| s.id == patamon.ultimate_skill)
        .expect("Patamon ultimate skill in book");
    assert!(
        patamon_ult
            .custom_signals
            .iter()
            .any(|signal| signal == &patamon_signal()),
        "Patamon ultimate missing Holy Support custom signal"
    );
}

#[test]
fn holy_support_metadata_remains_optional_for_backward_compatibility() {
    let roster = canonical_roster();
    let agumon = roster.0.iter().find(|u| u.name == "Agumon").unwrap();
    assert_eq!(agumon.holy_support.line, None);
    assert_eq!(agumon.holy_support.role, None);
}
