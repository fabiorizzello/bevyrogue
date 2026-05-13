//! Agent-first observability: dump Bevy `Schedule` graphs to `.gsd/schedules/`.
//!
//! Mirrors the message + resource registration in `src/main.rs` and registers
//! the combat kernel + the same combat-system chain used by the headless build,
//! then writes per-schedule Graphviz DOT files for downstream tooling and
//! agents to inspect without reading `add_systems` / `.before` / `.after`
//! sprinkled across the codebase.
//!
//! Run:
//!     cargo run --example dump_schedule
//!
//! Output:
//!     .gsd/schedules/update.dot
//!     .gsd/schedules/startup.dot

use std::path::PathBuf;

use bevy::ecs::schedule::Schedules;
use bevy::prelude::*;
use bevy_mod_debugdump::{schedule_graph, schedule_graph_dot};

use bevyrogue::combat::av::ActionValueUpdated;
use bevyrogue::combat::events::CombatEvent;
use bevyrogue::combat::floating::decay_floating_damage;
use bevyrogue::combat::follow_up::{
    FollowUpIntent, FollowUpTrace, follow_up_listener_system, form_identity_listener_system,
    resolve_follow_up_action_system,
};
use bevyrogue::combat::jsonl_logger::jsonl_logger_system;
use bevyrogue::combat::kernel::register_combat_kernel_runtime;
use bevyrogue::combat::log::ActionLog;
use bevyrogue::combat::sp::SpPool;
use bevyrogue::combat::state::CombatState;
use bevyrogue::combat::turn_order::{TurnAdvanced, TurnOrder};
use bevyrogue::combat::turn_system::{
    ActionIntent, advance_turn_system, apply_turn_advance_system, check_victory_system,
    resolve_action_system,
};
use bevyrogue::combat::ultimate::{UltGainQueue, flush_ult_gain_system, ult_accumulation_system};

fn main() {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins)
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

    register_combat_kernel_runtime(&mut app);

    app.add_systems(
        Update,
        (
            resolve_action_system,
            follow_up_listener_system,
            form_identity_listener_system,
            resolve_follow_up_action_system,
            ult_accumulation_system,
            flush_ult_gain_system,
            apply_turn_advance_system,
            advance_turn_system,
            check_victory_system,
            jsonl_logger_system,
        )
            .chain(),
    );

    let out_dir = PathBuf::from(".gsd/schedules");
    if let Err(err) = std::fs::create_dir_all(&out_dir) {
        eprintln!("failed to create {}: {err}", out_dir.display());
        std::process::exit(1);
    }

    let settings = schedule_graph::Settings::default();

    fn dump<L: bevy::ecs::schedule::ScheduleLabel + Clone>(
        app: &mut App,
        out_dir: &std::path::Path,
        settings: &schedule_graph::Settings,
        name: &str,
        label: L,
    ) {
        let exists = app
            .world()
            .get_resource::<Schedules>()
            .map(|s| s.contains(label.intern()))
            .unwrap_or(false);
        if !exists {
            println!("skipped {name}: schedule not registered");
            return;
        }
        let dot = schedule_graph_dot(app, label, settings);
        let path = out_dir.join(format!("{name}.dot"));
        if let Err(err) = std::fs::write(&path, &dot) {
            eprintln!("failed to write {}: {err}", path.display());
            std::process::exit(1);
        }
        println!(
            "wrote {} ({} bytes, {} lines)",
            path.display(),
            dot.len(),
            dot.lines().count()
        );
    }

    dump(&mut app, &out_dir, &settings, "update", Update);
    dump(&mut app, &out_dir, &settings, "startup", Startup);
    dump(&mut app, &out_dir, &settings, "preupdate", PreUpdate);
    dump(&mut app, &out_dir, &settings, "postupdate", PostUpdate);
}
