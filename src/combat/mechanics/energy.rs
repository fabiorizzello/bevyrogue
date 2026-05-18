#![allow(dead_code)]

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secondary_cap_at_10() {
        let mut tracker = RoundEnergyTracker::default();
        let gained = tracker.try_gain(EnergyGainSource::SecondaryAction, 15);
        assert_eq!(gained, 10);
        assert_eq!(tracker.secondary_gained, 10);
        let gained2 = tracker.try_gain(EnergyGainSource::SecondaryAction, 5);
        assert_eq!(gained2, 0);
    }

    #[test]
    fn external_cap_at_30() {
        let mut tracker = RoundEnergyTracker::default();
        let gained = tracker.try_gain(EnergyGainSource::External, 50);
        assert_eq!(gained, 30);
        assert_eq!(tracker.external_gained, 30);
        let gained2 = tracker.try_gain(EnergyGainSource::External, 10);
        assert_eq!(gained2, 0);
    }

    #[test]
    fn caps_are_independent() {
        let mut tracker = RoundEnergyTracker::default();
        tracker.try_gain(EnergyGainSource::SecondaryAction, 10);
        tracker.try_gain(EnergyGainSource::External, 30);
        assert_eq!(tracker.secondary_gained, 10);
        assert_eq!(tracker.external_gained, 30);
    }

    #[test]
    fn reset_restores_full_budget() {
        let mut tracker = RoundEnergyTracker::default();
        tracker.try_gain(EnergyGainSource::SecondaryAction, 10);
        tracker.try_gain(EnergyGainSource::External, 30);
        tracker.reset();
        assert_eq!(tracker.secondary_gained, 0);
        assert_eq!(tracker.external_gained, 0);
        let s = tracker.try_gain(EnergyGainSource::SecondaryAction, 5);
        let e = tracker.try_gain(EnergyGainSource::External, 20);
        assert_eq!(s, 5);
        assert_eq!(e, 20);
    }

    #[test]
    fn energy_gain_clamps_at_max() {
        let mut e = Energy::default();
        e.gain(150);
        assert_eq!(e.current, e.max);
        assert_eq!(e.current, 100);
    }
}
