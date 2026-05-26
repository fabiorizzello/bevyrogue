//! Agumon presentation: cosmetic cues, effect-id mappings, and skill-node
//! vocabulary. Registered into the generic engine registries via `register`.
//!
//! S04 extracts this out of the windowed engine files (`src/windowed/mod.rs`,
//! `src/windowed/render.rs`); the engine systems stay generic and read from the
//! registries this module populates.

use bevy::prelude::*;
use bevyrogue::animation::PlacementAnchor;
use bevyrogue::combat::bootstrap::AGUMON_DUMMY_ID;
use bevyrogue::combat::team::Team;
use bevyrogue::combat::types::UnitId;

use crate::windowed::demo::{WindowedDemoEntry, WindowedDemoRegistry};
use crate::windowed::render::{
    DetonateEffectRegistry, EnokiEffect, EnokiLifecycle, EnokiVfxRegistry, OnEnterEffectRegistry,
    SkillReleaseEffectRegistry, SkillStartNodeRegistry, SpritePresentationEntry,
    SpritePresentationRegistry,
};

// Agumon's animation-graph ids and skill/node vocabulary (S04). Owned by this
// module: the engine reads them from the registries below, never as consts.
const AGUMON_PRESENTATION_ID: &str = "agumon";
const AGUMON_STANCE_GRAPH_ID: &str = "agumon_stance";
const AGUMON_SKILL_GRAPH_ID: &str = "agumon_skill";
const SHARP_CLAWS_SKILL_ID: &str = "sharp_claws";
const SHARP_CLAWS_WINDUP_NODE: &str = "sharp_claws_windup";
const BABY_FLAME_SKILL_ID: &str = "baby_flame";
const AGUMON_ULT_SKILL_ID: &str = "agumon_ult";
const BABY_FLAME_CAST_NODE: &str = "baby_flame_cast";
const BABY_BURNER_CHARGE_NODE: &str = "baby_burner_charge";
// The skill FSM entry nodes above seed the player; the presentation nodes below
// complete the bridged-node vocabulary (matching anim_graph.ron) and are
// consumed when the Baby Flame / Baby Burner impact-release bridges go live.
#[allow(dead_code)]
const SHARP_CLAWS_STRIKE_NODE: &str = "sharp_claws_strike";
#[allow(dead_code)]
const BABY_FLAME_IMPACT_NODE: &str = "baby_flame_impact";
#[allow(dead_code)]
const BABY_BURNER_LAUNCH_NODE: &str = "baby_burner_launch";
#[allow(dead_code)]
const BABY_BURNER_RECOVERY_NODE: &str = "baby_burner_recovery";

/// Path (relative to `assets/`) of Agumon's sprite atlas image.
const AGUMON_ATLAS_IMAGE_PATH: &str = "digimon/agumon_atlas.png";
/// Index into `AnimationClipHandles` of Agumon's clip (atlas geometry source).
const AGUMON_CLIP_INDEX: usize = 0;

/// Namespaced effect ids of Agumon's effects. Owned by this module (S04); the
/// engine keys its generic registries on these strings and never matches them.
const CHARGE_EFFECT_ID: &str = "baby_flame.charge";
const EMBER_EFFECT_ID: &str = "baby_flame.ember";
const PROJECTILE_EFFECT_ID: &str = "baby_flame.projectile";
const IMPACT_EFFECT_ID: &str = "baby_flame.impact";
const SHARP_CLAWS_EFFECT_ID: &str = "sharp_claws.slash";
const DETONATE_EFFECT_ID: &str = "baby_burner.detonate";

/// Path (relative to `assets/`) of Agumon's enoki Baby Flame charge orb.
const ENOKI_CHARGE_PATH: &str = "digimon/agumon/baby_flame_charge.particle.ron";
/// Path of Agumon's enoki Baby Flame ember swirl (continuous emitter).
const ENOKI_EMBER_PATH: &str = "digimon/agumon/baby_flame_ember.particle.ron";
/// Path of Agumon's enoki Baby Flame traveling projectile.
const ENOKI_PROJECTILE_PATH: &str = "digimon/agumon/baby_flame_projectile.particle.ron";
/// Path of Agumon's enoki Baby Flame impact burst.
const ENOKI_IMPACT_PATH: &str = "digimon/agumon/baby_flame_impact.particle.ron";
/// Path of Agumon's enoki Sharp Claws slash burst.
const ENOKI_SHARP_CLAWS_PATH: &str = "digimon/agumon/sharp_claws_slash.particle.ron";
/// Path of Agumon's enoki Baby Burner detonate burst.
const ENOKI_DETONATE_PATH: &str = "digimon/agumon/baby_burner_detonate.particle.ron";

