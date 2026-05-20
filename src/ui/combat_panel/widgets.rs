#[cfg(feature = "windowed")]
use bevy_egui::egui;

#[cfg(feature = "windowed")]
use crate::combat::{
    action_query::{
        ActionAffordance, ActionQueryKind, ActionStatus, TargetStatus, first_enabled_target_id,
        query_action_affordance,
    },
    log::{ActionLog, LogEntry},
    sp::SpPool,
    toughness::DamageKind,
    types::UnitId,
};
#[cfg(feature = "windowed")]
use crate::data::{skills_ron::LegalityReasonCode, skills_ron::SkillBook};

#[cfg(feature = "windowed")]
use super::display::{FdDisplay, UnitDisplay};
#[cfg(feature = "windowed")]
use super::labels::{
    TelegraphChip, action_button_label, action_status_label, action_tooltip, attr_color,
    pending_label, target_button_label, target_status_label,
};
#[cfg(feature = "windowed")]
use super::{PendingAction, PendingKind, PreviewDamageCache};

#[cfg(feature = "windowed")]
#[allow(clippy::too_many_arguments)]
pub(super) fn render_action_bar(
    ui: &mut egui::Ui,
    active_actor_id: Option<UnitId>,
    action_snapshot: &Option<crate::combat::action_query::CombatQuerySnapshot>,
    skill_book: &SkillBook,
    active_display: Option<&UnitDisplay>,
    preview_cache: &PreviewDamageCache,
    pending_action: &PendingAction,
    selected_action_affordance: &Option<ActionAffordance<'_>>,
    selected_target_id: Option<UnitId>,
    telegraph_chip: Option<&TelegraphChip>,
    pending_request: &mut Option<Option<PendingKind>>,
) {
    ui.horizontal_wrapped(|ui| {
        if let (Some(actor_id), Some(snapshot)) = (active_actor_id, action_snapshot.as_ref()) {
            let basic_affordance =
                query_action_affordance(snapshot, skill_book, actor_id, ActionQueryKind::Basic);
            let basic_enabled = matches!(basic_affordance.action, ActionStatus::Enabled);
            let basic_pending = PendingKind::Basic;
            let basic_preview = preview_cache.label_for(
                active_actor_id,
                Some(&basic_pending),
                first_enabled_target_id(&basic_affordance),
            );
            let basic_response = ui
                .add_enabled(
                    basic_enabled,
                    egui::Button::new(action_button_label("Basic", &basic_affordance.action)),
                )
                .on_hover_text(action_tooltip(
                    "Basic",
                    &basic_affordance,
                    basic_preview.as_deref(),
                ));
            if basic_response.clicked() {
                *pending_request = Some(Some(basic_pending));
            }

            if let Some(active) = active_display {
                for skill in &active.skills {
                    let skill_affordance = query_action_affordance(
                        snapshot,
                        skill_book,
                        actor_id,
                        ActionQueryKind::Skill(&skill.id),
                    );
                    let skill_enabled = matches!(skill_affordance.action, ActionStatus::Enabled);
                    let skill_pending = PendingKind::Skill(skill.id.clone());
                    let skill_preview = preview_cache.label_for(
                        active_actor_id,
                        Some(&skill_pending),
                        first_enabled_target_id(&skill_affordance),
                    );
                    let skill_response = ui
                        .add_enabled(
                            skill_enabled,
                            egui::Button::new(action_button_label(
                                &skill.label,
                                &skill_affordance.action,
                            )),
                        )
                        .on_hover_text(action_tooltip(
                            &skill.label,
                            &skill_affordance,
                            skill_preview.as_deref(),
                        ));
                    if skill_response.clicked() {
                        *pending_request = Some(Some(skill_pending));
                    }
                }
            }

            let ultimate_affordance =
                query_action_affordance(snapshot, skill_book, actor_id, ActionQueryKind::Ultimate);
            let ultimate_enabled = matches!(ultimate_affordance.action, ActionStatus::Enabled);
            let ultimate_pending = PendingKind::Ultimate;
            let ultimate_preview = preview_cache.label_for(
                active_actor_id,
                Some(&ultimate_pending),
                first_enabled_target_id(&ultimate_affordance),
            );
            let ultimate_response = ui
                .add_enabled(
                    ultimate_enabled,
                    egui::Button::new(action_button_label("Ultimate", &ultimate_affordance.action)),
                )
                .on_hover_text(action_tooltip(
                    "Ultimate",
                    &ultimate_affordance,
                    ultimate_preview.as_deref(),
                ));
            if ultimate_response.clicked() {
                *pending_request = Some(Some(ultimate_pending));
            }
        } else {
            let disabled_status = ActionStatus::Disabled {
                reason: LegalityReasonCode::NotActiveUnit,
            };
            ui.add_enabled(
                false,
                egui::Button::new(action_button_label("Basic", &disabled_status)),
            )
            .on_hover_text("Basic unavailable: active actor missing");
            ui.add_enabled(
                false,
                egui::Button::new(action_button_label("Skill", &disabled_status)),
            )
            .on_hover_text("Skill unavailable: active actor missing");
            ui.add_enabled(
                false,
                egui::Button::new(action_button_label("Ultimate", &disabled_status)),
            )
            .on_hover_text("Ultimate unavailable: active actor missing");
        }

        if let Some(kind) = pending_action.kind.as_ref() {
            if let Some(affordance) = selected_action_affordance.as_ref() {
                if let Some(preview) =
                    preview_cache.label_for(active_actor_id, Some(kind), selected_target_id)
                {
                    ui.label(format!(
                        "Pending: {} [{}] · {}",
                        pending_label(kind),
                        action_status_label(&affordance.action),
                        preview,
                    ));
                } else {
                    ui.label(format!(
                        "Pending: {} [{}]",
                        pending_label(kind),
                        action_status_label(&affordance.action)
                    ));
                }
            } else {
                ui.label(format!("Pending: {}", pending_label(kind)));
            }
        } else {
            ui.label("Pending: choose an action, then click a target");
        }

        if let Some(chip) = telegraph_chip {
            egui::Frame::default()
                .fill(egui::Color32::from_rgb(95, 45, 20))
                .stroke(egui::Stroke::new(
                    1.0_f32,
                    egui::Color32::from_rgb(230, 170, 90),
                ))
                .corner_radius(6.0)
                .inner_margin(egui::Margin::symmetric(8, 4))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(&chip.label)
                            .small()
                            .color(egui::Color32::from_rgb(255, 232, 200))
                            .strong(),
                    )
                    .on_hover_text(&chip.tooltip);
                });
        }
    });
}

