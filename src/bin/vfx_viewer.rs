//! Standalone particle VFX viewer (windowed-only).
//!
//! A focused test bed for `.particle.ron` effects, isolated from the rest of the
//! game. It reproduces the *exact* render setup the combat renderer uses — HDR
//! `Camera2d` + `Bloom::NATURAL` + `TonyMcMapface` tonemapping + the soft-sprite
//! particle material (`vfx/soft_particle.png`) — so what you see here is what the
//! game draws. This is the only authoritative aesthetic test bed outside a full
//! `cargo run --features windowed`:
//!
//! - The **web editor** (lommix.github.io/bevy_enoki) renders with its own material
//!   and no bloom, and loads one file at a time, so it cannot show soft material,
//!   HDR glow, or a layered composite.
//! - Headless / SwiftShader capture renders flat 1px dots with no bloom.
//!
//! The dropdown enumerates every `.particle.ron` under `assets/` and lets you view
//! each layer singly, plus a set of **composite presets** that co-spawn the same
//! layer groups the game does (e.g. Baby Flame = flames + charge + core + ember on
//! one anchor) — the thing no single-file editor can show.
//!
//! Run (K001: a human runs this, never auto-mode):
//!   cargo run --features windowed --bin vfx_viewer
//!
//! Controls: dropdown to pick an effect · Left/Right arrows cycle · R reloads.

use std::path::Path;

use bevy::{
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::Bloom,
    prelude::*,
    render::view::Hdr,
};
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};
use bevy_enoki::prelude::{ParticleStore, SpriteParticle2dMaterial};
use bevy_enoki::{EnokiPlugin, Particle2dEffect, ParticleEffectHandle, ParticleSpawner};

/// Base Z for spawned layers; each layer steps forward so back-to-front alpha
/// blending matches the order in a composite group.
const LAYER_Z_BASE: f32 = 0.0;
const LAYER_Z_STEP: f32 = 0.5;

/// Composite presets — layer groups co-spawned together, mirroring the game's
/// `OnEnterEffectRegistry` fan-out (`src/windowed/digimon/agumon/mod.rs`).
/// Source of truth for the real groups lives there; this list is the dev-viewer
/// projection. Paths are relative to `assets/`. Order is back-to-front.
const COMPOSITES: &[(&str, &[&str])] = &[
    (
        "Baby Flame — charge body (flames+charge+core+ember)",
        &[
            "digimon/agumon/baby_flame_flames.particle.ron",
            "digimon/agumon/baby_flame_charge.particle.ron",
            "digimon/agumon/baby_flame_core.particle.ron",
            "digimon/agumon/baby_flame_ember.particle.ron",
        ],
    ),
    (
        "Baby Flame — spit + trail (projectile)",
        &["digimon/agumon/baby_flame_projectile.particle.ron"],
    ),
    (
        "Baby Flame — impact flames + dissolve",
        &["digimon/agumon/baby_flame_impact.particle.ron"],
    ),
];

/// Asset-relative paths of the "defined flame" layers that spawn through the
/// flipbook flame material (mirrors the `flame()` assignment in the Agumon enoki
/// registry — `src/windowed/digimon/agumon/mod.rs`). Every other layer spawns
/// through the soft-blob material. A dev-viewer projection of the game's
/// per-effect material routing; the source of truth is the registry.
const FLAME_FLIPBOOK_LAYERS: &[&str] = &[
    "digimon/agumon/baby_flame_flames.particle.ron",
    "digimon/agumon/baby_flame_projectile.particle.ron",
    "digimon/agumon/baby_flame_impact.particle.ron",
];

fn layer_uses_flame_flipbook(path: &str) -> bool {
    FLAME_FLIPBOOK_LAYERS.contains(&path)
}

/// One selectable entry in the dropdown: one or more `.particle.ron` layers.
#[derive(Clone)]
struct EffectEntry {
    label: String,
    /// Asset-relative paths (forward-slash), in spawn order.
    layers: Vec<String>,
}

