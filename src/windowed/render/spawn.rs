//! Sprite spawn and atlas-build systems for the windowed presentation layer.
//!
//! Owns camera setup, soft-particle material initialisation ([`setup_camera`],
//! [`init_soft_particle_material`]), atlas binding ([`build_digimon_atlases`]),
//! and unit-sprite spawning ([`spawn_unit_sprites`]).

use std::collections::{HashMap, HashSet};

use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::Bloom,
    prelude::*,
    render::view::Hdr,
};
use bevy_enoki::prelude::SpriteParticle2dMaterial;

use bevyrogue::animation::{
    AnimGraph, AnimGraphId, AnimationClipHandles, AnimationClipLoadState, AtlasGeometry, Clip,
    StanceGraphRegistry,
};
use bevyrogue::combat::team::Team;
use bevyrogue::combat::types::UnitId;
use bevyrogue::combat::unit::Unit;

use super::{
    DigimonSprite, PresentationAtlas, PresentationAtlasRegistry, SpriteRest, CameraRest,
    SPRITE_DISPLAY_SCALE, TEAM_COLUMN_X, SLOT_Z_STEP,
    presentation_entry_for_unit,
};
use super::playback::slot_offset_y;
use super::registries::{SoftParticleMaterial, SpritePresentationRegistry};

pub(super) fn setup_camera(mut commands: Commands) {
    let transform = Transform::default();
    commands.spawn((
        Camera2d,
        Hdr,
        // Bloom intensity pushed above NATURAL (0.15) so white-hot HDR particle cores
        // spill warm light onto the scene (Baby Flame reference look). Kept in sync
        // with the standalone vfx_viewer camera (src/bin/vfx_viewer.rs setup_camera).
        Bloom { intensity: 0.30, ..Bloom::NATURAL },
        Tonemapping::TonyMcMapface,
        DebandDither::Enabled,
        transform,
        // Capture the camera's spawn translation so camera-shake restores to the
        // exact rest without drift (never accumulate — MEM094). Same anti-drift
        // pattern as SpriteRest.
        CameraRest {
            translation: transform.translation,
        },
    ));
}

/// Build the shared soft-particle [`SpriteParticle2dMaterial`] at Startup: load the
/// radial-gradient PNG, register the material asset, and store its handle in
/// [`SoftParticleMaterial`]. Spawning every effect through this material (instead of
/// enoki's flat-square `ColorParticle2dMaterial` default) is the single
/// highest-leverage VFX fix — see [`SoftParticleMaterial`] and the `bevy-enoki-vfx`
/// skill. The texture is generated deterministically by `scripts/gen_soft_particle.py`.
pub(super) fn init_soft_particle_material(
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<SpriteParticle2dMaterial>>,
    mut commands: Commands,
) {
    let texture = asset_server.load("vfx/soft_particle.png");
    let handle = materials.add(SpriteParticle2dMaterial::from_texture(texture));
    commands.insert_resource(SoftParticleMaterial(handle));
    info!(
        target: "windowed.digimon_playback",
        texture = "vfx/soft_particle.png",
        "soft-particle sprite material built; enoki effects spawn as soft blobs (not flat squares)"
    );
}

/// Builds the shared `PresentationAtlas` (image + `TextureAtlasLayout` + geometry)
/// once the agumon `Clip` is readable. Idempotent: returns early once the
/// resource exists, so it runs at most one effective build. Emits a one-time
/// `info!` describing the grid and a one-time `warn!` if the clip never becomes
/// readable or the atlas image fails to load.
#[allow(clippy::too_many_arguments)]
pub(super) fn build_digimon_atlases(
    mut atlases: ResMut<PresentationAtlasRegistry>,
    clip_load_state: Res<AnimationClipLoadState>,
    clip_handles: Option<Res<AnimationClipHandles>>,
    clips: Res<Assets<Clip>>,
    asset_server: Res<AssetServer>,
    presentation: Res<SpritePresentationRegistry>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut warned: Local<HashSet<String>>,
) {
    let Some(handles) = clip_handles else {
        return;
    };

    for entry in &presentation.entries {
        let image_warn_key = format!("image:{}", entry.presentation_id);
        if let Some(bound) = atlases.atlases.get(&entry.presentation_id) {
            if matches!(
                asset_server.load_state(bound.image.id()),
                bevy::asset::LoadState::Failed(_)
            ) && warned.insert(image_warn_key)
            {
                warn!(
                    target: "windowed.digimon_playback",
                    presentation_id = entry.presentation_id.as_str(),
                    path = entry.atlas_image_path.as_str(),
                    "digimon atlas image load failed — sprites will render blank"
                );
            }
            continue;
        }

        let clip = handles
            .0
            .get(entry.clip_index)
            .and_then(|handle| clips.get(handle));
        let Some(clip) = clip else {
            let clip_warn_key = format!("clip:{}", entry.presentation_id);
            if clip_load_state.ready && warned.insert(clip_warn_key) {
                warn!(
                    target: "windowed.digimon_playback",
                    presentation_id = entry.presentation_id.as_str(),
                    path = entry.atlas_image_path.as_str(),
                    clip_index = entry.clip_index,
                    "presentation clip not readable after load state ready; atlas binding deferred — sprites stay blank"
                );
            }
            continue;
        };

        let geometry = AtlasGeometry::from_clip_meta(&clip.meta);
        let layout = TextureAtlasLayout::from_grid(
            UVec2::new(geometry.frame_size.w, geometry.frame_size.h),
            geometry.columns,
            geometry.rows,
            None,
            None,
        );
        let layout = layouts.add(layout);
        let image = asset_server.load(entry.atlas_image_path.clone());

        if matches!(
            asset_server.load_state(image.id()),
            bevy::asset::LoadState::Failed(_)
        ) && warned.insert(image_warn_key)
        {
            warn!(
                target: "windowed.digimon_playback",
                presentation_id = entry.presentation_id.as_str(),
                path = entry.atlas_image_path.as_str(),
                "digimon atlas image load failed — sprites will render blank"
            );
        }

        info!(
            target: "windowed.digimon_playback",
            presentation_id = entry.presentation_id.as_str(),
            path = entry.atlas_image_path.as_str(),
            frame_w = geometry.frame_size.w,
            frame_h = geometry.frame_size.h,
            columns = geometry.columns,
            rows = geometry.rows,
            total_frames = geometry.total_frames,
            "presentation atlas built (TextureAtlasLayout + image bound)"
        );

        atlases.atlases.insert(
            entry.presentation_id.clone(),
            PresentationAtlas {
                image,
                layout,
                geometry,
            },
        );
    }
}

