use bevyrogue::animation::{
    AnimGraph, AnimationValidationCatalogs, AnimationValidationCheck, Clip, ParticleId, StatusId,
    validate_anim_graph,
};

fn parse_valid_graph() -> AnimGraph {
    ron::from_str(include_str!(
        "../assets/test/animation_validation/valid_anim_graph.ron"
    ))
    .expect("valid_anim_graph.ron should parse")
}

fn parse_broken_graph() -> AnimGraph {
    ron::from_str(include_str!(
        "../assets/test/animation_validation/broken_anim_graph.ron"
    ))
    .expect("broken_anim_graph.ron should parse")
}

fn parse_valid_clip() -> Clip {
    ron::from_str(include_str!(
        "../assets/test/animation_validation/valid_clip.ron"
    ))
    .expect("valid_clip.ron should parse")
}

fn full_catalog() -> AnimationValidationCatalogs {
    let mut catalogs = AnimationValidationCatalogs::default();
    catalogs
        .particles
        .insert(ParticleId("impact_particle".into()));
    catalogs.statuses.insert(StatusId("Burn".into()));
    catalogs
}

#[test]
fn valid_graph_with_full_catalog_passes() {
    let graph = parse_valid_graph();
    let clip = parse_valid_clip();
    let catalogs = full_catalog();
    let report = validate_anim_graph(&graph, &clip, &catalogs);
    assert!(
        report.is_valid(),
        "valid graph should pass validation; diagnostics: {:?}",
        report.diagnostics
    );
}

#[test]
fn broken_graph_has_transition_target_error() {
    let graph = parse_broken_graph();
    let clip = parse_valid_clip();
    let catalogs = AnimationValidationCatalogs::default();
    let report = validate_anim_graph(&graph, &clip, &catalogs);
    assert!(report.has_errors(), "broken graph must produce errors");
    let has_transition_target_err = report
        .diagnostics
        .iter()
        .any(|d| d.check == AnimationValidationCheck::TransitionTarget);
    assert!(
        has_transition_target_err,
        "broken graph must report a TransitionTarget error for missing_target node"
    );
}

#[test]
fn graph_with_kernel_cue_predicate_validates_clean() {
    let ron_str = r#"(
        id: "stance_graph",
        clip: "skill",
        entry: "cast",
        nodes: {
            "cast": (frames: (0, 5)),
            "recover": (frames: (6, 7)),
        },
        transitions: [
            (from: "cast", to: Node("recover"), when: KernelCue),
            (from: "recover", to: Exit, when: Always),
        ]
    )"#;
    let graph: AnimGraph = ron::from_str(ron_str).expect("inline graph should parse");
    let clip = parse_valid_clip();
    let report = validate_anim_graph(&graph, &clip, &AnimationValidationCatalogs::default());
    assert!(
        report.is_valid(),
        "graph using KernelCue predicate should validate clean; diagnostics: {:?}",
        report.diagnostics
    );
}