#[derive(Resource, Default)]
struct ViewerState {
    effects: Vec<EffectEntry>,
    selected: usize,
    /// Index whose layers are currently spawned (None until first load).
    active: Option<usize>,
    /// Live handles for the active entry's layers (kept for hot-reload matching).
    handles: Vec<Handle<Particle2dEffect>>,
    /// True once the active entry's spawners have been created.
    spawned: bool,
    /// Set by the UI / hot-reload to request a (re)load of `selected`.
    needs_load: bool,
}

/// The soft-sprite material handle — identical to the combat renderer's.
#[derive(Resource)]
struct SoftMaterial(Handle<SpriteParticle2dMaterial>);

/// The "defined flame" flipbook material — identical to the combat renderer's
/// (`SpriteParticle2dMaterial::new(flame_sheet.png, 4, 4)`). Routed onto the flame
/// layers ([`FLAME_FLIPBOOK_LAYERS`]) so the viewer shows the animated flame-tongue
/// silhouette, not a soft blob.
#[derive(Resource)]
struct FlameMaterial(Handle<SpriteParticle2dMaterial>);

/// Tags an entity spawned by the viewer so we can clear it on selection change.
#[derive(Component)]
struct PreviewLayer;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "bevyrogue — VFX viewer".into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    // Live edit → reload of the open .particle.ron.
                    watch_for_changes_override: Some(true),
                    ..default()
                }),
        )
        .add_plugins(EnokiPlugin)
        .add_plugins(EguiPlugin::default())
        // Dim neutral background so HDR bloom reads honestly.
        .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.04)))
        .init_resource::<ViewerState>()
        .add_systems(
            Startup,
            (setup_camera, init_soft_material, init_flame_material, enumerate_effects),
        )
        .add_systems(
            Update,
            (keyboard_controls, apply_selection, ensure_spawner_when_ready, hot_reload),
        )
        .add_systems(EguiPrimaryContextPass, viewer_ui)
        .run();
}

/// Faithful copy of the combat renderer's camera (`src/windowed/render.rs`
/// `setup_camera`): HDR + bloom + tonemapping is what makes the >1.0 HDR core
/// channels glow. Without it, the soft particles render dim and flat.
fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Hdr,
        // Bloom intensity pushed above NATURAL (0.15) so the white-hot HDR cores
        // spill warm light onto the scene like the Baby Flame reference. Kept in
        // sync with the combat renderer's camera (render/spawn.rs setup_camera).
        Bloom { intensity: 0.30, ..Bloom::NATURAL },
        Tonemapping::TonyMcMapface,
        DebandDither::Enabled,
    ));
}

/// Builds the soft-sprite material from `vfx/soft_particle.png` — the same atom
/// the combat renderer spawns with, so particles read as glowing blobs, not the
/// default `ColorParticle2dMaterial` flat squares.
fn init_soft_material(
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<SpriteParticle2dMaterial>>,
    mut commands: Commands,
) {
    let texture = asset_server.load("vfx/soft_particle.png");
    let handle = materials.add(SpriteParticle2dMaterial::from_texture(texture));
    commands.insert_resource(SoftMaterial(handle));
}

/// Builds the flame flipbook material from `vfx/flame_sheet.png` (4x4) — the same
/// material the combat renderer routes onto Agumon's defined-flame layers. The frag
/// advances the 16 frames over each particle's lifetime, so a flame layer renders a
/// flickering tongue silhouette instead of a blob.
fn init_flame_material(
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<SpriteParticle2dMaterial>>,
    mut commands: Commands,
) {
    let texture = asset_server.load("vfx/flame_sheet.png");
    let handle = materials.add(SpriteParticle2dMaterial::new(texture, 4, 4));
    commands.insert_resource(FlameMaterial(handle));
}

