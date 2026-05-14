use bevy::prelude::Resource;

/// Global reactive signal bus (scaffold; full implementation in S04).
///
/// In S04 this carries a `VecDeque<Signal>` where `Signal` is the closed-enum
/// signal type registered at `App::finish()` (D028). Listeners subscribe per
/// signal variant; `PassiveRunner` drains the queue each pipeline step.
///
/// For S01 this is an empty marker `Resource` to establish ownership and
/// wiring. No methods are exposed until the signal taxonomy is finalised.
#[derive(Resource, Default)]
pub struct SignalBus {
    /// Placeholder pending count — replaced by `VecDeque<Signal>` in S04.
    _pending: u32,
}
