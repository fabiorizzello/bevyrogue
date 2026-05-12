use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

/// Canon status taxonomy v0 (M017 D004+D009). All variants are single-instance per target.
/// Re-application follows refresh_max_dur: keep the longer of old/new duration.
/// Per-status semantics (damage ticks, speed delta, cancel probability, ult boost)
/// are implemented in S03–S05; this module carries only the lifecycle skeleton.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatusEffectKind {
    Heated,
    Chilled,
    Paralyzed,
    Slowed,
    Blessed,
    /// Reserved §H.1 — vocabulary anchor for RON/log; no active effect in v0.
    #[allow(dead_code)]
    Burn,
    /// Reserved §H.1 — vocabulary anchor for RON/log; no active effect in v0.
    #[allow(dead_code)]
    Shock,
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusEffect {
    pub kind: StatusEffectKind,
    pub duration_remaining: u32,
}

impl StatusEffect {
    pub fn new(kind: StatusEffectKind, duration: u32) -> Self {
        StatusEffect { kind, duration_remaining: duration }
    }

    /// refresh_max_dur: keep the longer of the existing and incoming durations.
    pub fn refresh(&mut self, new_duration: u32) {
        self.duration_remaining = self.duration_remaining.max(new_duration);
    }

    /// Decrements the duration counter; returns true when the effect should be removed.
    pub fn tick(&mut self) -> bool {
        self.duration_remaining = self.duration_remaining.saturating_sub(1);
        self.duration_remaining == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn heated(duration: u32) -> StatusEffect {
        StatusEffect::new(StatusEffectKind::Heated, duration)
    }

    #[test]
    fn tick_keeps_component_while_turns_remain() {
        let mut effect = heated(2);
        assert!(!effect.tick());
        assert_eq!(effect.duration_remaining, 1);
    }

    #[test]
    fn tick_down_to_zero() {
        let mut effect = heated(1);
        assert!(effect.tick());
        assert_eq!(effect.duration_remaining, 0);
    }

    #[test]
    fn tick_no_op_at_zero() {
        let mut effect = heated(0);
        assert!(effect.tick());
        assert_eq!(effect.duration_remaining, 0);
    }

    #[test]
    fn refresh_max_dur_keeps_longer() {
        let mut effect = heated(3);
        effect.refresh(1);
        assert_eq!(effect.duration_remaining, 3);
        effect.refresh(5);
        assert_eq!(effect.duration_remaining, 5);
    }

    #[test]
    fn ron_roundtrip_heated() {
        let effect = StatusEffect::new(StatusEffectKind::Heated, 2);
        let s = ron::to_string(&effect).expect("serialize");
        let back: StatusEffect = ron::from_str(&s).expect("deserialize");
        assert_eq!(effect, back);
    }

    #[test]
    fn ron_roundtrip_chilled() {
        let effect = StatusEffect::new(StatusEffectKind::Chilled, 3);
        let s = ron::to_string(&effect).expect("serialize");
        let back: StatusEffect = ron::from_str(&s).expect("deserialize");
        assert_eq!(effect, back);
    }

    #[test]
    fn ron_roundtrip_paralyzed() {
        let effect = StatusEffect::new(StatusEffectKind::Paralyzed, 1);
        let s = ron::to_string(&effect).expect("serialize");
        let back: StatusEffect = ron::from_str(&s).expect("deserialize");
        assert_eq!(effect, back);
    }

    #[test]
    fn ron_roundtrip_slowed() {
        let effect = StatusEffect::new(StatusEffectKind::Slowed, 2);
        let s = ron::to_string(&effect).expect("serialize");
        let back: StatusEffect = ron::from_str(&s).expect("deserialize");
        assert_eq!(effect, back);
    }

    #[test]
    fn ron_roundtrip_blessed() {
        let effect = StatusEffect::new(StatusEffectKind::Blessed, 2);
        let s = ron::to_string(&effect).expect("serialize");
        let back: StatusEffect = ron::from_str(&s).expect("deserialize");
        assert_eq!(effect, back);
    }

    #[test]
    fn ron_roundtrip_reserved_burn() {
        let effect = StatusEffect::new(StatusEffectKind::Burn, 1);
        let s = ron::to_string(&effect).expect("serialize");
        let back: StatusEffect = ron::from_str(&s).expect("deserialize");
        assert_eq!(effect, back);
    }

    #[test]
    fn ron_roundtrip_reserved_shock() {
        let effect = StatusEffect::new(StatusEffectKind::Shock, 1);
        let s = ron::to_string(&effect).expect("serialize");
        let back: StatusEffect = ron::from_str(&s).expect("deserialize");
        assert_eq!(effect, back);
    }
}