#[cfg(feature = "windowed")]
#[allow(clippy::too_many_arguments)]
pub(super) fn render_columns(
    ui: &mut egui::Ui,
    allies: &[UnitDisplay],
    enemies: &[UnitDisplay],
    sp: &SpPool,
    log: &ActionLog,
    any_broken: bool,
    action_snapshot_is_none: bool,
    selected_action_affordance: &Option<ActionAffordance<'_>>,
    active_actor_id: Option<UnitId>,
    preview_cache: &PreviewDamageCache,
    pending_action: &PendingAction,
    clicked_target: &mut Option<UnitId>,
) {
    let pending_targets = selected_action_affordance.as_ref();
    let pending_enabled = pending_targets
        .is_some_and(|affordance| matches!(affordance.action, ActionStatus::Enabled));

    ui.columns(3, |cols| {
        for ally in allies {
            cols[0].group(|ui| {
                let bg = egui::Frame::default()
                    .fill(attr_color(ally.attribute))
                    .inner_margin(egui::Margin::symmetric(4, 2));
                bg.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(&ally.name);
                        if ally.is_commander {
                            ui.label(egui::RichText::new("COMMANDER").color(egui::Color32::YELLOW));
                        }
                    });
                });
                let hp_ratio = ally.hp_cur as f32 / ally.hp_max.max(1) as f32;
                ui.add(
                    egui::ProgressBar::new(hp_ratio)
                        .text(format!("{}/{}", ally.hp_cur, ally.hp_max)),
                );
                let ult_ratio = ally.ult_cur as f32 / ally.ult_cap.max(1) as f32;
                let ult_label = if ally.ult_cur >= ally.ult_trigger {
                    "READY".to_string()
                } else {
                    format!("{}/{}", ally.ult_cur, ally.ult_cap)
                };
                ui.add(egui::ProgressBar::new(ult_ratio).text(ult_label));
                if let (Some(current), Some(max)) = (ally.energy_cur, ally.energy_max) {
                    ui.label(format!("Energy: {current}/{max}"));
                    if let (Some(sec), Some(ext)) =
                        (ally.energy_secondary_gained, ally.energy_external_gained)
                    {
                        ui.label(format!("Round energy: +sec {sec} / +ext {ext}"));
                    }
                }
                if ally.is_stunned {
                    ui.label(egui::RichText::new("STUNNED").color(egui::Color32::YELLOW));
                }
                if ally.is_ko {
                    ui.label(egui::RichText::new("KO").color(egui::Color32::RED));
                }

                if let Some(target_affordance) = pending_targets.and_then(|affordance| {
                    affordance
                        .targets
                        .iter()
                        .find(|(id, _)| *id == ally.id)
                        .map(|(_, target)| target)
                }) {
                    ui.separator();
                    ui.label(format!(
                        "Target: {}",
                        target_status_label(&target_affordance.status)
                    ));
                    let target_enabled = matches!(target_affordance.status, TargetStatus::Enabled);
                    let target_preview = preview_cache.label_for(
                        active_actor_id,
                        pending_action.kind.as_ref(),
                        Some(ally.id),
                    );
                    let target_response = ui
                        .add_enabled(
                            target_enabled,
                            egui::Button::new(target_button_label(
                                &ally.name,
                                Some(target_affordance),
                            )),
                        )
                        .on_hover_text({
                            let mut text = format!(
                                "{}
HP: {}/{}
{}",
                                ally.name,
                                ally.hp_cur,
                                ally.hp_max,
                                if ally.is_commander {
                                    "Commander target"
                                } else {
                                    ""
                                }
                            );
                            if let Some(preview) = target_preview {
                                text.push('\n');
                                text.push_str(&preview);
                            }
                            text
                        });
                    if target_response.clicked() && pending_enabled {
                        *clicked_target = Some(ally.id);
                    }
                }
            });
        }

        if any_broken {
            cols[1].label(
                egui::RichText::new("BREAK!")
                    .size(24.0)
                    .color(egui::Color32::RED)
                    .strong(),
            );
        }
        cols[1].label(format!("SP {}/{}", sp.current, sp.max));
        let sp_ratio = sp.current as f32 / sp.max.max(1) as f32;
        cols[1].add(egui::ProgressBar::new(sp_ratio));
        if action_snapshot_is_none {
            cols[1].label(
                egui::RichText::new("Active actor unavailable; controls disabled")
                    .color(egui::Color32::YELLOW),
            );
        }
        cols[1].separator();
        for (i, ev) in log.events.iter().enumerate() {
            let s = match ev {
                LogEntry::BasicHit {
                    attacker,
                    target,
                    amount,
                    kind,
                } => format!(
                    "[{}] Hit({:?}→{:?}, {} {:?})",
                    i + 1,
                    attacker,
                    target,
                    amount,
                    kind
                ),
                LogEntry::Break { target, damage_tag } => {
                    format!("[{}] Break({:?}, {:?})", i + 1, target, damage_tag)
                }
                LogEntry::Ko { target } => {
                    format!("[{}] KO({:?})", i + 1, target)
                }
                LogEntry::Revive { target, hp_after } => {
                    format!("[{}] Revive({:?}, hp={})", i + 1, target, hp_after)
                }
                LogEntry::ActionFailed { reason } => {
                    format!("[{}] Failed({})", i + 1, reason)
                }
                LogEntry::AdvanceTurn { target, amount_pct } => {
                    format!("[{}] Advance({:?}, {}%)", i + 1, target, amount_pct)
                }
                LogEntry::DelayTurn { target, amount_pct } => {
                    format!("[{}] Delay({:?}, {}%)", i + 1, target, amount_pct)
                }
            };
            cols[1].label(egui::RichText::new(s).monospace());
        }

        for enemy in enemies {
            cols[2].group(|ui| {
                let chip =
                    egui::Button::new(egui::RichText::new(&enemy.name).color(egui::Color32::BLACK))
                        .fill(attr_color(enemy.attribute));
                let enemy_target = pending_targets.and_then(|affordance| {
                    affordance
                        .targets
                        .iter()
                        .find(|(id, _)| *id == enemy.id)
                        .map(|(_, target)| target)
                });
                if let Some(target_affordance) = enemy_target {
                    ui.label(format!(
                        "Target: {}",
                        target_status_label(&target_affordance.status)
                    ));
                    let target_enabled = matches!(target_affordance.status, TargetStatus::Enabled);
                    let target_preview = preview_cache.label_for(
                        active_actor_id,
                        pending_action.kind.as_ref(),
                        Some(enemy.id),
                    );
                    let response = ui.add_enabled(target_enabled, chip).on_hover_text({
                        let mut text = format!(
                            "{}
HP: {}/{}
{}",
                            enemy.name,
                            enemy.hp_cur,
                            enemy.hp_max,
                            if enemy.is_ko { "KO target" } else { "" }
                        );
                        if let Some(preview) = target_preview {
                            text.push('\n');
                            text.push_str(&preview);
                        }
                        text
                    });
                    if response.clicked() && pending_enabled {
                        *clicked_target = Some(enemy.id);
                    }
                } else {
                    ui.add_enabled(false, chip)
                        .on_hover_text(format!("{} (target unavailable)", enemy.name));
                }
                let hp_ratio = enemy.hp_cur as f32 / enemy.hp_max.max(1) as f32;
                ui.add(
                    egui::ProgressBar::new(hp_ratio)
                        .text(format!("{}/{}", enemy.hp_cur, enemy.hp_max)),
                );
                if let Some(toughness) = enemy.toughness.as_ref() {
                    let tough_ratio = toughness.current as f32 / toughness.max.max(1) as f32;
                    ui.add(
                        egui::ProgressBar::new(tough_ratio.max(0.0))
                            .fill(egui::Color32::ORANGE)
                            .text(format!("{}/{}", toughness.current, toughness.max)),
                    );
                    if !toughness.weaknesses.is_empty() {
                        ui.label(format!("WEAK: {:?}", toughness.weaknesses[0]));
                    }
                    if toughness.broken {
                        ui.label(egui::RichText::new("BROKEN").color(egui::Color32::RED));
                    }
                }
                if enemy.is_ko {
                    ui.label(egui::RichText::new("KO").color(egui::Color32::RED));
                }
                if enemy.is_commander {
                    ui.label(egui::RichText::new("COMMANDER").color(egui::Color32::YELLOW));
                }
            });
        }
    });
}

