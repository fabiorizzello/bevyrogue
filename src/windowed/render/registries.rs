//! Engine-generic presentation registries and shared types extracted from
//! `render.rs` (M006/S09). Pure structural move — no behavior change. The
//! per-Digimon modules populate these registries via their `register` entry
//! points; the engine-side presentation systems in `render.rs` read them
//! without species-specific consts or closed matches. Visibility stays
//! `pub(in crate::windowed)` so existing call sites keep compiling once
//! `render.rs` re-exports this module.

use std::collections::HashMap;

use bevy::prelude::*;
use bevy_enoki::Particle2dEffect;
use bevy_enoki::prelude::SpriteParticle2dMaterial;

use bevyrogue::animation::PlacementAnchor;
use bevyrogue::combat::types::UnitId;

/// Per-effect-id registry of enoki `Particle2dEffect` handles. Engine-generic
/// (S04): the per-Digimon module populates it via its `register` entry point with
/// the effect id, asset path, placement anchor, and lifecycle for each effect; the
/// spawn seam ([`crate::windowed::render::spawn_effect_by_id`]) looks an id up here and routes it through
/// bevy_enoki's GPU 2D backend, choosing the spawned lifecycle components from the
/// entry's [`EnokiLifecycle`]. A missing id spawns nothing. Loading these handles
/// does not move any particle lifetime into the kernel/FSM timeline (D031/D032).
#[derive(Resource, Debug, Clone, Default)]
pub(in crate::windowed) struct EnokiVfxRegistry {
    pub(in crate::windowed) handles: HashMap<String, EnokiEffect>,
}

/// Shared handle to the soft-particle sprite material every enoki effect spawns
/// with. enoki's default `ParticleSpawner` is `ColorParticle2dMaterial` — a flat
/// frag (`in.color * color`) that draws every particle as a hard solid-color
/// square, so no `color_curve`/HDR tuning makes an effect read as fire/water. This
/// material is `SpriteParticle2dMaterial::from_texture(soft_particle.png)`, whose
/// frag multiplies the particle color by the texture's radial alpha
/// (`particle_sprite_frag.wgsl`), turning each quad into a soft round blob;
/// overlapping blobs + HDR bloom read as a glowing body. Built once at Startup
/// ([`crate::windowed::render::init_soft_particle_material`]) and cloned per spawn in [`crate::windowed::render::spawn_effect_by_id`].
/// Both material plugins are already registered by `EnokiPlugin`, so this is purely
/// a spawn-site swap (no new plugin). See the `bevy-enoki-vfx` skill.
#[derive(Resource, Debug, Clone)]
pub(in crate::windowed) struct SoftParticleMaterial(
    pub(in crate::windowed) Handle<SpriteParticle2dMaterial>,
);

/// One entry in [`EnokiVfxRegistry`]: the loaded enoki effect handle, the source
/// asset `path` (carried for diagnostics so `diagnose_enoki_vfx_load` reports the
/// failing path without a const match), the placement `anchor`, and the
/// `lifecycle` that decides which lifecycle components the spawn seam attaches.
#[derive(Debug, Clone)]
pub(in crate::windowed) struct EnokiEffect {
    pub(in crate::windowed) handle: Handle<Particle2dEffect>,
    pub(in crate::windowed) anchor: PlacementAnchor,
    pub(in crate::windowed) path: String,
    pub(in crate::windowed) lifecycle: EnokiLifecycle,
}

/// How a spawned enoki effect behaves over time. Replaces the closed effect-id
/// match in [`crate::windowed::render::spawn_effect_by_id`] with data carried per registry entry (S04):
/// `PersistentEmitter` keeps emitting until cleared by marker at a launch
/// boundary, `Projectile` travels caster->target then chains `on_arrival`, and
/// `OneShot` is fire-and-forget and self-despawns once it drains.
#[derive(Debug, Clone)]
pub(in crate::windowed) enum EnokiLifecycle {
    PersistentEmitter,
    Projectile {
        flight_ticks: u32,
        on_arrival: String,
    },
    OneShot,
}

/// Engine-generic map of authored `SpawnParticle` name -> the owned effect id(s)
/// it spawns on node enter (S04). The per-Digimon module populates it; the
/// `on_enter` loop in [`crate::windowed::render::advance_digimon_presentation`] reads it instead of a closed
/// name match, so adding a Digimon's spawn vocabulary needs no engine edit.
#[derive(Resource, Debug, Clone, Default)]
pub(in crate::windowed) struct OnEnterEffectRegistry {
    pub(in crate::windowed) map: HashMap<String, Vec<String>>,
}

/// Engine-generic map of skill id -> the effect id spawned at the skill's release
/// boundary (S04). Replaces the `mode_skill_id == Some(BABY_FLAME_SKILL_ID)`
/// special-case in [`crate::windowed::render::advance_digimon_presentation`]: a skill present here spawns its
/// mapped effect (the projectile) on release; a skill absent here spawns nothing.
#[derive(Resource, Debug, Clone, Default)]
pub(in crate::windowed) struct SkillReleaseEffectRegistry {
    pub(in crate::windowed) map: HashMap<String, String>,
}

/// Engine-generic detonate effect id (S04). Replaces the per-Digimon detonate
/// const read in [`crate::windowed::render::spawn_detonate_particles`]: `None` spawns no detonate burst.
#[derive(Resource, Debug, Clone, Default)]
pub(in crate::windowed) struct DetonateEffectRegistry {
    pub(in crate::windowed) effect_id: Option<String>,
}

/// Engine-generic map of skill id -> its windowed FSM entry node (S04). The
/// per-Digimon module populates it; a skill present here is "bridged" (presents
/// its rendered FSM and releases on its `ReleaseKernel` cue), a skill absent here
/// is unbridged and takes the auto-release fallback. Replaces the closed
/// `skill_start_node` match: the presentation systems read this registry directly.
#[derive(Resource, Debug, Clone, Default)]
pub(in crate::windowed) struct SkillStartNodeRegistry {
    pub(in crate::windowed) map: HashMap<String, String>,
}

/// Engine-generic per-Digimon sprite presentation data (S04): the stance/skill
/// animation-graph ids and the atlas image path + clip index used by
/// [`crate::windowed::render::build_digimon_atlases`] / [`crate::windowed::render::spawn_unit_sprites`]. The per-Digimon module
/// populates it instead of the engine reading species-specific consts and a
/// hardcoded atlas path. For S04 it holds the single Agumon entry; S05 adds more.
#[derive(Resource, Debug, Clone, Default)]
pub(in crate::windowed) struct SpritePresentationRegistry {
    pub(in crate::windowed) entries: Vec<SpritePresentationEntry>,
}

/// One entry in [`SpritePresentationRegistry`]: the stable presentation id,
/// owned `UnitId` selectors, stance/skill graph ids, and atlas source used to
/// spawn/render a specific Digimon presentation without engine-side species
/// matches.
#[derive(Debug, Clone)]
pub(in crate::windowed) struct SpritePresentationEntry {
    pub(in crate::windowed) presentation_id: String,
    pub(in crate::windowed) unit_ids: Vec<UnitId>,
    pub(in crate::windowed) stance_graph_id: String,
    pub(in crate::windowed) skill_graph_id: String,
    pub(in crate::windowed) atlas_image_path: String,
    pub(in crate::windowed) clip_index: usize,
}

impl SpritePresentationEntry {
    pub(in crate::windowed) fn matches_unit(&self, unit_id: UnitId) -> bool {
        self.unit_ids.contains(&unit_id)
    }
}
