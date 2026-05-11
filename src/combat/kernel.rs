use bevy::prelude::{App, Resource, Update};
use serde::{Deserialize, Serialize};

use crate::combat::types::UnitId;

// CombatKernelRegistry must be Resource so it can be accessed via Bevy ECS Res<>.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TacticalCyclePhase {
    Declared,
    PreApp,
    Impact,
    Applied,
}

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

    pub fn decay(&mut self, amount: u16) -> StrainTransition {
        let before = self.current;
        let after = before.saturating_sub(amount);
        self.current = after;
        StrainTransition {
            before,
            after,
            requested: amount,
            applied: before - after,
            kind: StrainChangeKind::Decayed,
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

    pub fn decay(&mut self, config: &CombatKernelConfig) -> FlowTransition {
        let before = *self;
        let after = match before {
            Self::Dormant => Self::Dormant,
            Self::Active { momentum } => {
                let next = momentum.saturating_sub(config.flow_decay_per_advance);
                if next == 0 {
                    Self::Dormant
                } else {
                    Self::Active { momentum: next }
                }
            }
        };
        *self = after;
        FlowTransition {
            before,
            after,
            kind: match (before, after) {
                (Self::Dormant, Self::Dormant) => FlowChangeKind::Decayed,
                (_, Self::Dormant) => FlowChangeKind::Exited,
                _ => FlowChangeKind::Decayed,
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

    pub fn carry_over(&mut self, config: &CombatKernelConfig) -> FatigueTransition {
        let before = self.current;
        let carried_over = before.min(config.fatigue_carryover_cap);
        self.current = 0;
        self.carried_over = carried_over;
        FatigueTransition {
            before,
            after: 0,
            requested: before,
            applied: before,
            carried_over,
            kind: FatigueChangeKind::CarriedOver,
        }
    }

    pub fn reset(&mut self) -> FatigueTransition {
        let before = self.current;
        self.current = 0;
        self.carried_over = 0;
        FatigueTransition {
            before,
            after: 0,
            requested: before,
            applied: before,
            carried_over: 0,
            kind: FatigueChangeKind::Reset,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TwinCoreSignal {
    BuildCrossResonance,
    SpendCrossResonance,
    ThermalSpark,
    TwinBurst,
    Shatter,
    FireSpendMarker,
    IceSpendMarker,
    CycleReset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TwinCoreTransition {
    pub signal: TwinCoreSignal,
    pub amount: u8,
}

impl TwinCoreTransition {
    pub const fn build_cross_resonance(amount: u8) -> Self {
        Self {
            signal: TwinCoreSignal::BuildCrossResonance,
            amount,
        }
    }

    pub const fn spend_cross_resonance(amount: u8) -> Self {
        Self {
            signal: TwinCoreSignal::SpendCrossResonance,
            amount,
        }
    }

    pub const fn thermal_spark(amount: u8) -> Self {
        Self {
            signal: TwinCoreSignal::ThermalSpark,
            amount,
        }
    }

    pub const fn twin_burst(amount: u8) -> Self {
        Self {
            signal: TwinCoreSignal::TwinBurst,
            amount,
        }
    }

    pub const fn shatter(amount: u8) -> Self {
        Self {
            signal: TwinCoreSignal::Shatter,
            amount,
        }
    }

    pub const fn fire_spend_marker(amount: u8) -> Self {
        Self {
            signal: TwinCoreSignal::FireSpendMarker,
            amount,
        }
    }

    pub const fn ice_spend_marker(amount: u8) -> Self {
        Self {
            signal: TwinCoreSignal::IceSpendMarker,
            amount,
        }
    }

    pub const fn cycle_reset() -> Self {
        Self {
            signal: TwinCoreSignal::CycleReset,
            amount: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatteryLoopChargeKind {
    Static,
    Circuit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatteryLoopBlockedReason {
    ChargeCapReached { charge: BatteryLoopChargeKind },
    ChargeUnderflow { charge: BatteryLoopChargeKind },
    MissingPreExistingShock,
    NoEligibleAlly,
    UnsupportedRequest,
    MalformedData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatteryLoopStep {
    BuildStaticCharge { amount: u8 },
    BuildCircuitCharge { amount: u8 },
    SpendCircuitCharge { amount: u8 },
    GrantEnergy { amount: u8 },
    SelfEnergyGain { amount: u8 },
    TransferEnergy { amount: u8 },
    CycleReset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatteryLoopSignal {
    BuildStaticCharge,
    BuildCircuitCharge,
    SpendCircuitCharge,
    GrantEnergy,
    SelfEnergyGain,
    TransferEnergy,
    CycleReset,
    Rejected,
    Ignored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatteryLoopTransition {
    pub signal: BatteryLoopSignal,
    pub amount: u8,
    pub attempted: Option<BatteryLoopStep>,
    pub reason: Option<BatteryLoopBlockedReason>,
}

impl BatteryLoopTransition {
    pub const fn build_static_charge(amount: u8) -> Self {
        Self {
            signal: BatteryLoopSignal::BuildStaticCharge,
            amount,
            attempted: None,
            reason: None,
        }
    }

    pub const fn build_circuit_charge(amount: u8) -> Self {
        Self {
            signal: BatteryLoopSignal::BuildCircuitCharge,
            amount,
            attempted: None,
            reason: None,
        }
    }

    pub const fn spend_circuit_charge(amount: u8) -> Self {
        Self {
            signal: BatteryLoopSignal::SpendCircuitCharge,
            amount,
            attempted: None,
            reason: None,
        }
    }

    pub const fn grant_energy(amount: u8) -> Self {
        Self {
            signal: BatteryLoopSignal::GrantEnergy,
            amount,
            attempted: None,
            reason: None,
        }
    }

    pub const fn self_energy_gain(amount: u8) -> Self {
        Self {
            signal: BatteryLoopSignal::SelfEnergyGain,
            amount,
            attempted: None,
            reason: None,
        }
    }

    pub const fn transfer_energy(amount: u8) -> Self {
        Self {
            signal: BatteryLoopSignal::TransferEnergy,
            amount,
            attempted: None,
            reason: None,
        }
    }

    pub const fn cycle_reset() -> Self {
        Self {
            signal: BatteryLoopSignal::CycleReset,
            amount: 0,
            attempted: None,
            reason: None,
        }
    }

    pub const fn rejected(attempted: BatteryLoopStep, reason: BatteryLoopBlockedReason) -> Self {
        Self {
            signal: BatteryLoopSignal::Rejected,
            amount: 0,
            attempted: Some(attempted),
            reason: Some(reason),
        }
    }

    pub const fn ignored(attempted: BatteryLoopStep) -> Self {
        Self {
            signal: BatteryLoopSignal::Ignored,
            amount: 0,
            attempted: Some(attempted),
            reason: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrecisionWindowKind {
    Momentum,
    Counterplay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrecisionCommitment {
    Press,
    Hold,
    Feint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrecisionReveal {
    Guarded,
    Baited,
    Trapped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrecisionOutcome {
    Success,
    Countered,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrecisionMindGamePhase {
    Dormant,
    WindowOpen,
    CommitmentLocked,
    CounterplayRevealed,
    Resolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrecisionMindGameRejectReason {
    NoOpenWindow,
    WindowAlreadyOpen,
    DuplicateCommitment,
    MissingCommitment,
    DuplicateReveal,
    MissingReveal,
    AlreadyResolved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrecisionMindGameStep {
    OpenWindow { window: PrecisionWindowKind },
    Commit { commitment: PrecisionCommitment },
    Reveal { reveal: PrecisionReveal },
    Resolve { outcome: PrecisionOutcome },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrecisionMindGameTransition {
    OpenWindow {
        window: PrecisionWindowKind,
    },
    Commit {
        commitment: PrecisionCommitment,
    },
    Reveal {
        reveal: PrecisionReveal,
    },
    Resolve {
        outcome: PrecisionOutcome,
    },
    Rejected {
        attempted: PrecisionMindGameStep,
        reason: PrecisionMindGameRejectReason,
    },
    Ignored {
        attempted: PrecisionMindGameStep,
    },
}

impl PrecisionMindGameTransition {
    pub const fn open_window(window: PrecisionWindowKind) -> Self {
        Self::OpenWindow { window }
    }

    pub const fn commit(commitment: PrecisionCommitment) -> Self {
        Self::Commit { commitment }
    }

    pub const fn reveal(reveal: PrecisionReveal) -> Self {
        Self::Reveal { reveal }
    }

    pub const fn resolve(outcome: PrecisionOutcome) -> Self {
        Self::Resolve { outcome }
    }

    pub const fn rejected(
        attempted: PrecisionMindGameStep,
        reason: PrecisionMindGameRejectReason,
    ) -> Self {
        Self::Rejected { attempted, reason }
    }

    pub const fn ignored(attempted: PrecisionMindGameStep) -> Self {
        Self::Ignored { attempted }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredatorLoopCapKind {
    Exploit,
    PreyLock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredatorLoopBlockedReason {
    CapReached { cap: PredatorLoopCapKind },
    MissingExploit,
    MissingPreyLock,
    ExpiredPreyLock,
    InvalidTarget,
    BerserkBlockedByStrain { current: u16, threshold: u16 },
    UnsupportedRequest,
    MalformedData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredatorLoopStep {
    BuildExploit { target: UnitId, amount: u16 },
    ApplyPreyLock { target: UnitId },
    ConsumePreyLockPayoff { target: UnitId },
    EnterBerserk,
    Tick,
    Expire { target: UnitId },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredatorLoopSignal {
    BuildExploit,
    ApplyPreyLock,
    ConsumePreyLockPayoff,
    EnterBerserk,
    Tick,
    Expire,
    Rejected,
    Ignored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PredatorLoopTransition {
    pub signal: PredatorLoopSignal,
    pub target: Option<UnitId>,
    pub amount: u16,
    pub attempted: Option<PredatorLoopStep>,
    pub reason: Option<PredatorLoopBlockedReason>,
}

impl PredatorLoopTransition {
    pub const fn build_exploit(target: UnitId, amount: u16) -> Self {
        Self {
            signal: PredatorLoopSignal::BuildExploit,
            target: Some(target),
            amount,
            attempted: None,
            reason: None,
        }
    }

    pub const fn apply_prey_lock(target: UnitId, duration: u16) -> Self {
        Self {
            signal: PredatorLoopSignal::ApplyPreyLock,
            target: Some(target),
            amount: duration,
            attempted: None,
            reason: None,
        }
    }

    pub const fn consume_prey_lock_payoff(target: UnitId, amount: u16) -> Self {
        Self {
            signal: PredatorLoopSignal::ConsumePreyLockPayoff,
            target: Some(target),
            amount,
            attempted: None,
            reason: None,
        }
    }

    pub const fn enter_berserk(strain_current: u16) -> Self {
        Self {
            signal: PredatorLoopSignal::EnterBerserk,
            target: None,
            amount: strain_current,
            attempted: None,
            reason: None,
        }
    }

    pub const fn tick() -> Self {
        Self {
            signal: PredatorLoopSignal::Tick,
            target: None,
            amount: 0,
            attempted: None,
            reason: None,
        }
    }

    pub const fn expire(target: UnitId) -> Self {
        Self {
            signal: PredatorLoopSignal::Expire,
            target: Some(target),
            amount: 0,
            attempted: None,
            reason: None,
        }
    }

    pub const fn rejected(attempted: PredatorLoopStep, reason: PredatorLoopBlockedReason) -> Self {
        Self {
            signal: PredatorLoopSignal::Rejected,
            target: match attempted {
                PredatorLoopStep::BuildExploit { target, .. }
                | PredatorLoopStep::ApplyPreyLock { target }
                | PredatorLoopStep::ConsumePreyLockPayoff { target }
                | PredatorLoopStep::Expire { target } => Some(target),
                PredatorLoopStep::EnterBerserk | PredatorLoopStep::Tick => None,
            },
            amount: 0,
            attempted: Some(attempted),
            reason: Some(reason),
        }
    }

    pub const fn ignored(attempted: PredatorLoopStep) -> Self {
        Self {
            signal: PredatorLoopSignal::Ignored,
            target: match attempted {
                PredatorLoopStep::BuildExploit { target, .. }
                | PredatorLoopStep::ApplyPreyLock { target }
                | PredatorLoopStep::ConsumePreyLockPayoff { target }
                | PredatorLoopStep::Expire { target } => Some(target),
                PredatorLoopStep::EnterBerserk | PredatorLoopStep::Tick => None,
            },
            amount: 0,
            attempted: Some(attempted),
            reason: None,
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
    TwinCore(TwinCoreTransition),
    BatteryLoop(BatteryLoopTransition),
    HolySupport(HolySupportTransition),
    PredatorLoop(PredatorLoopTransition),
    PrecisionMindGame(PrecisionMindGameTransition),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TacticalCycleTransition {
    pub before: TacticalCycleStep,
    pub after: TacticalCycleStep,
    pub wrapped_phase: bool,
    pub wrapped_cycle: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HolySupportSignal {
    BuildGrace,
    SpendGrace,
    MarkMartyrLight,
    ConsumeMartyrLight,
    CycleReset,
    Rejected,
    Ignored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HolySupportStep {
    BuildGrace { amount: u8 },
    SpendGrace { amount: u8 },
    MarkMartyrLight,
    ConsumeMartyrLight,
    CycleReset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HolySupportRejectReason {
    GraceUnderflow,
    MartyrAlreadyMarked,
    MartyrNotMarked,
    MartyrAlreadyConsumed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HolySupportTransition {
    pub signal: HolySupportSignal,
    pub amount: u8,
    pub attempted: Option<HolySupportStep>,
    pub reason: Option<HolySupportRejectReason>,
}

impl HolySupportTransition {
    pub const fn build_grace(amount: u8) -> Self {
        Self {
            signal: HolySupportSignal::BuildGrace,
            amount,
            attempted: None,
            reason: None,
        }
    }

    pub const fn spend_grace(amount: u8) -> Self {
        Self {
            signal: HolySupportSignal::SpendGrace,
            amount,
            attempted: None,
            reason: None,
        }
    }

    pub const fn mark_martyr_light() -> Self {
        Self {
            signal: HolySupportSignal::MarkMartyrLight,
            amount: 0,
            attempted: None,
            reason: None,
        }
    }

    pub const fn consume_martyr_light() -> Self {
        Self {
            signal: HolySupportSignal::ConsumeMartyrLight,
            amount: 0,
            attempted: None,
            reason: None,
        }
    }

    pub const fn cycle_reset() -> Self {
        Self {
            signal: HolySupportSignal::CycleReset,
            amount: 0,
            attempted: None,
            reason: None,
        }
    }

    pub const fn rejected(attempted: HolySupportStep, reason: HolySupportRejectReason) -> Self {
        Self {
            signal: HolySupportSignal::Rejected,
            amount: 0,
            attempted: Some(attempted),
            reason: Some(reason),
        }
    }

    pub const fn ignored(attempted: HolySupportStep) -> Self {
        Self {
            signal: HolySupportSignal::Ignored,
            amount: 0,
            attempted: Some(attempted),
            reason: None,
        }
    }
}

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

#[derive(Default, Resource)]
pub struct CombatKernelRegistry {
    hooks: Vec<Box<dyn CombatKernelHook>>,
}

impl CombatKernelRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<H>(&mut self, hook: H)
    where
        H: CombatKernelHook + 'static,
    {
        self.hooks.push(Box::new(hook));
    }

    pub fn dispatch(&self, transition: CombatKernelTransition) -> Vec<CombatKernelTransition> {
        let mut out = vec![transition.clone()];
        for hook in &self.hooks {
            hook.on_transition(&transition, &mut out);
        }
        out
    }
}

pub fn register_combat_kernel_runtime(app: &mut App) {
    let mut registry = CombatKernelRegistry::new();
    registry.register(crate::combat::twin_core::TwinCoreHook);
    registry.register(crate::combat::battery_loop::BatteryLoopHook);
    registry.register(crate::combat::holy_support::HolySupportHook);
    registry.register(crate::combat::predator_loop::PredatorLoopHook);
    registry.register(crate::combat::precision_mind_game::PrecisionMindGameHook);

    app.init_resource::<CombatKernelState>()
        .init_resource::<crate::combat::twin_core::TwinCoreState>()
        .init_resource::<crate::combat::battery_loop::BatteryLoopState>()
        .init_resource::<crate::combat::holy_support::HolySupportState>()
        .init_resource::<crate::combat::predator_loop::PredatorLoopState>()
        .init_resource::<crate::combat::precision_mind_game::PrecisionMindGameState>()
        .add_systems(
            Update,
            (
                crate::combat::battery_loop::apply_battery_loop_transitions_system,
                crate::combat::predator_loop::apply_predator_loop_transitions_system,
                crate::combat::twin_core::apply_twin_core_transitions_system,
                crate::combat::holy_support::apply_holy_support_transitions_system,
                crate::combat::precision_mind_game::apply_precision_mind_game_transitions_system,
            ),
        )
        .insert_resource(registry);
}

pub trait CombatKernelHook: Send + Sync {
    fn domain(&self) -> CombatKernelHookDomain;

    fn on_transition(
        &self,
        _transition: &CombatKernelTransition,
        _out: &mut Vec<CombatKernelTransition>,
    ) {
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CombatKernelHookDomain {
    Digimon,
    Enemy,
    Party,
    Shared,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Resource)]
pub struct CombatKernelState {
    pub config: CombatKernelConfig,
    pub tactical_cycle: TacticalCycleStep,
    pub strain: Strain,
    pub flow: FlowState,
    pub fatigue: Fatigue,
    pub tags: Vec<CombatTagState>,
}

impl Default for CombatKernelState {
    fn default() -> Self {
        Self {
            config: CombatKernelConfig::default(),
            tactical_cycle: TacticalCycleStep::default(),
            strain: Strain::default(),
            flow: FlowState::default(),
            fatigue: Fatigue::default(),
            tags: Vec::new(),
        }
    }
}

impl CombatKernelState {
    pub fn gain_strain(&mut self, amount: u16) -> StrainTransition {
        self.strain.gain(amount, &self.config)
    }

    pub fn spend_strain(&mut self, amount: u16) -> StrainTransition {
        self.strain.spend(amount)
    }

    pub fn decay_strain(&mut self) -> StrainTransition {
        self.strain.decay(self.config.strain_decay_per_advance)
    }

    pub fn enter_flow(&mut self) -> FlowTransition {
        self.flow.enter(&self.config)
    }

    pub fn decay_flow(&mut self) -> FlowTransition {
        self.flow.decay(&self.config)
    }

    pub fn exit_flow(&mut self) -> FlowTransition {
        self.flow.exit()
    }

    pub fn gain_fatigue(&mut self, amount: u16) -> FatigueTransition {
        self.fatigue.gain(amount, &self.config)
    }

    pub fn carry_over_fatigue(&mut self) -> FatigueTransition {
        self.fatigue.carry_over(&self.config)
    }

    pub fn reset_fatigue(&mut self) -> FatigueTransition {
        self.fatigue.reset()
    }

    pub fn advance_tactical_cycle(&mut self) -> TacticalCycleTransition {
        let transition = self.tactical_cycle.advance(&self.config);
        self.tactical_cycle = transition.after;
        transition
    }

    pub fn add_tag(&mut self, id: impl Into<CombatTagId>) -> CombatTagTransition {
        let tag = CombatTagState::new(id, self.config.tag_default_lifetime);
        let before = tag.clone();
        self.tags.push(tag.clone());
        CombatTagTransition {
            before,
            after: tag,
            kind: CombatTagChangeKind::Added,
        }
    }

    pub fn consume_tag(&mut self, id: &CombatTagId) -> Option<CombatTagTransition> {
        let tag = self.tags.iter_mut().find(|tag| &tag.id == id)?;
        Some(tag.consume())
    }

    pub fn tick_tags(&mut self) -> Vec<CombatTagTransition> {
        self.tags.iter_mut().map(CombatTagState::tick).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strain_clamps_gain_and_spend() {
        let config = CombatKernelConfig::default();
        let mut strain = Strain::default();

        let gain = strain.gain(150, &config);
        assert_eq!(strain.current, config.strain_max);
        assert_eq!(gain.after, config.strain_max);
        assert_eq!(gain.applied, config.strain_max);

        let spend = strain.spend(80);
        assert_eq!(strain.current, 20);
        assert_eq!(spend.kind, StrainChangeKind::Spent);
        assert_eq!(spend.applied, 80);

        let decay = strain.decay(50);
        assert_eq!(strain.current, 0);
        assert_eq!(decay.applied, 20);
    }

    #[test]
    fn flow_enters_decays_and_exits() {
        let config = CombatKernelConfig::default();
        let mut flow = FlowState::default();

        let entered = flow.enter(&config);
        assert_eq!(entered.kind, FlowChangeKind::Entered);
        assert_eq!(
            flow,
            FlowState::Active {
                momentum: config.flow_default_momentum
            }
        );

        let decayed = flow.decay(&config);
        assert_eq!(decayed.kind, FlowChangeKind::Decayed);
        assert_eq!(
            flow.momentum(),
            config.flow_default_momentum - config.flow_decay_per_advance
        );

        let exited = flow.exit();
        assert_eq!(exited.kind, FlowChangeKind::Exited);
        assert_eq!(flow, FlowState::Dormant);
    }

    #[test]
    fn fatigue_carryover_and_reset_are_deterministic() {
        let config = CombatKernelConfig::default();
        let mut fatigue = Fatigue::default();

        let gained = fatigue.gain(80, &config);
        assert_eq!(fatigue.current, 80);
        assert_eq!(gained.applied, 80);

        let carried = fatigue.carry_over(&config);
        assert_eq!(carried.kind, FatigueChangeKind::CarriedOver);
        assert_eq!(fatigue.current, 0);
        assert_eq!(fatigue.carried_over, config.fatigue_carryover_cap);
        assert_eq!(carried.carried_over, config.fatigue_carryover_cap);

        let reset = fatigue.reset();
        assert_eq!(reset.kind, FatigueChangeKind::Reset);
        assert_eq!(fatigue.current, 0);
        assert_eq!(fatigue.carried_over, 0);
    }

    #[test]
    fn tactical_cycle_advances_through_phases_and_wraps_cycle() {
        let config = CombatKernelConfig::default();
        let mut step = TacticalCycleStep::default();

        let first = step.advance(&config);
        assert_eq!(first.after.step_in_phase, 1);
        assert!(!first.wrapped_phase);
        assert_eq!(first.after.phase, TacticalCyclePhase::Declared);

        let second = first.after.advance(&config);
        assert!(second.wrapped_phase);
        assert_eq!(second.after.phase, TacticalCyclePhase::PreApp);
        assert_eq!(second.after.step_in_phase, 0);

        let cycle_wrap = TacticalCycleStep {
            phase: TacticalCyclePhase::Applied,
            step_in_phase: config.tactical_cycle_steps_per_phase - 1,
            cycle_index: 3,
        }
        .advance(&config);
        assert!(cycle_wrap.wrapped_cycle);
        assert_eq!(cycle_wrap.after.phase, TacticalCyclePhase::Declared);
        assert_eq!(cycle_wrap.after.cycle_index, 4);
    }

    #[test]
    fn canonical_beat_ids_are_stable_and_ordered() {
        let labels: Vec<&'static str> = CombatBeatId::ALL
            .into_iter()
            .map(CombatBeatId::as_str)
            .collect();
        assert_eq!(
            labels,
            vec![
                "declared",
                "pre-app",
                "impact",
                "damage",
                "extra-hit",
                "applied",
                "resolved"
            ]
        );
    }

    struct HookSmokeTest;

    impl CombatKernelHook for HookSmokeTest {
        fn domain(&self) -> CombatKernelHookDomain {
            CombatKernelHookDomain::Shared
        }

        fn on_transition(
            &self,
            transition: &CombatKernelTransition,
            out: &mut Vec<CombatKernelTransition>,
        ) {
            if matches!(
                transition,
                CombatKernelTransition::Flow(FlowTransition {
                    kind: FlowChangeKind::Entered,
                    ..
                })
            ) {
                out.push(CombatKernelTransition::Beat(CombatBeatId::Impact));
            }
        }
    }

    #[test]
    fn registry_can_attach_extension_behavior_without_core_match_ladders() {
        let mut registry = CombatKernelRegistry::new();
        registry.register(HookSmokeTest);

        let transitions = registry.dispatch(CombatKernelTransition::Flow(FlowTransition {
            before: FlowState::Dormant,
            after: FlowState::Active { momentum: 3 },
            kind: FlowChangeKind::Entered,
        }));

        assert_eq!(transitions.len(), 2);
        assert!(matches!(transitions[0], CombatKernelTransition::Flow(_)));
        assert!(matches!(
            transitions[1],
            CombatKernelTransition::Beat(CombatBeatId::Impact)
        ));
    }

    #[test]
    fn tag_lifetime_ticks_and_consumes() {
        let mut tag = CombatTagState::new("burning", 2);

        let tick = tag.tick();
        assert_eq!(tick.kind, CombatTagChangeKind::Ticked);
        assert_eq!(tag.turns_left, 1);
        assert!(tag.is_active());

        let consumed = tag.consume();
        assert_eq!(consumed.kind, CombatTagChangeKind::Consumed);
        assert!(tag.consumed);
        assert_eq!(tag.turns_left, 0);
        assert!(!tag.is_active());
    }

    #[test]
    fn state_delegates_to_kernel_primitives() {
        let mut kernel = CombatKernelState::default();

        let strain = kernel.gain_strain(55);
        let flow = kernel.enter_flow();
        let cycle = kernel.advance_tactical_cycle();
        let tag = kernel.add_tag("combo_ready");
        let consumed = kernel.consume_tag(&tag.after.id).expect("tag exists");

        assert_eq!(strain.after, 55);
        assert!(matches!(flow.after, FlowState::Active { .. }));
        assert_eq!(cycle.after.step_in_phase, 1);
        assert_eq!(kernel.tags.len(), 1);
        assert!(consumed.after.consumed);
    }
}
