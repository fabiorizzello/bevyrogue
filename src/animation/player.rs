use super::anim_graph::{
    AnimEdge, AnimGraph, AnimNode, NodeId, PlaybackModifier, Predicate, TransitionTarget,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnimAdvanceResult {
    pub frame: u32,
    pub exited: bool,
}

/// Feature-agnostic FSM core — no `#[cfg(feature)]` anywhere in this file.
///
/// The player tracks which node is active and how many animation ticks have
/// elapsed within that node. `advance` is the single entry point: call it once
/// per animation tick and use the returned sprite sheet frame index to drive
/// the renderer.
#[derive(Debug, Clone)]
pub struct AnimGraphPlayer {
    pub current_node: NodeId,
    pub elapsed_anim_frames: u32,
    pending_kernel_cue: bool,
}

impl AnimGraphPlayer {
    pub fn new(entry: NodeId) -> Self {
        Self {
            current_node: entry,
            elapsed_anim_frames: 0,
            pending_kernel_cue: false,
        }
    }

    /// Latch a one-shot kernel cue for the current node.
    ///
    /// The cue stays pending until a `Predicate::KernelCue` transition fires,
    /// then it is consumed so one release cannot satisfy later gates.
    pub fn fire_kernel_cue(&mut self) {
        self.pending_kernel_cue = true;
    }

    /// Advance one animation tick.
    ///
    /// Evaluates `TimeInNode`, `Always`, and `KernelCue` transitions.
    /// Returns the sprite-sheet frame index for the current animation state.
    pub fn advance(&mut self, graph: &AnimGraph) -> u32 {
        self.advance_result(graph).frame
    }

    /// Advance one animation tick and surface whether the graph exited.
    pub fn advance_result(&mut self, graph: &AnimGraph) -> AnimAdvanceResult {
        let Some(node) = graph.nodes.get(&self.current_node) else {
            return AnimAdvanceResult {
                frame: 0,
                exited: false,
            };
        };

        let duration = node_duration(node);
        let transition = self.select_transition(graph, duration);

        if let Some(edge) = transition {
            if matches!(edge.when, Predicate::KernelCue) {
                self.pending_kernel_cue = false;
            }

            match &edge.to {
                TransitionTarget::Node(id) => {
                    self.current_node = id.clone();
                    self.elapsed_anim_frames = 0;
                    return AnimAdvanceResult {
                        frame: frame_index(
                            graph
                                .nodes
                                .get(&self.current_node)
                                .expect("transition targets a valid node"),
                            0,
                        ),
                        exited: false,
                    };
                }
                TransitionTarget::Exit => {
                    return AnimAdvanceResult {
                        frame: frame_index(node, duration.saturating_sub(1)),
                        exited: true,
                    };
                }
            }
        }

        let idx = frame_index(node, self.elapsed_anim_frames);
        self.elapsed_anim_frames = self.elapsed_anim_frames.wrapping_add(1);
        AnimAdvanceResult {
            frame: idx,
            exited: false,
        }
    }

    fn select_transition<'a>(&self, graph: &'a AnimGraph, duration: u32) -> Option<&'a AnimEdge> {
        graph
            .transitions
            .iter()
            .filter(|edge| edge.from == self.current_node)
            .filter(|edge| self.predicate_matches(&edge.when, duration))
            .max_by_key(|edge| edge.priority.map_or(0, |priority| priority.0))
    }

    fn predicate_matches(&self, predicate: &Predicate, duration: u32) -> bool {
        match predicate {
            Predicate::Always => true,
            Predicate::TimeInNode => self.elapsed_anim_frames >= duration,
            Predicate::KernelCue => self.pending_kernel_cue,
            _ => false,
        }
    }
}

/// Total animation-tick duration of a node before a `TimeInNode` transition fires.
fn node_duration(node: &AnimNode) -> u32 {
    let range_len = node.frames.end() - node.frames.start() + 1;
    match &node.modifier {
        None => range_len,
        Some(PlaybackModifier::Loop { count: 0 }) => u32::MAX,
        Some(PlaybackModifier::Loop { count }) => range_len.saturating_mul(*count as u32),
        Some(PlaybackModifier::Hold { extra_frames }) => range_len.saturating_add(*extra_frames),
        Some(PlaybackModifier::SpeedMul { pct }) => {
            range_len.saturating_mul(100) / (*pct as u32).max(1)
        }
    }
}

/// Sprite-sheet frame index at `elapsed` ticks into `node`.
fn frame_index(node: &AnimNode, elapsed: u32) -> u32 {
    let range = node.frames;
    let range_len = range.end() - range.start() + 1;

    let local = match &node.modifier {
        None => elapsed.min(range_len - 1),
        Some(PlaybackModifier::Loop { count: 0 }) => elapsed % range_len,
        Some(PlaybackModifier::Loop { count }) => {
            let total = range_len.saturating_mul(*count as u32);
            elapsed.min(total - 1) % range_len
        }
        Some(PlaybackModifier::Hold { extra_frames }) => {
            elapsed.min(range_len - 1 + extra_frames).min(range_len - 1)
        }
        Some(PlaybackModifier::SpeedMul { pct }) => {
            let anim_frame = ((elapsed as u64 * *pct as u64) / 100) as u32;
            anim_frame.min(range_len - 1)
        }
    };

    if node.reverse {
        range.end() - local
    } else {
        range.start() + local
    }
}
