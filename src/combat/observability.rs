use std::fmt;

use bevy::prelude::World;
use serde::{Deserialize, Serialize};

use crate::combat::{
    av::ActionValue,
    battery_loop::{
        BatteryLoopBlockedReason, BatteryLoopSignal, BatteryLoopState, BatteryLoopStep,
        BatteryLoopTransition,
    },
    blueprints::{
        dorumon::{PredatorLoopSignal, PredatorLoopStep, PredatorLoopTransition},
        patamon::identity::{HolySupportSignal, HolySupportStep, HolySupportTransition},
        twin_core::{TwinCoreSignal, TwinCoreTransition},
    },
    floating::FloatingDamage,
    log::{ActionLog, LogEntry},
    sp::SpPool,
    state::{CombatPhase, CombatState},
    status_effect::{StatusBag, StatusEffectKind},
    stun::Stunned,
    team::Team,
    toughness::{DamageKind, Toughness, visible_toughness},
    types::{DamageTag, UnitId},
    ultimate::UltimateCharge,
    unit::{Ko, Unit},
};
pub use crate::combat::api::registry::{ValidationField, ValidationSection};
use crate::combat::api::ExtRegistries;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationSnapshot {
    pub phase: CombatPhase,
    pub winner: Option<Team>,
    pub sp_current: i32,
    pub sp_max: i32,
    pub turn_preview: Vec<UnitId>,
    pub action_log_tail: Vec<ValidationLogEntry>,
    pub floating_live: usize,
    pub units: Vec<ValidationUnitSnapshot>,
    pub owner_sections: Vec<ValidationSection>,
}

