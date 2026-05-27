// Reproduces the single-graph-per-batch starvation bug in `populate_graph_registries`.
//
// The system contains `return` statements at lines 275 and 279 of
// `src/animation/registry.rs` that exit the entire event loop after the first
// matching graph is processed. When two AssetEvents arrive in the same frame —
// one for a skill-path graph and one for a stance-path graph — the system inserts
// the skill graph into `SkillGraphRegistry` and then immediately returns, leaving
// the stance event unprocessed and `StanceGraphRegistry` empty.
//
// This test MUST remain RED against the current code. That red result is the
// evidence of the bug; T02 will make it green by replacing the `return`s with
// `continue`.
//
// Setup:
//   • `AssetPlugin` is included so `AssetServer` can register path→index mappings
//     via `load()` calls (path registration is synchronous, file I/O is not needed).
//   • Both graphs are inserted into `Assets<AnimGraph>` directly from inline RON so
//     the test is deterministic and file-I/O-free beyond the initial `load()` call.
//   • Both `AssetEvent::LoadedWithDependencies` messages are queued in the SAME
//     frame before `populate_graph_registries` runs, guaranteeing they land in the
//     same event batch.

use bevy::asset::{AssetEvent, AssetPlugin};
use bevy::prelude::TaskPoolPlugin;
use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevyrogue::animation::{
    AnimGraph, AnimGraphId, AnimationGraphHandles, SkillGraphPaths, SkillGraphRegistry,
    StanceGraphPaths, StanceGraphRegistry,
};
use bevyrogue::animation::registry::populate_graph_registries;

// Inline skill graph — parsed once, inserted via a known handle.
const SKILL_RON: &str = r#"(
    id: "agumon_skill",
    clip: "all",
    entry: "cast",
    nodes: {
        "cast": (frames: (0, 10)),
    },
    transitions: []
)"#;

// Inline stance graph — parsed once, inserted via a known handle.
const STANCE_RON: &str = r#"(
    id: "agumon_stance",
    clip: "all",
    entry: "idle",
    nodes: {
        "idle": (
            frames: (53, 58),
            modifier: Some(Loop(count: 0)),
        ),
    },
    transitions: [
        (from: "idle", to: Node("idle"), when: TimeInNode),
    ]
)"#;

/// The asset path the project uses for the agumon skill graph.
/// Must match the value stored in `SkillGraphPaths` so the system classifies
/// the event as a skill-graph load.
const SKILL_ASSET_PATH: &str = "digimon/agumon/anim_graph.ron";

/// The asset path the project uses for the agumon stance graph.
/// Must match the value stored in `StanceGraphPaths`.
const STANCE_ASSET_PATH: &str = "digimon/agumon/stance.ron";

/// Builds a minimal App that is sufficient to run `populate_graph_registries`:
///   - `TaskPoolPlugin` + `AssetPlugin` + `RonAssetPlugin<AnimGraph>` wire up
///     `AssetServer` with a shared index allocator for `Assets<AnimGraph>`.
///   - All registry resources are initialised in their empty/default state.
fn build_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        TaskPoolPlugin::default(),
        AssetPlugin {
            // Disable hot-reloading; we control events manually.
            watch_for_changes_override: Some(false),
            use_asset_processor_override: Some(false),
            ..Default::default()
        },
        RonAssetPlugin::<AnimGraph>::new(&["ron"]),
    ))
    .add_message::<AssetEvent<AnimGraph>>()
    .init_resource::<SkillGraphPaths>()
    .init_resource::<StanceGraphPaths>()
    .init_resource::<SkillGraphRegistry>()
    .init_resource::<StanceGraphRegistry>()
    .add_systems(Update, populate_graph_registries);

    app
}

