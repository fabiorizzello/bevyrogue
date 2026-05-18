use std::fmt;

use bevy::prelude::World;

use crate::combat::{
    av::ActionValue,
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
pub use crate::combat::api::registry::ValidationSection;
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
    // Consumed by tests/holy_support_mechanics.rs, holy_support_affordance.rs, holy_support_resolution.rs.
    #[allow(dead_code)]
    pub fn section(&self, owner: &str) -> Option<&ValidationSection> {
        self.owner_sections.iter().find(|section| section.owner == owner)
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
    let mut parts = vec![
        format!("phase={}", format_phase(snapshot.phase)),
        format!("winner={}", format_winner(snapshot.winner)),
        format!("sp={}/{}", snapshot.sp_current, snapshot.sp_max),
    ];

    for section in &snapshot.owner_sections {
        parts.push(format_section_generic(section));
    }

    parts.push(format!(
        "turn_preview={}",
        format_unit_ids(&snapshot.turn_preview)
    ));
    parts.push(format!(
        "action_log_tail={}",
        format_action_log_tail(&snapshot.action_log_tail)
    ));
    parts.push(format!("floating_live={}", snapshot.floating_live));
    parts.push(format!("units={}", format_units(&snapshot.units)));

    parts.join(" ")
}

fn format_section_generic(section: &ValidationSection) -> String {
    let fields = section
        .fields
        .iter()
        .map(|f| format!("{}={}", f.key, f.value))
        .collect::<Vec<_>>()
        .join(" ");
    format!("{}={}", section.owner, fields)
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

pub(crate) fn format_predator_targets(
    targets: &[crate::combat::blueprints::dorumon::identity::PredatorTargetSnapshot],
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
