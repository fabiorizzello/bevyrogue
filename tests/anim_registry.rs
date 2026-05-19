use bevy::prelude::*;
use bevyrogue::animation::{AnimGraphId, SkillGraphRegistry, StanceGraphRegistry};

fn make_id(s: &str) -> AnimGraphId {
    AnimGraphId(s.to_string())
}

#[test]
fn skill_registry_hit_returns_handle() {
    let mut reg = SkillGraphRegistry::default();
    let handle: Handle<bevyrogue::animation::AnimGraph> = Handle::default();
    let id = make_id("agumon_skill");
    reg.0.insert(id.clone(), handle.clone());
    assert!(
        reg.resolve(&id).is_some(),
        "known id must resolve to a handle"
    );
}

#[test]
fn skill_registry_miss_returns_none() {
    let reg = SkillGraphRegistry::default();
    assert!(
        reg.resolve(&make_id("unknown")).is_none(),
        "unknown id must return None"
    );
}

#[test]
fn stance_registry_hit_returns_handle() {
    let mut reg = StanceGraphRegistry::default();
    let handle: Handle<bevyrogue::animation::AnimGraph> = Handle::default();
    let id = make_id("agumon_stance");
    reg.0.insert(id.clone(), handle.clone());
    assert!(
        reg.resolve(&id).is_some(),
        "known id must resolve to a handle"
    );
}

#[test]
fn stance_registry_miss_returns_none() {
    let reg = StanceGraphRegistry::default();
    assert!(
        reg.resolve(&make_id("unknown")).is_none(),
        "unknown id must return None"
    );
}

#[test]
fn registries_are_independent() {
    let mut skill_reg = SkillGraphRegistry::default();
    let stance_reg = StanceGraphRegistry::default();
    let id = make_id("shared_id");
    skill_reg
        .0
        .insert(id.clone(), Handle::default());
    assert!(skill_reg.resolve(&id).is_some());
    assert!(
        stance_reg.resolve(&id).is_none(),
        "inserting into skill registry must not affect stance registry"
    );
}