#[cfg(feature = "windowed")]
pub(super) fn render_floating(ui: &mut egui::Ui, fd_displays: &[FdDisplay]) {
    let painter = ui.painter();
    let content_rect = ui.ctx().content_rect();
    let panel_w = content_rect.width();
    for fd in fd_displays {
        let (prefix, color) = match fd.kind {
            DamageKind::Normal => (
                "",
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, fd.alpha),
            ),
            DamageKind::Weak => (
                "WEAK ",
                egui::Color32::from_rgba_unmultiplied(255, 220, 0, fd.alpha),
            ),
            DamageKind::Resist => (
                "RES ",
                egui::Color32::from_rgba_unmultiplied(160, 160, 160, fd.alpha),
            ),
            DamageKind::Break => (
                "BRK ",
                egui::Color32::from_rgba_unmultiplied(255, 140, 0, fd.alpha),
            ),
        };
        let jitter = (fd.target_idx as f32 * 17.0) % 60.0 - 30.0;
        let x = panel_w * 5.0 / 6.0 + jitter;
        let y = content_rect.top() + 150.0 + (fd.target_idx as f32 * 17.0) % 40.0;
        painter.text(
            egui::pos2(x, y),
            egui::Align2::CENTER_CENTER,
            format!("{}{}", prefix, fd.amount),
            egui::FontId::proportional(18.0),
            color,
        );
    }
}