/// Spawns one `DigimonSprite` entity per unit that does not yet have one.
/// Runs every frame but is idempotent: once a sprite exists for a unit it is skipped.
/// Waits for the stance graph to be loaded before spawning anything.
pub(super) fn spawn_unit_sprites(
    mut commands: Commands,
    stance_reg: Res<StanceGraphRegistry>,
    graphs: Res<Assets<AnimGraph>>,
    atlases: Res<PresentationAtlasRegistry>,
    presentation: Res<SpritePresentationRegistry>,
    units: Query<(&Unit, &Team)>,
    sprites: Query<&DigimonSprite>,
    mut warned: Local<HashSet<String>>,
) {
    let spawned: HashSet<UnitId> = sprites.iter().map(|s| s.unit_id).collect();

    // Deterministic per-team slot assignment computed across ALL units (not just
    // the ones spawned this frame), so a multi-actor team fans out to distinct
    // positions instead of every member stacking at one (x, z). Slot is the
    // unit's index within its team after sorting by UnitId — stable regardless of
    // spawn order or which units already exist. Replaces the previous team-only
    // ±200 placement that collapsed two allies onto the same point.
    let slot_of: HashMap<UnitId, (usize, usize)> = {
        let mut roster: Vec<(UnitId, Team)> = units.iter().map(|(u, t)| (u.id, *t)).collect();
        roster.sort_by_key(|(id, _)| id.0);
        let mut map = HashMap::new();
        for assigned in [Team::Ally, Team::Enemy] {
            let members: Vec<UnitId> = roster
                .iter()
                .filter(|(_, t)| *t == assigned)
                .map(|(id, _)| *id)
                .collect();
            let count = members.len();
            for (slot, id) in members.into_iter().enumerate() {
                map.insert(id, (slot, count));
            }
        }
        map
    };

    for (unit, team) in &units {
        if spawned.contains(&unit.id) {
            continue;
        }

        let Some(entry) = presentation_entry_for_unit(&presentation, unit.id) else {
            let warn_key = format!("missing-presentation:{}", unit.id.0);
            if warned.insert(warn_key) {
                warn!(
                    target: "windowed.digimon_playback",
                    unit_id = ?unit.id,
                    "no sprite presentation registered for unit; sprite spawn deferred"
                );
            }
            continue;
        };

        let Some(stance_graph) = stance_reg
            .resolve_snapshot(&AnimGraphId(entry.stance_graph_id.clone().into()), &graphs)
        else {
            let warn_key = format!("missing-stance:{}", unit.id.0);
            if warned.insert(warn_key) {
                warn!(
                    target: "windowed.digimon_playback",
                    unit_id = ?unit.id,
                    presentation_id = entry.presentation_id.as_str(),
                    graph_id = entry.stance_graph_id.as_str(),
                    "stance graph not yet readable; sprite spawn deferred"
                );
            }
            continue;
        };

        let Some(atlas) = atlases.atlases.get(&entry.presentation_id) else {
            let warn_key = format!("missing-atlas:{}", unit.id.0);
            if warned.insert(warn_key) {
                warn!(
                    target: "windowed.digimon_playback",
                    unit_id = ?unit.id,
                    presentation_id = entry.presentation_id.as_str(),
                    path = entry.atlas_image_path.as_str(),
                    "presentation atlas binding unavailable; sprite spawn deferred"
                );
            }
            continue;
        };

        let flip_x = *team == Team::Enemy;
        let x = if flip_x { TEAM_COLUMN_X } else { -TEAM_COLUMN_X };
        let (slot, count) = slot_of.get(&unit.id).copied().unwrap_or((0, 1));
        let y = slot_offset_y(slot, count);
        let z = slot as f32 * SLOT_Z_STEP;
        commands.spawn((
            DigimonSprite::idle_for(
                unit.id,
                entry.presentation_id.clone(),
                stance_graph.clone(),
                entry.stance_graph_id.clone(),
                entry.skill_graph_id.clone(),
            ),
            Sprite {
                image: atlas.image.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: atlas.layout.clone(),
                    index: 0,
                }),
                flip_x,
                ..default()
            },
            Transform::from_xyz(x, y, z).with_scale(Vec3::splat(SPRITE_DISPLAY_SCALE)),
            SpriteRest {
                xy: Vec2::new(x, y),
            },
        ));
    }
}
