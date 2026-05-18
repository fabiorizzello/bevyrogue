use bevy::prelude::*;

use crate::combat::events::CombatEvent;

/// Writes each CombatEvent as a JSON line to stdout when BEVYROGUE_JSONL is set.
/// Consumes events unconditionally to avoid backpressure even when disabled.
pub fn jsonl_logger_system(mut events: MessageReader<CombatEvent>) {
    if std::env::var_os("BEVYROGUE_JSONL").is_none() {
        for _ in events.read() {}
        return;
    }
    for event in events.read() {
        match serde_json::to_string(event) {
            Ok(line) => println!("{line}"),
            Err(err) => eprintln!("jsonl_logger: serialize error: {err}"),
        }
    }
}
