use std::collections::HashSet;

use bevy::prelude::*;

use bevyrogue::animation::{
    AnimGraph, AnimGraphId, AnimGraphPlayer, AnimationClipHandles, AnimationClipLoadState,
    AnimationGraphLookupDiagnostics, AtlasGeometry, Clip, FrameCueCommand, NodeId,
    ResolvedAnimGraph, ResolvedAnimGraphSource, SkillGraphRegistry, StanceGraphRegistry,
};
use bevyrogue::combat::runtime::{CueBarrierStatus, CueReleaseResult, SuspendedTimelineState};
use bevyrogue::combat::team::Team;
use bevyrogue::combat::turn_system::{continue_suspended_timeline_system, resolve_action_system};
use bevyrogue::combat::types::UnitId;
use bevyrogue::combat::unit::Unit;

use super::{
    AGUMON_SKILL_GRAPH_ID, AGUMON_STANCE_GRAPH_ID, SHARP_CLAWS_SKILL_ID, SHARP_CLAWS_STRIKE_NODE,
    SHARP_CLAWS_WINDUP_NODE,
};

/// Marker + FSM state for the on-screen Agumon preview actor.
#[derive(Component, Debug, Clone)]
pub(super) struct AgumonSprite {
    pub(super) unit_id: UnitId,
    pub(super) player: AnimGraphPlayer,
    graph: ResolvedAnimGraph,
    mode: AgumonPlaybackMode,
    last_release_frame: Option<ReleaseFrameKey>,
    last_missing_skill_graph_cue: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AgumonPlaybackMode {
    Idle,
    SharpClaws {
        cue_id: String,
        presentation_node: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ReleaseFrameKey {
    pub(super) cue_id: String,
    pub(super) node: String,
    pub(super) local_frame: u32,
}

impl AgumonSprite {
    pub(super) fn idle_for(unit_id: UnitId, graph: ResolvedAnimGraph) -> Self {
        let entry = graph.graph().entry.clone();
        Self {
            unit_id,
            player: AnimGraphPlayer::new(entry),
            graph,
            mode: AgumonPlaybackMode::Idle,
            last_release_frame: None,
            last_missing_skill_graph_cue: None,
        }
    }

    fn start_sharp_claws(
        &mut self,
        cue_id: &str,
        presentation_node: &str,
        graph: ResolvedAnimGraph,
    ) {
        self.player = AnimGraphPlayer::new(sharp_claws_start_node(presentation_node));
        self.graph = graph;
        self.mode = AgumonPlaybackMode::SharpClaws {
            cue_id: cue_id.to_string(),
            presentation_node: presentation_node.to_string(),
        };
        self.last_release_frame = None;
        self.last_missing_skill_graph_cue = None;
    }

    pub(super) fn return_to_idle(
        &mut self,
        graph: ResolvedAnimGraph,
        preserve_missing_skill_graph_cue: Option<String>,
    ) {
        let entry = graph.graph().entry.clone();
        self.player = AnimGraphPlayer::new(entry);
        self.graph = graph;
        self.mode = AgumonPlaybackMode::Idle;
        self.last_release_frame = None;
        self.last_missing_skill_graph_cue = preserve_missing_skill_graph_cue;
    }
}

/// Bound atlas handles + geometry for the on-screen Agumon sprites. Built once
/// the agumon `Clip` is readable; both actors share the same image/layout, and
/// the geometry drives the player-frame -> tile-index map each tick.
#[derive(Resource, Debug, Clone)]
pub(super) struct AgumonAtlas {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
    geometry: AtlasGeometry,
}

/// Sprite camera + Agumon presentation state machine. Feature-agnostic player,
/// windowed playback bridge.
pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_systems(Update, build_agumon_atlas.before(spawn_unit_sprites))
            .add_systems(Update, spawn_unit_sprites)
            .add_systems(
                Update,
                advance_agumon_presentation
                    .after(spawn_unit_sprites)
                    .after(resolve_action_system)
                    .before(continue_suspended_timeline_system),
            );
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Builds the shared `AgumonAtlas` (image + `TextureAtlasLayout` + geometry)
/// once the agumon `Clip` is readable. Idempotent: returns early once the
/// resource exists, so it runs at most one effective build. Emits a one-time
/// `info!` describing the grid and a one-time `warn!` if the clip never becomes
/// readable or the atlas image fails to load.
#[allow(clippy::too_many_arguments)]
fn build_agumon_atlas(
    mut commands: Commands,
    existing: Option<Res<AgumonAtlas>>,
    clip_load_state: Res<AnimationClipLoadState>,
    clip_handles: Option<Res<AnimationClipHandles>>,
    clips: Res<Assets<Clip>>,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut warned: Local<bool>,
) {
    if existing.is_some() {
        return;
    }

    let Some(handles) = clip_handles else {
        return;
    };

    // The agumon clip is index 0 (DEFAULT_ANIM_CLIP_PATHS) and is the geometry
    // source for both on-screen actors.
    let clip = handles.0.first().and_then(|handle| clips.get(handle));
    let Some(clip) = clip else {
        // Only a real failure (load state reports ready but the asset is
        // missing) is worth surfacing; the transient loading state is silent.
        if clip_load_state.ready && !*warned {
            warn!(
                target: "windowed.agumon_playback",
                "agumon clip not readable after load state ready; atlas binding deferred — sprites stay blank"
            );
            *warned = true;
        }
        return;
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
    let image = asset_server.load("digimon/agumon_atlas.png");

    if matches!(
        asset_server.load_state(image.id()),
        bevy::asset::LoadState::Failed(_)
    ) && !*warned
    {
        warn!(
            target: "windowed.agumon_playback",
            "agumon atlas image load failed: digimon/agumon_atlas.png — sprites will render blank"
        );
        *warned = true;
    }

    info!(
        target: "windowed.agumon_playback",
        frame_w = geometry.frame_size.w,
        frame_h = geometry.frame_size.h,
        columns = geometry.columns,
        rows = geometry.rows,
        total_frames = geometry.total_frames,
        "agumon atlas built (TextureAtlasLayout + image bound)"
    );

    commands.insert_resource(AgumonAtlas {
        image,
        layout,
        geometry,
    });
}

/// Spawns one `AgumonSprite` entity per unit that does not yet have one.
/// Runs every frame but is idempotent: once a sprite exists for a unit it is skipped.
/// Waits for the stance graph to be loaded before spawning anything.
fn spawn_unit_sprites(
    mut commands: Commands,
    stance_reg: Res<StanceGraphRegistry>,
    graphs: Res<Assets<AnimGraph>>,
    atlas: Option<Res<AgumonAtlas>>,
    units: Query<(&Unit, &Team)>,
    sprites: Query<&AgumonSprite>,
) {
    let Some(stance_graph) = stance_reg.resolve_snapshot(
        &AnimGraphId(AGUMON_STANCE_GRAPH_ID.into()),
        &graphs,
    ) else {
        return;
    };

    // Both the stance graph and the bound atlas must be ready before we can
    // spawn a sprite that renders real pixels.
    let Some(atlas) = atlas else {
        return;
    };

    let spawned: HashSet<UnitId> = sprites.iter().map(|s| s.unit_id).collect();

    for (unit, team) in &units {
        if spawned.contains(&unit.id) {
            continue;
        }
        let flip_x = *team == Team::Enemy;
        let x = if flip_x { 200.0_f32 } else { -200.0_f32 };
        commands.spawn((
            AgumonSprite::idle_for(unit.id, stance_graph.clone()),
            Sprite {
                image: atlas.image.clone(),
                texture_atlas: Some(TextureAtlas {
                    layout: atlas.layout.clone(),
                    index: 0,
                }),
                flip_x,
                ..default()
            },
            Transform::from_xyz(x, 0.0, 0.0),
        ));
    }
}

pub(super) fn advance_agumon_presentation(
    stance_reg: Res<StanceGraphRegistry>,
    skill_reg: Res<SkillGraphRegistry>,
    graphs: Res<Assets<AnimGraph>>,
    mut lookup_diagnostics: ResMut<AnimationGraphLookupDiagnostics>,
    mut barrier: ResMut<SuspendedTimelineState>,
    atlas: Option<Res<AgumonAtlas>>,
    mut sprites: Query<(&mut AgumonSprite, &mut Sprite)>,
) {
    let stance_graph = stance_reg.resolve_snapshot(&AnimGraphId(AGUMON_STANCE_GRAPH_ID.into()), &graphs);
    let active_barrier = barrier.active_status().cloned();

    if let Some(status) = active_barrier.as_ref() {
        if status.awaiting_release && status.skill_id.0 != SHARP_CLAWS_SKILL_ID {
            debug!(
                target: "windowed.agumon_playback",
                skill_id = %status.skill_id.0,
                beat_id = status.beat_id,
                cue_id = status.cue_id,
                hop_index = ?status.hop_index,
                "unsupported windowed cue bridge; auto-releasing barrier to avoid stalled resolve"
            );
            let _ = barrier.request_release(status.cue_id);
            return;
        }
    }

    for (mut sprite, mut render_sprite) in &mut sprites {
        sync_agumon_mode(
            &mut sprite,
            active_barrier.as_ref(),
            &skill_reg,
            &stance_reg,
            &graphs,
            &mut lookup_diagnostics,
        );

        let graph = sprite.graph.graph().clone();
        let advance = sprite.player.advance_result(&graph);
        let current_node = sprite.player.current_node.0.clone();
        let local_frame = local_frame_for(&graph, &sprite.player.current_node, advance.frame);

        // Drive the rendered tile from the player frame via the identity
        // frame -> atlas-index map. Leave the index unchanged on an
        // out-of-range frame (atlas_index == None).
        let atlas_index = atlas
            .as_ref()
            .and_then(|atlas| atlas.geometry.atlas_index(advance.frame));
        if let (Some(index), Some(texture_atlas)) =
            (atlas_index, render_sprite.texture_atlas.as_mut())
        {
            texture_atlas.index = index as usize;
        }

        if active_barrier.is_some() {
            barrier.annotate_active_animation(&current_node, advance.frame as usize);
        }

        let awaiting = active_barrier
            .as_ref()
            .is_some_and(|status| status.awaiting_release);
        let released = active_barrier
            .as_ref()
            .is_some_and(|status| status.released);
        trace!(
            target: "windowed.agumon_playback",
            mode = ?sprite.mode,
            graph_source = ?sprite.graph.source,
            node = current_node.as_str(),
            clip_frame = advance.frame,
            local_frame,
            atlas_index,
            awaiting,
            released,
            barrier = ?active_barrier.as_ref().map(barrier_trace_tuple),
            "agumon windowed playback tick"
        );

        let pending_release = if let AgumonPlaybackMode::SharpClaws { cue_id, .. } = &sprite.mode {
            if let (Some(lf), Some(node)) =
                (local_frame, graph.nodes.get(&sprite.player.current_node))
            {
                if should_release_kernel(node, lf)
                    && !already_released_frame(
                        sprite.last_release_frame.as_ref(),
                        cue_id,
                        &current_node,
                        lf,
                    )
                {
                    Some((cue_id.clone(), lf))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        if let Some((cue_id, lf)) = pending_release {
            let result = barrier.request_release(&cue_id);
            trace!(
                target: "windowed.agumon_playback",
                cue_id = cue_id.as_str(),
                node = current_node.as_str(),
                clip_frame = advance.frame,
                local_frame = lf,
                ?result,
                "sharp claws release frame observed"
            );
            if matches!(
                result,
                CueReleaseResult::Released | CueReleaseResult::DuplicateRelease
            ) {
                sprite.player.fire_kernel_cue();
                sprite.last_release_frame = Some(ReleaseFrameKey {
                    cue_id,
                    node: current_node.clone(),
                    local_frame: lf,
                });
            }
        }

        if advance.exited {
            if let Some(stance_graph) = stance_graph.clone() {
                let preserve_missing = active_barrier.as_ref().and_then(|status| {
                    (sprite.graph.source == ResolvedAnimGraphSource::InstantFallback
                        && sprite.last_missing_skill_graph_cue.as_deref() == Some(status.cue_id))
                        .then(|| status.cue_id.to_string())
                });
                sprite.return_to_idle(stance_graph, preserve_missing);
                trace!(
                    target: "windowed.agumon_playback",
                    "agumon playback returned to idle"
                );
            }
        }
    }
}

fn sync_agumon_mode(
    sprite: &mut AgumonSprite,
    active_barrier: Option<&CueBarrierStatus>,
    skill_reg: &SkillGraphRegistry,
    stance_reg: &StanceGraphRegistry,
    graphs: &Assets<AnimGraph>,
    lookup_diagnostics: &mut AnimationGraphLookupDiagnostics,
) {
    let Some(status) = sharp_claws_barrier(active_barrier) else {
        return;
    };

    let presentation_node = status
        .animation_node
        .as_deref()
        .unwrap_or(SHARP_CLAWS_STRIKE_NODE);

    if matches!(
        &sprite.mode,
        AgumonPlaybackMode::SharpClaws { cue_id, .. } if cue_id == status.cue_id
    ) {
        sprite.last_missing_skill_graph_cue = None;
        return;
    }

    if sprite.last_missing_skill_graph_cue.as_deref() == Some(status.cue_id) {
        return;
    }

    let resolved_graph = skill_reg.resolve_snapshot_or_instant_fallback(
        &AnimGraphId(AGUMON_SKILL_GRAPH_ID.into()),
        graphs,
        lookup_diagnostics,
    );

    if resolved_graph.source == ResolvedAnimGraphSource::InstantFallback {
        warn!(
            target: "windowed.agumon_playback",
            cue_id = status.cue_id,
            skill_id = %status.skill_id.0,
            graph_id = AGUMON_SKILL_GRAPH_ID,
            diagnostic = lookup_diagnostics.last_message.as_deref().unwrap_or("missing"),
            "Sharp Claws presentation graph missing; running deterministic instant fallback"
        );
        sprite.last_missing_skill_graph_cue = Some(status.cue_id.to_string());
    }

    sprite.start_sharp_claws(status.cue_id, presentation_node, resolved_graph);
    trace!(
        target: "windowed.agumon_playback",
        cue_id = status.cue_id,
        skill_id = %status.skill_id.0,
        presentation_node,
        start_node = %sprite.player.current_node.0,
        graph_source = ?sprite.graph.source,
        "Sharp Claws playback entered windup"
    );

    if sprite.graph.source == ResolvedAnimGraphSource::InstantFallback {
        if let Some(stance_graph) = stance_reg.resolve_snapshot(
            &AnimGraphId(AGUMON_STANCE_GRAPH_ID.into()),
            graphs,
        ) {
            trace!(
                target: "windowed.agumon_playback",
                graph_id = AGUMON_STANCE_GRAPH_ID,
                stance_entry = %stance_graph.graph().entry.0,
                "stance snapshot remains available for post-fallback idle restore"
            );
        }
    }
}

fn sharp_claws_barrier(status: Option<&CueBarrierStatus>) -> Option<&CueBarrierStatus> {
    status.filter(|status| status.skill_id.0 == SHARP_CLAWS_SKILL_ID)
}

fn local_frame_for(graph: &AnimGraph, node_id: &NodeId, clip_frame: u32) -> Option<u32> {
    let node = graph.nodes.get(node_id)?;
    Some(if node.reverse {
        node.frames.end().saturating_sub(clip_frame)
    } else {
        clip_frame.saturating_sub(node.frames.start())
    })
}

fn should_release_kernel(node: &bevyrogue::animation::AnimNode, local_frame: u32) -> bool {
    node.cues.iter().any(|cue| {
        cue.at == local_frame && matches!(cue.command, FrameCueCommand::ReleaseKernel(_))
    })
}

fn already_released_frame(
    last_release_frame: Option<&ReleaseFrameKey>,
    cue_id: &str,
    node: &str,
    local_frame: u32,
) -> bool {
    last_release_frame.is_some_and(|last| {
        last.cue_id == cue_id && last.node == node && last.local_frame == local_frame
    })
}

pub(super) fn sharp_claws_start_node(presentation_node: &str) -> NodeId {
    if presentation_node == SHARP_CLAWS_STRIKE_NODE {
        NodeId(SHARP_CLAWS_WINDUP_NODE.into())
    } else {
        NodeId(presentation_node.to_string())
    }
}

fn barrier_trace_tuple(status: &CueBarrierStatus) -> (&str, &str, &str, bool, bool) {
    (
        status.skill_id.0.as_str(),
        status.beat_id,
        status.cue_id,
        status.awaiting_release,
        status.released,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sharp_claws_start_node_rewinds_strike_presentation_back_to_windup() {
        assert_eq!(
            sharp_claws_start_node(SHARP_CLAWS_STRIKE_NODE),
            NodeId(SHARP_CLAWS_WINDUP_NODE.into())
        );
        assert_eq!(
            sharp_claws_start_node("sharp_claws_recover"),
            NodeId("sharp_claws_recover".into())
        );
    }

    #[test]
    fn duplicate_release_guard_matches_same_cue_node_and_local_frame_only() {
        let last = ReleaseFrameKey {
            cue_id: "agumon/sharp_claws/impact".into(),
            node: SHARP_CLAWS_STRIKE_NODE.into(),
            local_frame: 1,
        };

        assert!(already_released_frame(
            Some(&last),
            "agumon/sharp_claws/impact",
            SHARP_CLAWS_STRIKE_NODE,
            1,
        ));
        assert!(!already_released_frame(
            Some(&last),
            "agumon/sharp_claws/impact",
            SHARP_CLAWS_STRIKE_NODE,
            2,
        ));
        assert!(!already_released_frame(
            Some(&last),
            "other/cue",
            SHARP_CLAWS_STRIKE_NODE,
            1,
        ));
    }
}
