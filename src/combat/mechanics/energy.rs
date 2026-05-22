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
