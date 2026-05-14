use crate::combat::kernel::{
    CombatKernelHook, CombatKernelHookDomain, CombatKernelTransition, PredatorLoopTransition,
    TacticalCycleTransition,
};

use super::identity::PredatorLoopHook;

impl CombatKernelHook for PredatorLoopHook {
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
            CombatKernelTransition::TacticalCycle(TacticalCycleTransition {
                wrapped_cycle: true,
                ..
            })
        ) {
            out.push(CombatKernelTransition::PredatorLoop(
                PredatorLoopTransition::tick(),
            ));
        }
    }
}
