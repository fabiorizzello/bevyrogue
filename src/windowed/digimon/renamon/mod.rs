//! Renamon presentation: sprite-presentation ownership, bridged skill vocabulary,
//! and windowed demo composition.
//!
//! Like Agumon, this module owns only Renamon-specific data and registry writes.
//! The engine stays species-agnostic and consumes the generic registries.
//!
//! S08 adds the `diamond_storm_leaf` on-enter cue: `register_renamon_on_enter_effects`
//! maps the authored `SpawnParticle(name: "diamond_storm_leaf")` to the owned
//! `diamond_storm.leaf` effect id, and `register_renamon_enoki_vfx` loads the
//! `.particle.ron` handle into `EnokiVfxRegistry` with `PlacementAnchor::CasterCenter`
//! and `EnokiLifecycle::Projectile`.

use bevy::prelude::*;
use bevyrogue::animation::{AnimationStancePaths, PlacementAnchor};
use bevyrogue::combat::team::Team;
use bevyrogue::combat::types::UnitId;

use crate::windowed::demo::{WindowedDemoEntry, WindowedDemoRegistry};
use crate::windowed::render::registries::{
    EnokiEffect, EnokiLifecycle, EnokiVfxRegistry, OnEnterEffectRegistry, SkillStartNodeRegistry,
    SpritePresentationEntry, SpritePresentationRegistry,
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

/// Namespaced owned effect id for Renamon's diamond storm leaf projectile.
/// Keyed by `register_renamon_enoki_vfx` into `EnokiVfxRegistry`; consumed by
/// `register_renamon_on_enter_effects` as the target of the authored
/// `SpawnParticle(name: "diamond_storm_leaf")` cue. Namespaced under
/// `diamond_storm.*` to avoid collision with other species' effect ids.
const DIAMOND_STORM_LEAF_EFFECT_ID: &str = "diamond_storm.leaf";

/// Path (relative to `assets/`) of Renamon's enoki diamond storm leaf projectile.
const ENOKI_DIAMOND_STORM_LEAF_PATH: &str = "digimon/renamon/diamond_storm_leaf.particle.ron";

/// Animation ticks the diamond storm leaf projectile takes to travel
/// caster→target before chaining the impact. Mirrors Baby Flame's `ttl_ticks`
/// feel (5 ticks at 12fps ≈ 0.4s) so cross-screen travel feels consistent.
const DIAMOND_STORM_FLIGHT_TICKS: u32 = 5;

/// Effect id chained on arrival of the diamond storm projectile. There is no
/// separate impact effect registered in S08; this id is intentionally absent
/// from `EnokiVfxRegistry`, so `spawn_effect_by_id` gracefully spawns nothing
/// on arrival. A future slice can register `"diamond_storm.impact"` without
/// touching this module's control flow.
const DIAMOND_STORM_IMPACT_EFFECT_ID: &str = "diamond_storm.impact";

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
            register_renamon_on_enter_effects,
            register_renamon_enoki_vfx,
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

/// Map Renamon's authored `SpawnParticle` particle name(s) to the owned effect
/// id(s) that the engine spawns on node enter. The single authored cue
/// `SpawnParticle(name: "diamond_storm_leaf")` in `diamond_storm_cast` maps to
/// `DIAMOND_STORM_LEAF_EFFECT_ID` — a 1:1 mapping for this skill. The engine's
/// `on_enter` loop in `advance_digimon_presentation` reads
/// `OnEnterEffectRegistry` by name; a name absent here spawns nothing.
fn register_renamon_on_enter_effects(mut registry: ResMut<OnEnterEffectRegistry>) {
    registry.map.insert(
        "diamond_storm_leaf".to_string(),
        vec![DIAMOND_STORM_LEAF_EFFECT_ID.to_string()],
    );
}

/// Load Renamon's `diamond_storm.leaf` enoki effect handle into
/// `EnokiVfxRegistry`. `PlacementAnchor::CasterCenter` mirrors the authored
/// `origin: CasterCenter` in `anim_graph.ron`. `EnokiLifecycle::Projectile` is
/// the closest existing lifecycle for the authored `motion: ArcToTarget`:
/// there is no `ArcToTarget`-specific lifecycle variant; `Projectile` drives
/// the caster→target travel via `advance_enoki_projectiles` and chains
/// `on_arrival` on arrival — the arc shape is not expressed in the lifecycle
/// enum but the travel + chain behavior is correct (S08 design note: a future
/// slice can add an `Arc` variant without changing this registry entry).
fn register_renamon_enoki_vfx(
    asset_server: Res<AssetServer>,
    mut registry: ResMut<EnokiVfxRegistry>,
) {
    registry.handles.insert(
        DIAMOND_STORM_LEAF_EFFECT_ID.to_string(),
        EnokiEffect {
            handle: asset_server.load(ENOKI_DIAMOND_STORM_LEAF_PATH),
            anchor: PlacementAnchor::CasterCenter,
            path: ENOKI_DIAMOND_STORM_LEAF_PATH.to_string(),
            lifecycle: EnokiLifecycle::Projectile {
                flight_ticks: DIAMOND_STORM_FLIGHT_TICKS,
                on_arrival: DIAMOND_STORM_IMPACT_EFFECT_ID.to_string(),
            },
            // Diamond Storm leaves stay soft blobs (the shared material); no flipbook.
            material_override: None,
        },
    );
    info!(
        target: "windowed.renamon_playback",
        diamond_storm_leaf_path = ENOKI_DIAMOND_STORM_LEAF_PATH,
        "renamon enoki effects load requested"
    );
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
        app.init_resource::<OnEnterEffectRegistry>();
        // EnokiVfxRegistry requires AssetServer — only test the non-asset systems
        // in this broad smoke test. The enoki entry is covered by
        // `enoki_registry_holds_the_diamond_storm_leaf_entry`.
        // We add the non-asset Startup systems only, so skip `app.update()` call
        // that would trigger register_renamon_enoki_vfx (which needs AssetServer).
        // Instead, call register and then manually invoke only the systems that
        // do not require AssetServer via a targeted approach: init EnokiVfxRegistry
        // and add AssetPlugin so all Startup systems can run.
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.init_asset::<bevy_enoki::Particle2dEffect>();
        app.init_resource::<EnokiVfxRegistry>();

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

    /// The authored `SpawnParticle(name: "diamond_storm_leaf")` in
    /// `diamond_storm_cast`'s `on_enter` maps to exactly the owned
    /// `diamond_storm.leaf` effect id (1:1, no fan-out for this skill).
    #[test]
    fn on_enter_diamond_storm_leaf_maps_to_the_owned_effect_id() {
        let mut app = App::new();
        app.init_resource::<OnEnterEffectRegistry>();
        app.add_systems(Startup, register_renamon_on_enter_effects);
        app.update();

        let reg = app.world().resource::<OnEnterEffectRegistry>();
        let ids = reg
            .map
            .get("diamond_storm_leaf")
            .expect("diamond_storm_leaf must be in OnEnterEffectRegistry after register");
        assert_eq!(ids.as_slice(), [DIAMOND_STORM_LEAF_EFFECT_ID]);
        assert_eq!(DIAMOND_STORM_LEAF_EFFECT_ID, "diamond_storm.leaf");

        // An unknown particle name must NOT resolve to the diamond storm effect:
        // the bridge is an exact name map, not a substring match.
        for name in [
            "diamond_storm",
            "leaf",
            "diamond_storm_impact",
            "baby_flame_charge",
            "",
        ] {
            assert!(
                reg.map
                    .get(name)
                    .map(Vec::as_slice)
                    .unwrap_or(&[])
                    .iter()
                    .all(|id| id.as_str() != DIAMOND_STORM_LEAF_EFFECT_ID),
                "`{name}` must not map to the diamond storm leaf effect id"
            );
        }
    }

    /// `register_renamon_enoki_vfx` populates `EnokiVfxRegistry` with the
    /// `diamond_storm.leaf` entry. Requires `AssetServer` (via `TaskPoolPlugin`
    /// + `AssetPlugin` + `init_asset::<Particle2dEffect>()`); the handle is a
    /// load request — not yet resolved — so we assert the registry key, path,
    /// anchor, and lifecycle shape without touching the loaded asset.
    #[test]
    fn enoki_registry_holds_the_diamond_storm_leaf_entry() {
        let mut app = App::new();
        app.add_plugins(bevy::prelude::TaskPoolPlugin::default());
        app.add_plugins(bevy::asset::AssetPlugin::default());
        app.init_asset::<bevy_enoki::Particle2dEffect>();
        app.init_resource::<EnokiVfxRegistry>();
        app.add_systems(Startup, register_renamon_enoki_vfx);
        app.update();

        let reg = app.world().resource::<EnokiVfxRegistry>();
        let entry = reg
            .handles
            .get(DIAMOND_STORM_LEAF_EFFECT_ID)
            .expect("diamond_storm.leaf must be in EnokiVfxRegistry after register");

        assert_eq!(entry.path, ENOKI_DIAMOND_STORM_LEAF_PATH);
        assert!(matches!(entry.anchor, PlacementAnchor::CasterCenter));
        assert!(matches!(
            &entry.lifecycle,
            EnokiLifecycle::Projectile {
                flight_ticks,
                on_arrival,
            } if *flight_ticks == DIAMOND_STORM_FLIGHT_TICKS
              && on_arrival == DIAMOND_STORM_IMPACT_EFFECT_ID
        ));
    }
}
