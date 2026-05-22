use bevy::prelude::*;
use inquire::Select;

use bevyrogue::combat::action_query::{
    ActionQueryKind, ActionStatus, TargetStatus, build_snapshot_from_ecs_with_sp,
    first_enabled_target_id, mark_unit_active, query_action_affordance,
};
use bevyrogue::combat::energy::Energy;
use bevyrogue::combat::kit::UnitSkills;
use bevyrogue::combat::sp::SpPool;
use bevyrogue::combat::state::{CombatPhase, CombatState};
use bevyrogue::combat::stun::Stunned;
use bevyrogue::combat::team::Team;
use bevyrogue::combat::toughness::Toughness;
use bevyrogue::combat::turn_order::TurnOrder;
use bevyrogue::combat::turn_system::{ActionIntent, UltBurstRequest};
use bevyrogue::combat::types::UnitId;
use bevyrogue::combat::ultimate::UltimateCharge;
use bevyrogue::combat::ult_gauge::UltGaugeMetadata;
use bevyrogue::combat::unit::{Commander, Ko, Unit};
use bevyrogue::data::SkillBookHandle;
use bevyrogue::data::skills_ron::SkillBook;

use super::config::{IsInteractive, PlayerActed};
use super::menu::{
    action_kind_label, action_status_label, build_action_entries, print_action_entries,
    print_target_entries, target_entry_label,
};