/// Animation ticks the Baby Flame projectile emitter takes to travel caster->target
/// before chaining the impact burst. Mirrors the deleted quad projectile's
/// `ttl_ticks: 5` in `vfx.ron` so the flight feels identical.
const PROJECTILE_FLIGHT_TICKS: u32 = 5;

/// Authored `SpawnParticle` name -> the owned effect id(s) it spawns on node
/// enter. Pure data so it can be unit-tested without an `App`;
/// `register_agumon_on_enter_effects` copies it into the engine's
/// [`OnEnterEffectRegistry`]. The `baby_flame_charge` command fans out to the
/// charge orb plus the inward ember swirl.
fn on_enter_effect_specs() -> &'static [(&'static str, &'static [&'static str])] {
    &[
        ("baby_flame_charge", &[CHARGE_EFFECT_ID, EMBER_EFFECT_ID]),
        ("baby_flame_projectile", &[PROJECTILE_EFFECT_ID]),
        ("baby_flame_impact", &[IMPACT_EFFECT_ID]),
        ("sharp_claws_slash", &[SHARP_CLAWS_EFFECT_ID]),
    ]
}

/// Populate the engine registries with all Agumon-specific presentation data.
/// Called once from `crate::windowed::digimon::register_all`.
pub(in crate::windowed) fn register(app: &mut App) {
    app.add_systems(
        Startup,
        (
            register_agumon_cues,
            register_agumon_enoki_vfx,
            register_agumon_on_enter_effects,
            register_agumon_skill_release_effects,
            register_agumon_detonate_effect,
            register_agumon_skill_start_nodes,
            register_agumon_sprite_presentation,
            register_agumon_windowed_demo,
        ),
    );
}

/// Register the three Agumon-specific cosmetic cues with the legacy
/// `hit_feedback` const values: the colour flash, the positional sprite-shake,
/// and the camera-shake. Behaviour-preserving param sourcing (D048 model a) —
/// the parametric fns at these params are bit-for-bit identical to the legacy
/// `flash_tint`/`shake_offset`. Registration is collision-free (D047 panics on
/// a conflicting def).
fn register_agumon_cues(mut registry: ResMut<bevyrogue::ui::cues::CueRegistry>) {
    use bevyrogue::ui::cues::CueDef;
    registry.register(
        "hit_flash",
        CueDef::Flash {
            peak: (1.0, 0.45, 0.45),
            ticks: 8,
        },
    );
    registry.register(
        "hit_shake",
        CueDef::SpriteShake {
            amp: 4.0,
            freq_x: 1.7,
            freq_y: 2.3,
            ticks: 8,
        },
    );
    registry.register(
        "camera_impact",
        CueDef::CameraShake {
            amp: 4.0,
            freq_x: 1.7,
            freq_y: 2.3,
            ticks: 8,
        },
    );
}

