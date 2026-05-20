
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

// Tests relocated to `tests/sp_mechanics_internals.rs` (R003).
