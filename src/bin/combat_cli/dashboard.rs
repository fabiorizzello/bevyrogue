use bevy::prelude::*;

use bevyrogue::combat::events::CombatEvent;
use bevyrogue::combat::events::CombatEventKind;
use bevyrogue::combat::sp::SpPool;
use bevyrogue::combat::team::Team;
use bevyrogue::combat::toughness::Toughness;
use bevyrogue::combat::turn_order::{TurnAdvanced, TurnOrder};
use bevyrogue::combat::ultimate::UltimateCharge;
use bevyrogue::combat::unit::Unit;

pub fn combat_dashboard_system(
    mut turn_advanced: MessageReader<TurnAdvanced>,
    mut combat_events: MessageReader<CombatEvent>,
    units: Query<(&Unit, &Team, Option<&UltimateCharge>, Option<&Toughness>)>,
    turn_order: Res<TurnOrder>,
    sp_pool: Res<SpPool>,
) {
    let mut should_draw = false;
    for _ in turn_advanced.read() {
        should_draw = true;
    }
    for event in combat_events.read() {
        if matches!(
            event.kind,
            CombatEventKind::OnActionResolved | CombatEventKind::TurnOrderSeeded { .. }
        ) {
            should_draw = true;
        }
    }

    if !should_draw {
        return;
    }

    println!("\n{}", "=".repeat(60));
    println!(
        " COMBAT DASHBOARD | SP: {}/{}",
        sp_pool.current, sp_pool.max
    );
    println!("{}", "-".repeat(60));

    // Turn Order (AV system: show active unit)
    let mut names = Vec::new();
    if let Some(active_id) = turn_order.active_unit {
        if let Some((unit, _, _, _)) = units.iter().find(|(u, _, _, _)| u.id == active_id) {
            names.push(unit.name.clone());
        }
    }
    println!(" TURN ORDER: {}", names.join(" -> "));
    println!("{}", "-".repeat(60));

    // Units
    let mut all_units: Vec<_> = units.iter().collect();
    all_units.sort_by_key(|(_, team, _, _)| match team {
        Team::Ally => 0,
        Team::Enemy => 1,
    });

    for (unit, team, ult, toughness) in all_units {
        let team_str = match team {
            Team::Ally => "[ALLY]",
            Team::Enemy => "[ENEM]",
        };

        let ult_str = if let Some(u) = ult {
            format!("ULT: {}/{}", u.current, u.trigger)
        } else {
            "ULT: N/A".to_string()
        };

        let toughness_str = if let Some(t) = toughness {
            format!("TGH: {}/{}", t.current, t.max)
        } else {
            "TGH: N/A".to_string()
        };

        println!(
            "{:<6} {:<12} | HP: {:>4}/{:<4} | {} | {}",
            team_str, unit.name, unit.hp_current, unit.hp_max, ult_str, toughness_str
        );
    }
    println!("{}\n", "=".repeat(60));
}
