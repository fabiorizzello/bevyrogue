use bevy::prelude::Component;

// Used by S06/T02.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Stunned {
    pub turns_left: u32,
}

impl Stunned {
    /// Decrements the stun counter and returns true when the component can be removed.
    // Used by S06/T02.
    pub fn tick(&mut self) -> bool {
        self.turns_left = self.turns_left.saturating_sub(1);
        self.turns_left == 0
    }
}
