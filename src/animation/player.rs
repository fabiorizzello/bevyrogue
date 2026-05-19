use super::anim_graph::{AnimGraph, AnimNode, NodeId, PlaybackModifier, Predicate, TransitionTarget};

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
}

impl AnimGraphPlayer {
    pub fn new(entry: NodeId) -> Self {
        Self { current_node: entry, elapsed_anim_frames: 0 }
    }

    /// Advance one animation tick.
    ///
    /// Evaluates `TimeInNode` and `Always` transitions; ignores `KernelCue` and
    /// all other predicates (those are driven by the combat kernel, not this layer).
    /// Returns the sprite-sheet frame index for the current animation state.
    pub fn advance(&mut self, graph: &AnimGraph) -> u32 {
        let Some(node) = graph.nodes.get(&self.current_node) else {
            return 0;
        };

        let duration = node_duration(node);

        let transition = graph
            .transitions
            .iter()
            .filter(|e| e.from == self.current_node)
            .filter(|e| match &e.when {
                Predicate::Always => true,
                Predicate::TimeInNode => self.elapsed_anim_frames >= duration,
                _ => false,
            })
            .max_by_key(|e| e.priority.map_or(0, |p| p.0));

        if let Some(edge) = transition {
            match &edge.to {
                TransitionTarget::Node(id) => {
                    self.current_node = id.clone();
                    self.elapsed_anim_frames = 0;
                    return frame_index(
                        graph.nodes.get(&self.current_node).expect("transition targets a valid node"),
                        0,
                    );
                }
                TransitionTarget::Exit => {
                    return frame_index(node, duration.saturating_sub(1));
                }
            }
        }

        let idx = frame_index(node, self.elapsed_anim_frames);
        self.elapsed_anim_frames = self.elapsed_anim_frames.wrapping_add(1);
        idx
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