/// Load Agumon's enoki `Particle2dEffect` handles and register each into the
/// engine-generic [`EnokiVfxRegistry`] with its anchor + lifecycle. Anchors mirror
/// the old `vfx.ron` placement so the enoki path reproduces the quad placement
/// exactly; charge/ember are persistent emitters, the projectile travels then
/// chains `baby_flame.impact`, and the contact bursts are fire-and-forget.
fn register_agumon_enoki_vfx(
    asset_server: Res<AssetServer>,
    mut registry: ResMut<EnokiVfxRegistry>,
) {
    let entries: [(&str, &str, PlacementAnchor, EnokiLifecycle); 6] = [
        (
            CHARGE_EFFECT_ID,
            ENOKI_CHARGE_PATH,
            PlacementAnchor::Mouth,
            EnokiLifecycle::PersistentEmitter,
        ),
        (
            EMBER_EFFECT_ID,
            ENOKI_EMBER_PATH,
            PlacementAnchor::Mouth,
            EnokiLifecycle::PersistentEmitter,
        ),
        (
            PROJECTILE_EFFECT_ID,
            ENOKI_PROJECTILE_PATH,
            PlacementAnchor::CasterCenter,
            EnokiLifecycle::Projectile {
                flight_ticks: PROJECTILE_FLIGHT_TICKS,
                on_arrival: IMPACT_EFFECT_ID.to_string(),
            },
        ),
        (
            SHARP_CLAWS_EFFECT_ID,
            ENOKI_SHARP_CLAWS_PATH,
            PlacementAnchor::TargetCenter,
            EnokiLifecycle::OneShot,
        ),
        (
            IMPACT_EFFECT_ID,
            ENOKI_IMPACT_PATH,
            PlacementAnchor::TargetCenter,
            EnokiLifecycle::OneShot,
        ),
        (
            DETONATE_EFFECT_ID,
            ENOKI_DETONATE_PATH,
            PlacementAnchor::TargetCenter,
            EnokiLifecycle::OneShot,
        ),
    ];
    for (effect_id, path, anchor, lifecycle) in entries {
        registry.handles.insert(
            effect_id.to_string(),
            EnokiEffect {
                handle: asset_server.load(path),
                anchor,
                path: path.to_string(),
                lifecycle,
            },
        );
    }
    info!(
        target: "windowed.agumon_playback",
        charge_path = ENOKI_CHARGE_PATH,
        ember_path = ENOKI_EMBER_PATH,
        projectile_path = ENOKI_PROJECTILE_PATH,
        sharp_claws_path = ENOKI_SHARP_CLAWS_PATH,
        impact_path = ENOKI_IMPACT_PATH,
        detonate_path = ENOKI_DETONATE_PATH,
        "agumon enoki effects load requested"
    );
}

/// Copy [`on_enter_effect_specs`] into the engine-generic [`OnEnterEffectRegistry`].
fn register_agumon_on_enter_effects(mut registry: ResMut<OnEnterEffectRegistry>) {
    for (name, ids) in on_enter_effect_specs() {
        registry.map.insert(
            (*name).to_string(),
            ids.iter().map(|id| (*id).to_string()).collect(),
        );
    }
}

/// Register Baby Flame's release effect: launching the flame spawns the traveling
/// projectile (the engine clears the charge/ember emitters at the same boundary).
fn register_agumon_skill_release_effects(mut registry: ResMut<SkillReleaseEffectRegistry>) {
    registry.map.insert(
        BABY_FLAME_SKILL_ID.to_string(),
        PROJECTILE_EFFECT_ID.to_string(),
    );
}

/// Register Baby Burner's detonate burst as the engine's detonate effect.
fn register_agumon_detonate_effect(mut registry: ResMut<DetonateEffectRegistry>) {
    registry.effect_id = Some(DETONATE_EFFECT_ID.to_string());
}

/// Authored skill id -> its windowed FSM entry node for each of Agumon's bridged
/// skills. Pure data so it can be unit-tested without an `App`;
/// `register_agumon_skill_start_nodes` copies it into the engine's
/// [`SkillStartNodeRegistry`]. A skill absent here is unbridged (auto-release
/// fallback in the engine).
fn skill_start_node_specs() -> &'static [(&'static str, &'static str)] {
    &[
        (SHARP_CLAWS_SKILL_ID, SHARP_CLAWS_WINDUP_NODE),
        (BABY_FLAME_SKILL_ID, BABY_FLAME_CAST_NODE),
        (AGUMON_ULT_SKILL_ID, BABY_BURNER_CHARGE_NODE),
    ]
}

/// Populate the engine-generic [`SkillStartNodeRegistry`] with Agumon's bridged
/// skill -> FSM entry node map, replacing the closed `skill_start_node` match.
fn register_agumon_skill_start_nodes(mut registry: ResMut<SkillStartNodeRegistry>) {
    for (skill_id, node) in skill_start_node_specs() {
        registry
            .map
            .insert((*skill_id).to_string(), (*node).to_string());
    }
}