impl ValidationSnapshot {
    pub fn section(&self, owner: &str) -> Option<&ValidationSection> {
        self.owner_sections.iter().find(|section| section.owner == owner)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatteryLoopSnapshot {
    pub static_charge: u8,
    pub static_charge_cap: u8,
    pub circuit_charge: u8,
    pub circuit_charge_cap: u8,
    pub static_charge_threshold: u8,
    pub threshold_grant_emitted_this_cycle: bool,
    pub block_reaction_armed: bool,
    pub last_block_reaction_cast_id: Option<crate::combat::api::intent::CastId>,
    pub last_transition: Option<BatteryLoopTransition>,
    pub last_blocked_reason: Option<BatteryLoopBlockedReason>,
}

impl From<&BatteryLoopState> for BatteryLoopSnapshot {
    fn from(state: &BatteryLoopState) -> Self {
        Self {
            static_charge: state.static_charge,
            static_charge_cap: state.static_charge_cap,
            circuit_charge: state.circuit_charge,
            circuit_charge_cap: state.circuit_charge_cap,
            static_charge_threshold: state.static_charge_threshold,
            threshold_grant_emitted_this_cycle: state.threshold_grant_emitted_this_cycle,
            block_reaction_armed: state.block_reaction_armed,
            last_block_reaction_cast_id: state.last_block_reaction_cast_id,
            last_transition: state.last_transition,
            last_blocked_reason: state.last_blocked_reason,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationLogEntry {
    Hit {
        attacker: UnitId,
        target: UnitId,
        amount: i32,
        kind: DamageKind,
    },
    Break {
        target: UnitId,
        damage_tag: DamageTag,
    },
    Ko {
        target: UnitId,
    },
    Revive {
        target: UnitId,
        hp_after: i32,
    },
    ActionFailed {
        reason: String,
    },
    AdvanceTurn {
        target: UnitId,
        amount_pct: u32,
    },
    DelayTurn {
        target: UnitId,
        amount_pct: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationStatusSnapshot {
    pub kind: StatusEffectKind,
    pub duration_remaining: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationToughnessSnapshot {
    pub current: i32,
    pub max: i32,
    pub weaknesses: Vec<DamageTag>,
    pub broken: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationUnitSnapshot {
    pub id: UnitId,
    pub team: Team,
    pub hp_current: i32,
    pub hp_max: i32,
    pub toughness: Option<ValidationToughnessSnapshot>,
    pub ultimate_current: i32,
    pub ultimate_trigger: i32,
    pub ultimate_cap: i32,
    pub ko: bool,
    pub stun_turns: u32,
    pub statuses: Vec<ValidationStatusSnapshot>,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationSnapshotError {
    MissingResource(&'static str),
    MissingTeam { unit: UnitId },
    MissingToughness { unit: UnitId },
    MissingUltimateCharge { unit: UnitId },
}

impl fmt::Display for ValidationSnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingResource(resource) => write!(f, "missing resource {resource}"),
            Self::MissingTeam { unit } => write!(f, "unit {:?} missing Team", unit),
            Self::MissingToughness { unit } => write!(f, "unit {:?} missing Toughness", unit),
            Self::MissingUltimateCharge { unit } => {
                write!(f, "unit {:?} missing UltimateCharge", unit)
            }
        }
    }
}

impl std::error::Error for ValidationSnapshotError {}

pub fn capture_validation_snapshot(
    world: &mut World,
) -> Result<ValidationSnapshot, ValidationSnapshotError> {
    let (phase, winner) = {
        let combat_state = world
            .get_resource::<CombatState>()
            .ok_or(ValidationSnapshotError::MissingResource("CombatState"))?;
        (combat_state.phase, combat_state.winner)
    };
    let (sp_current, sp_max) = {
        let sp = world
            .get_resource::<SpPool>()
            .ok_or(ValidationSnapshotError::MissingResource("SpPool"))?;
        (sp.current, sp.max)
    };
    let turn_preview = {
        let mut av_query = world.query::<(&Unit, Option<&ActionValue>, Option<&Ko>)>();
        let mut entries: Vec<(i32, UnitId)> = av_query
            .iter(world)
            .filter(|(_, _, ko)| ko.is_none())
            .map(|(unit, av, _)| (av.map_or(0, |a| a.0), unit.id))
            .collect();
        // Highest AV first (closest to taking a turn), tiebreak by UnitId ascending.
        entries.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.0.cmp(&b.1.0)));
        entries.into_iter().map(|(_, id)| id).collect::<Vec<_>>()
    };
    let action_log_tail = world
        .get_resource::<ActionLog>()
        .ok_or(ValidationSnapshotError::MissingResource("ActionLog"))?
        .events
        .iter()
        .map(|event| match event {
            LogEntry::BasicHit {
                attacker,
                target,
                amount,
                kind,
            } => ValidationLogEntry::Hit {
                attacker: *attacker,
                target: *target,
                amount: *amount,
                kind: *kind,
            },
            LogEntry::Break { target, damage_tag } => ValidationLogEntry::Break {
                target: *target,
                damage_tag: *damage_tag,
            },
            LogEntry::Ko { target } => ValidationLogEntry::Ko { target: *target },
            LogEntry::Revive { target, hp_after } => ValidationLogEntry::Revive {
                target: *target,
                hp_after: *hp_after,
            },
            LogEntry::ActionFailed { reason } => ValidationLogEntry::ActionFailed {
                reason: reason.clone(),
            },
            LogEntry::AdvanceTurn { target, amount_pct } => ValidationLogEntry::AdvanceTurn {
                target: *target,
                amount_pct: *amount_pct,
            },
            LogEntry::DelayTurn { target, amount_pct } => ValidationLogEntry::DelayTurn {
                target: *target,
                amount_pct: *amount_pct,
            },
        })
        .collect();

    let mut units_query = world.query::<(
        &Unit,
        Option<&Team>,
        Option<&Toughness>,
        Option<&UltimateCharge>,
        Option<&Ko>,
        Option<&Stunned>,
        Option<&StatusBag>,
    )>();
    let mut units = Vec::new();
    for (unit, team, toughness, ultimate, ko, stunned, bag) in units_query.iter(world) {
        let team = team
            .copied()
            .ok_or(ValidationSnapshotError::MissingTeam { unit: unit.id })?;
        let toughness = match (team, toughness) {
            (Team::Enemy, None) => {
                return Err(ValidationSnapshotError::MissingToughness { unit: unit.id });
            }
            (_, maybe_toughness) => maybe_toughness,
        };
        let ultimate = ultimate
            .cloned()
            .ok_or(ValidationSnapshotError::MissingUltimateCharge { unit: unit.id })?;

        let toughness =
            visible_toughness(team, toughness).map(|view| ValidationToughnessSnapshot {
                current: view.current,
                max: view.max,
                weaknesses: view.weaknesses,
                broken: view.broken,
            });

        let mut statuses: Vec<ValidationStatusSnapshot> = bag
            .map(|b| {
                b.iter()
                    .map(|inst| ValidationStatusSnapshot {
                        kind: inst.kind.clone(),
                        duration_remaining: inst.duration_remaining,
                    })
                    .collect()
            })
            .unwrap_or_default();
        statuses.sort_by_key(|s| status_kind_ord(&s.kind));

        units.push(ValidationUnitSnapshot {
            id: unit.id,
            team,
            hp_current: unit.hp_current,
            hp_max: unit.hp_max,
            toughness,
            ultimate_current: ultimate.current,
            ultimate_trigger: ultimate.trigger,
            ultimate_cap: ultimate.cap,
            ko: ko.is_some(),
            stun_turns: stunned.map_or(0, |state| state.turns_left),
            statuses,
        });
    }
    units.sort_by_key(|unit| unit.id.0);

    let mut floating_query = world.query::<&FloatingDamage>();
    let floating_live = floating_query.iter(world).count();

    let owner_sections = collect_validation_sections(world);

    Ok(ValidationSnapshot {
        phase,
        winner,
        sp_current,
        sp_max,
        turn_preview,
        action_log_tail,
        floating_live,
        units,
        owner_sections,
    })
}

fn collect_validation_sections(world: &World) -> Vec<ValidationSection> {
    let Some(regs) = world.get_resource::<ExtRegistries>() else {
        return Vec::new();
    };

    let mut sections = regs
        .validation
        .iter()
        .filter_map(|(_, contributor)| (*contributor)(world))
        .collect::<Vec<_>>();
    sections.sort_by(|a, b| a.owner.cmp(b.owner));
    for section in &mut sections {
        section.fields.sort_by(|a, b| a.key.cmp(b.key));
    }
    sections
}

pub fn format_validation_snapshot(snapshot: &ValidationSnapshot) -> String {
    format!(
        "phase={} winner={} sp={}/{} twin_core={} support={} predator={} mind_game={} battery={} turn_preview={} action_log_tail={} floating_live={} units={}",
        format_phase(snapshot.phase),
        format_winner(snapshot.winner),
        snapshot.sp_current,
        snapshot.sp_max,
        format_twin_core_section(snapshot.section("twin_core")),
        format_holy_support_section(snapshot.section("support")),
        format_predator_loop_section(snapshot.section("predator")),
        format_precision_mind_game_section(snapshot.section("mind_game")),
        snapshot
            .section("battery")
            .map(format_battery_loop_section)
            .unwrap_or_else(|| "none".to_string()),
        format_unit_ids(&snapshot.turn_preview),
        format_action_log_tail(&snapshot.action_log_tail),
        snapshot.floating_live,
        format_units(&snapshot.units),
    )
}

fn format_phase(phase: CombatPhase) -> &'static str {
    match phase {
        CombatPhase::WaitingForTurn => "WaitingForTurn",
        CombatPhase::WaitingAction => "WaitingAction",
        CombatPhase::Resolving => "Resolving",
        CombatPhase::Victory => "Victory",
        CombatPhase::Defeat => "Defeat",
    }
}

fn format_winner(winner: Option<Team>) -> &'static str {
    match winner {
        Some(Team::Ally) => "Ally",
        Some(Team::Enemy) => "Enemy",
        None => "none",
    }
}

pub(crate) fn format_unit_ids(ids: &[UnitId]) -> String {
    let joined = ids
        .iter()
        .map(|id| id.0.to_string())
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn format_twin_core_section(section: Option<&ValidationSection>) -> String {
    match section {
        Some(section) => format!(
            "cr={} spark_targets={} fire={} ice={} burst_guard={} shatter_guard={} last={}",
            validation_field(section, "cr"),
            validation_field(section, "spark_targets"),
            validation_field(section, "fire"),
            validation_field(section, "ice"),
            validation_field(section, "burst_guard"),
            validation_field(section, "shatter_guard"),
            validation_field(section, "last"),
        ),
        None => "none".to_string(),
    }
}

fn format_holy_support_section(section: Option<&ValidationSection>) -> String {
    match section {
        Some(section) => format!(
            "grace={}/{} martyr_marked={} martyr_consumed={} last={}",
            validation_field(section, "grace"),
            validation_field(section, "grace_cap"),
            validation_field(section, "martyr_marked"),
            validation_field(section, "martyr_consumed"),
            validation_field(section, "last"),
        ),
        None => "none".to_string(),
    }
}

fn format_predator_loop_section(section: Option<&ValidationSection>) -> String {
    match section {
        Some(section) => format!(
            "exploit_cap={} prey_lock_duration={} berserk_threshold={} targets={} last={} blocked={}",
            validation_field(section, "exploit_cap"),
            validation_field(section, "prey_lock_duration"),
            validation_field(section, "berserk_threshold"),
            validation_field(section, "targets"),
            validation_field(section, "last"),
            validation_field(section, "blocked"),
        ),
        None => "none".to_string(),
    }
}

fn format_precision_mind_game_section(section: Option<&ValidationSection>) -> String {
    match section {
        Some(section) => format!(
            "phase={},window_index={},window={},commitment={},reveal={},outcome={},last={}",
            validation_field(section, "phase"),
            validation_field(section, "window_index"),
            validation_field(section, "window"),
            validation_field(section, "commitment"),
            validation_field(section, "reveal"),
            validation_field(section, "outcome"),
            validation_field(section, "last"),
        ),
        None => "none".to_string(),
    }
}

fn format_battery_loop_section(section: &ValidationSection) -> String {
    format!(
        "static={}/{} circuit={}/{} threshold={} grant_guard={} block_ready={} last_block_cast={} last={} blocked={}",
        validation_field(section, "static"),
        validation_field(section, "static_cap"),
        validation_field(section, "circuit"),
        validation_field(section, "circuit_cap"),
        validation_field(section, "threshold"),
        validation_field(section, "grant_guard"),
        validation_field(section, "block_ready"),
        validation_field(section, "last_block_cast"),
        validation_field(section, "last"),
        validation_field(section, "blocked"),
    )
}

fn validation_field<'a>(section: &'a ValidationSection, key: &str) -> &'a str {
    section.field(key).unwrap_or("none")
}

pub(crate) fn format_twin_core_transition(transition: TwinCoreTransition) -> String {
    let signal = match transition.signal {
        TwinCoreSignal::BuildCrossResonance => "build",
        TwinCoreSignal::SpendCrossResonance => "spend",
        TwinCoreSignal::ThermalSpark => "spark",
        TwinCoreSignal::TwinBurst => "twin-burst",
        TwinCoreSignal::Shatter => "shatter",
        TwinCoreSignal::FireSpendMarker => "fire-spend",
        TwinCoreSignal::IceSpendMarker => "ice-spend",
        TwinCoreSignal::CycleReset => "cycle-reset",
    };
    format!("{signal}({})", transition.amount)
}

pub(crate) fn format_holy_support_transition(transition: HolySupportTransition) -> String {
    let signal = match transition.signal {
        HolySupportSignal::BuildGrace => "build",
        HolySupportSignal::SpendGrace => "spend",
        HolySupportSignal::MarkMartyrLight => "mark-martyr",
        HolySupportSignal::ConsumeMartyrLight => "consume-martyr",
        HolySupportSignal::CycleReset => "cycle-reset",
        HolySupportSignal::Rejected => "rejected",
        HolySupportSignal::Ignored => "ignored",
    };

    match transition.signal {
        HolySupportSignal::BuildGrace
        | HolySupportSignal::SpendGrace => {
            format!("{signal}({})", transition.amount)
        }
        HolySupportSignal::Rejected | HolySupportSignal::Ignored => {
            match (transition.attempted, transition.reason) {
                (Some(attempted), Some(reason)) => {
                    format!(
                        "{signal}({};reason={reason:?})",
                        format_holy_support_step(attempted)
                    )
                }
                (Some(attempted), None) => {
                    format!("{signal}({})", format_holy_support_step(attempted))
                }
                _ => signal.to_string(),
            }
        }
        _ => signal.to_string(),
    }
}

fn format_holy_support_step(step: HolySupportStep) -> String {
    match step {
        HolySupportStep::BuildGrace { amount } => format!("build({amount})"),
        HolySupportStep::SpendGrace { amount } => format!("spend({amount})"),
        HolySupportStep::MarkMartyrLight => "mark-martyr".to_string(),
        HolySupportStep::ConsumeMartyrLight => "consume-martyr".to_string(),
        HolySupportStep::CycleReset => "cycle-reset".to_string(),
    }
}

pub(crate) fn format_predator_targets(
    targets: &[crate::combat::blueprints::dorumon::PredatorTargetSnapshot],
) -> String {
    let joined = targets
        .iter()
        .map(|target| {
            format!(
                "{}:e{}:p{}",
                target.unit_id.0,
                target.exploit_stacks,
                target
                    .prey_lock
                    .map(|lock| format!(
                        "{}{}",
                        lock.turns_left,
                        if lock.consumed { "c" } else { "" }
                    ))
                    .unwrap_or_else(|| "none".to_string())
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

pub(crate) fn format_predator_loop_transition(
    transition: PredatorLoopTransition,
) -> String {
    let signal = match transition.signal {
        PredatorLoopSignal::BuildExploit => "build-exploit",
        PredatorLoopSignal::ApplyPreyLock => "prey-lock",
        PredatorLoopSignal::ConsumePreyLockPayoff => "payoff",
        PredatorLoopSignal::EnterBerserk => "berserk",
        PredatorLoopSignal::Tick => "tick",
        PredatorLoopSignal::Expire => "expire",
        PredatorLoopSignal::Rejected => "rejected",
        PredatorLoopSignal::Ignored => "ignored",
    };
    match transition.signal {
        PredatorLoopSignal::BuildExploit
        | PredatorLoopSignal::ApplyPreyLock
        | PredatorLoopSignal::ConsumePreyLockPayoff
        | PredatorLoopSignal::EnterBerserk
        | PredatorLoopSignal::Expire => {
            format!(
                "{signal}(target={:?};amount={})",
                transition.target, transition.amount
            )
        }
        PredatorLoopSignal::Tick => signal.to_string(),
        PredatorLoopSignal::Rejected | PredatorLoopSignal::Ignored => {
            match (transition.attempted, transition.reason) {
                (Some(attempted), Some(reason)) => format!(
                    "{signal}({};reason={reason:?})",
                    format_predator_loop_step(attempted)
                ),
                (Some(attempted), None) => {
                    format!("{signal}({})", format_predator_loop_step(attempted))
                }
                _ => signal.to_string(),
            }
        }
    }
}

fn format_predator_loop_step(step: PredatorLoopStep) -> String {
    match step {
        PredatorLoopStep::BuildExploit { target, amount } => {
            format!("build({}:{})", target.0, amount)
        }
        PredatorLoopStep::ApplyPreyLock { target } => {
            format!("prey-lock({})", target.0)
        }
        PredatorLoopStep::ConsumePreyLockPayoff { target } => {
            format!("payoff({})", target.0)
        }
        PredatorLoopStep::EnterBerserk => "berserk".to_string(),
        PredatorLoopStep::Tick => "tick".to_string(),
        PredatorLoopStep::Expire { target } => {
            format!("expire({})", target.0)
        }
    }
}

pub fn format_battery_loop_snapshot(snapshot: &BatteryLoopSnapshot) -> String {
    format!(
        "static={}/{} circuit={}/{} threshold={} grant_guard={} block_ready={} last_block_cast={} last={} blocked={}",
        snapshot.static_charge,
        snapshot.static_charge_cap,
        snapshot.circuit_charge,
        snapshot.circuit_charge_cap,
        snapshot.static_charge_threshold,
        snapshot.threshold_grant_emitted_this_cycle,
        snapshot.block_reaction_armed,
        snapshot
            .last_block_reaction_cast_id
            .map(|cast_id| cast_id.0.get().to_string())
            .unwrap_or_else(|| "none".to_string()),
        snapshot
            .last_transition
            .map(format_battery_loop_transition)
            .unwrap_or_else(|| "none".to_string()),
        snapshot
            .last_blocked_reason
            .map(format_battery_loop_blocked_reason)
            .unwrap_or_else(|| "none".to_string()),
    )
}

pub(crate) fn format_battery_loop_transition(transition: BatteryLoopTransition) -> String {
    match transition.signal {
        BatteryLoopSignal::BuildStaticCharge => {
            format!("build-static({})", transition.amount)
        }
        BatteryLoopSignal::BuildCircuitCharge => {
            format!("build-circuit({})", transition.amount)
        }
        BatteryLoopSignal::SpendCircuitCharge => {
            format!("spend-circuit({})", transition.amount)
        }
        BatteryLoopSignal::BlockReady => "block-ready".to_string(),
        BatteryLoopSignal::BlockProc => "block-proc".to_string(),
        BatteryLoopSignal::GrantEnergy => {
            format!("grant({})", transition.amount)
        }
        BatteryLoopSignal::SelfEnergyGain => {
            format!("self-gain({})", transition.amount)
        }
        BatteryLoopSignal::TransferEnergy => {
            format!("transfer({})", transition.amount)
        }
        BatteryLoopSignal::CycleReset => "cycle-reset".to_string(),
        BatteryLoopSignal::Rejected => match (transition.attempted, transition.reason) {
            (Some(attempted), Some(reason)) => {
                format!(
                    "rejected({};reason={reason:?})",
                    format_battery_loop_step(attempted)
                )
            }
            (Some(attempted), None) => {
                format!("rejected({})", format_battery_loop_step(attempted))
            }
            _ => "rejected".to_string(),
        },
        BatteryLoopSignal::Ignored => match transition.attempted {
            Some(attempted) => format!("ignored({})", format_battery_loop_step(attempted)),
            None => "ignored".to_string(),
        },
    }
}

fn format_battery_loop_step(step: BatteryLoopStep) -> String {
    match step {
        BatteryLoopStep::BuildStaticCharge { amount } => {
            format!("build-static({amount})")
        }
        BatteryLoopStep::BuildCircuitCharge { amount } => {
            format!("build-circuit({amount})")
        }
        BatteryLoopStep::SpendCircuitCharge { amount } => {
            format!("spend-circuit({amount})")
        }
        BatteryLoopStep::BlockReady => "block-ready".to_string(),
        BatteryLoopStep::BlockProc => "block-proc".to_string(),
        BatteryLoopStep::GrantEnergy { amount } => format!("grant({amount})"),
        BatteryLoopStep::SelfEnergyGain { amount } => {
            format!("self-gain({amount})")
        }
        BatteryLoopStep::TransferEnergy { amount } => format!("transfer({amount})"),
        BatteryLoopStep::CycleReset => "cycle-reset".to_string(),
    }
}

fn format_battery_loop_blocked_reason(reason: BatteryLoopBlockedReason) -> String {
    match reason {
        BatteryLoopBlockedReason::ChargeCapReached { charge } => {
            format!("cap-reached({charge:?})")
        }
        BatteryLoopBlockedReason::ChargeUnderflow { charge } => {
            format!("underflow({charge:?})")
        }
        BatteryLoopBlockedReason::MissingPreExistingShock => "missing-shock".to_string(),
        BatteryLoopBlockedReason::NoEligibleAlly => "no-eligible-ally".to_string(),
        BatteryLoopBlockedReason::UnsupportedRequest => "unsupported".to_string(),
        BatteryLoopBlockedReason::MalformedData => "malformed".to_string(),
    }
}

fn format_action_log_tail(events: &[ValidationLogEntry]) -> String {
    let joined = events
        .iter()
        .map(format_log_entry)
        .collect::<Vec<_>>()
        .join("|");
    format!("[{joined}]")
}

fn format_log_entry(entry: &ValidationLogEntry) -> String {
    match entry {
        ValidationLogEntry::Hit {
            attacker,
            target,
            amount,
            kind,
        } => format!(
            "hit(attacker={},target={},amount={},kind={:?})",
            attacker.0, target.0, amount, kind
        ),
        ValidationLogEntry::Break { target, damage_tag } => {
            format!("break(target={},element={:?})", target.0, damage_tag)
        }
        ValidationLogEntry::Ko { target } => format!("ko(target={})", target.0),
        ValidationLogEntry::Revive { target, hp_after } => {
            format!("revive(target={},hp_after={})", target.0, hp_after)
        }
        ValidationLogEntry::ActionFailed { reason } => format!("fail(reason={})", reason),
        ValidationLogEntry::AdvanceTurn { target, amount_pct } => {
            format!("advance(target={},amount={})", target.0, amount_pct)
        }
        ValidationLogEntry::DelayTurn { target, amount_pct } => {
            format!("delay(target={},amount={})", target.0, amount_pct)
        }
    }
}

fn format_units(units: &[ValidationUnitSnapshot]) -> String {
    let joined = units.iter().map(format_unit).collect::<Vec<_>>().join(";");
    format!("[{joined}]")
}

fn format_unit(unit: &ValidationUnitSnapshot) -> String {
    let toughness = unit
        .toughness
        .as_ref()
        .map(|t| {
            format!(
                "{}/{},weaknesses={},broken={}",
                t.current,
                t.max,
                format_weaknesses(&t.weaknesses),
                t.broken
            )
        })
        .unwrap_or_else(|| "N/A".to_string());

    format!(
        "id={},team={:?},hp={}/{},tough={},ult={}/{}/{},ko={},stun={},statuses={}",
        unit.id.0,
        unit.team,
        unit.hp_current,
        unit.hp_max,
        toughness,
        unit.ultimate_current,
        unit.ultimate_trigger,
        unit.ultimate_cap,
        unit.ko,
        unit.stun_turns,
        format_statuses(&unit.statuses),
    )
}

fn format_statuses(statuses: &[ValidationStatusSnapshot]) -> String {
    let joined = statuses
        .iter()
        .map(|s| format!("{:?}({})", s.kind, s.duration_remaining))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn status_kind_ord(kind: &StatusEffectKind) -> u8 {
    match kind {
        StatusEffectKind::Heated => 0,
        StatusEffectKind::Chilled => 1,
        StatusEffectKind::Paralyzed => 2,
        StatusEffectKind::Slowed => 3,
        StatusEffectKind::Blessed => 4,
        StatusEffectKind::Burn => 5,
        StatusEffectKind::Shock => 6,
    }
}

fn format_weaknesses(weaknesses: &[DamageTag]) -> String {
    let joined = weaknesses
        .iter()
        .map(|tag| format!("{tag:?}"))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}
