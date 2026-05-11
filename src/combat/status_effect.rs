// Freeze does NOT re-seed TurnOrder (static VecDeque) — speed reduction is observable via
// SpeedModifier reads only.
use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatusEffectKind {
    Burn { damage_per_turn: i32 },
    Freeze { speed_reduction: i32 },
    Shock { cancel_chance_pct: u8 },
    DeepFreeze,
}

#[allow(dead_code)]
#[derive(Component, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusEffect {
    pub kind: StatusEffectKind,
    pub duration_remaining: u32,
}

impl StatusEffect {
    /// Decrements the duration counter and returns true when the component can be removed.
    #[allow(dead_code)]
    pub fn tick(&mut self) -> bool {
        self.duration_remaining = self.duration_remaining.saturating_sub(1);
        self.duration_remaining == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn burn_effect(duration: u32) -> StatusEffect {
        StatusEffect {
            kind: StatusEffectKind::Burn { damage_per_turn: 5 },
            duration_remaining: duration,
        }
    }

    #[test]
    fn tick_keeps_component_while_turns_remain() {
        let mut effect = burn_effect(2);
        assert!(!effect.tick());
        assert_eq!(effect.duration_remaining, 1);
    }

    #[test]
    fn tick_down_to_zero() {
        let mut effect = burn_effect(1);
        assert!(effect.tick());
        assert_eq!(effect.duration_remaining, 0);
    }

    #[test]
    fn tick_no_op_at_zero() {
        let mut effect = burn_effect(0);
        assert!(effect.tick());
        assert_eq!(effect.duration_remaining, 0);
    }

    #[test]
    fn ron_roundtrip_burn() {
        let effect = StatusEffect {
            kind: StatusEffectKind::Burn { damage_per_turn: 8 },
            duration_remaining: 2,
        };
        let s = ron::to_string(&effect).expect("serialize");
        let back: StatusEffect = ron::from_str(&s).expect("deserialize");
        assert_eq!(effect, back);
    }

    #[test]
    fn ron_roundtrip_freeze() {
        let effect = StatusEffect {
            kind: StatusEffectKind::Freeze {
                speed_reduction: 10,
            },
            duration_remaining: 3,
        };
        let s = ron::to_string(&effect).expect("serialize");
        let back: StatusEffect = ron::from_str(&s).expect("deserialize");
        assert_eq!(effect, back);
    }

    #[test]
    fn ron_roundtrip_shock() {
        let effect = StatusEffect {
            kind: StatusEffectKind::Shock {
                cancel_chance_pct: 75,
            },
            duration_remaining: 1,
        };
        let s = ron::to_string(&effect).expect("serialize");
        let back: StatusEffect = ron::from_str(&s).expect("deserialize");
        assert_eq!(effect, back);
    }

    // Negative damage_per_turn (i32) is accepted at parse time — i32 structurally allows it.
    // Semantic rejection (negative burn damage is nonsensical) is deferred to apply time.
    #[test]
    fn negative_damage_per_turn_accepted_at_parse_time() {
        let effect = StatusEffect {
            kind: StatusEffectKind::Burn {
                damage_per_turn: -5,
            },
            duration_remaining: 3,
        };
        let s = ron::to_string(&effect).expect("serialize");
        let back: StatusEffect = ron::from_str(&s).expect("deserialize — accepted at parse time");
        assert_eq!(effect, back);
    }
}
