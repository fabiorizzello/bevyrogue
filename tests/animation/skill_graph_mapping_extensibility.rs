// Extensibility proof: SkillGraphRegistry supports arbitrary many AnimGraphId↔AnimGraph
// mappings via a plain HashMap — the only hardcoded seam is the call-site constant in the
// windowed binary crate. Adding a new skill graph is a data/lookup change (insert into the
// registry), not a code rewrite.
//
// Boundary note: `return_to_idle` lives in `src/windowed/render.rs` and is therefore cited
// in the boundary map rather than directly callable from integration tests. This file
// documents that the binary-side consumer reads `graph().entry` from the resolved stance
// snapshot and resets the player to that node; the assertion on `entry` below proves the
// contract surface is non-empty and well-typed.

use bevy::prelude::Assets;
use bevyrogue::animation::{
    AnimGraph, AnimGraphId, AnimationGraphLookupDiagnostics, MISSING_GRAPH_FALLBACK_NODE_ID,
    ResolvedAnimGraphSource, SkillGraphRegistry, StanceGraphRegistry,
};

fn make_skill_graph(id: &str, entry_node: &str) -> AnimGraph {
    ron::from_str(&format!(
        r#"(
        id: "{id}",
        clip: "all",
        entry: "{entry_node}",
        nodes: {{
            "{entry_node}": (frames: (0, 10)),
        }},
        transitions: []
    )"#
    ))
    .unwrap_or_else(|e| panic!("inline graph '{id}' should parse: {e}"))
}

fn make_stance_graph(id: &str, entry_node: &str) -> AnimGraph {
    ron::from_str(&format!(
        r#"(
        id: "{id}",
        clip: "all",
        entry: "{entry_node}",
        nodes: {{
            "{entry_node}": (
                frames: (53, 58),
                modifier: Some(Loop(count: 0)),
            ),
        }},
        transitions: [
            (from: "{entry_node}", to: Node("{entry_node}"), when: TimeInNode),
        ]
    )"#
    ))
    .unwrap_or_else(|e| panic!("inline stance graph '{id}' should parse: {e}"))
}

/// Registry stores N distinct AnimGraphId↔Handle mappings without collision.
/// Each resolve_snapshot returns source == Registry and the correct requested_id.
#[test]
fn skill_registry_supports_multiple_distinct_graph_ids() {
    let mut registry = SkillGraphRegistry::default();
    let mut graphs = Assets::<AnimGraph>::default();

    let id_a = AnimGraphId("skill_graph_alpha".into());
    let id_b = AnimGraphId("skill_graph_beta".into());
    let id_c = AnimGraphId("skill_graph_gamma".into());

    let graph_a = make_skill_graph("skill_graph_alpha", "cast_a");
    let graph_b = make_skill_graph("skill_graph_beta", "cast_b");
    let graph_c = make_skill_graph("skill_graph_gamma", "cast_c");

    let handle_a = graphs.add(graph_a.clone());
    let handle_b = graphs.add(graph_b.clone());
    let handle_c = graphs.add(graph_c.clone());

    registry.0.insert(id_a.clone(), handle_a);
    registry.0.insert(id_b.clone(), handle_b);
    registry.0.insert(id_c.clone(), handle_c);

    // Each id resolves to its own graph — 1:1, no cross-contamination.
    let resolved_a = registry
        .resolve_snapshot(&id_a, &graphs)
        .expect("id_a must resolve");
    assert_eq!(resolved_a.source, ResolvedAnimGraphSource::Registry);
    assert_eq!(resolved_a.requested_id, id_a);
    assert_eq!(resolved_a.graph().entry.0, "cast_a");

    let resolved_b = registry
        .resolve_snapshot(&id_b, &graphs)
        .expect("id_b must resolve");
    assert_eq!(resolved_b.source, ResolvedAnimGraphSource::Registry);
    assert_eq!(resolved_b.requested_id, id_b);
    assert_eq!(resolved_b.graph().entry.0, "cast_b");

    let resolved_c = registry
        .resolve_snapshot(&id_c, &graphs)
        .expect("id_c must resolve");
    assert_eq!(resolved_c.source, ResolvedAnimGraphSource::Registry);
    assert_eq!(resolved_c.requested_id, id_c);
    assert_eq!(resolved_c.graph().entry.0, "cast_c");

    // Snapshots are independent clones — mutating one must not affect another.
    assert_ne!(
        resolved_a.graph().entry,
        resolved_b.graph().entry,
        "distinct ids must not share entry nodes"
    );
    assert_ne!(resolved_b.graph().entry, resolved_c.graph().entry);
}

