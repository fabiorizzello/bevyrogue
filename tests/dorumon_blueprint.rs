use bevyrogue::combat::runtime::SignalPayload;
use bevyrogue::combat::blueprints::{self, CustomSignalDispatchError};
use bevyrogue::combat::kernel::CombatKernelTransition;
use bevyrogue::combat::state::{ResolvedAction, UltEffect};
use bevyrogue::combat::types::{DamageTag, SkillId, UnitId};
use bevyrogue::data::skills_ron::{CustomSignalPayload, SkillCustomSignal};

fn base_action() -> ResolvedAction {
    ResolvedAction {
        source: UnitId(1),
        target: UnitId(2),
        skill_id: SkillId("test".into()),
        damage_tag: DamageTag::Dark,
        base_damage: 10,
        toughness_damage: 5,
        revive_pct: 0,
        heal_pct: 0,
        sp_cost: 0,
        ult_effect: UltEffect::None,
        grant_free_skill_count: 0,
        status_to_apply: None,
        advance_pct: 0,
        delay_pct: 0,
        energy_grant: 0,
        self_advance_pct: 0,
        target_shape: bevyrogue::data::skills_ron::TargetShape::Single,
        custom_signals: Vec::new(),
        damage_curve: Default::default(),
        cleanse_count: None,
    }
}

fn custom_signal(owner: &str, signal: &str, payload: CustomSignalPayload) -> SkillCustomSignal {
    SkillCustomSignal::blueprint(owner, signal, payload)
}

fn dorumon_blueprint_transition(name: &str, payload: SignalPayload) -> CombatKernelTransition {
    CombatKernelTransition::Blueprint {
        owner: "dorumon".to_string(),
        name: name.to_string(),
        payload,
    }
}

fn dorumon_transitions(
    action: &ResolvedAction,
    signal: &str,
    payload: CustomSignalPayload,
) -> Vec<CombatKernelTransition> {
    let custom = custom_signal("dorumon", signal, payload);
    blueprints::dispatch_custom_signal(&custom, action).expect("dispatch")
}

#[test]
fn dorumon_build_exploit_maps_to_blueprint_transition() {
    let action = base_action();
    let transitions = dorumon_transitions(
        &action,
        "build_exploit",
        CustomSignalPayload::Amount { amount: 2 },
    );

    assert_eq!(
        transitions,
        vec![dorumon_blueprint_transition(
            "build_exploit",
            SignalPayload::Amount(2)
        )]
    );
}

#[test]
fn dorumon_apply_prey_lock_uses_blueprint_owner_envelope() {
    let action = base_action();
    let transitions = dorumon_transitions(&action, "apply_prey_lock", CustomSignalPayload::Empty);

    assert_eq!(
        transitions,
        vec![dorumon_blueprint_transition(
            "apply_prey_lock",
            SignalPayload::Amount(0)
        )]
    );
}

#[test]
fn dorumon_consume_payoff_maps_to_blueprint_transition() {
    let action = base_action();
    let transitions = dorumon_transitions(
        &action,
        "consume_prey_lock_payoff",
        CustomSignalPayload::Empty,
    );

    assert_eq!(
        transitions,
        vec![dorumon_blueprint_transition(
            "consume_prey_lock_payoff",
            SignalPayload::Amount(1),
        )]
    );
}

#[test]
fn dorumon_enter_berserk_maps_to_blueprint_transition() {
    let action = base_action();
    let transitions = dorumon_transitions(&action, "enter_berserk", CustomSignalPayload::Empty);

    assert_eq!(
        transitions,
        vec![dorumon_blueprint_transition(
            "enter_berserk",
            SignalPayload::Amount(0)
        )]
    );
}

#[test]
fn multiple_dorumon_signals_preserve_order() {
    let mut action = base_action();
    action.custom_signals = vec![
        custom_signal(
            "dorumon",
            "build_exploit",
            CustomSignalPayload::Amount { amount: 1 },
        ),
        custom_signal("dorumon", "apply_prey_lock", CustomSignalPayload::Empty),
        custom_signal(
            "dorumon",
            "consume_prey_lock_payoff",
            CustomSignalPayload::Empty,
        ),
    ];

    let transitions = blueprints::transitions_for_action(&action);
    assert_eq!(
        transitions,
        vec![
            dorumon_blueprint_transition("build_exploit", SignalPayload::Amount(1)),
            dorumon_blueprint_transition("apply_prey_lock", SignalPayload::Amount(0)),
            dorumon_blueprint_transition("consume_prey_lock_payoff", SignalPayload::Amount(1)),
        ]
    );
}

#[test]
fn unknown_owner_and_signal_are_rejected() {
    let action = base_action();
    let unknown_owner = custom_signal(
        "unknown",
        "build_exploit",
        CustomSignalPayload::Amount { amount: 1 },
    );
    let owner_error = blueprints::dispatch_custom_signal(&unknown_owner, &action)
        .expect_err("unknown owner should be rejected");
    assert_eq!(
        owner_error,
        CustomSignalDispatchError::UnknownOwner {
            owner: "unknown".into()
        }
    );

    let unknown_signal =
        custom_signal("dorumon", "nope", CustomSignalPayload::Amount { amount: 1 });
    let signal_error = blueprints::dispatch_custom_signal(&unknown_signal, &action)
        .expect_err("unknown signal should be rejected");
    assert_eq!(
        signal_error,
        CustomSignalDispatchError::UnknownSignal {
            owner: "dorumon".into(),
            signal: "nope".into(),
        }
    );
}

#[test]
fn malformed_envelope_is_rejected_by_serde() {
    let err = ron::from_str::<SkillCustomSignal>(
        r#"(owner: "dorumon", signal: "build_exploit", payload: Amount(amount: "oops"))"#,
    )
    .expect_err("malformed envelope must fail parsing");

    assert!(
        err.to_string().contains("Expected integer")
            || err.to_string().contains("Expected integer type")
            || err.to_string().contains("Invalid value"),
        "unexpected parse error: {err}"
    );
}
