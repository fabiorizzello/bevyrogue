use std::collections::{BTreeMap, BTreeSet};

use bevyrogue::animation::{
    AnimEdge, AnimGraph, AnimNode, AnimationValidationCatalogs, AnimationValidationCheck,
    AnimationValidationDiagnostic, AnimationValidationReason, Clip, ClipId, ClipMeta, ClipRange,
    Command, FrameRange, FrameSize, KernelEventFilter, NodeId, ParamKey, ParamRef, ParticleId,
    Predicate, StatusId, TargetShape, TransitionTarget, VfxLocus, VfxMotion,
    validate_anim_graph,
};

fn mini_clip() -> Clip {
    Clip {
        meta: ClipMeta {
            frame_size: FrameSize { w: 64, h: 64 },
            columns: 4,
            rows: 4,
            total_frames: 16,
        },
        ranges: BTreeMap::from([("skill".into(), ClipRange { start: 4, end: 9 })]),
    }
}

fn mini_catalogs() -> AnimationValidationCatalogs {
    AnimationValidationCatalogs {
        params: BTreeSet::from([
            ParamKey("hits".into()),
            ParamKey("mul".into()),
            ParamKey("burn_duration".into()),
        ]),
        statuses: BTreeSet::from([StatusId("burn".into()), StatusId("guard".into())]),
        particles: BTreeSet::from([ParticleId("impact".into())]),
        skills: BTreeSet::new(),
    }
}

fn valid_graph() -> AnimGraph {
    AnimGraph {
        clip: ClipId("skill".into()),
        entry: NodeId("windup".into()),
        nodes: BTreeMap::from([
            (
                NodeId("impact".into()),
                AnimNode {
                    frames: FrameRange(6, 8),
                    on_enter: vec![
                        Command::EmitDamage {
                            hits: ParamRef::Static(ParamKey("hits".into())),
                            mul: ParamRef::Static(ParamKey("mul".into())),
                            status: Some(StatusId("burn".into())),
                            chance_pct: None,
                            duration: Some(ParamRef::Static(ParamKey("burn_duration".into()))),
                            target: TargetShape::Primary,
                        },
                        Command::SpawnParticle {
                            name: ParticleId("impact".into()),
                            origin: VfxLocus::TargetCenter,
                            motion: VfxMotion::Static,
                        },
                    ],
                    modifier: None,
                    reverse: false,
                },
            ),
            (
                NodeId("windup".into()),
                AnimNode {
                    frames: FrameRange(4, 6),
                    on_enter: vec![],
                    modifier: None,
                    reverse: false,
                },
            ),
        ]),
        transitions: vec![
            AnimEdge {
                from: NodeId("windup".into()),
                to: TransitionTarget::Node(NodeId("impact".into())),
                when: Predicate::TimeInNode,
                priority: None,
            },
            AnimEdge {
                from: NodeId("impact".into()),
                to: TransitionTarget::Exit,
                when: Predicate::And(
                    Box::new(Predicate::Unlock(NodeId("windup".into()))),
                    Box::new(Predicate::KernelEvent(KernelEventFilter::StatusApplied {
                        status: StatusId("guard".into()),
                    })),
                ),
                priority: None,
            },
        ],
    }
}

fn has_diag(
    diags: &[AnimationValidationDiagnostic],
    check: AnimationValidationCheck,
    reason: AnimationValidationReason,
) -> bool {
    diags
        .iter()
        .any(|diag| diag.check == check && diag.reason == reason)
}

#[test]
fn valid_graph_passes_without_diagnostics() {
    let report = validate_anim_graph(&valid_graph(), &mini_clip(), &mini_catalogs());

    assert!(report.is_valid(), "unexpected diagnostics: {report:#?}");
    assert!(report.diagnostics.is_empty());
}

#[test]
fn missing_cross_asset_ids_return_typed_diagnostics() {
    let mut graph = valid_graph();
    graph.clip = ClipId("missing".into());
    graph.entry = NodeId("missing_entry".into());
    graph.transitions.push(AnimEdge {
        from: NodeId("windup".into()),
        to: TransitionTarget::Node(NodeId("missing_target".into())),
        when: Predicate::TimeInNode,
        priority: None,
    });

    let report = validate_anim_graph(&graph, &mini_clip(), &mini_catalogs());

    assert!(!report.is_valid());
    assert!(has_diag(
        &report.diagnostics,
        AnimationValidationCheck::GraphClipRange,
        AnimationValidationReason::MissingClipRange,
    ));
    assert!(has_diag(
        &report.diagnostics,
        AnimationValidationCheck::EntryNode,
        AnimationValidationReason::MissingEntryNode,
    ));
    assert!(has_diag(
        &report.diagnostics,
        AnimationValidationCheck::TransitionTarget,
        AnimationValidationReason::UnknownNodeReference,
    ));
}

