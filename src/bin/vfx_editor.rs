//! `vfx_editor` — standalone live preview for bevy_enoki `.particle.ron` effects.
//!
//! Windowed-only, manual-use tool (K001: never launched from auto-mode). It
//! renders a chosen `.particle.ron` on the SAME render path as the real combat
//! renderer — `Camera2d + Hdr + Bloom::NATURAL + Tonemapping::TonyMcMapface` —
//! so the overbright/bloom look is faithful to what ships. Hot-reload is on
//! (the base build enables bevy's `file_watcher`): edit the `.ron` on disk and
//! the preview updates live. This is the human-in-the-loop authoring loop the
//! `bevy-enoki-vfx` skill assumes — there is no headless VFX preview (the look
//! only exists in the windowed render stack).
//!
//! Controls (also drawn on screen):
//!   1 / 2          select fire / water preset
//!   Left / Right   cycle presets
//!   R              force-reload the current effect from disk
//!   Space          respawn the emitter (clean restart)
//!   Esc            quit
//!
//! Run: `cargo run --features windowed --bin vfx_editor`

use bevy::{
    asset::LoadState,
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::Bloom,
    prelude::*,
    render::view::Hdr,
};
use bevy_enoki::prelude::{ParticleEffectInstance, ParticleSpawnerState, ParticleStore};
use bevy_enoki::{EnokiPlugin, Particle2dEffect, ParticleEffectHandle, ParticleSpawner};

/// The previewable effects. Each is `(on-screen label, asset path under assets/)`.
/// Add a row here to expose another `.particle.ron` in the editor.
const PRESETS: &[(&str, &str)] = &[
    ("fire", "vfx/fire_test.particle.ron"),
    ("water", "vfx/water_test.particle.ron"),
];

/// Which preset is currently shown.
#[derive(Resource)]
struct Selected(usize);

/// Handle to the currently-loaded effect asset (kept alive + reloadable).
#[derive(Resource)]
struct CurrentHandle(Handle<Particle2dEffect>);

/// Marker on the live emitter entity so a preset switch can despawn it.
#[derive(Component)]
struct PreviewSpawner;

/// Marker on the on-screen status line so input handlers can rewrite it.
#[derive(Component)]
struct StatusText;

/// Set once we've logged a load failure for the current handle, so the warning
/// fires a single time instead of every frame the load stays `Failed`.
#[derive(Resource, Default)]
struct LoadFailureLogged(bool);

/// Throttle for the once-per-second emitter diagnostic so the log is readable.
#[derive(Resource)]
struct DiagTimer(Timer);

