use std::sync::Arc;

use bevy::prelude::*;

use crate::combat::api::registry::{ValidationField, ValidationSection};
use crate::combat::{
    api::{
        Beat, BeatEvent, BeatKind, BlueprintState, CompiledTimeline, EventFilter, Intent,
        PassiveListeners, PassiveRunner, SignalPayload, SignalTaxonomy, SkillCtx,
    },
    events::{CombatEvent, CombatEventKind},
    kernel::{CombatKernelRegistry, CombatKernelTransition},
    precision_mind_game::{
        PrecisionCommitment, PrecisionMindGamePhase, PrecisionMindGameState,
        PrecisionMindGameTransition, PrecisionOutcome, PrecisionReveal, PrecisionWindowKind,
        apply_precision_mind_game_transition,
    },
    team::Team,
    types::UnitId,
    unit::Unit,
};
use crate::data::skills_ron::SkillCustomSignal;

use super::CustomSignalDispatchError;

pub const OWNER: &str = "renamon";

const SIGNAL_OPEN_MOMENTUM_WINDOW: &str = "open_momentum_window";
const SIGNAL_COMMIT_PRECISION_PRESS: &str = "commit_precision_press";
const SIGNAL_REVEAL_BAIT: &str = "reveal_bait";
const SIGNAL_RESOLVE_PRECISION_SUCCESS: &str = "resolve_precision_success";

const PASSIVE_SIGNAL_NAME: &str = "kitsune_grace";
const PASSIVE_TRIGGER_KEY: &str = "renamon/kitsune_grace/triggered";
const PASSIVE_TIMELINE_ID: &str = "renamon_kitsune_grace_passive";
const PASSIVE_OWNER: UnitId = UnitId(7);

fn blueprint_transition(name: &str) -> CombatKernelTransition {
    CombatKernelTransition::Blueprint {
        owner: OWNER.to_owned(),
        name: name.to_owned(),
        payload: SignalPayload::Empty,
    }
}

pub fn register_validation_ext(regs: &mut crate::combat::api::ExtRegistries) {
    regs.validation
        .register("mind_game/validation", precision_validation_section);
}

fn precision_validation_section(world: &World) -> Option<ValidationSection> {
    let state = world.get_resource::<PrecisionMindGameState>()?;
    Some(ValidationSection::new(
        "mind_game",
        vec![
            ValidationField::new("phase", format_precision_phase(state.phase)),
            ValidationField::new("window_index", state.window_index.to_string()),
            ValidationField::new(
                "window",
                state
                    .current_window
                    .map(|window| format_precision_window(Some(window)))
                    .unwrap_or("none"),
            ),
            ValidationField::new(
                "commitment",
                state
                    .commitment
                    .map(|commitment| format_precision_commitment(Some(commitment)))
                    .unwrap_or("none"),
            ),
            ValidationField::new(
                "reveal",
                state
                    .reveal
                    .map(|reveal| format_precision_reveal(Some(reveal)))
                    .unwrap_or("none"),
            ),
            ValidationField::new(
                "outcome",
                state
                    .outcome
                    .map(|outcome| format_precision_outcome(Some(outcome)))
                    .unwrap_or("none"),
            ),
            ValidationField::new(
                "last",
                state
                    .last_signal
                    .map(format_precision_transition)
                    .unwrap_or_else(|| "none".to_string()),
            ),
        ],
    ))
}

fn format_precision_transition(transition: PrecisionMindGameTransition) -> String {
    match transition {
        PrecisionMindGameTransition::OpenWindow { window } => {
            format!("open_window({})", format_precision_window(Some(window)))
        }
        PrecisionMindGameTransition::Commit { commitment } => {
            format!("commit({})", format_precision_commitment(Some(commitment)))
        }
        PrecisionMindGameTransition::Reveal { reveal } => {
            format!("reveal({})", format_precision_reveal(Some(reveal)))
        }
        PrecisionMindGameTransition::Resolve { outcome } => {
            format!("resolve({})", format_precision_outcome(Some(outcome)))
        }
        PrecisionMindGameTransition::Rejected { attempted, reason } => format!(
            "rejected({:?};reason={:?})",
            attempted, reason
        ),
        PrecisionMindGameTransition::Ignored { attempted } => {
            format!("ignored({:?})", attempted)
        }
    }
}

