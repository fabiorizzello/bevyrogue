use bevy::prelude::Component;

/// Single damage-reduction instance with a turn-based duration.
///
/// Multiple instances coexist in a [`DrBag`]; their values sum (unclamped) via
/// [`sum_dr`] when applied to incoming damage.
#[derive(Debug, Clone)]
pub struct DrInstance {
    pub value: f32,
    pub duration: u32,
}

/// Per-unit DR storage. Instances accumulate by summation (no cap, no merging).
/// M021 will expose mutation via `Intent::ApplyDR`; for now [`DrBag::apply`] is
/// the public seam.
#[derive(Component, Default, Debug, Clone)]
pub struct DrBag(Vec<DrInstance>);

// apply consumed by tests/dr_pipeline.rs and tests/block_reaction_pipeline.rs.
// instances is public API; not yet consumed outside this module.
impl DrBag {
    /// Push a new DR instance. No merging or capping — caller controls semantics.
    pub fn apply(&mut self, value: f32, duration: u32) {
        self.0.push(DrInstance { value, duration });
    }

    /// Decrement every instance's duration; drop expired entries.
    /// Returns the number of instances removed (mirrors `StatusBag::tick_all` shape).
    pub fn tick_all(&mut self) -> usize {
        for inst in self.0.iter_mut() {
            inst.duration = inst.duration.saturating_sub(1);
        }
        let before = self.0.len();
        self.0.retain(|i| i.duration > 0);
        before - self.0.len()
    }

    /// Read-only view of active instances.
    pub fn instances(&self) -> &[DrInstance] {
        &self.0
    }
}

/// Pure summation of every active DR instance. Returns `0.0` for an absent bag.
/// Unclamped: callers are responsible for any policy on damage flooring.
pub fn sum_dr(bag: Option<&DrBag>) -> f32 {
    bag.map(|b| b.0.iter().map(|i| i.value).sum())
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sum_dr_none_is_zero() {
        assert_eq!(sum_dr(None), 0.0);
    }

    #[test]
    fn sum_dr_sums_unclamped() {
        let mut bag = DrBag::default();
        bag.apply(0.3, 2);
        bag.apply(0.5, 1);
        bag.apply(0.4, 3);
        assert!((sum_dr(Some(&bag)) - 1.2).abs() < f32::EPSILON);
    }

    #[test]
    fn tick_all_drops_expired() {
        let mut bag = DrBag::default();
        bag.apply(0.2, 1);
        bag.apply(0.3, 2);
        let dropped = bag.tick_all();
        assert_eq!(dropped, 1);
        assert_eq!(bag.instances().len(), 1);
        assert_eq!(bag.instances()[0].duration, 1);
    }
}
