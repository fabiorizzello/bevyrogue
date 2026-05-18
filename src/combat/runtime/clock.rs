use bevy::prelude::Resource;

/// Execution clock mode for the combat pipeline (F7 / D026).
///
/// - `HeadlessAuto`: consumes `BeatEvent` immediately; no presentation stalls.
///   Used by integration tests and `cargo run` (headless).
/// - `Windowed`: stalls on `BeatKind::Presentation::Cue(CueId)` until the
///   animation engine signals completion, then advances. Used with
///   `--features windowed`.
///
/// Invariant I3: both modes produce the same `Intent` stream end-of-cast.
/// The difference is timing only — never observable game state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Resource)]
pub enum Clock {
    #[default]
    HeadlessAuto,
    Windowed,
}
