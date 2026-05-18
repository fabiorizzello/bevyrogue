//! Renamon blueprint: custom-signal dispatch + identity (MIND GAME) wiring.
//!
//! `RenamonPlugin` owns Renamon-specific kernel-runtime registrations
//! (MIND GAME resource, applier system, hook) so adding or removing
//! the digimon is a single `add_plugins` line at the call site.

use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::combat::bevy_types::*;

use crate::combat::runtime::registry::{ValidationField, ValidationSection};
use crate::combat::{
    runtime::{
        Beat, BeatEvent, BeatKind, BlueprintState, CompiledTimeline, EventFilter, Intent,
        PassiveListeners, PassiveRunner, SignalPayload, SignalTaxonomy, SkillCtx,
    },
    events::{CombatEvent, CombatEventKind},
    kernel::{CombatKernelRegistry, CombatKernelTransition, CombatKernelHook, CombatKernelHookDomain},
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

    // Constructor not consumed; kept for API symmetry with rejected().
    pub const fn ignored(attempted: PrecisionMindGameStep) -> Self {
        Self::Ignored { attempted }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Resource)]
pub struct PrecisionMindGameState {
    pub phase: PrecisionMindGamePhase,
    pub window_index: u32,
    pub current_window: Option<PrecisionWindowKind>,
    pub commitment: Option<PrecisionCommitment>,
    pub reveal: Option<PrecisionReveal>,
    pub outcome: Option<PrecisionOutcome>,
    pub last_signal: Option<PrecisionMindGameTransition>,
}

impl Default for PrecisionMindGameState {
    fn default() -> Self {
        Self {
            phase: PrecisionMindGamePhase::Dormant,
            window_index: 0,
            current_window: None,
            commitment: None,
            reveal: None,
            outcome: None,
            last_signal: None,
        }
    }
}

// Snapshot type; not yet consumed by integration tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrecisionMindGameSnapshot {
    pub phase: PrecisionMindGamePhase,
    pub window_index: u32,
    pub current_window: Option<PrecisionWindowKind>,
    pub commitment: Option<PrecisionCommitment>,
    pub reveal: Option<PrecisionReveal>,
    pub outcome: Option<PrecisionOutcome>,
    pub last_signal: Option<PrecisionMindGameTransition>,
}

impl From<&PrecisionMindGameState> for PrecisionMindGameSnapshot {
    fn from(state: &PrecisionMindGameState) -> Self {
        Self {
            phase: state.phase,
            window_index: state.window_index,
            current_window: state.current_window,
            commitment: state.commitment,
            reveal: state.reveal,
            outcome: state.outcome,
            last_signal: state.last_signal,
        }
    }
}

impl PrecisionMindGameState {
    // Consumed by tests/renamon_precision_runtime.rs and tests/validation_snapshot.rs.
    pub fn is_window_open(&self) -> bool {
        self.phase == PrecisionMindGamePhase::WindowOpen
    }

    // Public snapshot method; not yet consumed by tests.
    pub fn snapshot(&self) -> PrecisionMindGameSnapshot {
        PrecisionMindGameSnapshot::from(self)
    }
}

pub struct PrecisionMindGameHook;

impl CombatKernelHook for PrecisionMindGameHook {
    fn domain(&self) -> CombatKernelHookDomain {
        CombatKernelHookDomain::Shared
    }

    fn on_transition(
        &self,
        _transition: &CombatKernelTransition,
        _out: &mut Vec<CombatKernelTransition>,
    ) {
    }
}

pub struct RenamonPlugin;

impl Plugin for RenamonPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PrecisionMindGameState>()
            .add_systems(Update, apply_precision_mind_game_transitions_system);

        app.world_mut()
            .resource_mut::<CombatKernelRegistry>()
            .register(PrecisionMindGameHook);
    }
}

fn blueprint_transition(name: &str) -> CombatKernelTransition {
    CombatKernelTransition::Blueprint {
        owner: OWNER.to_owned(),
        name: name.to_owned(),
        payload: SignalPayload::Empty,
    }
}

pub fn register_renamon_ext(regs: &mut crate::combat::runtime::ExtRegistries) {
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
        PrecisionMindGameTransition::Rejected { attempted, reason } => {
            format!("rejected({:?};reason={:?})", attempted, reason)
        }
        PrecisionMindGameTransition::Ignored { attempted } => {
            format!("ignored({:?})", attempted)
        }
    }
}

