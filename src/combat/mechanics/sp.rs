use bevy::prelude::Resource;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpGainSource {
    Basic,
    NonBasic,
}

#[derive(Resource, Debug, Clone)]
pub struct RoundSpTracker {
    non_basic_gained: i32,
    max_non_basic_per_round: i32,
}

impl Default for RoundSpTracker {
    fn default() -> Self {
        Self {
            non_basic_gained: 0,
            max_non_basic_per_round: 2,
        }
    }
}

impl RoundSpTracker {
    pub fn try_gain_non_basic(&mut self, amount: i32) -> i32 {
        let remaining_cap = self.max_non_basic_per_round - self.non_basic_gained;
        if remaining_cap <= 0 {
            return 0;
        }
        let actual_gain = amount.min(remaining_cap);
        self.non_basic_gained += actual_gain;
        actual_gain
    }

    pub fn reset(&mut self) {
        self.non_basic_gained = 0;
    }
}

#[derive(Resource, Debug, Clone)]
pub struct SpPool {
    pub current: i32,
    pub max: i32,
}

impl Default for SpPool {
    fn default() -> Self {
        Self { current: 3, max: 5 }
    }
}

impl SpPool {
    pub fn spend(&mut self, cost: i32) -> bool {
        if self.current >= cost {
            self.current -= cost;
            true
        } else {
            false
        }
    }

    pub fn gain(&mut self, amount: i32) {
        self.current = (self.current + amount).min(self.max);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sp_pool_default_max_is_5() {
        assert_eq!(SpPool::default().max, 5);
    }

    #[test]
    fn round_sp_tracker_caps_non_basic_at_2() {
        let mut tracker = RoundSpTracker::default();
        // Attempt +3: only +2 allowed
        let gained = tracker.try_gain_non_basic(3);
        assert_eq!(gained, 2);
        // Budget exhausted: further gain returns 0
        let gained2 = tracker.try_gain_non_basic(1);
        assert_eq!(gained2, 0);
    }

    #[test]
    fn round_sp_tracker_reset_restores_full_budget() {
        let mut tracker = RoundSpTracker::default();
        tracker.try_gain_non_basic(2); // exhaust budget
        tracker.reset();
        let gained = tracker.try_gain_non_basic(2);
        assert_eq!(gained, 2);
    }

    #[test]
    fn round_sp_tracker_partial_gain_then_remainder() {
        let mut tracker = RoundSpTracker::default();
        let first = tracker.try_gain_non_basic(1);
        assert_eq!(first, 1);
        let second = tracker.try_gain_non_basic(1);
        assert_eq!(second, 1);
        // Budget now exhausted
        let third = tracker.try_gain_non_basic(1);
        assert_eq!(third, 0);
    }
}
