use bevy::prelude::*;

#[cfg(not(feature = "windowed"))]
mod headless;
#[cfg(feature = "windowed")]
mod windowed;

use bevyrogue::combat::CombatPlugin;
use bevyrogue::combat::av::ActionValueUpdated;
use bevyrogue::combat::events::CombatEvent;
use bevyrogue::combat::floating::decay_floating_damage;
use bevyrogue::combat::follow_up::{FollowUpIntent, FollowUpTrace};
use bevyrogue::combat::log::ActionLog;
use bevyrogue::combat::sp::SpPool;
use bevyrogue::combat::state::CombatState;
use bevyrogue::combat::turn_order::{TurnAdvanced, TurnOrder};
use bevyrogue::combat::turn_system::ActionIntent;
use bevyrogue::combat::ultimate::UltGainQueue;

// Used by S03+ turn state transitions
#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    #[default]
    CombatSandbox,
}

fn main() -> AppExit {
    #[cfg(feature = "windowed")]
    let windowed_validation = match windowed::config_from_env() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("windowed validation config error: {err}");
            return AppExit::error();
        }
    };

    let mut app = App::new();

    #[cfg(not(feature = "windowed"))]
    headless::register(&mut app);

    #[cfg(feature = "windowed")]
    windowed::register(&mut app, windowed_validation);

    app.init_state::<AppState>()
        .init_resource::<TurnOrder>()
        .init_resource::<SpPool>()
        .init_resource::<ActionLog>()
        .init_resource::<CombatState>()
        .init_resource::<UltGainQueue>()
        .add_message::<TurnAdvanced>()
        .add_message::<ActionIntent>()
        .add_message::<FollowUpIntent>()
        .add_message::<FollowUpTrace>()
        .add_message::<CombatEvent>()
        .add_message::<ActionValueUpdated>()
        .add_systems(Update, decay_floating_damage)
        .add_plugins(CombatPlugin);

    #[cfg(not(feature = "windowed"))]
    headless::register_combat_systems(&mut app);

    #[cfg(feature = "windowed")]
    windowed::register_combat_systems(&mut app);

    app.run()
}
