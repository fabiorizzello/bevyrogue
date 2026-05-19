use bevyrogue::animation::{
    AnimGraph, AnimationValidationCatalogs, AnimationValidationCheck, validate_anim_graph,
    Clip, ParticleId,
};

fn parse_valid_clip() -> Clip {
    ron::from_str(include_str!("../assets/test/animation_validation/valid_clip.ron"))
        .expect("valid_clip.ron should parse")
}

/// Anti-DRY gate: production agumon graph must contain zero gameplay commands (D001).
#[test]
fn agumon_graph_has_no_gameplay_commands() {
    let ron_str = include_str!("../assets/digimon/agumon/anim_graph.ron");
    let graph: AnimGraph = ron::from_str(ron_str)
        .expect("agumon anim_graph.ron should parse");
    let mut catalogs = AnimationValidationCatalogs::default();
    catalogs.particles.insert(ParticleId("baby_flame".into()));
    let clip = parse_valid_clip();
    let report = validate_anim_graph(&graph, &clip, &catalogs);
    let forbidden: Vec<_> = report
        .diagnostics
        .iter()
        .filter(|d| d.check == AnimationValidationCheck::GameplayCommandForbidden)
        .collect();
    assert!(
        forbidden.is_empty(),
        "agumon graph must contain zero gameplay commands (D001); found: {:?}",
        forbidden
    );
}

/// Validator must reject EmitDamage in on_enter.
#[test]
fn emit_damage_in_on_enter_fails_validation() {
    let ron_str = r#"(
        id: "test_forbidden",
        clip: "skill",
        entry: "cast",
        nodes: {
            "cast": (
                frames: (0, 5),
                on_enter: [
                    EmitDamage(hits: Literal(1), mul: Literal(10), target: Primary),
                ],
            ),
        },
        transitions: [
            (from: "cast", to: Exit, when: Always),
        ]
    )"#;
    let graph: AnimGraph = ron::from_str(ron_str).expect("inline graph should parse");
    let clip = parse_valid_clip();
    let report = validate_anim_graph(&graph, &clip, &AnimationValidationCatalogs::default());
    assert!(
        report.diagnostics.iter().any(|d| d.check == AnimationValidationCheck::GameplayCommandForbidden),
        "EmitDamage in on_enter must produce GameplayCommandForbidden diagnostic"
    );
}

/// Validator must reject a gameplay command wrapped in a cue Presentation.
#[test]
fn emit_damage_in_cue_presentation_fails_validation() {
    let ron_str = r#"(
        id: "test_cue_forbidden",
        clip: "skill",
        entry: "cast",
        nodes: {
            "cast": (
                frames: (0, 5),
                cues: [
                    (at: 3, command: Presentation(EmitDamage(hits: Literal(1), mul: Literal(10), target: Primary))),
                ],
            ),
        },
        transitions: [
            (from: "cast", to: Exit, when: Always),
        ]
    )"#;
    let graph: AnimGraph = ron::from_str(ron_str).expect("inline graph should parse");
    let clip = parse_valid_clip();
    let report = validate_anim_graph(&graph, &clip, &AnimationValidationCatalogs::default());
    assert!(
        report.diagnostics.iter().any(|d| d.check == AnimationValidationCheck::GameplayCommandForbidden),
        "EmitDamage in cue Presentation must produce GameplayCommandForbidden diagnostic"
    );
}

/// ReleaseKernelCue in cues must not trigger GameplayCommandForbidden.
#[test]
fn release_kernel_cue_is_allowed() {
    let ron_str = r#"(
        id: "test_kernel_cue_ok",
        clip: "skill",
        entry: "cast",
        nodes: {
            "cast": (
                frames: (0, 5),
                cues: [
                    (at: 3, command: ReleaseKernel(())),
                ],
            ),
        },
        transitions: [
            (from: "cast", to: Exit, when: KernelCue),
        ]
    )"#;
    let graph: AnimGraph = ron::from_str(ron_str).expect("inline graph should parse");
    let clip = parse_valid_clip();
    let report = validate_anim_graph(&graph, &clip, &AnimationValidationCatalogs::default());
    let forbidden: Vec<_> = report
        .diagnostics
        .iter()
        .filter(|d| d.check == AnimationValidationCheck::GameplayCommandForbidden)
        .collect();
    assert!(
        forbidden.is_empty(),
        "ReleaseKernelCue must not trigger GameplayCommandForbidden; diagnostics: {:?}",
        forbidden
    );
}
