use crate::combat::{
    state::CombatPhase,
    team::Team,
    types::{DamageTag, UnitId},
};

use super::snapshot::{
    ValidationLogEntry, ValidationSection, ValidationSnapshot, ValidationStatusSnapshot,
    ValidationUnitSnapshot,
};

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

fn format_weaknesses(weaknesses: &[DamageTag]) -> String {
    let joined = weaknesses
        .iter()
        .map(|tag| format!("{tag:?}"))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}
