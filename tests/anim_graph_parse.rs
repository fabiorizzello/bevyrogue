use bevyrogue::animation::{
    AdjMetric, AnimGraph, BuffKind, Command, FrameRange, KernelEventFilter, NodeId, ParamKey,
    ParamRef, PlaybackModifier, Predicate, Priority, QteKind, QteOutcome, ScanDirection,
    SeedSource, Side, StatusId, TargetAnchor, TargetShape, TransitionTarget, UserInputFilter,
};

#[test]
fn valid_anim_graph_parses_into_typed_variants() {
    let graph: AnimGraph = ron::from_str(
        r#"(
            clip: "skill",
            entry: "windup",
            nodes: {
                "windup": (
                    frames: (0, 12),
                    modifier: Some(Hold(extra_frames: 3)),
                ),
                "impact": (
                    frames: (12, 14),
                    on_enter: [
                        EmitDamage(
                            hits: Static("hits"),
                            mul: Static("atk_mul"),
                            status: Some("Heated"),
                            chance_pct: Some(Static("burn_chance_pct")),
                            duration: Some(Static("burn_duration")),
                            target: Blast(Primary),
                        ),
                        ApplyBuff(
                            id: "Blessed",
                            duration: Static("buff_duration"),
                            kind: Aura,
                            target: AoE(side: AllyTeam, exclude_dead: true),
                        ),
                        StartQte(
                            kind: Mash,
                            window: Static("qte_window_ms"),
                            headless_default: Success,
                        ),
                    ],
                ),
                "recovery": (
                    frames: (14, 17),
                    reverse: true,
                ),
            },
            transitions: [
                (from: "windup", to: Node("impact"), when: TimeInNode, priority: Some((10))),
                (
                    from: "impact",
                    to: Node("recovery"),
                    when: And(KernelEvent(StatusApplied(status: "Heated")), Not(UserInput(QteFail))),
                    priority: Some((5)),
                ),
                (
                    from: "recovery",
                    to: Exit,
                    when: Or(Always, Unlock("super_charge")),
                    priority: None,
                ),
            ],
        )"#,
    )
    .expect("valid graph should parse");

    assert_eq!(graph.entry, NodeId("windup".into()));
    assert_eq!(
        graph.nodes[&NodeId("windup".into())].frames,
        FrameRange(0, 12)
    );
    assert_eq!(graph.nodes[&NodeId("impact".into())].on_enter.len(), 3);
    assert_eq!(
        graph.nodes[&NodeId("windup".into())].modifier,
        Some(PlaybackModifier::Hold { extra_frames: 3 })
    );
    assert_eq!(
        graph.transitions[0].to,
        TransitionTarget::Node(NodeId("impact".into()))
    );

    match &graph.nodes[&NodeId("impact".into())].on_enter[0] {
        Command::EmitDamage {
            hits,
            mul,
            status,
            chance_pct,
            duration,
            target,
        } => {
            assert_eq!(hits, &ParamRef::Static(ParamKey("hits".into())));
            assert_eq!(mul, &ParamRef::Static(ParamKey("atk_mul".into())));
            assert_eq!(status, &Some(StatusId("Heated".into())));
            assert_eq!(
                chance_pct,
                &Some(ParamRef::Static(ParamKey("burn_chance_pct".into())))
            );
            assert_eq!(
                duration,
                &Some(ParamRef::Static(ParamKey("burn_duration".into())))
            );
            assert_eq!(target, &TargetShape::Blast(TargetAnchor::Primary));
        }
        other => panic!("expected EmitDamage, got {other:?}"),
    }

    match &graph.nodes[&NodeId("impact".into())].on_enter[1] {
        Command::ApplyBuff {
            id,
            duration,
            kind,
            target,
        } => {
            assert_eq!(id, &StatusId("Blessed".into()));
            assert_eq!(
                duration,
                &ParamRef::Static(ParamKey("buff_duration".into()))
            );
            assert_eq!(kind, &BuffKind::Aura);
            assert_eq!(
                target,
                &TargetShape::AoE {
                    side: Side::AllyTeam,
                    exclude_dead: true,
                }
            );
        }
        other => panic!("expected ApplyBuff, got {other:?}"),
    }

    match &graph.nodes[&NodeId("impact".into())].on_enter[2] {
        Command::StartQte {
            kind,
            window,
            headless_default,
        } => {
            assert_eq!(kind, &QteKind::Mash);
            assert_eq!(window, &ParamRef::Static(ParamKey("qte_window_ms".into())));
            assert_eq!(headless_default, &QteOutcome::Success);
        }
        other => panic!("expected StartQte, got {other:?}"),
    }

    assert!(graph.nodes[&NodeId("recovery".into())].reverse);
    assert_eq!(graph.transitions[0].priority, Some(Priority(10)));
    assert_eq!(graph.transitions[1].priority, Some(Priority(5)));
    assert_eq!(graph.transitions[2].priority, None);

    match &graph.transitions[1].when {
        Predicate::And(left, right) => {
            assert_eq!(
                left.as_ref(),
                &Predicate::KernelEvent(KernelEventFilter::StatusApplied {
                    status: StatusId("Heated".into()),
                })
            );
            assert_eq!(
                right.as_ref(),
                &Predicate::Not(Box::new(Predicate::UserInput(UserInputFilter::QteFail)))
            );
        }
        other => panic!("expected And predicate, got {other:?}"),
    }

    assert_eq!(graph.transitions[2].to, TransitionTarget::Exit);
}

