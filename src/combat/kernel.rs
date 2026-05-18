use bevy::prelude::{App, Resource};
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
    app.init_resource::<CombatKernelState>()
        .init_resource::<crate::combat::modifiers::DamageModifierLedger>()
        .init_resource::<crate::combat::rng::CombatRng>()
        .init_resource::<crate::combat::api::ExtRegistries>()
        .insert_resource(CombatKernelRegistry::new());
}

pub trait CombatKernelHook: Send + Sync {
    // domain() is part of the hook trait API; implemented by blueprint hooks.
    #[allow(dead_code)]
    fn domain(&self) -> CombatKernelHookDomain;

    fn on_transition(
        &self,
        _transition: &CombatKernelTransition,
        _out: &mut Vec<CombatKernelTransition>,
    ) {
    }
}

// Used by blueprint hook impls (dorumon, patamon, renamon, tentomon) to classify hooks.
#[allow(dead_code)]
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

// Methods consumed in cfg(test) within this file; public API for blueprint systems.
#[allow(dead_code)]
impl CombatKernelState {
    pub fn gain_strain(&mut self, amount: u16) -> StrainTransition {
        self.strain.gain(amount, &self.config)
    }

    pub fn spend_strain(&mut self, amount: u16) -> StrainTransition {
        self.strain.spend(amount)
    }

    pub fn enter_flow(&mut self) -> FlowTransition {
        self.flow.enter(&self.config)
    }

    pub fn gain_fatigue(&mut self, amount: u16) -> FatigueTransition {
        self.fatigue.gain(amount, &self.config)
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
    }

    #[test]
    fn flow_enters_and_exits() {
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

        let intensified = flow.enter(&config);
        assert_eq!(intensified.kind, FlowChangeKind::Intensified);
        assert_eq!(
            flow,
            FlowState::Active {
                momentum: config.flow_default_momentum
            }
        );

        let exited = flow.exit();
        assert_eq!(exited.kind, FlowChangeKind::Exited);
        assert_eq!(flow, FlowState::Dormant);
    }

    #[test]
    fn fatigue_gain_clamps_deterministically() {
        let config = CombatKernelConfig::default();
        let mut fatigue = Fatigue::default();

        let gained = fatigue.gain(80, &config);
        assert_eq!(fatigue.current, 80);
        assert_eq!(gained.applied, 80);
        assert_eq!(gained.kind, FatigueChangeKind::Gained);

        let capped = fatigue.gain(config.fatigue_max, &config);
        assert_eq!(fatigue.current, config.fatigue_max);
        assert_eq!(capped.after, config.fatigue_max);
    }

    #[test]
    fn tactical_cycle_advances_through_phases_and_wraps_cycle() {
        let config = CombatKernelConfig::default();
        let step = TacticalCycleStep::default();

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
