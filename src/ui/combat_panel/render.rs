#[cfg(feature = "windowed")]
use bevy::ecs::system::ParamSet;
#[cfg(feature = "windowed")]
use bevy::prelude::*;
#[cfg(feature = "windowed")]
use bevy_egui::{EguiContexts, egui};
#[cfg(feature = "windowed")]
use moonshine_kind::Instance;

#[cfg(feature = "windowed")]
use crate::combat::{
    action_query::{
        ActionStatus, TargetStatus, build_snapshot_from_ecs_with_sp, first_enabled_target_id,
    },
    floating::{FLOATING_LIFETIME_SECS, FloatingDamage},
    log::ActionLog,
    sp::SpPool,
    state::{CombatPhase, CombatState},
    team::Team,
    toughness::visible_toughness,
    turn_order::TurnOrder,
    turn_system::ActionIntent,
    types::UnitId,
    unit::Unit,
};
#[cfg(feature = "windowed")]
use crate::combat::runtime::SuspendedTimelineState;
use crate::data::skills_ron::SkillBook;

#[cfg(feature = "windowed")]
use super::display::{FdDisplay, SkillDisplay, UnitDisplay};
#[cfg(feature = "windowed")]
use super::labels::{cue_barrier_chip, query_pending_action_affordance, skill_name};
#[cfg(feature = "windowed")]
use super::widgets::{render_action_bar, render_columns, render_floating};
#[cfg(feature = "windowed")]
use super::{CombatPanelUnitsQuery, PendingAction, PendingKind, PreviewDamageCache};

