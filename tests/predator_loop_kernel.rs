use bevyrogue::combat::events::{CombatEvent, CombatEventKind};
use bevyrogue::combat::kernel::{
    PredatorLoopBlockedReason, PredatorLoopCapKind, PredatorLoopSignal,
    PredatorLoopStep, PredatorLoopTransition,
};
use bevyrogue::combat::observability::{
    ValidationSnapshot, ValidationTwinCoreSnapshot, format_validation_snapshot,
};
use bevyrogue::combat::blueprints::dorumon::{PredatorLoopSnapshot, PredatorLoopState};
use bevyrogue::combat::state::CombatPhase;
use bevyrogue::combat::types::UnitId;

fn tracked_state() -> PredatorLoopState {
    let mut state = PredatorLoopState::default();
    state.track_target(UnitId(7));
    state
}

fn rejected_build(
    target: UnitId,
    amount: u16,
    reason: PredatorLoopBlockedReason,
) -> PredatorLoopTransition {
    PredatorLoopTransition::rejected(PredatorLoopStep::BuildExploit { target, amount }, reason)
}

#[test]
fn predator_loop_builds_exploit_applies_prey_lock_and_consumes_payoff() {
    let target = UnitId(7);
    let mut state = tracked_state();

    let build = state.build_exploit(target, 2);
    assert_eq!(build, PredatorLoopTransition::build_exploit(target, 2));
    assert_eq!(state.targets.get(&target).unwrap().exploit_stacks, 2);
    assert_eq!(state.last_transition, Some(build));
    assert_eq!(state.last_blocked_reason, None);

    let prey_lock = state.apply_prey_lock(target);
    assert_eq!(
        prey_lock,
        PredatorLoopTransition::apply_prey_lock(target, state.prey_lock_duration as u16)
    );
    assert_eq!(
        state
            .targets
            .get(&target)
            .unwrap()
            .prey_lock
            .unwrap()
            .turns_left,
        state.prey_lock_duration
    );

    let payoff = state.consume_prey_lock_payoff(target);
    assert_eq!(
        payoff,
        PredatorLoopTransition::consume_prey_lock_payoff(target, 1)
    );
    assert_eq!(state.targets.get(&target).unwrap().exploit_stacks, 1);
    let lock = state.targets.get(&target).unwrap().prey_lock.unwrap();
    assert!(lock.consumed);
    assert_eq!(lock.turns_left, 0);
    assert_eq!(state.last_transition, Some(payoff));
    assert_eq!(state.last_blocked_reason, None);
}

#[test]
fn predator_loop_rejects_cap_overflow_and_records_blocked_reason() {
    let target = UnitId(7);
    let mut state = tracked_state();

    let build = state.build_exploit(target, 3);
    assert_eq!(build, PredatorLoopTransition::build_exploit(target, 3));

    let blocked = state.build_exploit(target, 1);
    assert_eq!(
        blocked,
        PredatorLoopTransition::rejected(
            PredatorLoopStep::BuildExploit { target, amount: 1 },
            PredatorLoopBlockedReason::CapReached {
                cap: PredatorLoopCapKind::Exploit,
            },
        )
    );
    assert_eq!(state.last_transition, Some(blocked));
    assert_eq!(
        state.last_blocked_reason,
        Some(PredatorLoopBlockedReason::CapReached {
            cap: PredatorLoopCapKind::Exploit,
        })
    );
}

#[test]
fn predator_loop_requires_exploit_before_prey_lock() {
    let target = UnitId(7);
    let mut state = tracked_state();

    let blocked = state.apply_prey_lock(target);
    assert_eq!(
        blocked,
        PredatorLoopTransition::rejected(
            PredatorLoopStep::ApplyPreyLock { target },
            PredatorLoopBlockedReason::MissingExploit,
        )
    );
    assert_eq!(
        state.last_blocked_reason,
        Some(PredatorLoopBlockedReason::MissingExploit)
    );
}

#[test]
fn predator_loop_ticking_expires_prey_lock_and_blocks_consumption_after_expiry() {
    let target = UnitId(7);
    let mut state = tracked_state();
    state.build_exploit(target, 1);
    state.apply_prey_lock(target);

    let tick_1 = state.tick_all_prey_locks();
    assert_eq!(tick_1.signal, PredatorLoopSignal::Tick);
    assert_eq!(
        state
            .targets
            .get(&target)
            .unwrap()
            .prey_lock
            .unwrap()
            .turns_left,
        1
    );

    let tick_2 = state.tick_all_prey_locks();
    assert_eq!(tick_2.signal, PredatorLoopSignal::Tick);
    assert_eq!(
        state
            .targets
            .get(&target)
            .unwrap()
            .prey_lock
            .unwrap()
            .turns_left,
        0
    );

    let blocked = state.consume_prey_lock_payoff(target);
    assert_eq!(
        blocked,
        PredatorLoopTransition::rejected(
            PredatorLoopStep::ConsumePreyLockPayoff { target },
            PredatorLoopBlockedReason::ExpiredPreyLock,
        )
    );
    assert_eq!(
        state.last_blocked_reason,
        Some(PredatorLoopBlockedReason::ExpiredPreyLock)
    );
}