/// Queuing two `AssetEvent::LoadedWithDependencies` messages — one for a
/// skill-path graph and one for a stance-path graph — in the same frame causes
/// `populate_graph_registries` to insert the skill graph and then immediately
/// `return`, starving the stance graph.
///
/// EXPECTED RESULT: **FAIL** against the current code.
/// The assertion `stance_reg.0.contains_key(&stance_id)` will fail because
/// the `return` on line 275 of `registry.rs` prevents the stance event from
/// ever being processed.
#[test]
fn populate_graph_registries_starves_second_event_when_first_matches() {
    let mut app = build_app();

    // Parse graphs from inline RON — deterministic, no filesystem dependency.
    let skill_graph: AnimGraph =
        ron::from_str(SKILL_RON).expect("inline skill RON must parse");
    let stance_graph: AnimGraph =
        ron::from_str(STANCE_RON).expect("inline stance RON must parse");

    let skill_id = skill_graph.id.clone();
    let stance_id = stance_graph.id.clone();

    // Obtain handles via `asset_server.load()`.  This call is synchronous for
    // path registration: `AssetServer` immediately writes the path→index
    // mapping into its `AssetInfos`, so `asset_server.get_path(handle.id())`
    // will return `Some(path)` even before the file has been read from disk.
    // The actual file read is irrelevant — we will insert the asset data
    // ourselves using the same index.
    let asset_server = app.world().resource::<AssetServer>().clone();
    let handle_skill: Handle<AnimGraph> = asset_server.load(SKILL_ASSET_PATH);
    let handle_stance: Handle<AnimGraph> = asset_server.load(STANCE_ASSET_PATH);

    // Record the asset IDs for event construction.
    let skill_asset_id = handle_skill.id();
    let stance_asset_id = handle_stance.id();

    // Insert graph data into `Assets<AnimGraph>` under the same indexed IDs
    // that `asset_server.load()` reserved.  The shared index allocator ensures
    // the storage slots exist; `insert` expands them via `flush()`.
    {
        let mut graphs = app.world_mut().resource_mut::<Assets<AnimGraph>>();
        graphs
            .insert(skill_asset_id, skill_graph)
            .expect("skill graph insertion must succeed with a freshly reserved index");
        graphs
            .insert(stance_asset_id, stance_graph)
            .expect("stance graph insertion must succeed with a freshly reserved index");
    }

    // Register both handles so `populate_graph_registries` can map
    // asset IDs back to handles via `AnimationGraphHandles`.
    app.world_mut().insert_resource(AnimationGraphHandles(vec![
        handle_skill.clone(),
        handle_stance.clone(),
    ]));

    // Classify paths: agumon/anim_graph.ron → skill, agumon/stance.ron → stance.
    app.world_mut()
        .resource_mut::<SkillGraphPaths>()
        .0
        .push(SKILL_ASSET_PATH.to_string());
    app.world_mut()
        .resource_mut::<StanceGraphPaths>()
        .0
        .push(STANCE_ASSET_PATH.to_string());

    // Queue BOTH load events in the same frame.  The skill event comes first so
    // the bug's `return` fires on the skill branch, starving the stance event.
    app.world_mut()
        .write_message(AssetEvent::<AnimGraph>::LoadedWithDependencies {
            id: skill_asset_id,
        });
    app.world_mut()
        .write_message(AssetEvent::<AnimGraph>::LoadedWithDependencies {
            id: stance_asset_id,
        });

    // Run one update so `populate_graph_registries` processes the event batch.
    app.update();

    // --- Assertions ---

    let skill_reg = app.world().resource::<SkillGraphRegistry>();
    let stance_reg = app.world().resource::<StanceGraphRegistry>();

    assert!(
        skill_reg.0.contains_key(&AnimGraphId("agumon_skill".into())),
        "SkillGraphRegistry must contain the skill graph after a LoadedWithDependencies event \
         (id={skill_id:?}); if this fails the test setup is broken, not the starvation bug"
    );

    // This assertion FAILS against the current code: the `return` at line 275
    // of registry.rs exits populate_graph_registries after inserting the skill
    // graph, so the stance event is never consumed and StanceGraphRegistry
    // stays empty — reproducing the starvation bug.
    assert!(
        stance_reg.0.contains_key(&AnimGraphId("agumon_stance".into())),
        "StanceGraphRegistry must contain the stance graph after a LoadedWithDependencies event \
         in the same batch as the skill event (id={stance_id:?}); \
         EXPECTED FAILURE: the `return` at registry.rs:275 starves this event — \
         both registries must be populated when the bug is fixed"
    );
}