impl Default for DiagTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0, TimerMode::Repeating))
    }
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "bevyrogue — enoki VFX editor".into(),
                    ..default()
                }),
                ..default()
            }),
        )
        .add_plugins(EnokiPlugin)
        // Dark, slightly cool backdrop so both warm (fire) and cool (water)
        // effects read against it; matches the "value contrast first" rule.
        .insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.04)))
        .insert_resource(Selected(0))
        .init_resource::<LoadFailureLogged>()
        .init_resource::<DiagTimer>()
        .add_systems(Startup, setup)
        // Order matters: react to reloads (despawn) first, then (re)spawn once the
        // asset is ready, then process input. Keeps the spawn lifecycle coherent
        // within a single frame.
        .add_systems(
            Update,
            (respawn_on_reload, ensure_spawner_when_ready, handle_input).chain(),
        )
        .add_systems(Update, diagnose_emitter)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, selected: Res<Selected>) {
    // Same HDR/bloom/tonemapping stack as `windowed::render::setup_camera`, so
    // overbright color-curve channels (> 1.0) bloom exactly as they will in combat.
    commands.spawn((
        Camera2d,
        Hdr,
        Bloom::NATURAL,
        Tonemapping::TonyMcMapface,
        DebandDither::Enabled,
    ));

    // Kick off the async load but do NOT spawn the emitter here. enoki's
    // `clone_effect` copies the effect into the spawner only on the frame the
    // `ParticleSpawnerState` is `Added`, and on `Startup` the `.particle.ron` has
    // not finished loading yet — so an emitter spawned now would latch an empty
    // effect and never emit. `ensure_spawner_when_ready` spawns it once the asset
    // reaches `LoadState::Loaded`.
    let (_, path) = PRESETS[selected.0];
    let handle: Handle<Particle2dEffect> = asset_server.load(path);
    commands.insert_resource(CurrentHandle(handle));

    // On-screen control legend + current preset (absolute-positioned overlay).
    commands.spawn((
        StatusText,
        Text::new(status_line(selected.0)),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.85, 0.85, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

/// Spawn the enoki emitter for `handle` at the origin. Continuous effects
/// (`spawn_rate > 0`) loop forever; no `OneShot` is attached.
fn spawn_preview(commands: &mut Commands, handle: &Handle<Particle2dEffect>) {
    commands.spawn((
        PreviewSpawner,
        ParticleSpawner::default(),
        ParticleEffectHandle(handle.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

/// Spawn the preview emitter exactly when its effect asset is ready, and never
/// before. This is the fix for "the effect doesn't show": enoki's `clone_effect`
/// copies the `Particle2dEffect` into the spawner only on the frame the spawner's
/// state is `Added`, so spawning against a still-loading handle latches an empty
/// effect forever. Spawning only on `LoadState::Loaded` guarantees `clone_effect`
/// sees the real asset. A `Failed` load is logged once (the editor's only
/// load-failure signal — there is no headless preview to fall back on, K001).
fn ensure_spawner_when_ready(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    current: Res<CurrentHandle>,
    spawners: Query<(), With<PreviewSpawner>>,
    mut failure_logged: ResMut<LoadFailureLogged>,
) {
    if !spawners.is_empty() {
        return;
    }
    match asset_server.load_state(current.0.id()) {
        LoadState::Loaded => {
            spawn_preview(&mut commands, &current.0);
            failure_logged.0 = false;
            info!(target: "vfx_editor", "effect loaded; spawned preview emitter");
        }
        LoadState::Failed(err) => {
            if !failure_logged.0 {
                failure_logged.0 = true;
                error!(
                    target: "vfx_editor",
                    "effect failed to load (no particles will show): {err}"
                );
            }
        }
        // Still loading: do nothing this frame, retry next.
        _ => {}
    }
}

/// Make hot-reload actually work. enoki only clones the effect into a spawner on
/// `Added<ParticleSpawnerState>`, so editing the `.ron` on disk (or pressing `R`)
/// does NOT update a live emitter on its own. On a `Modified`/reload event for the
/// current handle, despawn the emitter; `ensure_spawner_when_ready` respawns it
/// next frame, re-triggering the clone with the freshly-parsed effect.
fn respawn_on_reload(
    mut commands: Commands,
    mut events: MessageReader<AssetEvent<Particle2dEffect>>,
    current: Res<CurrentHandle>,
    spawners: Query<Entity, With<PreviewSpawner>>,
) {
    let current_id = current.0.id();
    let mut should_respawn = false;
    for event in events.read() {
        match event {
            AssetEvent::Modified { id } | AssetEvent::LoadedWithDependencies { id }
                if *id == current_id =>
            {
                should_respawn = true;
            }
            _ => {}
        }
    }
    if should_respawn {
        for e in &spawners {
            commands.entity(e).despawn();
        }
    }
}

/// Once per second, report the live emitter's pipeline state so a black screen
/// can be bisected without a debugger (K001: the editor is the only place these
/// particles render, so this log is the diagnostic of record). Three signals:
///   - `instance`  — has enoki cloned the asset into this spawner yet? `false`
///     means a load/clone-timing miss (no effect to emit).
///   - `active`    — is the spawner still emitting? `false` on a continuous
///     effect would be unexpected (OneShot is never attached here).
///   - `particles` — live count in the `ParticleStore`. `> 0` with a black
///     screen isolates the fault to rendering, not simulation.
fn diagnose_emitter(
    time: Res<Time>,
    mut timer: ResMut<DiagTimer>,
    emitter: Query<
        (&ParticleEffectInstance, &ParticleSpawnerState, &ParticleStore),
        With<PreviewSpawner>,
    >,
) {
    if !timer.0.tick(time.delta()).just_finished() {
        return;
    }
    match emitter.single() {
        Ok((instance, state, store)) => {
            info!(
                target: "vfx_editor",
                instance = instance.0.is_some(),
                active = state.active,
                particles = store.len(),
                "emitter status"
            );
        }
        Err(_) => {
            info!(target: "vfx_editor", "no live emitter yet (asset still loading?)");
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_input(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
    mut selected: ResMut<Selected>,
    spawners: Query<Entity, With<PreviewSpawner>>,
    mut status: Query<&mut Text, With<StatusText>>,
    mut exit: MessageWriter<AppExit>,
    mut failure_logged: ResMut<LoadFailureLogged>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
        return;
    }

    // Reload the current effect from disk (manual hot-reload). `reload` emits a
    // `Modified` event that `respawn_on_reload` turns into a despawn + respawn, so
    // the freshly-parsed effect is re-cloned into the emitter.
    if keys.just_pressed(KeyCode::KeyR) {
        asset_server.reload(PRESETS[selected.0].1);
        return;
    }

    // Clean restart: despawn now; `ensure_spawner_when_ready` respawns once the
    // (already-loaded) effect is ready, re-triggering enoki's clone.
    if keys.just_pressed(KeyCode::Space) {
        for e in &spawners {
            commands.entity(e).despawn();
        }
        return;
    }

    // Select / cycle presets.
    let next = if keys.just_pressed(KeyCode::Digit1) {
        Some(0)
    } else if keys.just_pressed(KeyCode::Digit2) {
        Some(1.min(PRESETS.len() - 1))
    } else if keys.just_pressed(KeyCode::ArrowRight) {
        Some((selected.0 + 1) % PRESETS.len())
    } else if keys.just_pressed(KeyCode::ArrowLeft) {
        Some((selected.0 + PRESETS.len() - 1) % PRESETS.len())
    } else {
        None
    };

    let Some(next) = next else { return };
    if next == selected.0 {
        return;
    }
    selected.0 = next;

    // Despawn the old emitter and load the new effect. The emitter is NOT spawned
    // here — `ensure_spawner_when_ready` spawns it once the new handle is `Loaded`
    // (immediately next frame if it was cached, otherwise after the async load).
    for e in &spawners {
        commands.entity(e).despawn();
    }
    let handle: Handle<Particle2dEffect> = asset_server.load(PRESETS[next].1);
    commands.insert_resource(CurrentHandle(handle));
    // Allow a fresh load-failure log for the newly selected effect.
    failure_logged.0 = false;

    if let Ok(mut text) = status.single_mut() {
        *text = Text::new(status_line(next));
    }
}

/// The overlay text: current preset + the control legend.
fn status_line(selected: usize) -> String {
    let (label, path) = PRESETS[selected];
    format!(
        "effect: {label}  ({path})\n\
         [1] fire   [2] water   [<-/->] cycle\n\
         [R] reload from disk   [Space] respawn   [Esc] quit\n\
         hot-reload is ON: edit the .ron and watch it update",
    )
}
