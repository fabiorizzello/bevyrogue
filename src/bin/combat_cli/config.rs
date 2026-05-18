use bevy::prelude::*;
use std::env;

use bevyrogue::combat::bootstrap::EncounterPreset;
use bevyrogue::combat::types::UnitId;

pub const DEFAULT_CLI_PROOF_TICK_LIMIT: u32 = 120;

#[derive(Resource, Default)]
pub struct TickCounter(pub u32);

/// Tracks whether a player action was just submitted and the next WaitingAction
/// tick should advance the turn instead of prompting again.
#[derive(Resource, Default)]
pub struct PlayerActed(pub bool);

/// Set once terminal mode is determined so we don't re-check stdin every tick.
#[derive(Resource)]
pub struct IsInteractive(pub bool);

/// The 4 ally UnitIds chosen by the player at startup.
#[derive(Resource)]
pub struct SelectedAllies(pub Vec<UnitId>);

/// The encounter preset chosen by the player at startup (or defaulted to BossEncounter in CI).
#[derive(Resource)]
pub struct SelectedEncounter(pub EncounterPreset);

/// Env-gated proof-mode settings. This is an observation/exit surface only;
/// gameplay still flows through the shared ECS systems registered below.
#[derive(Resource, Debug, Clone, Copy)]
pub struct CliProofConfig {
    pub tick_limit: u32,
}

#[derive(Resource, Debug, Default)]
pub struct CliProofState {
    pub ticks: u32,
    pub finished: bool,
}

pub fn cli_proof_enabled() -> bool {
    env::var("BEVYROGUE_CLI_PROOF")
        .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "on"))
        .unwrap_or(false)
}

pub fn parse_cli_proof_tick_limit(raw: Option<&str>) -> u32 {
    raw.and_then(|value| value.parse::<u32>().ok())
        .filter(|limit| *limit > 0)
        .unwrap_or(DEFAULT_CLI_PROOF_TICK_LIMIT)
}

pub fn cli_proof_tick_limit_from_env() -> u32 {
    parse_cli_proof_tick_limit(env::var("BEVYROGUE_CLI_TICK_LIMIT").ok().as_deref())
}
