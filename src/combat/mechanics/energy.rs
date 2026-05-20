
use bevy::prelude::Component;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Energy {
    pub current: i32,
    pub max: i32,
}

impl Default for Energy {
    fn default() -> Self {
        Self {
            current: 0,
            max: 100,
        }
    }
}

impl Energy {
    pub fn gain(&mut self, amount: i32) {
        self.gain_capped(amount);
    }

    pub fn gain_capped(&mut self, amount: i32) -> i32 {
        let before = self.current;
        self.current = (self.current + amount).min(self.max);
        self.current - before
    }

    pub fn spend(&mut self, amount: i32) -> bool {
        if self.current >= amount {
            self.current -= amount;
            true
        } else {
            false
        }
    }

    pub fn is_full(&self) -> bool {
        self.current >= self.max
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnergyGainSource {
    SecondaryAction,
    External,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RoundEnergyTracker {
    pub secondary_gained: i32,
    pub external_gained: i32,
}

impl Default for RoundEnergyTracker {
    fn default() -> Self {
        Self {
            secondary_gained: 0,
            external_gained: 0,
        }
    }
}

impl RoundEnergyTracker {
    pub fn try_gain(&mut self, source: EnergyGainSource, amount: i32) -> i32 {
        match source {
            EnergyGainSource::SecondaryAction => {
                let cap = 10;
                let remaining = (cap - self.secondary_gained).max(0);
                let actual = amount.min(remaining);
                self.secondary_gained += actual;
                actual
            }
            EnergyGainSource::External => {
                let cap = 30;
                let remaining = (cap - self.external_gained).max(0);
                let actual = amount.min(remaining);
                self.external_gained += actual;
                actual
            }
        }
    }

    pub fn reset(&mut self) {
        self.secondary_gained = 0;
        self.external_gained = 0;
    }
}