/// Scans `assets/` for every `*.particle.ron`, then prepends the composite
/// presets. Runs once at startup. The viewer's CWD is the package root (cargo
/// sets it), so `assets/` resolves directly.
fn enumerate_effects(mut state: ResMut<ViewerState>) {
    let mut singles: Vec<String> = Vec::new();
    collect_particle_files(Path::new("assets"), &mut singles);
    singles.sort();

    let mut effects: Vec<EffectEntry> = Vec::new();

    // Composites first — they are the reason this viewer exists.
    for (label, layers) in COMPOSITES {
        effects.push(EffectEntry {
            label: format!("[composite] {label}"),
            layers: layers.iter().map(|s| (*s).to_string()).collect(),
        });
    }

    // Then every single file, labelled by its asset-relative path sans suffix.
    for rel in singles {
        let label = rel
            .strip_suffix(".particle.ron")
            .unwrap_or(&rel)
            .to_string();
        effects.push(EffectEntry { label, layers: vec![rel] });
    }

    if effects.is_empty() {
        warn!(
            "vfx_viewer: no .particle.ron found under assets/ (cwd={:?})",
            std::env::current_dir().ok()
        );
    } else {
        info!("vfx_viewer: {} effects available", effects.len());
    }

    state.effects = effects;
    state.needs_load = !state.effects.is_empty();
}

/// Recursively collects asset-relative paths (forward-slash) of `*.particle.ron`.
fn collect_particle_files(dir: &Path, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_particle_files(&path, out);
        } else if path.to_string_lossy().ends_with(".particle.ron") {
            if let Some(rel) = asset_relative(&path) {
                out.push(rel);
            }
        }
    }
}