#[cfg(feature = "windowed")]
#[allow(clippy::too_many_arguments)]
pub fn combat_panel(
    mut contexts: EguiContexts,
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut order: ResMut<TurnOrder>,
    mut pending_action: ResMut<PendingAction>,
    mut combat_state: ResMut<CombatState>,
    mut sp: ResMut<SpPool>,
    mut log: ResMut<ActionLog>,
    preview_cache: Res<PreviewDamageCache>,
    skill_books: Res<Assets<SkillBook>>,
    barrier: Res<SuspendedTimelineState>,
    mut action_intent: MessageWriter<ActionIntent>,
    units_q: CombatPanelUnitsQuery,
    floating_q: Query<&FloatingDamage>,
    mut despawn_q: ParamSet<(Query<Instance<Unit>>, Query<Instance<FloatingDamage>>)>,
) -> Result {
    let fallback_skill_book = SkillBook(Vec::new());
    let skill_book = skill_books
        .iter()
        .next()
        .map(|(_, book)| book)
        .unwrap_or(&fallback_skill_book);

    let mut unit_displays: Vec<UnitDisplay> = Vec::new();
    let mut units_data = Vec::new();
    for (unit, team, tough, counterplay, ult, kit, ko, commander, stunned, energy, tracker) in
        &units_q
    {
        units_data.push((
            unit.id,
            *team,
            unit,
            Some(kit),
            Some(ult),
            tough,
            counterplay,
            ko.is_some(),
            stunned.is_some(),
            commander.is_some(),
            energy,
            tracker,
        ));

        unit_displays.push(UnitDisplay {
            id: unit.id,
            team: *team,
            name: unit.name.clone(),
            attribute: unit.attribute,
            hp_cur: unit.hp_current,
            hp_max: unit.hp_max,
            ult_cur: ult.current,
            ult_trigger: ult.trigger,
            ult_cap: ult.cap,
            skills: kit
                .skills
                .iter()
                .cloned()
                .map(|id| SkillDisplay {
                    label: skill_name(Some(skill_book), &id),
                    id,
                })
                .collect(),
            is_ko: ko.is_some(),
            is_stunned: stunned.is_some(),
            is_commander: commander.is_some(),
            toughness: visible_toughness(*team, tough.as_deref()),
            energy_cur: energy.map(|value| value.current),
            energy_max: energy.map(|value| value.max),
            energy_secondary_gained: tracker.map(|value| value.secondary_gained),
            energy_external_gained: tracker.map(|value| value.external_gained),
        });
    }
    unit_displays.sort_by_key(|unit| unit.id.0);

    let mut allies: Vec<UnitDisplay> = unit_displays
        .iter()
        .filter(|unit| unit.team == Team::Ally)
        .cloned()
        .collect();
    allies.sort_by_key(|unit| unit.id.0);

    let mut enemies: Vec<UnitDisplay> = unit_displays
        .iter()
        .filter(|unit| unit.team == Team::Enemy)
        .cloned()
        .collect();
    enemies.sort_by_key(|unit| unit.id.0);

    let active_actor_id = order.active_unit;
    let active_display =
        active_actor_id.and_then(|id| unit_displays.iter().find(|unit| unit.id == id));

    let action_snapshot = active_actor_id.map(|actor_id| {
        build_snapshot_from_ecs_with_sp(
            &combat_state,
            &order,
            sp.current,
            actor_id,
            actor_id,
            units_data,
        )
    });

    let selected_action_affordance = match (
        action_snapshot.as_ref(),
        active_actor_id,
        pending_action.kind.as_ref(),
    ) {
        (Some(snapshot), Some(actor_id), Some(kind)) => Some(query_pending_action_affordance(
            snapshot, skill_book, actor_id, kind,
        )),
        _ => None,
    };
    let selected_target_id = selected_action_affordance
        .as_ref()
        .and_then(first_enabled_target_id);

    let any_broken = enemies
        .iter()
        .any(|enemy| enemy.toughness.as_ref().is_some_and(|t| t.broken) && !enemy.is_ko);

    let now = time.elapsed_secs();
    let fd_displays: Vec<FdDisplay> = floating_q
        .iter()
        .filter_map(|fd| {
            let elapsed = now - fd.spawn_time;
            if elapsed >= FLOATING_LIFETIME_SECS {
                return None;
            }
            let alpha = ((1.0 - elapsed / FLOATING_LIFETIME_SECS).clamp(0.0, 1.0) * 255.0) as u8;
            Some(FdDisplay {
                target_idx: fd.target.0,
                amount: fd.amount,
                kind: fd.kind,
                alpha,
            })
        })
        .collect();

    let telegraph_chip = cue_barrier_chip(barrier.active_status(), Some(skill_book));

    let mut pending_request: Option<Option<PendingKind>> = None;
    let mut clicked_target: Option<UnitId> = None;
    let mut restart_clicked = false;

    let ctx = contexts.ctx_mut()?;
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("Combat Sandbox");
            ui.separator();
            ui.label(format!("Phase: {:?}", combat_state.phase));
            match (active_actor_id, active_display) {
                (Some(_), Some(active)) => {
                    ui.label(format!("Active: {}", active.name));
                }
                (None, Some(active)) if combat_state.phase == CombatPhase::WaitingAction => {
                    ui.label(format!("Preview: {}", active.name));
                }
                _ if combat_state.phase == CombatPhase::WaitingAction => {
                    ui.label("Active: unavailable");
                }
                _ => {}
            }
        });

        render_action_bar(
            ui,
            active_actor_id,
            &action_snapshot,
            skill_book,
            active_display,
            &preview_cache,
            &pending_action,
            &selected_action_affordance,
            selected_target_id,
            telegraph_chip.as_ref(),
            &mut pending_request,
        );
        ui.separator();

        render_columns(
            ui,
            &allies,
            &enemies,
            &sp,
            &log,
            any_broken,
            action_snapshot.is_none(),
            &selected_action_affordance,
            active_actor_id,
            &preview_cache,
            &pending_action,
            &mut clicked_target,
        );

        render_floating(ui, &fd_displays);
    });

    if matches!(
        combat_state.phase,
        CombatPhase::Victory | CombatPhase::Defeat
    ) {
        let (title, color) = match combat_state.phase {
            CombatPhase::Victory => ("Victory", egui::Color32::LIGHT_GREEN),
            CombatPhase::Defeat => ("Defeat", egui::Color32::LIGHT_RED),
            _ => unreachable!(),
        };
        egui::Area::new(egui::Id::new("banner"))
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new(title).size(32.0).color(color).strong());
                        ui.add_space(8.0);
                        if ui.button("Restart").clicked() {
                            restart_clicked = true;
                        }
                    });
                });
            });
    }

    let selected_action_enabled = selected_action_affordance
        .as_ref()
        .is_some_and(|affordance| matches!(affordance.action, ActionStatus::Enabled));
    let should_clear_pending = pending_action.kind.is_some() && !selected_action_enabled;

    if let (Some(target), Some(active), Some(affordance)) = (
        clicked_target,
        active_actor_id,
        selected_action_affordance.as_ref(),
    ) {
        if matches!(affordance.action, ActionStatus::Enabled) {
            if let Some((_, target_affordance)) =
                affordance.targets.iter().find(|(id, _)| *id == target)
            {
                if matches!(target_affordance.status, TargetStatus::Enabled) {
                    let intent = match pending_action.kind.as_ref().expect("pending action") {
                        PendingKind::Basic => ActionIntent::Basic {
                            attacker: active,
                            target,
                        },
                        PendingKind::Skill(skill_id) => ActionIntent::Skill {
                            attacker: active,
                            skill_id: skill_id.clone(),
                            target,
                        },
                        PendingKind::Ultimate => ActionIntent::Ultimate {
                            attacker: active,
                            target,
                        },
                    };
                    action_intent.write(intent);
                    pending_action.kind = None;
                }
            }
        }
    }

    if should_clear_pending {
        pending_action.kind = None;
    }
    if let Some(request) = pending_request {
        pending_action.kind = request;
    }

    if restart_clicked {
        pending_action.kind = None;
        combat_state.reset();
        *order = TurnOrder::default();
        log.events.clear();
        *sp = SpPool::default();
        for floating in &despawn_q.p1() {
            commands.entity(floating.entity()).despawn();
        }
        for unit in &despawn_q.p0() {
            commands.entity(unit.entity()).despawn();
        }
        // Reload per-digimon unit sources to trigger re-assembly.
        for path in crate::data::DIGIMON_UNIT_PATHS
            .iter()
            .chain(crate::data::ENEMY_UNIT_PATHS.iter())
        {
            asset_server.reload(*path);
        }
        info!("restart: roster reloaded");
    }

    Ok(())
}