#[test]
fn node_frame_errors_are_collected_without_panicking() {
    let mut graph = valid_graph();
    graph.nodes.insert(
        NodeId("broken".into()),
        AnimNode {
            frames: FrameRange(20, 3),
            on_enter: vec![],
            modifier: None,
            reverse: false,
        },
    );
    graph.transitions.push(AnimEdge {
        from: NodeId("impact".into()),
        to: TransitionTarget::Node(NodeId("broken".into())),
        when: Predicate::TimeInNode,
        priority: None,
    });

    let report = validate_anim_graph(&graph, &mini_clip(), &mini_catalogs());

    assert!(!report.is_valid());
    assert!(has_diag(
        &report.diagnostics,
        AnimationValidationCheck::NodeFrames,
        AnimationValidationReason::FrameRangeOutOfOrder,
    ));
    assert!(has_diag(
        &report.diagnostics,
        AnimationValidationCheck::NodeFrames,
        AnimationValidationReason::FrameOutsideClipTotal,
    ));
    assert!(has_diag(
        &report.diagnostics,
        AnimationValidationCheck::NodeFrames,
        AnimationValidationReason::FrameOutsideNamedClipRange,
    ));
}

#[test]
fn unknown_catalog_and_graph_refs_are_reported_with_context() {
    let mut graph = valid_graph();
    graph.nodes.get_mut(&NodeId("impact".into())).unwrap().on_enter = vec![
        Command::EmitDamage {
            hits: ParamRef::Static(ParamKey("missing_hits".into())),
            mul: ParamRef::Static(ParamKey("missing_mul".into())),
            status: Some(StatusId("missing_status".into())),
            chance_pct: None,
            duration: None,
            target: TargetShape::Primary,
        },
        Command::SpawnParticle {
            name: ParticleId("missing_particle".into()),
            origin: VfxLocus::TargetCenter,
            motion: VfxMotion::Static,
        },
    ];
    graph.transitions.push(AnimEdge {
        from: NodeId("impact".into()),
        to: TransitionTarget::Exit,
        when: Predicate::And(
            Box::new(Predicate::Unlock(NodeId("missing_unlock".into()))),
            Box::new(Predicate::KernelEvent(KernelEventFilter::StatusApplied {
                status: StatusId("missing_status".into()),
            })),
        ),
        priority: None,
    });

    let report = validate_anim_graph(&graph, &mini_clip(), &mini_catalogs());

    assert!(!report.is_valid());
    assert!(has_diag(
        &report.diagnostics,
        AnimationValidationCheck::PredicateUnlock,
        AnimationValidationReason::UnknownNodeReference,
    ));
    assert!(has_diag(
        &report.diagnostics,
        AnimationValidationCheck::PredicateStatus,
        AnimationValidationReason::UnknownStatusReference,
    ));
    assert!(has_diag(
        &report.diagnostics,
        AnimationValidationCheck::CommandParam,
        AnimationValidationReason::UnknownParamReference,
    ));
    assert!(has_diag(
        &report.diagnostics,
        AnimationValidationCheck::CommandStatus,
        AnimationValidationReason::UnknownStatusReference,
    ));
    assert!(has_diag(
        &report.diagnostics,
        AnimationValidationCheck::CommandParticle,
        AnimationValidationReason::UnknownParticleReference,
    ));

    let command_diag = report
        .diagnostics
        .iter()
        .find(|diag| {
            diag.check == AnimationValidationCheck::CommandParam
                && diag.reason == AnimationValidationReason::UnknownParamReference
        })
        .expect("missing command param diagnostic");

    assert_eq!(command_diag.context.node_id, Some(NodeId("impact".into())));
    assert_eq!(command_diag.context.command_index, Some(0));
    assert_eq!(command_diag.context.command_field.as_deref(), Some("hits"));
}
