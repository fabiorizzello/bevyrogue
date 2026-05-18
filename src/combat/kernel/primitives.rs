use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TacticalCyclePhase {
    Declared,
    PreApp,
    Impact,
    Applied,
}

// ALL/next/as_str used in cfg(test) in this file; public API surface for blueprint callers.
#[allow(dead_code)]
impl TacticalCyclePhase {
    pub const ALL: [Self; 4] = [Self::Declared, Self::PreApp, Self::Impact, Self::Applied];

    pub const fn next(self) -> Self {
        match self {
            Self::Declared => Self::PreApp,
            Self::PreApp => Self::Impact,
            Self::Impact => Self::Applied,
            Self::Applied => Self::Declared,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Declared => "declared",
            Self::PreApp => "pre-app",
            Self::Impact => "impact",
            Self::Applied => "applied",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TacticalCycleStep {
    pub phase: TacticalCyclePhase,
    pub step_in_phase: u8,
    pub cycle_index: u32,
}

impl Default for TacticalCycleStep {
    fn default() -> Self {
        Self {
            phase: TacticalCyclePhase::Declared,
            step_in_phase: 0,
            cycle_index: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatKernelConfig {
    pub tactical_cycle_steps_per_phase: u8,
    pub strain_max: u16,
    pub strain_decay_per_advance: u16,
    pub flow_entry_threshold: u16,
    pub flow_default_momentum: u8,
    pub flow_decay_per_advance: u8,
    pub fatigue_max: u16,
    pub fatigue_carryover_cap: u16,
    pub tag_default_lifetime: u8,
}

impl Default for CombatKernelConfig {
    fn default() -> Self {
        Self {
            tactical_cycle_steps_per_phase: 2,
            strain_max: 100,
            strain_decay_per_advance: 5,
            flow_entry_threshold: 50,
            flow_default_momentum: 3,
            flow_decay_per_advance: 1,
            fatigue_max: 100,
            fatigue_carryover_cap: 25,
            tag_default_lifetime: 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrainChangeKind {
    Gained,
    Spent,
    Decayed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrainTransition {
    pub before: u16,
    pub after: u16,
    pub requested: u16,
    pub applied: u16,
    pub kind: StrainChangeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Strain {
    pub current: u16,
}

impl Default for Strain {
    fn default() -> Self {
        Self { current: 0 }
    }
}

// gain/spend consumed in cfg(test) and via CombatKernelState delegation.
#[allow(dead_code)]
impl Strain {
    pub fn gain(&mut self, amount: u16, config: &CombatKernelConfig) -> StrainTransition {
        let before = self.current;
        let after = before.saturating_add(amount).min(config.strain_max);
        self.current = after;
        StrainTransition {
            before,
            after,
            requested: amount,
            applied: after - before,
            kind: StrainChangeKind::Gained,
        }
    }

    pub fn spend(&mut self, amount: u16) -> StrainTransition {
        let before = self.current;
        let after = before.saturating_sub(amount);
        self.current = after;
        StrainTransition {
            before,
            after,
            requested: amount,
            applied: before - after,
            kind: StrainChangeKind::Spent,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlowChangeKind {
    Entered,
    Intensified,
    Decayed,
    Exited,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlowState {
    Dormant,
    Active { momentum: u8 },
}

impl Default for FlowState {
    fn default() -> Self {
        Self::Dormant
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlowTransition {
    pub before: FlowState,
    pub after: FlowState,
    pub kind: FlowChangeKind,
}

// enter/exit/momentum consumed in cfg(test) and via CombatKernelState delegation.
#[allow(dead_code)]
impl FlowState {
    pub fn enter(&mut self, config: &CombatKernelConfig) -> FlowTransition {
        let before = *self;
        let after = Self::Active {
            momentum: config.flow_default_momentum,
        };
        *self = after;
        FlowTransition {
            before,
            after,
            kind: match before {
                Self::Dormant => FlowChangeKind::Entered,
                Self::Active { .. } => FlowChangeKind::Intensified,
            },
        }
    }

    pub fn exit(&mut self) -> FlowTransition {
        let before = *self;
        *self = Self::Dormant;
        FlowTransition {
            before,
            after: Self::Dormant,
            kind: FlowChangeKind::Exited,
        }
    }

    pub fn momentum(self) -> u8 {
        match self {
            Self::Dormant => 0,
            Self::Active { momentum } => momentum,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FatigueChangeKind {
    Gained,
    CarriedOver,
    Reset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fatigue {
    pub current: u16,
    pub carried_over: u16,
}

impl Default for Fatigue {
    fn default() -> Self {
        Self {
            current: 0,
            carried_over: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FatigueTransition {
    pub before: u16,
    pub after: u16,
    pub requested: u16,
    pub applied: u16,
    pub carried_over: u16,
    pub kind: FatigueChangeKind,
}

// gain consumed in cfg(test) and via CombatKernelState::gain_fatigue.
#[allow(dead_code)]
impl Fatigue {
    pub fn gain(&mut self, amount: u16, config: &CombatKernelConfig) -> FatigueTransition {
        let before = self.current;
        let after = before.saturating_add(amount).min(config.fatigue_max);
        self.current = after;
        FatigueTransition {
            before,
            after,
            requested: amount,
            applied: after - before,
            carried_over: self.carried_over,
            kind: FatigueChangeKind::Gained,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CombatTagId(pub String);

impl From<&str> for CombatTagId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl From<String> for CombatTagId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatTagChangeKind {
    Added,
    Ticked,
    Consumed,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatTagState {
    pub id: CombatTagId,
    pub turns_left: u8,
    pub consumed: bool,
}

// is_active/tick/consume used in cfg(test) and CombatKernelState::consume_tag; public API.
#[allow(dead_code)]
impl CombatTagState {
    pub fn new(id: impl Into<CombatTagId>, turns_left: u8) -> Self {
        Self {
            id: id.into(),
            turns_left,
            consumed: false,
        }
    }

    pub fn is_active(&self) -> bool {
        !self.consumed && self.turns_left > 0
    }

    pub fn tick(&mut self) -> CombatTagTransition {
        let before = self.clone();
        if self.turns_left > 0 {
            self.turns_left -= 1;
        }
        let after = self.clone();
        let kind = if after.turns_left == 0 && !after.consumed {
            CombatTagChangeKind::Expired
        } else {
            CombatTagChangeKind::Ticked
        };
        CombatTagTransition {
            before,
            after,
            kind,
        }
    }

    pub fn consume(&mut self) -> CombatTagTransition {
        let before = self.clone();
        self.consumed = true;
        self.turns_left = 0;
        let after = self.clone();
        CombatTagTransition {
            before,
            after,
            kind: CombatTagChangeKind::Consumed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CombatTagTransition {
    pub before: CombatTagState,
    pub after: CombatTagState,
    pub kind: CombatTagChangeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CombatBeatId {
    Declared,
    PreApp,
    Impact,
    Damage,
    ExtraHit,
    Applied,
    Resolved,
}

// ALL/as_str reserved public API; not yet consumed by tests or binary.
#[allow(dead_code)]
impl CombatBeatId {
    pub const ALL: [Self; 7] = [
        Self::Declared,
        Self::PreApp,
        Self::Impact,
        Self::Damage,
        Self::ExtraHit,
        Self::Applied,
        Self::Resolved,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Declared => "declared",
            Self::PreApp => "pre-app",
            Self::Impact => "impact",
            Self::Damage => "damage",
            Self::ExtraHit => "extra-hit",
            Self::Applied => "applied",
            Self::Resolved => "resolved",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatKernelTransition {
    TacticalCycle(TacticalCycleTransition),
    Strain(StrainTransition),
    Flow(FlowTransition),
    Fatigue(FatigueTransition),
    Tag(CombatTagTransition),
    Beat(CombatBeatId),
    Blueprint {
        owner: String,
        name: String,
        payload: crate::combat::api::SignalPayload,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TacticalCycleTransition {
    pub before: TacticalCycleStep,
    pub after: TacticalCycleStep,
    pub wrapped_phase: bool,
    pub wrapped_cycle: bool,
}

// advance consumed in cfg(test) and CombatKernelState::advance_tactical_cycle.
#[allow(dead_code)]
impl TacticalCycleStep {
    pub fn advance(self, config: &CombatKernelConfig) -> TacticalCycleTransition {
        let before = self;
        let mut after = self;
        let mut wrapped_phase = false;
        let mut wrapped_cycle = false;

        after.step_in_phase = after.step_in_phase.saturating_add(1);
        if after.step_in_phase >= config.tactical_cycle_steps_per_phase {
            after.step_in_phase = 0;
            wrapped_phase = true;
            let next_phase = after.phase.next();
            wrapped_cycle = matches!(after.phase, TacticalCyclePhase::Applied);
            after.phase = next_phase;
            if wrapped_cycle {
                after.cycle_index = after.cycle_index.saturating_add(1);
            }
        }

        TacticalCycleTransition {
            before,
            after,
            wrapped_phase,
            wrapped_cycle,
        }
    }
}