#[test]
fn unknown_command_variant_is_rejected() {
    let err = ron::from_str::<AnimGraph>(
        r#"(
            clip: "skill",
            entry: "only",
            nodes: {
                "only": (
                    frames: (0, 1),
                    on_enter: [LaunchMissile(power: Static("atk_mul"))],
                ),
            },
            transitions: [(from: "only", to: Exit, when: Always)],
        )"#,
    )
    .expect_err("unknown command should fail");

    let msg = err.to_string();
    assert!(
        msg.contains("LaunchMissile") || msg.contains("variant") || msg.contains("identifier"),
        "unexpected parse error: {msg}"
    );
}

#[test]
fn unknown_predicate_variant_is_rejected() {
    let err = ron::from_str::<AnimGraph>(
        r#"(
            clip: "skill",
            entry: "only",
            nodes: {
                "only": (
                    frames: (0, 1),
                ),
            },
            transitions: [(from: "only", to: Exit, when: HpBelow(50))],
        )"#,
    )
    .expect_err("unknown predicate should fail");

    let msg = err.to_string();
    assert!(
        msg.contains("HpBelow") || msg.contains("variant") || msg.contains("identifier"),
        "unexpected parse error: {msg}"
    );
}

#[test]
fn unknown_target_shape_variant_is_rejected() {
    let err = ron::from_str::<AnimGraph>(
        r#"(
            clip: "skill",
            entry: "only",
            nodes: {
                "only": (
                    frames: (0, 1),
                    on_enter: [
                        EmitDamage(
                            hits: Static("hits"),
                            mul: Static("atk_mul"),
                            target: Cone(radius: 3),
                        ),
                    ],
                ),
            },
            transitions: [(from: "only", to: Exit, when: Always)],
        )"#,
    )
    .expect_err("unknown target shape should fail");

    let msg = err.to_string();
    assert!(
        msg.contains("Cone") || msg.contains("variant") || msg.contains("identifier"),
        "unexpected parse error: {msg}"
    );
}

#[test]
fn target_shape_supports_closed_nested_variants() {
    let shape: TargetShape =
        ron::from_str("Bounce(hits: 3, selector: NextAliveAdj(side: EnemyTeam, scan: ClockWise))")
            .expect("nested shape should parse");

    assert_eq!(
        shape,
        TargetShape::Bounce {
            hits: 3,
            selector: Box::new(TargetShape::NextAliveAdj {
                side: Side::EnemyTeam,
                scan: ScanDirection::ClockWise,
            }),
        }
    );

    let aux_shape: TargetShape = ron::from_str("AdjLowest(metric: HpPctMin, side: EnemyTeam)")
        .expect("adj metric shape should parse");
    assert_eq!(
        aux_shape,
        TargetShape::AdjLowest {
            metric: AdjMetric::HpPctMin,
            side: Side::EnemyTeam,
        }
    );

    let random_shape: TargetShape =
        ron::from_str("RandomEnemyAlive(seed: TurnRng)").expect("random target should parse");
    assert_eq!(
        random_shape,
        TargetShape::RandomEnemyAlive {
            seed: SeedSource::TurnRng,
        }
    );
}