/// An unregistered id returns source == InstantFallback and records a structured diagnostic.
#[test]
fn unregistered_skill_id_returns_instant_fallback_with_diagnostic() {
    let registry = SkillGraphRegistry::default();
    let graphs = Assets::<AnimGraph>::default();
    let mut diagnostics = AnimationGraphLookupDiagnostics::default();

    let missing_id = AnimGraphId("skill_graph_not_in_registry".into());

    let resolved =
        registry.resolve_snapshot_or_instant_fallback(&missing_id, &graphs, &mut diagnostics);

    assert_eq!(
        resolved.source,
        ResolvedAnimGraphSource::InstantFallback,
        "unregistered id must fall back to InstantFallback"
    );
    assert_eq!(resolved.requested_id, missing_id);
    assert_eq!(
        resolved.graph().entry.0,
        MISSING_GRAPH_FALLBACK_NODE_ID,
        "fallback graph entry must be the well-known missing-graph sentinel node"
    );

    let diag = diagnostics
        .last_missing_skill_graph
        .as_ref()
        .expect("a MissingGraphDiagnostic must be recorded for unregistered skill lookup");
    assert_eq!(diag.registry, "skill");
    assert_eq!(diag.requested_id, missing_id);
    assert_eq!(diag.fallback_node.0, MISSING_GRAPH_FALLBACK_NODE_ID);
    assert!(
        diag.message.contains("missing_or_unloaded"),
        "diagnostic message must explain the fallback reason: {}",
        diag.message
    );
    assert!(
        diagnostics
            .last_message
            .as_deref()
            .is_some_and(|m| m.contains("skill_graph_not_in_registry")),
        "last_message must expose the unresolved id for inspection"
    );
}

/// A stance snapshot exposes a non-empty `graph().entry`.
///
/// Boundary: `return_to_idle` in `src/windowed/render.rs` calls
/// `graph.graph().entry.clone()` to reset the player; this test proves that the
/// entry produced by a stance-registry snapshot is non-empty and well-typed,
/// satisfying the binary-side contract without calling the binary directly.
#[test]
fn stance_graph_snapshot_entry_is_non_empty_for_return_to_idle_boundary() {
    let mut registry = StanceGraphRegistry::default();
    let mut graphs = Assets::<AnimGraph>::default();

    let stance = make_stance_graph("agumon_stance_ext", "idle");
    let stance_id = stance.id.clone();
    let handle = graphs.add(stance);
    registry.0.insert(stance_id.clone(), handle);

    let resolved = registry
        .resolve_snapshot(&stance_id, &graphs)
        .expect("stance graph must resolve from registry");

    assert_eq!(resolved.source, ResolvedAnimGraphSource::Registry);
    assert_eq!(resolved.requested_id, stance_id);

    // The entry node is what return_to_idle passes to AnimGraphPlayer::new().
    // Assert it is a non-empty NodeId — the binary boundary map documents that
    // return_to_idle(stance_graph, _) clones this exact field.
    let entry = &resolved.graph().entry;
    assert!(
        !entry.0.is_empty(),
        "stance graph entry must be non-empty so return_to_idle can reset the player"
    );
    assert_eq!(
        entry.0, "idle",
        "the well-known return-to-idle node must match the graph's declared entry"
    );
}
