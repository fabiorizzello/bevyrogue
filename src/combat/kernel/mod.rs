mod primitives;

pub use primitives::*;

use bevy::prelude::{App, Resource};
use serde::{Deserialize, Serialize};

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
        .init_resource::<crate::combat::runtime::ExtRegistries>()
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
