use bevy::prelude::*;

mod combat;
mod data;
mod party_validation;
mod ui;

#[cfg(not(feature = "windowed"))]
mod headless;
#[cfg(feature = "windowed")]
mod windowed;

use combat::av::ActionValueUpdated;
use combat::events::CombatEvent;
use combat::floating::decay_floating_damage;
use combat::follow_up::{FollowUpIntent, FollowUpTrace};
use combat::log::ActionLog;
use combat::sp::SpPool;
use combat::state::CombatState;
use combat::turn_order::{TurnAdvanced, TurnOrder};
use combat::turn_system::ActionIntent;
use combat::ultimate::UltGainQueue;

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
        .add_systems(Update, decay_floating_damage);

    combat::kernel::register_combat_kernel_runtime(&mut app);

    #[cfg(not(feature = "windowed"))]
    headless::register_combat_systems(&mut app);

    #[cfg(feature = "windowed")]
    windowed::register_combat_systems(&mut app);

    app.run()
}
