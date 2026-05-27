//! Generic warn-once dedup.
//!
//! Recurring diagnostics (asset-event spawn misses, cue misses, verb misses)
//! often re-fire every frame for the same offending id. This util tracks the
//! ids already warned about so the caller logs once per id rather than each
//! frame. Keyed by an arbitrary id type so animation and windowed consumers
//! share one consistent, testable surface instead of re-implementing the
//! `Local<HashSet<Id>>` pattern.

use std::collections::HashSet;
use std::hash::Hash;

/// Tracks ids that have already been warned about.
///
/// Use as a `Local<WarnOnce<K>>` inside a system, or as a plain field. Call
/// [`WarnOnce::should_warn`] guarding the actual `warn!` so the diagnostic
/// fires once per distinct key.
#[derive(Debug, Clone)]
pub struct WarnOnce<K: Eq + Hash>(HashSet<K>);

impl<K: Eq + Hash> Default for WarnOnce<K> {
    fn default() -> Self {
        Self(HashSet::new())
    }
}

impl<K: Eq + Hash> WarnOnce<K> {
    /// Returns `true` the first time `key` is seen (the caller should warn),
    /// and `false` on every subsequent call with the same key (already warned).
    ///
    /// Mirrors `HashSet::insert`'s semantics: the boolean reports whether the
    /// key was newly inserted.
    pub fn should_warn(&mut self, key: K) -> bool {
        self.0.insert(key)
    }

    /// Returns `true` if `key` has already been warned about.
    pub fn has_warned(&self, key: &K) -> bool {
        self.0.contains(key)
    }

    /// Forgets all recorded keys, so each will warn again on next sight.
    pub fn clear(&mut self) {
        self.0.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warns_once_per_key() {
        let mut w: WarnOnce<u32> = WarnOnce::default();
        assert!(w.should_warn(1), "first sight of a key warns");
        assert!(!w.should_warn(1), "repeat sight is deduped");
        assert!(w.should_warn(2), "a distinct key warns independently");
        assert!(!w.should_warn(2));
    }

    #[test]
    fn has_warned_reflects_state_and_clear_resets() {
        let mut w: WarnOnce<&'static str> = WarnOnce::default();
        assert!(!w.has_warned(&"a"));
        w.should_warn("a");
        assert!(w.has_warned(&"a"));
        w.clear();
        assert!(!w.has_warned(&"a"), "clear forgets recorded keys");
        assert!(w.should_warn("a"), "warns again after clear");
    }
}
