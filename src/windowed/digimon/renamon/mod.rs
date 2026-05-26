//! Renamon presentation: sprite-presentation ownership, bridged skill vocabulary,
//! and windowed demo composition.
//!
//! Like Agumon, this module owns only Renamon-specific data and registry writes.
//! The engine stays species-agnostic and consumes the generic registries.

use bevy::prelude::*;
use bevyrogue::animation::AnimationStancePaths;
use bevyrogue::combat::team::Team;
use bevyrogue::combat::types::UnitId;

use crate::windowed::demo::{WindowedDemoEntry, WindowedDemoRegistry};
use crate::windowed::render::{
    SkillStartNodeRegistry, SpritePresentationEntry, SpritePresentationRegistry,
};

const RENAMON_UNIT_ID: UnitId = UnitId(7);
const RENAMON_PRESENTATION_ID: &str = "renamon";
const RENAMON_STANCE_GRAPH_ID: &str = "renamon_stance";
const RENAMON_SKILL_GRAPH_ID: &str = "renamon_skill";
const RENAMON_STANCE_PATH: &str = "digimon/renamon/stance.ron";
const RENAMON_ATLAS_IMAGE_PATH: &str = "digimon/renamon_atlas.png";
const RENAMON_CLIP_INDEX: usize = 1;

const DIAMOND_STORM_SKILL_ID: &str = "diamond_storm";
const DIAMOND_STORM_CAST_NODE: &str = "diamond_storm_cast";

/// Populate the engine registries with all Renamon-specific presentation data.
/// Called once from `crate::windowed::digimon::register_all`.
pub(in crate::windowed) fn register(app: &mut App) {
    register_renamon_stance_path(app);
    app.add_systems(
        Startup,
        (
            register_renamon_skill_start_nodes,
            register_renamon_sprite_presentation,
            register_renamon_windowed_demo,
        ),
    );
}

/// Extend the boot-time stance graph path list during app construction, before
/// `AnimationAssetPlugin`'s Startup load system snapshots `AnimationStancePaths`.
fn register_renamon_stance_path(app: &mut App) {
    let mut stance_paths = app.world_mut().resource_mut::<AnimationStancePaths>();
    if !stance_paths
        .0
        .iter()
        .any(|path| path == RENAMON_STANCE_PATH)
    {
        stance_paths.0.push(RENAMON_STANCE_PATH.to_string());
    }
}

fn register_renamon_skill_start_nodes(mut registry: ResMut<SkillStartNodeRegistry>) {
    registry.map.insert(
        DIAMOND_STORM_SKILL_ID.to_string(),
        DIAMOND_STORM_CAST_NODE.to_string(),
    );
}

fn register_renamon_sprite_presentation(mut registry: ResMut<SpritePresentationRegistry>) {
    registry.entries.push(SpritePresentationEntry {
        presentation_id: RENAMON_PRESENTATION_ID.to_string(),
        unit_ids: vec![RENAMON_UNIT_ID],
        stance_graph_id: RENAMON_STANCE_GRAPH_ID.to_string(),
        skill_graph_id: RENAMON_SKILL_GRAPH_ID.to_string(),
        atlas_image_path: RENAMON_ATLAS_IMAGE_PATH.to_string(),
        clip_index: RENAMON_CLIP_INDEX,
    });
}

fn register_renamon_windowed_demo(mut registry: ResMut<WindowedDemoRegistry>) {
    registry.entries.push(WindowedDemoEntry {
        demo_id: "renamon_ally".to_string(),
        source_unit_id: RENAMON_UNIT_ID,
        spawned_unit_id: RENAMON_UNIT_ID,
        team: Team::Ally,
        name_override: None,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_inserts_the_renamon_stance_path_at_build_time() {
        let mut app = App::new();
        app.init_resource::<AnimationStancePaths>();

        register(&mut app);

        let stance_paths = app.world().resource::<AnimationStancePaths>();
        assert!(
            stance_paths
                .0
                .iter()
                .any(|path| path == RENAMON_STANCE_PATH)
        );
    }

    #[test]
    fn register_does_not_duplicate_the_stance_path() {
        let mut app = App::new();
        app.init_resource::<AnimationStancePaths>();

        register(&mut app);
        register(&mut app);

        let stance_paths = app.world().resource::<AnimationStancePaths>();
        assert_eq!(
            stance_paths
                .0
                .iter()
                .filter(|path| path.as_str() == RENAMON_STANCE_PATH)
                .count(),
            1
        );
    }

    #[test]
    fn register_populates_the_windowed_registries() {
        let mut app = App::new();
        app.init_resource::<AnimationStancePaths>();
        app.init_resource::<SkillStartNodeRegistry>();
        app.init_resource::<SpritePresentationRegistry>();
        app.init_resource::<WindowedDemoRegistry>();

        register(&mut app);
        app.update();

        let skill_start = app.world().resource::<SkillStartNodeRegistry>();
        assert_eq!(
            skill_start
                .map
                .get(DIAMOND_STORM_SKILL_ID)
                .map(String::as_str),
            Some(DIAMOND_STORM_CAST_NODE)
        );

        let presentation = app.world().resource::<SpritePresentationRegistry>();
        let entry = presentation
            .entries
            .iter()
            .find(|entry| entry.presentation_id == RENAMON_PRESENTATION_ID)
            .expect("renamon presentation entry");
        assert_eq!(entry.unit_ids, vec![RENAMON_UNIT_ID]);
        assert_eq!(entry.stance_graph_id, RENAMON_STANCE_GRAPH_ID);
        assert_eq!(entry.skill_graph_id, RENAMON_SKILL_GRAPH_ID);
        assert_eq!(entry.atlas_image_path, RENAMON_ATLAS_IMAGE_PATH);
        assert_eq!(entry.clip_index, RENAMON_CLIP_INDEX);

        let demo = app.world().resource::<WindowedDemoRegistry>();
        let entry = demo
            .entries
            .iter()
            .find(|entry| entry.demo_id == "renamon_ally")
            .expect("renamon demo entry");
        assert_eq!(entry.source_unit_id, RENAMON_UNIT_ID);
        assert_eq!(entry.spawned_unit_id, RENAMON_UNIT_ID);
        assert_eq!(entry.team, Team::Ally);
        assert_eq!(entry.name_override, None);
    }
}
