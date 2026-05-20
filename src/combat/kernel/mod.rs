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
    fn domain(&self) -> CombatKernelHookDomain;

    fn on_transition(
        &self,
        _transition: &CombatKernelTransition,
        _out: &mut Vec<CombatKernelTransition>,
    ) {
    }
}

// Used by blueprint hook impls (dorumon, patamon, renamon, tentomon) to classify hooks.
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