fn format_precision_phase(phase: PrecisionMindGamePhase) -> String {
    format!("{:?}", phase)
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

pub fn register_passive_runtime(app: &mut App) {
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

fn build_passive_timeline() -> Arc<CompiledTimeline> {
    Arc::new(CompiledTimeline {
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
            crate::combat::runtime::timeline::BeatEdge {
                from: "dormant",
                to: "proc",
                gate: Some("renamon/kitsune_grace/passive_trigger"),
            },
            crate::combat::runtime::timeline::BeatEdge {
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

fn register_passive_hooks(app: &mut App) {
    let mut regs = app
        .world_mut()
        .resource_mut::<crate::combat::runtime::ExtRegistries>();
    regs.predicates
        .register("renamon/kitsune_grace/passive_trigger", passive_trigger);
    regs.hooks
        .register("renamon/kitsune_grace/passive_proc", passive_proc);
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
        SIGNAL_OPEN_MOMENTUM_WINDOW => Ok(vec![blueprint_transition(SIGNAL_OPEN_MOMENTUM_WINDOW)]),
        SIGNAL_COMMIT_PRECISION_PRESS => {
            Ok(vec![blueprint_transition(SIGNAL_COMMIT_PRECISION_PRESS)])
        }
        SIGNAL_REVEAL_BAIT => Ok(vec![blueprint_transition(SIGNAL_REVEAL_BAIT)]),
        SIGNAL_RESOLVE_PRECISION_SUCCESS => {
            Ok(vec![blueprint_transition(SIGNAL_RESOLVE_PRECISION_SUCCESS)])
        }
        _ => Err(CustomSignalDispatchError::UnknownSignal {
            owner: OWNER.to_owned(),
            signal: signal.signal().to_owned(),
        }),
    }
}

pub fn apply_precision_mind_game_transitions_system(
    mut events: MessageReader<CombatEvent>,
    mut state: ResMut<PrecisionMindGameState>,
) {
    for event in events.read() {
        if let CombatEventKind::OnKernelTransition {
            transition: CombatKernelTransition::Blueprint { owner, name, .. },
        } = &event.kind
        {
            if owner != OWNER {
                continue;
            }

            let step = match name.as_str() {
                SIGNAL_OPEN_MOMENTUM_WINDOW => {
                    Some(PrecisionMindGameTransition::open_window(PrecisionWindowKind::Momentum))
                }
                SIGNAL_COMMIT_PRECISION_PRESS => {
                    Some(PrecisionMindGameTransition::commit(PrecisionCommitment::Press))
                }
                SIGNAL_REVEAL_BAIT => {
                    Some(PrecisionMindGameTransition::reveal(PrecisionReveal::Baited))
                }
                SIGNAL_RESOLVE_PRECISION_SUCCESS => {
                    Some(PrecisionMindGameTransition::resolve(PrecisionOutcome::Success))
                }
                _ => None,
            };

            if let Some(transition) = step {
                apply_precision_mind_game_transition(&mut state, transition);
            }
        }
    }
}

pub fn apply_precision_mind_game_transition(
    state: &mut PrecisionMindGameState,
    transition: PrecisionMindGameTransition,
) {
    let before = state.clone();
    let mut accepted = false;

    match transition {
        PrecisionMindGameTransition::OpenWindow { window } => {
            if matches!(
                state.phase,
                PrecisionMindGamePhase::Dormant | PrecisionMindGamePhase::Resolved
            ) {
                state.phase = PrecisionMindGamePhase::WindowOpen;
                state.window_index = state.window_index.saturating_add(1);
                state.current_window = Some(window);
                state.commitment = None;
                state.reveal = None;
                state.outcome = None;
                accepted = true;
            } else {
                state.last_signal = Some(PrecisionMindGameTransition::rejected(
                    PrecisionMindGameStep::OpenWindow { window },
                    PrecisionMindGameRejectReason::WindowAlreadyOpen,
                ));
            }
        }
        PrecisionMindGameTransition::Commit { commitment } => {
            if state.phase == PrecisionMindGamePhase::WindowOpen && state.commitment.is_none() {
                state.phase = PrecisionMindGamePhase::CommitmentLocked;
                state.commitment = Some(commitment);
                accepted = true;
            } else {
                let reason = if state.current_window.is_none() {
                    PrecisionMindGameRejectReason::NoOpenWindow
                } else if state.commitment.is_some() {
                    PrecisionMindGameRejectReason::DuplicateCommitment
                } else {
                    PrecisionMindGameRejectReason::NoOpenWindow
                };
                state.last_signal = Some(PrecisionMindGameTransition::rejected(
                    PrecisionMindGameStep::Commit { commitment },
                    reason,
                ));
            }
        }
        PrecisionMindGameTransition::Reveal { reveal } => {
            if state.phase == PrecisionMindGamePhase::CommitmentLocked && state.reveal.is_none() {
                state.phase = PrecisionMindGamePhase::CounterplayRevealed;
                state.reveal = Some(reveal);
                accepted = true;
            } else {
                let reason = if state.commitment.is_none() {
                    PrecisionMindGameRejectReason::MissingCommitment
                } else if state.reveal.is_some() {
                    PrecisionMindGameRejectReason::DuplicateReveal
                } else {
                    PrecisionMindGameRejectReason::NoOpenWindow
                };
                state.last_signal = Some(PrecisionMindGameTransition::rejected(
                    PrecisionMindGameStep::Reveal { reveal },
                    reason,
                ));
            }
        }
        PrecisionMindGameTransition::Resolve { outcome } => {
            if state.phase == PrecisionMindGamePhase::CounterplayRevealed && state.outcome.is_none()
            {
                state.phase = PrecisionMindGamePhase::Resolved;
                state.outcome = Some(outcome);
                accepted = true;
            } else {
                let reason = if state.reveal.is_none() {
                    PrecisionMindGameRejectReason::MissingReveal
                } else if matches!(state.phase, PrecisionMindGamePhase::Resolved) {
                    PrecisionMindGameRejectReason::AlreadyResolved
                } else {
                    PrecisionMindGameRejectReason::MissingReveal
                };
                state.last_signal = Some(PrecisionMindGameTransition::rejected(
                    PrecisionMindGameStep::Resolve { outcome },
                    reason,
                ));
            }
        }
        PrecisionMindGameTransition::Rejected { attempted, reason } => {
            state.last_signal = Some(PrecisionMindGameTransition::Rejected { attempted, reason });
        }
        PrecisionMindGameTransition::Ignored { attempted } => {
            state.last_signal = Some(PrecisionMindGameTransition::Ignored { attempted });
        }
    }

    if accepted {
        state.last_signal = Some(transition);
    }

    debug!(
        "PrecisionMindGameState before={:?} after={:?} last={:?}",
        before, state, state.last_signal
    );
}