fn format_precision_phase(phase: PrecisionMindGamePhase) -> &'static str {
    match phase {
        PrecisionMindGamePhase::Dormant => "Dormant",
        PrecisionMindGamePhase::WindowOpen => "WindowOpen",
        PrecisionMindGamePhase::CommitmentLocked => "CommitmentLocked",
        PrecisionMindGamePhase::CounterplayRevealed => "CounterplayRevealed",
        PrecisionMindGamePhase::Resolved => "Resolved",
    }
}

fn format_precision_window(window: Option<PrecisionWindowKind>) -> &'static str {
    match window {
        Some(PrecisionWindowKind::Momentum) => "Momentum",
        Some(PrecisionWindowKind::Counterplay) => "Counterplay",
        None => "none",
    }
}

fn format_precision_commitment(commitment: Option<PrecisionCommitment>) -> &'static str {
    match commitment {
        Some(PrecisionCommitment::Press) => "Press",
        Some(PrecisionCommitment::Hold) => "Hold",
        Some(PrecisionCommitment::Feint) => "Feint",
        None => "none",
    }
}

fn format_precision_reveal(reveal: Option<PrecisionReveal>) -> &'static str {
    match reveal {
        Some(PrecisionReveal::Guarded) => "Guarded",
        Some(PrecisionReveal::Baited) => "Baited",
        Some(PrecisionReveal::Trapped) => "Trapped",
        None => "none",
    }
}

fn format_precision_outcome(outcome: Option<PrecisionOutcome>) -> &'static str {
    match outcome {
        Some(PrecisionOutcome::Success) => "Success",
        Some(PrecisionOutcome::Countered) => "Countered",
        Some(PrecisionOutcome::Failed) => "Failed",
        None => "none",
    }
}

pub fn dispatch(
    signal: &SkillCustomSignal,
    _action: &crate::combat::state::ResolvedAction,
) -> Result<Vec<CombatKernelTransition>, CustomSignalDispatchError> {
    if signal.owner() != OWNER {
        return Err(CustomSignalDispatchError::UnknownOwner {
            owner: signal.owner().to_owned(),
        });
    }

    match signal.signal() {
        SIGNAL_OPEN_MOMENTUM_WINDOW
        | SIGNAL_COMMIT_PRECISION_PRESS
        | SIGNAL_REVEAL_BAIT
        | SIGNAL_RESOLVE_PRECISION_SUCCESS => Ok(vec![blueprint_transition(signal.signal())]),
        other => Err(CustomSignalDispatchError::UnknownSignal {
            owner: OWNER.to_string(),
            signal: other.to_string(),
        }),
    }
}

pub struct RenamonPlugin;

impl Plugin for RenamonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PrecisionMindGameState>()
            .add_systems(Update, apply_renamon_precision_blueprint_system);

        app.world_mut()
            .resource_mut::<CombatKernelRegistry>()
            .register(crate::combat::precision_mind_game::PrecisionMindGameHook);
    }
}

fn decode_precision_blueprint_transition(name: &str) -> Option<PrecisionMindGameTransition> {
    match name {
        SIGNAL_OPEN_MOMENTUM_WINDOW => Some(PrecisionMindGameTransition::open_window(
            PrecisionWindowKind::Momentum,
        )),
        SIGNAL_COMMIT_PRECISION_PRESS => Some(PrecisionMindGameTransition::commit(
            PrecisionCommitment::Press,
        )),
        SIGNAL_REVEAL_BAIT => Some(PrecisionMindGameTransition::reveal(PrecisionReveal::Baited)),
        SIGNAL_RESOLVE_PRECISION_SUCCESS => Some(PrecisionMindGameTransition::resolve(
            PrecisionOutcome::Success,
        )),
        _ => None,
    }
}

