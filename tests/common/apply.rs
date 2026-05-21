//! Thin wrapper over `apply_legacy_ops` (13 args) that owns the mutable
//! state required by the call and exposes it for post-call assertions.

#![allow(dead_code)]

use bevyrogue::combat::StatusBag;
use bevyrogue::combat::buffs::DrBag;
use bevyrogue::combat::events::CombatEventKind;
use bevyrogue::combat::resolution::{ResolutionOutcome, apply_legacy_ops};
use bevyrogue::combat::sp::{RoundSpTracker, SpPool};
use bevyrogue::combat::state::ResolvedAction;
use bevyrogue::combat::team::Team;
use bevyrogue::combat::toughness::Toughness;
use bevyrogue::combat::ultimate::UltimateCharge;
use bevyrogue::combat::unit::{BasicStreak, Unit};

use super::actions::default_ult;
use super::units::{attacker, defender};

/// Per-call flags & optional status bags for `apply_legacy_ops`.
#[derive(Default)]
pub struct ApplyOpts<'a> {
    pub attacker_status: Option<&'a StatusBag>,
    pub defender_status: Option<&'a StatusBag>,
    pub defender_dr: Option<&'a DrBag>,
    pub defender_is_commander: bool,
    pub defender_break_sealed: bool,
}

/// Owns the mutable state for one `apply_legacy_ops` invocation so tests can
/// inspect `harness.ult.current`, `harness.defender.hp_current`, … afterwards.
pub struct LegacyOpsHarness {
    pub attacker: Unit,
    pub defender: Unit,
    pub defender_team: Team,
    pub tough: Toughness,
    pub ult: UltimateCharge,
    pub sp: SpPool,
    pub round_sp: RoundSpTracker,
    pub streak: BasicStreak,
}

impl Default for LegacyOpsHarness {
    fn default() -> Self {
        Self {
            attacker: attacker(),
            defender: defender(),
            defender_team: Team::Enemy,
            tough: Toughness::new(1_000, vec![]),
            ult: default_ult(),
            sp: SpPool { current: 5, max: 5 },
            round_sp: RoundSpTracker::default(),
            streak: BasicStreak::default(),
        }
    }
}

impl LegacyOpsHarness {
    pub fn with_ult(mut self, ult: UltimateCharge) -> Self {
        self.ult = ult;
        self
    }

    pub fn apply(
        &mut self,
        action: &ResolvedAction,
        opts: ApplyOpts<'_>,
    ) -> (ResolutionOutcome, Vec<CombatEventKind>) {
        apply_legacy_ops(
            action,
            &self.attacker,
            &mut self.defender,
            self.defender_team,
            Some(&mut self.tough),
            &mut self.ult,
            &mut self.sp,
            &mut self.round_sp,
            &mut self.streak,
            opts.defender_is_commander,
            opts.defender_break_sealed,
            opts.defender_status,
            opts.attacker_status,
            opts.defender_dr,
            None,
            None,
        )
    }
}

/// Convenience: run one action against the default harness, return the damage
/// from the first `OnDamageDealt` event (panics if none).
pub fn run_damage(action: &ResolvedAction, attacker_status: Option<&StatusBag>) -> i32 {
    let mut harness = LegacyOpsHarness::default();
    let (_, events) = harness.apply(
        action,
        ApplyOpts {
            attacker_status,
            ..Default::default()
        },
    );
    events
        .iter()
        .find_map(|e| match e {
            CombatEventKind::OnDamageDealt { amount, .. } => Some(*amount),
            _ => None,
        })
        .expect("OnDamageDealt must be emitted")
}

/// Convenience: run one action and return the delta added to the attacker's
/// ult meter (`final - initial`).
pub fn run_ult_delta(action: &ResolvedAction, attacker_status: Option<&StatusBag>) -> i32 {
    let mut harness = LegacyOpsHarness::default();
    let before = harness.ult.current;
    harness.apply(
        action,
        ApplyOpts {
            attacker_status,
            ..Default::default()
        },
    );
    harness.ult.current - before
}
