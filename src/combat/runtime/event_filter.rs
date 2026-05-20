use std::sync::Arc;

use crate::combat::{events::CombatEvent, runtime::signal::Signal};

/// Typed runtime filter for passive listeners.
///
/// The filter surface stays generic: blueprints are matched by opaque owner/name
/// strings, combat-envelope listeners can inspect the bridged `CombatEvent`, and
/// custom predicates are available for one-off cases without expanding the enum.
// Consumed by tests/passive_event_filters.rs and tests/passive_kitsune_grace.rs via public API.
#[derive(Clone)]
pub enum EventFilter {
    Any(Vec<EventFilter>),
    All(Vec<EventFilter>),
    Not(Box<EventFilter>),
    Blueprint {
        owner: &'static str,
        name: &'static str,
    },
    CombatEvent(Arc<dyn Fn(&CombatEvent) -> bool + Send + Sync>),
    Custom(Arc<dyn Fn(&Signal) -> bool + Send + Sync>),
}

// Constructor functions consumed by tests/passive_event_filters.rs and passive_kitsune_grace.rs.
impl EventFilter {
    pub fn any(filters: impl IntoIterator<Item = EventFilter>) -> Self {
        Self::Any(filters.into_iter().collect())
    }

    pub fn all(filters: impl IntoIterator<Item = EventFilter>) -> Self {
        Self::All(filters.into_iter().collect())
    }

    pub fn not(filter: EventFilter) -> Self {
        Self::Not(Box::new(filter))
    }

    pub fn blueprint(owner: &'static str, name: &'static str) -> Self {
        Self::Blueprint { owner, name }
    }

    pub fn combat<F>(predicate: F) -> Self
    where
        F: Fn(&CombatEvent) -> bool + Send + Sync + 'static,
    {
        Self::CombatEvent(Arc::new(predicate))
    }

    pub fn custom<F>(predicate: F) -> Self
    where
        F: Fn(&Signal) -> bool + Send + Sync + 'static,
    {
        Self::Custom(Arc::new(predicate))
    }

    pub fn matches(&self, signal: &Signal) -> bool {
        match self {
            Self::Any(filters) => filters.iter().any(|filter| filter.matches(signal)),
            Self::All(filters) => filters.iter().all(|filter| filter.matches(signal)),
            Self::Not(filter) => !filter.matches(signal),
            Self::Blueprint { owner, name } => matches!(
                signal,
                Signal::Blueprint {
                    owner: signal_owner,
                    name: signal_name,
                    ..
                } if signal_owner == owner && signal_name == name
            ),
            Self::CombatEvent(predicate) => {
                matches!(signal, Signal::CombatEvent(event) if predicate(event))
            }
            Self::Custom(predicate) => predicate(signal),
        }
    }
}

impl std::fmt::Debug for EventFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Any(filters) => f.debug_tuple("Any").field(filters).finish(),
            Self::All(filters) => f.debug_tuple("All").field(filters).finish(),
            Self::Not(filter) => f.debug_tuple("Not").field(filter).finish(),
            Self::Blueprint { owner, name } => f
                .debug_struct("Blueprint")
                .field("owner", owner)
                .field("name", name)
                .finish(),
            Self::CombatEvent(_) => f.write_str("CombatEvent(<predicate>)"),
            Self::Custom(_) => f.write_str("Custom(<predicate>)"),
        }
    }
}

// Tests relocated to `tests/event_filter_internals.rs` (R003).
