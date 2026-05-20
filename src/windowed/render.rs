use bevy::prelude::*;

use bevyrogue::animation::{
    AnimGraph, AnimGraphId, AnimGraphPlayer, FrameCueCommand, NodeId, SkillGraphRegistry,
    StanceGraphRegistry,
};
use bevyrogue::combat::runtime::{CueBarrierStatus, CueReleaseResult, SuspendedTimelineState};
use bevyrogue::combat::turn_system::{continue_suspended_timeline_system, resolve_action_system};

use super::{
    AGUMON_SKILL_GRAPH_ID, AGUMON_STANCE_GRAPH_ID, SHARP_CLAWS_SKILL_ID, SHARP_CLAWS_STRIKE_NODE,
    SHARP_CLAWS_WINDUP_NODE,
};

/// Marker + FSM state for the on-screen Agumon preview actor.
#[derive(Component, Debug, Clone)]
pub(super) struct AgumonSprite {
    pub(super) player: AnimGraphPlayer,
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
    pub(super) fn idle(entry: NodeId) -> Self {
        Self {
            player: AnimGraphPlayer::new(entry),
            mode: AgumonPlaybackMode::Idle,
            last_release_frame: None,
            last_missing_skill_graph_cue: None,
        }
    }

    fn start_sharp_claws(&mut self, cue_id: &str, presentation_node: &str) {
        self.player = AnimGraphPlayer::new(sharp_claws_start_node(presentation_node));
        self.mode = AgumonPlaybackMode::SharpClaws {
            cue_id: cue_id.to_string(),
            presentation_node: presentation_node.to_string(),
        };
        self.last_release_frame = None;
        self.last_missing_skill_graph_cue = None;
    }

    pub(super) fn return_to_idle(&mut self, stance_entry: NodeId) {
        self.player = AnimGraphPlayer::new(stance_entry);
        self.mode = AgumonPlaybackMode::Idle;
        self.last_release_frame = None;
        self.last_missing_skill_graph_cue = None;
    }
}

/// Sprite camera + Agumon presentation state machine. Feature-agnostic player,
/// windowed playback bridge.
pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera).add_systems(
            Update,
            advance_agumon_presentation
                .after(resolve_action_system)
                .before(continue_suspended_timeline_system),
        );
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub(super) fn advance_agumon_presentation(
    mut commands: Commands,
    stance_reg: Res<StanceGraphRegistry>,
    skill_reg: Res<SkillGraphRegistry>,
    graphs: Res<Assets<AnimGraph>>,
    mut barrier: ResMut<SuspendedTimelineState>,
    mut sprites: Query<&mut AgumonSprite>,
) {
    let stance_graph = stance_reg
        .resolve(&AnimGraphId(AGUMON_STANCE_GRAPH_ID.into()))
        .and_then(|handle| graphs.get(handle));
    let skill_graph = skill_reg
        .resolve(&AnimGraphId(AGUMON_SKILL_GRAPH_ID.into()))
        .and_then(|handle| graphs.get(handle));
    let active_barrier = barrier.active_status().cloned();

    if sprites.is_empty() {
        if let Some(stance_graph) = stance_graph {
            commands.spawn(AgumonSprite::idle(stance_graph.entry.clone()));
        } else {
            return;
        }
    }

    for mut sprite in &mut sprites {
        sync_agumon_mode(&mut sprite, active_barrier.as_ref(), skill_graph, stance_graph);

        let Some(graph) = current_graph_for_mode(&sprite, stance_graph, skill_graph) else {
            continue;
        };

        let advance = sprite.player.advance_result(graph);
        let current_node = sprite.player.current_node.0.clone();
        let local_frame = local_frame_for(graph, &sprite.player.current_node, advance.frame);

        if active_barrier.is_some() {
            barrier.annotate_active_animation(&current_node, advance.frame as usize);
        }

        let awaiting = active_barrier
            .as_ref()
            .is_some_and(|status| status.awaiting_release);
        let released = active_barrier.as_ref().is_some_and(|status| status.released);
        trace!(
            target: "windowed.agumon_playback",
            mode = ?sprite.mode,
            node = current_node.as_str(),
            clip_frame = advance.frame,
            local_frame,
            awaiting,
            released,
            barrier = ?active_barrier.as_ref().map(barrier_trace_tuple),
            "agumon windowed playback tick"
        );

        let pending_release =
            if let AgumonPlaybackMode::SharpClaws { cue_id, .. } = &sprite.mode {
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
            if let Some(stance_graph) = stance_graph {
                sprite.return_to_idle(stance_graph.entry.clone());
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
    skill_graph: Option<&AnimGraph>,
    stance_graph: Option<&AnimGraph>,
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

    if skill_graph.is_none() {
        if sprite.last_missing_skill_graph_cue.as_deref() != Some(status.cue_id) {
            warn!(
                target: "windowed.agumon_playback",
                cue_id = status.cue_id,
                skill_id = %status.skill_id.0,
                graph_id = AGUMON_SKILL_GRAPH_ID,
                "Sharp Claws presentation graph missing; staying/falling back to idle while the kernel barrier remains suspended"
            );
        }
        sprite.last_missing_skill_graph_cue = Some(status.cue_id.to_string());
        if let Some(stance_graph) = stance_graph {
            sprite.return_to_idle(stance_graph.entry.clone());
        }
        return;
    }

    sprite.start_sharp_claws(status.cue_id, presentation_node);
    trace!(
        target: "windowed.agumon_playback",
        cue_id = status.cue_id,
        skill_id = %status.skill_id.0,
        presentation_node,
        start_node = %sprite.player.current_node.0,
        "Sharp Claws playback entered windup"
    );
}

fn current_graph_for_mode<'a>(
    sprite: &AgumonSprite,
    stance_graph: Option<&'a AnimGraph>,
    skill_graph: Option<&'a AnimGraph>,
) -> Option<&'a AnimGraph> {
    match sprite.mode {
        AgumonPlaybackMode::Idle => stance_graph,
        AgumonPlaybackMode::SharpClaws { .. } => skill_graph.or(stance_graph),
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