fn apply_renamon_precision_blueprint_system(
    mut events: MessageReader<CombatEvent>,
    mut state: ResMut<PrecisionMindGameState>,
) {
    for event in events.read() {
        let CombatEventKind::OnKernelTransition { transition } = &event.kind else {
            continue;
        };
        let CombatKernelTransition::Blueprint { owner, name, .. } = transition else {
            continue;
        };
        if owner != OWNER {
            continue;
        }
        let Some(precision_transition) = decode_precision_blueprint_transition(name) else {
            continue;
        };
        apply_precision_mind_game_transition(&mut state, precision_transition);
    }
}

pub fn register_passive_runtime(app: &mut bevy::prelude::App) {
    register_passive_hooks(app);

    app.world_mut()
        .resource_mut::<SignalTaxonomy>()
        .register(OWNER, PASSIVE_SIGNAL_NAME);

    app.world_mut()
        .resource_mut::<PassiveListeners>()
        .runners
        .push(PassiveRunner::new(
            build_passive_timeline(),
            PASSIVE_OWNER,
            vec![EventFilter::blueprint("kernel", "ult_used")],
        ));
}

fn build_passive_timeline() -> Arc<crate::combat::api::CompiledTimeline> {
    Arc::new(crate::combat::api::CompiledTimeline {
        id: PASSIVE_TIMELINE_ID,
        entry: "dormant",
        beats: vec![
            Beat {
                id: "dormant",
                kind: BeatKind::Impact,
                hook: None,
                selector: None,
                presentation: None,
                payload: None,
            },
            Beat {
                id: "proc",
                kind: BeatKind::Impact,
                hook: Some("renamon/kitsune_grace/passive_proc"),
                selector: None,
                presentation: None,
                payload: None,
            },
            Beat {
                id: "resolve",
                kind: BeatKind::Impact,
                hook: None,
                selector: None,
                presentation: None,
                payload: None,
            },
        ],
        edges: vec![
            crate::combat::api::timeline::BeatEdge {
                from: "dormant",
                to: "proc",
                gate: Some("renamon/kitsune_grace/passive_trigger"),
            },
            crate::combat::api::timeline::BeatEdge {
                from: "proc",
                to: "resolve",
                gate: None,
            },
        ],
    })
}

fn passive_trigger(evt: &BeatEvent, ctx: &SkillCtx<'_>) -> bool {
    let world = ctx.world;

    let Some(mut units) = world.try_query::<(&Unit, &Team)>() else {
        return false;
    };

    let Some((_, target_team)) = units
        .iter(world)
        .find(|(unit, _)| unit.id == ctx.primary_target)
    else {
        return false;
    };

    let Some((self_unit, self_team)) = units.iter(world).find(|(unit, _)| unit.id == ctx.caster)
    else {
        return false;
    };

    let guard_key = (ctx.caster, PASSIVE_TRIGGER_KEY.to_string());
    let guard_written = world
        .resource::<BlueprintState>()
        .map
        .get(&guard_key)
        .copied()
        .unwrap_or_default()
        != 0;

    ctx.primary_target != ctx.caster
        && self_unit.hp_current > 0
        && self_team == target_team
        && evt.beat_id == "dormant"
        && !guard_written
}

fn passive_proc(evt: &BeatEvent, ctx: &mut SkillCtx<'_>) {
    ctx.enqueue(Intent::SetBlueprintState {
        actor: ctx.caster,
        key: PASSIVE_TRIGGER_KEY.to_string(),
        value: 1,
        cast_id: evt.cast_id,
    });
    ctx.enqueue(Intent::BlueprintSignal {
        source: ctx.caster,
        owner: OWNER,
        name: PASSIVE_SIGNAL_NAME,
        payload: SignalPayload::UnitTarget(ctx.primary_target),
        cast_id: evt.cast_id,
    });
}

fn register_passive_hooks(app: &mut bevy::prelude::App) {
    let mut regs = app
        .world_mut()
        .resource_mut::<crate::combat::api::ExtRegistries>();
    regs.predicates
        .register("renamon/kitsune_grace/passive_trigger", passive_trigger);
    regs.hooks
        .register("renamon/kitsune_grace/passive_proc", passive_proc);
}
