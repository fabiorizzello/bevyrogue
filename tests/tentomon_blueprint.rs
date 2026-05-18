use bevy::prelude::*;
use bevyrogue::combat::blueprints::tentomon::{
    OWNER as TENTOMON_OWNER, SIG_BUILD_CIRCUIT_CHARGE, SIG_BUILD_STATIC_CHARGE,
    SIG_SPEND_CIRCUIT_CHARGE,
};
use bevyrogue::combat::kernel::CombatKernelTransition;
use bevyrogue::combat::plugin::CombatPlugin;
use bevyrogue::combat::state::{ResolvedAction, UltEffect};
use bevyrogue::combat::types::{DamageTag, SkillId, UnitId};
use bevyrogue::combat::blueprints::{self, tentomon::BatteryLoopState};
use bevyrogue::data::skills_ron::{CustomSignalPayload, SkillCustomSignal, TargetShape};

fn base_action() -> ResolvedAction {
    ResolvedAction {
        source: UnitId(1),
        target: UnitId(2),
        skill_id: SkillId("test".into()),
        damage_tag: DamageTag::Electric,
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
        target_shape: TargetShape::Single,
        custom_signals: Vec::new(),
        damage_curve: Default::default(),
        cleanse_count: None,
    }
}

fn signal(owner: &str, signal: &str, amount: u16) -> SkillCustomSignal {
    SkillCustomSignal::blueprint(
        owner,
        signal,
        CustomSignalPayload::Amount {
            amount: amount as i32,
        },
    )
}

#[test]
fn tentomon_blueprint_maps_static_charge() {
    let mut action = base_action();
    action
        .custom_signals
        .push(signal("tentomon", "build_static_charge", 1));

    let transitions = blueprints::transitions_for_action(&action);
    assert_eq!(transitions.len(), 1);
    assert_eq!(
        transitions[0],
        CombatKernelTransition::Blueprint {
            owner: TENTOMON_OWNER.to_string(),
            name: SIG_BUILD_STATIC_CHARGE.to_string(),
            payload: bevyrogue::combat::runtime::SignalPayload::Amount(1),
        }
    );
}

#[test]
fn tentomon_blueprint_maps_circuit_charge() {
    let mut action = base_action();
    action
        .custom_signals
        .push(signal("tentomon", "build_circuit_charge", 1));

    let transitions = blueprints::transitions_for_action(&action);
    assert_eq!(transitions.len(), 1);
    assert_eq!(
        transitions[0],
        CombatKernelTransition::Blueprint {
            owner: TENTOMON_OWNER.to_string(),
            name: SIG_BUILD_CIRCUIT_CHARGE.to_string(),
            payload: bevyrogue::combat::runtime::SignalPayload::Amount(1),
        }
    );
}

#[test]
fn tentomon_blueprint_maps_spend_circuit_charge() {
    let mut action = base_action();
    action
        .custom_signals
        .push(signal("tentomon", "spend_circuit_charge", 2));

    let transitions = blueprints::transitions_for_action(&action);
    assert_eq!(transitions.len(), 1);
    assert_eq!(
        transitions[0],
        CombatKernelTransition::Blueprint {
            owner: TENTOMON_OWNER.to_string(),
            name: SIG_SPEND_CIRCUIT_CHARGE.to_string(),
            payload: bevyrogue::combat::runtime::SignalPayload::Amount(2),
        }
    );
}

#[test]
fn integration_blueprint_to_kernel_state() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_message::<CombatEvent>();
    app.add_plugins(CombatPlugin);

    {
        let state = app.world().resource::<BatteryLoopState>();
        assert_eq!(state.static_charge, 0);
        assert_eq!(state.circuit_charge, 0);
    }

    let mut action = base_action();
    action
        .custom_signals
        .push(signal("tentomon", "build_static_charge", 1));

    let transitions = blueprints::transitions_for_action(&action);
    use bevyrogue::combat::runtime::intent::CastId;
    use bevyrogue::combat::events::{CombatEvent, CombatEventKind};
    for transition in transitions {
        app.world_mut().write_message(CombatEvent {
            kind: CombatEventKind::OnKernelTransition { transition },
            source: action.source,
            target: action.target,
            follow_up_depth: 0,
            cast_id: CastId::ROOT,
        });
    }

    app.update();

    {
        let state = app.world().resource::<BatteryLoopState>();
        assert_eq!(state.static_charge, 1);
    }
}
