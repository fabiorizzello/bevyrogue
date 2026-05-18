//! Agent-oriented logging configuration.
//!
//! Bevy's `LogPlugin` owns the global tracing subscriber. To avoid installing a
//! competing subscriber, headless entry points route structured output through
//! `LogPlugin::fmt_layer` when `BEVYROGUE_TRACE_FORMAT=json` is set.

use bevy::{app::App, log::LogPlugin};

const TRACE_FORMAT_ENV: &str = "BEVYROGUE_TRACE_FORMAT";

/// Returns the Bevy log plugin configured for optional agent-readable output.
///
/// Default behavior is identical to `LogPlugin::default()`. Set
/// `BEVYROGUE_TRACE_FORMAT=json` to replace the default formatter with JSON
/// including current span metadata and explicit span enter/close events.
pub fn log_plugin_from_env() -> LogPlugin {
    let mut plugin = LogPlugin::default();
    plugin.fmt_layer = structured_fmt_layer_from_env;
    plugin
}

fn structured_fmt_layer_from_env(_app: &mut App) -> Option<bevy::log::BoxedFmtLayer> {
    match std::env::var(TRACE_FORMAT_ENV) {
        Ok(value) if value.eq_ignore_ascii_case("json") => Some(Box::new(
            tracing_subscriber::fmt::Layer::default()
                .json()
                .with_current_span(true)
                .with_span_list(true)
                .with_span_events(
                    tracing_subscriber::fmt::format::FmtSpan::ENTER
                        | tracing_subscriber::fmt::format::FmtSpan::CLOSE,
                )
                .with_writer(std::io::stderr),
        )),
        Ok(value) if value.trim().is_empty() => None,
        Ok(value) => {
            eprintln!(
                "unsupported {TRACE_FORMAT_ENV}={value:?}; falling back to Bevy's default log formatter"
            );
            None
        }
        Err(_) => None,
    }
}