#[test]
fn predator_loop_rejects_invalid_target_and_malformed_builds() {
    let mut state = PredatorLoopState::default();

    let blocked = state.build_exploit(UnitId(99), 1);
    assert_eq!(
        blocked,
        PredatorLoopTransition::rejected(
            PredatorLoopStep::BuildExploit {
                target: UnitId(99),
                amount: 1,
            },
            PredatorLoopBlockedReason::InvalidTarget,
        )
    );
    assert_eq!(
        state.last_blocked_reason,
        Some(PredatorLoopBlockedReason::InvalidTarget)
    );

    let malformed = state.build_exploit(UnitId(0), 0);
    assert_eq!(
        malformed,
        PredatorLoopTransition::rejected(
            PredatorLoopStep::BuildExploit {
                target: UnitId(0),
                amount: 0,
            },
            PredatorLoopBlockedReason::MalformedData,
        )
    );
    assert_eq!(
        state.last_blocked_reason,
        Some(PredatorLoopBlockedReason::MalformedData)
    );
}

#[test]
fn predator_loop_blocks_berserk_when_strain_is_at_threshold() {
    let mut state = PredatorLoopState::default();
    state.berserk_strain_threshold = 50;

    let blocked = state.enter_berserk(50);
    assert_eq!(
        blocked,
        PredatorLoopTransition::rejected(
            PredatorLoopStep::EnterBerserk,
            PredatorLoopBlockedReason::BerserkBlockedByStrain {
                current: 50,
                threshold: 50,
            },
        )
    );
    assert_eq!(
        state.last_blocked_reason,
        Some(PredatorLoopBlockedReason::BerserkBlockedByStrain {
            current: 50,
            threshold: 50,
        })
    );

    let allowed = state.enter_berserk(49);
    assert_eq!(allowed.signal, PredatorLoopSignal::EnterBerserk);
    assert_eq!(allowed.amount, 49);
    assert_eq!(state.last_blocked_reason, None);
}

#[test]
fn predator_loop_event_and_snapshot_surfaces_are_serializable_and_readable() {
    let target = UnitId(7);
    let mut state = tracked_state();
    state.build_exploit(target, 2);
    state.apply_prey_lock(target);
    state.consume_prey_lock_payoff(target);

    let snapshot: PredatorLoopSnapshot = state.snapshot();
    assert_eq!(snapshot.targets.len(), 1);
    assert_eq!(snapshot.targets[0].unit_id, target);

    let validation = ValidationSnapshot {
        phase: CombatPhase::WaitingAction,
        winner: None,
        sp_current: 3,
        sp_max: 5,
        turn_preview: vec![target],
        action_log_tail: vec![],
        floating_live: 0,
        units: vec![],
        twin_core: ValidationTwinCoreSnapshot {
            active_thermal_spark_targets: vec![],
            cross_resonance: 0,
            fire_spend_markers: 0,
            ice_spend_markers: 0,
            twin_burst_used_this_cycle: false,
            shatter_used_this_cycle: false,
            last_signal: None,
        },
        holy_support: None,
        predator_loop: Some(snapshot.clone()),
        battery_loop: None,
        precision_mind_game: None,
    };
    let rendered = format_validation_snapshot(&validation);
    assert!(rendered.contains("predator"));
    assert!(rendered.contains("battery_loop=none"));
    assert!(rendered.contains("exploit_cap=3"));
    assert!(rendered.contains("targets=[7"));

    let transition = PredatorLoopTransition::build_exploit(target, 2);
    let event = CombatEvent {
        kind: CombatEventKind::PredatorLoopResolved { transition },
        source: target,
        target,
        follow_up_depth: 0,
    };
    let json = serde_json::to_string(&event).expect("serialize predator event");
    assert!(json.contains("PredatorLoopResolved"));
    assert!(json.contains("BuildExploit"));
    assert!(json.contains("7"));

    let transition_json =
        serde_json::to_string(&transition).expect("serialize predator transition");
    let roundtrip: PredatorLoopTransition =
        serde_json::from_str(&transition_json).expect("roundtrip predator transition");
    assert_eq!(roundtrip, transition);
}