/// Populate the engine-generic [`SpritePresentationRegistry`] with Agumon's
/// stance/skill graph ids and atlas image path + clip index, replacing the engine
/// `AGUMON_*` consts and the hardcoded atlas path in `build_digimon_atlas` /
/// `spawn_unit_sprites`.
fn register_agumon_sprite_presentation(mut registry: ResMut<SpritePresentationRegistry>) {
    registry.entries.push(SpritePresentationEntry {
        presentation_id: AGUMON_PRESENTATION_ID.to_string(),
        unit_ids: vec![UnitId(1), AGUMON_DUMMY_ID],
        stance_graph_id: AGUMON_STANCE_GRAPH_ID.to_string(),
        skill_graph_id: AGUMON_SKILL_GRAPH_ID.to_string(),
        atlas_image_path: AGUMON_ATLAS_IMAGE_PATH.to_string(),
        clip_index: AGUMON_CLIP_INDEX,
    });
}

fn register_agumon_windowed_demo(mut registry: ResMut<WindowedDemoRegistry>) {
    registry.entries.extend([
        WindowedDemoEntry {
            demo_id: "agumon_ally".to_string(),
            source_unit_id: UnitId(1),
            spawned_unit_id: UnitId(1),
            team: Team::Ally,
            name_override: None,
        },
        WindowedDemoEntry {
            demo_id: "agumon_dummy".to_string(),
            source_unit_id: UnitId(1),
            spawned_unit_id: AGUMON_DUMMY_ID,
            team: Team::Enemy,
            name_override: Some("Agumon (Dummy)".to_string()),
        },
    ]);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn on_enter_ids(name: &str) -> &'static [&'static str] {
        on_enter_effect_specs()
            .iter()
            .find(|(spec_name, _)| *spec_name == name)
            .map(|(_, ids)| *ids)
            .unwrap_or(&[])
    }

    #[test]
    fn on_enter_charge_seeds_both_the_orb_and_the_ember_swirl() {
        // The single authored `baby_flame_charge` SpawnParticle fans out to the
        // owned charge + ember effect ids; the projectile maps to its own id.
        assert_eq!(
            on_enter_ids("baby_flame_charge"),
            &[CHARGE_EFFECT_ID, EMBER_EFFECT_ID]
        );
        assert_eq!(
            on_enter_ids("baby_flame_projectile"),
            &[PROJECTILE_EFFECT_ID]
        );
        // An unknown particle name maps to no effects (spawns nothing, no panic).
        assert!(on_enter_ids("unknown_particle").is_empty());
    }

    #[test]
    fn on_enter_sharp_claws_maps_only_to_the_slash_effect() {
        // The `sharp_claws_slash` SpawnParticle maps to exactly the owned slash
        // effect id — proving the data-driven bridge, not a VFX-kind branch.
        assert_eq!(on_enter_ids("sharp_claws_slash"), &[SHARP_CLAWS_EFFECT_ID]);
        assert_eq!(SHARP_CLAWS_EFFECT_ID, "sharp_claws.slash");

        // Unrelated / near-miss names must NOT resolve to the Sharp Claws effect:
        // the bridge is an exact name map, not a substring/string-kind match.
        for name in [
            "sharp_claws",
            "slash",
            "baby_flame_charge",
            "sharp_claws_strike",
            "",
        ] {
            assert!(
                !on_enter_ids(name).contains(&SHARP_CLAWS_EFFECT_ID),
                "`{name}` must not map to the Sharp Claws effect id"
            );
        }
    }

    #[test]
    fn skill_release_maps_baby_flame_to_the_projectile_effect() {
        // The release boundary is data-driven: baby_flame launches the projectile,
        // and no other skill is registered to spawn a release effect.
        let mut app = App::new();
        app.init_resource::<SkillReleaseEffectRegistry>();
        app.add_systems(Startup, register_agumon_skill_release_effects);
        app.update();
        let reg = app.world().resource::<SkillReleaseEffectRegistry>();
        assert_eq!(
            reg.map.get(BABY_FLAME_SKILL_ID).map(String::as_str),
            Some(PROJECTILE_EFFECT_ID)
        );
        assert_eq!(reg.map.len(), 1);
    }

    #[test]
    fn detonate_registry_holds_the_baby_burner_burst() {
        let mut app = App::new();
        app.init_resource::<DetonateEffectRegistry>();
        app.add_systems(Startup, register_agumon_detonate_effect);
        app.update();
        let reg = app.world().resource::<DetonateEffectRegistry>();
        assert_eq!(reg.effect_id.as_deref(), Some(DETONATE_EFFECT_ID));
    }

    #[test]
    fn skill_start_nodes_map_each_bridged_skill_to_its_fsm_entry() {
        // The owned spec maps each bridged skill to its authored FSM entry node;
        // unbridged skills are simply absent (engine auto-release fallback).
        let specs = skill_start_node_specs();
        let lookup = |skill: &str| {
            specs
                .iter()
                .find(|(s, _)| *s == skill)
                .map(|(_, node)| *node)
        };
        assert_eq!(lookup(SHARP_CLAWS_SKILL_ID), Some(SHARP_CLAWS_WINDUP_NODE));
        assert_eq!(lookup(BABY_FLAME_SKILL_ID), Some(BABY_FLAME_CAST_NODE));
        assert_eq!(lookup(AGUMON_ULT_SKILL_ID), Some(BABY_BURNER_CHARGE_NODE));
        assert_eq!(lookup("greymon_basic"), None);
    }

    #[test]
    fn register_populates_the_skill_start_node_registry() {
        let mut app = App::new();
        app.init_resource::<SkillStartNodeRegistry>();
        app.add_systems(Startup, register_agumon_skill_start_nodes);
        app.update();
        let reg = app.world().resource::<SkillStartNodeRegistry>();
        assert_eq!(
            reg.map.get(AGUMON_ULT_SKILL_ID).map(String::as_str),
            Some(BABY_BURNER_CHARGE_NODE)
        );
        assert_eq!(reg.map.len(), 3);
    }

    /// Baby Flame and Baby Burner are bridged, so the engine's fallback auto-release
    /// branch must NOT fire for them — they release on their rendered `ReleaseKernel`
    /// cue. Only skills with no windowed FSM entry take the auto-release fallback.
    #[test]
    fn auto_release_fallback_only_targets_unbridged_skills() {
        use crate::windowed::render::should_auto_release_unbridged;
        let mut app = App::new();
        app.init_resource::<SkillStartNodeRegistry>();
        app.add_systems(Startup, register_agumon_skill_start_nodes);
        app.update();
        let reg = app.world().resource::<SkillStartNodeRegistry>();
        assert!(!should_auto_release_unbridged(reg, SHARP_CLAWS_SKILL_ID));
        assert!(!should_auto_release_unbridged(reg, BABY_FLAME_SKILL_ID));
        assert!(!should_auto_release_unbridged(reg, AGUMON_ULT_SKILL_ID));
        // An unbridged skill (no windowed presentation graph) still auto-releases.
        assert!(should_auto_release_unbridged(reg, "greymon_basic"));
    }

    #[test]
    fn register_populates_the_sprite_presentation_registry() {
        let mut app = App::new();
        app.init_resource::<SpritePresentationRegistry>();
        app.add_systems(Startup, register_agumon_sprite_presentation);
        app.update();
        let reg = app.world().resource::<SpritePresentationRegistry>();
        let entry = reg.entries.first().expect("agumon presentation entry");
        assert_eq!(entry.presentation_id, AGUMON_PRESENTATION_ID);
        assert_eq!(entry.unit_ids, vec![UnitId(1), AGUMON_DUMMY_ID]);
        assert_eq!(entry.stance_graph_id, AGUMON_STANCE_GRAPH_ID);
        assert_eq!(entry.skill_graph_id, AGUMON_SKILL_GRAPH_ID);
        assert_eq!(entry.atlas_image_path, AGUMON_ATLAS_IMAGE_PATH);
        assert_eq!(entry.clip_index, AGUMON_CLIP_INDEX);
    }

    #[test]
    fn register_populates_the_windowed_demo_registry() {
        let mut app = App::new();
        app.init_resource::<WindowedDemoRegistry>();
        app.add_systems(Startup, register_agumon_windowed_demo);
        app.update();
        let reg = app.world().resource::<WindowedDemoRegistry>();
        assert_eq!(reg.entries.len(), 2);
        assert_eq!(reg.entries[0].demo_id, "agumon_ally");
        assert_eq!(reg.entries[0].spawned_unit_id, UnitId(1));
        assert_eq!(reg.entries[0].team, Team::Ally);
        assert_eq!(reg.entries[1].demo_id, "agumon_dummy");
        assert_eq!(reg.entries[1].spawned_unit_id, AGUMON_DUMMY_ID);
        assert_eq!(reg.entries[1].team, Team::Enemy);
        assert_eq!(
            reg.entries[1].name_override.as_deref(),
            Some("Agumon (Dummy)")
        );
    }
}