/// Strips the leading `assets/` and normalises to forward slashes for AssetServer.
fn asset_relative(path: &Path) -> Option<String> {
    let rel = path.strip_prefix("assets").ok()?;
    Some(rel.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/"))
}

/// Arrow keys cycle the selection; R forces a reload of the current effect.
fn keyboard_controls(keys: Res<ButtonInput<KeyCode>>, mut state: ResMut<ViewerState>) {
    if state.effects.is_empty() {
        return;
    }
    let count = state.effects.len();
    if keys.just_pressed(KeyCode::ArrowRight) {
        state.selected = (state.selected + 1) % count;
        state.needs_load = true;
    }
    if keys.just_pressed(KeyCode::ArrowLeft) {
        state.selected = (state.selected + count - 1) % count;
        state.needs_load = true;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        state.needs_load = true;
    }
}

/// On a load request: despawn the previous layers and (re)load the selected
/// entry's handles. Actual spawning waits for the assets to finish loading.
fn apply_selection(
    mut state: ResMut<ViewerState>,
    asset_server: Res<AssetServer>,
    existing: Query<Entity, With<PreviewLayer>>,
    mut commands: Commands,
) {
    if !state.needs_load || state.effects.is_empty() {
        return;
    }
    for entity in &existing {
        commands.entity(entity).despawn();
    }

    let idx = state.selected.min(state.effects.len() - 1);
    // Own the paths before loading: AssetServer::load wants AssetPath<'static>,
    // so it can't borrow from `state` while we mutate `state` below.
    let paths: Vec<String> = state.effects[idx].layers.clone();
    let handles: Vec<Handle<Particle2dEffect>> = paths
        .into_iter()
        .map(|path| asset_server.load(path))
        .collect();

    state.handles = handles;
    state.active = Some(idx);
    state.spawned = false;
    state.needs_load = false;
}

/// Once every layer asset is loaded, spawn a soft-material `ParticleSpawner` per
/// layer at the origin (the game co-spawns all layers on one anchor). enoki only
/// clones the effect on `Added<ParticleSpawnerState>`, so spawning the entity is
/// what kicks emission off.
fn ensure_spawner_when_ready(
    mut state: ResMut<ViewerState>,
    asset_server: Res<AssetServer>,
    soft: Option<Res<SoftMaterial>>,
    flame: Option<Res<FlameMaterial>>,
    mut commands: Commands,
) {
    if state.spawned || state.handles.is_empty() {
        return;
    }
    let (Some(soft), Some(flame)) = (soft, flame) else { return };

    let all_ready = state
        .handles
        .iter()
        .all(|h| matches!(asset_server.load_state(h.id()), bevy::asset::LoadState::Loaded));
    if !all_ready {
        return;
    }

    // Layers and handles are parallel by index (apply_selection builds handles from
    // the active entry's layers in order), so the layer path selects the material:
    // the defined-flame layers use the flipbook, everything else the soft blob.
    let layers = state
        .active
        .map(|idx| state.effects[idx].layers.clone())
        .unwrap_or_default();
    for (i, handle) in state.handles.iter().enumerate() {
        let z = LAYER_Z_BASE + (i as f32) * LAYER_Z_STEP;
        let material = match layers.get(i) {
            Some(path) if layer_uses_flame_flipbook(path) => flame.0.clone(),
            _ => soft.0.clone(),
        };
        commands.spawn((
            PreviewLayer,
            ParticleSpawner(material),
            ParticleEffectHandle(handle.clone()),
            Transform::from_xyz(0.0, 0.0, z),
        ));
    }
    state.spawned = true;
}

/// Live file edits: when one of the active handles is modified, re-trigger a load
/// so the despawn+respawn cycle picks up the new asset (enoki won't re-clone an
/// existing spawner in place).
fn hot_reload(
    mut events: MessageReader<AssetEvent<Particle2dEffect>>,
    mut state: ResMut<ViewerState>,
) {
    for event in events.read() {
        if let AssetEvent::Modified { id } = event {
            if state.handles.iter().any(|h| h.id() == *id) {
                state.needs_load = true;
            }
        }
    }
}

/// The egui overlay: effect dropdown, layer list, and the authoritative live
/// particle count (the signal that exposed the `spawn_rate`-is-an-interval bug).
fn viewer_ui(
    mut contexts: EguiContexts,
    mut state: ResMut<ViewerState>,
    stores: Query<&ParticleStore, With<PreviewLayer>>,
) -> Result {
    let ctx = contexts.ctx_mut()?;

    let live: usize = stores.iter().map(|s| s.particles.len()).sum();
    let layer_count = stores.iter().count();

    egui::Window::new("VFX Viewer")
        .default_pos(egui::pos2(12.0, 12.0))
        .show(ctx, |ui| {
            if state.effects.is_empty() {
                ui.label("No .particle.ron found under assets/.");
                return;
            }

            let selected = state.selected.min(state.effects.len() - 1);
            let current_label = state.effects[selected].label.clone();

            let mut new_selection = selected;
            egui::ComboBox::from_label("Effect")
                .selected_text(current_label)
                .width(360.0)
                .show_ui(ui, |ui| {
                    for (i, entry) in state.effects.iter().enumerate() {
                        ui.selectable_value(&mut new_selection, i, entry.label.as_str());
                    }
                });
            if new_selection != selected {
                state.selected = new_selection;
                state.needs_load = true;
            }

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Reload (R)").clicked() {
                    state.needs_load = true;
                }
                ui.label("· ←/→ to cycle");
            });

            ui.separator();
            ui.label(format!("Spawners alive: {layer_count}"));
            ui.label(
                egui::RichText::new(format!("Live particles: {live}"))
                    .strong()
                    .color(if live == 0 {
                        egui::Color32::from_rgb(220, 120, 120)
                    } else {
                        egui::Color32::from_rgb(140, 220, 140)
                    }),
            );
            if live == 0 {
                ui.label(
                    egui::RichText::new("0 live → check spawn_rate (it's an INTERVAL: 1/rate)")
                        .small()
                        .italics(),
                );
            }

            ui.separator();
            ui.label("Layers:");
            for layer in &state.effects[selected].layers {
                ui.label(format!("  • {layer}"));
            }
        });

    Ok(())
}
