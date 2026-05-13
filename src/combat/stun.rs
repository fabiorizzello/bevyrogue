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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_keeps_component_while_turns_remain() {
        let mut stunned = Stunned { turns_left: 2 };

        assert!(!stunned.tick());
        assert_eq!(stunned.turns_left, 1);
    }

    #[test]
    fn tick_down_to_zero() {
        let mut stunned = Stunned { turns_left: 1 };

        assert!(stunned.tick());
        assert_eq!(stunned.turns_left, 0);
    }

    #[test]
    fn tick_no_op_at_zero() {
        let mut stunned = Stunned { turns_left: 0 };

        assert!(stunned.tick());
        assert_eq!(stunned.turns_left, 0);
    }
}
