use bevyrogue::animation::{
    AnimGraph, AnimationValidationCatalogs, AnimationValidationSeverity, Clip, StanceGraphRegistry,
    validate_anim_graph,
};

fn parse_agumon_clip() -> Clip {
    ron::from_str(include_str!("../../assets/digimon/agumon/clip.ron"))
        .expect("agumon clip.ron should parse")
}

fn parse_stance() -> AnimGraph {
    ron::from_str(include_str!("../../assets/digimon/agumon/stance.ron"))
        .expect("stance.ron should parse")
}

#[test]
fn stance_ron_parses() {
    let graph = parse_stance();
    assert_eq!(graph.id.0, "agumon_stance");
    assert_eq!(graph.clip.0, "all");
    assert_eq!(graph.entry.0, "idle");
    assert_eq!(graph.nodes.len(), 4);
}

#[test]
fn stance_validates_with_zero_errors() {
    let graph = parse_stance();
    let clip = parse_agumon_clip();
    let catalogs = AnimationValidationCatalogs::default();
    let report = validate_anim_graph(&graph, &clip, &catalogs);
    let errors: Vec<_> = report
        .diagnostics
        .iter()
        .filter(|d| d.severity == AnimationValidationSeverity::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "stance graph must validate with zero errors; got: {:?}",
        errors
    );
}

#[test]
fn stance_registry_resolves_agumon_stance() {
    let graph = parse_stance();
    let mut reg = StanceGraphRegistry::default();
    reg.0
        .insert(graph.id.clone(), bevy::prelude::Handle::default());
    assert!(
        reg.resolve(&graph.id).is_some(),
        "StanceGraphRegistry must resolve 'agumon_stance'"
    );
}