pub fn player_action_system(
    state: Res<CombatState>,
    mut player_acted: ResMut<PlayerActed>,
    mut order: ResMut<TurnOrder>,
    mut intent_writer: MessageWriter<ActionIntent>,
    mut burst_writer: MessageWriter<UltBurstRequest>,
    units: Query<(
        &Unit,
        &Team,
        Option<&UltimateCharge>,
        Option<&UnitSkills>,
        Option<&Ko>,
        Option<&Commander>,
        Option<&Toughness>,
        Option<&bevyrogue::combat::counterplay::EnemyCounterplayKit>,
        Option<&Stunned>,
        Option<&Energy>,
        Option<&UltGaugeMetadata>,
    )>,
    sp_pool: Res<SpPool>,
    skill_books: Res<Assets<SkillBook>>,
    skill_book_handle: Option<Res<SkillBookHandle>>,
    interactive: Res<IsInteractive>,
    mut exit: MessageWriter<AppExit>,
) {
    if state.phase == CombatPhase::Victory {
        println!("\n[VICTORY] All enemies defeated!");
        exit.write(AppExit::Success);
        return;
    }
    if state.phase == CombatPhase::Defeat {
        println!("\n[DEFEAT] All allies fallen!");
        exit.write(AppExit::Success);
        return;
    }
    if state.phase != CombatPhase::WaitingAction {
        return;
    }

    if player_acted.0 {
        player_acted.0 = false;
        order.active_unit = None;
        return;
    }

    let Some(actor_id) = order.active_unit else {
        return;
    };

    let Some((
        actor_unit,
        _actor_team,
        _actor_ult,
        actor_skills,
        actor_ko,
        _actor_commander,
        _,
        _,
        _,
        _,
        _,
    )) = units
        .iter()
        .find(|(u, _, _, _, _, _, _, _, _, _, _)| u.id == actor_id)
    else {
        return;
    };

    if actor_ko.is_some() {
        order.active_unit = None;
        return;
    }

    let Some(skill_book) = skill_book_handle
        .as_ref()
        .and_then(|handle| skill_books.get(&handle.0))
    else {
        println!("[QUERY] Skill book loading; waiting for shared SkillBook asset.");
        return;
    };

    let units_data: Vec<_> = units
        .iter()
        .map(
            |(
                unit,
                team,
                ult,
                skills,
                ko,
                commander,
                toughness,
                counterplay,
                stunned,
                energy,
                ult_metadata,
            )| {
                (
                    unit.id,
                    *team,
                    unit,
                    skills,
                    ult,
                    toughness,
                    counterplay,
                    ko.is_some(),
                    stunned.is_some(),
                    commander.is_some(),
                    energy,
                    ult_metadata,
                )
            },
        )
        .collect();

    let snapshot = build_snapshot_from_ecs_with_sp(
        &state,
        &order,
        sp_pool.current,
        actor_id,
        actor_id,
        units_data.clone(),
    );

    let action_entries = build_action_entries(&snapshot, skill_book, actor_id, actor_skills);
    print_action_entries(&action_entries);

    let basic_entry = action_entries
        .iter()
        .find(|entry| matches!(entry.kind, ActionQueryKind::Basic));

    if !interactive.0 {
        let preferred_entry = action_entries
            .iter()
            .find(|entry| {
                matches!(entry.affordance.action, ActionStatus::Enabled)
                    && matches!(entry.kind, ActionQueryKind::Skill(_))
            })
            .or_else(|| {
                action_entries.iter().find(|entry| {
                    matches!(entry.affordance.action, ActionStatus::Enabled)
                        && matches!(entry.kind, ActionQueryKind::Ultimate)
                })
            })
            .or_else(|| {
                action_entries.iter().find(|entry| {
                    matches!(entry.affordance.action, ActionStatus::Enabled)
                        && matches!(entry.kind, ActionQueryKind::Basic)
                })
            });

        if let Some(entry) = preferred_entry {
            if let Some(target_id) = first_enabled_target_id(&entry.affordance) {
                let intent = match entry.kind {
                    ActionQueryKind::Basic => ActionIntent::Basic {
                        attacker: actor_id,
                        target: target_id,
                    },
                    ActionQueryKind::Skill(skill_id) => {
                        println!("[CLI_PROOF] OnSkillCast intent skill_id={}", skill_id.0);
                        ActionIntent::Skill {
                            attacker: actor_id,
                            skill_id: skill_id.clone(),
                            target: target_id,
                        }
                    }
                    ActionQueryKind::Ultimate => {
                        println!("[CLI_PROOF] OnSkillCast intent skill_id=ultimate");
                        ActionIntent::Ultimate {
                            attacker: actor_id,
                            target: target_id,
                        }
                    }
                };
                intent_writer.write(intent);
                player_acted.0 = true;
                return;
            }
            println!(
                "[QUERY] Auto-selected action has no enabled target: {}",
                action_status_label(&entry.affordance.action)
            );
        } else if let Some(entry) = basic_entry {
            println!(
                "[QUERY] Basic Attack unavailable: {}",
                action_status_label(&entry.affordance.action)
            );
        }

        player_acted.0 = true;
        order.active_unit = None;
        return;
    }

    // ── Out-of-turn ultimate burst (interactive only) ─────────────────────
    // Offer a free burst for any ready off-turn ally before the active unit
    // commits. Firing a burst does NOT consume the active unit's turn — it just
    // emits an UltBurstRequest and returns; the player is re-prompted next frame
    // (with the burst already resolved). The non-interactive proof path returns
    // earlier, so this never perturbs deterministic CI runs (R004).
    let mut burst_choices: Vec<(UnitId, String, Vec<(UnitId, String)>)> = Vec::new();
    for (unit, team, _ult, _skills, ko, _cmd, _tough, _cp, _stun, _energy, _meta) in units.iter() {
        if *team != Team::Ally || unit.id == actor_id || ko.is_some() {
            continue;
        }
        let mut snap = build_snapshot_from_ecs_with_sp(
            &state,
            &order,
            sp_pool.current,
            unit.id,
            unit.id,
            units_data.clone(),
        );
        mark_unit_active(&mut snap, unit.id);
        let aff = query_action_affordance(&snap, skill_book, unit.id, ActionQueryKind::Ultimate);
        if !matches!(aff.action, ActionStatus::Enabled) {
            continue;
        }
        let targets: Vec<(UnitId, String)> = aff
            .targets
            .iter()
            .filter(|(_, t)| matches!(t.status, TargetStatus::Enabled))
            .filter_map(|(tid, t)| {
                units
                    .iter()
                    .find(|(u, _, _, _, _, _, _, _, _, _, _)| u.id == *tid)
                    .map(|(u, tteam, _, _, _, _, _, _, _, _, _)| {
                        (*tid, target_entry_label(u, tteam, t))
                    })
            })
            .collect();
        if !targets.is_empty() {
            burst_choices.push((unit.id, format!("⚡ Burst: {} Ultimate", unit.name), targets));
        }
    }

    if !burst_choices.is_empty() {
        const CONTINUE: &str = "▶ Continue my turn";
        let mut labels: Vec<String> = burst_choices.iter().map(|(_, label, _)| label.clone()).collect();
        labels.push(CONTINUE.to_string());
        let picked = Select::new("Out-of-turn Burst available:", labels)
            .prompt()
            .unwrap_or_else(|_| CONTINUE.to_string());
        if picked != CONTINUE {
            if let Some((attacker, _, targets)) =
                burst_choices.into_iter().find(|(_, label, _)| *label == picked)
            {
                let target_id = if targets.len() == 1 {
                    targets[0].0
                } else {
                    let tlabels: Vec<String> = targets.iter().map(|(_, l)| l.clone()).collect();
                    let chosen = Select::new("Burst target:", tlabels.clone())
                        .prompt()
                        .unwrap_or_else(|_| tlabels[0].clone());
                    targets
                        .iter()
                        .find(|(_, l)| *l == chosen)
                        .map(|(id, _)| *id)
                        .unwrap_or(targets[0].0)
                };
                println!("[CLI] out-of-turn burst requested: {attacker:?} -> {target_id:?}");
                burst_writer.write(UltBurstRequest {
                    attacker,
                    target: target_id,
                });
                // Free action: leave active_unit / PlayerActed untouched.
                return;
            }
        }
    }

    let enabled_actions: Vec<_> = action_entries
        .iter()
        .filter(|entry| matches!(entry.affordance.action, ActionStatus::Enabled))
        .collect();

    if enabled_actions.is_empty() {
        if let Some(entry) = basic_entry {
            println!(
                "[QUERY] No enabled actions; Basic Attack state: {}",
                action_status_label(&entry.affordance.action)
            );
        } else {
            println!("[QUERY] No enabled actions available.");
        }
        player_acted.0 = true;
        order.active_unit = None;
        return;
    }

    println!(
        "\n>>> {}'s turn  (SP: {}/{})",
        actor_unit.name, sp_pool.current, sp_pool.max
    );
    let enabled_action_labels: Vec<String> = enabled_actions
        .iter()
        .map(|entry| entry.label.clone())
        .collect();
    let selected_label = match Select::new("Action:", enabled_action_labels.clone()).prompt() {
        Ok(label) => label,
        Err(_) => {
            println!("Input cancelled — defaulting to first enabled action.");
            enabled_action_labels[0].clone()
        }
    };

    let selected_entry = enabled_actions
        .iter()
        .find(|entry| entry.label == selected_label)
        .copied()
        .unwrap_or(enabled_actions[0]);

    let target_entries: Vec<(UnitId, String)> = selected_entry
        .affordance
        .targets
        .iter()
        .filter(|(_, affordance)| matches!(affordance.status, TargetStatus::Enabled))
        .filter_map(|(target_id, affordance)| {
            units
                .iter()
                .find(|(unit, _, _, _, _, _, _, _, _, _, _)| unit.id == *target_id)
                .map(|(unit, team, _, _, _, _, _, _, _, _, _)| {
                    (*target_id, target_entry_label(unit, team, affordance))
                })
        })
        .collect();

    print_target_entries(&target_entries);

    if target_entries.is_empty() {
        println!(
            "[QUERY] No enabled targets for {}: {}",
            action_kind_label(selected_entry.kind),
            action_status_label(&selected_entry.affordance.action)
        );
        player_acted.0 = true;
        order.active_unit = None;
        return;
    }

    let enabled_target_labels: Vec<String> = target_entries
        .iter()
        .map(|(_, label)| label.clone())
        .collect();

    let selected_target_label = match Select::new("Target:", enabled_target_labels.clone()).prompt()
    {
        Ok(label) => label,
        Err(_) => {
            println!("Input cancelled — defaulting to first enabled target.");
            enabled_target_labels[0].clone()
        }
    };

    let target_id = target_entries
        .iter()
        .find(|(_, label)| *label == selected_target_label)
        .map(|(id, _)| *id)
        .unwrap_or(target_entries[0].0);

    let intent = match selected_entry.kind {
        ActionQueryKind::Basic => ActionIntent::Basic {
            attacker: actor_id,
            target: target_id,
        },
        ActionQueryKind::Skill(skill_id) => ActionIntent::Skill {
            attacker: actor_id,
            skill_id: skill_id.clone(),
            target: target_id,
        },
        ActionQueryKind::Ultimate => ActionIntent::Ultimate {
            attacker: actor_id,
            target: target_id,
        },
    };

    intent_writer.write(intent);
    player_acted.0 = true;
}
