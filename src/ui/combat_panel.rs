#[cfg(feature = "windowed")]
use bevy::prelude::*;
#[cfg(feature = "windowed")]
use bevy_egui::{EguiContexts, egui};
#[cfg(feature = "windowed")]
use moonshine_kind::Instance;

#[cfg(feature = "windowed")]
use crate::combat::{
    action_query::{
        ActionAffordance, ActionQueryKind, ActionStatus, CombatQuerySnapshot,
        TargetAffordance, TargetStatus,
        build_snapshot_from_ecs_with_sp, first_enabled_target_id, query_action_affordance,
    },
    api::intent::CastIdGen,
    enemy_counterplay::EnemyCounterplayKit,
    energy::{Energy, RoundEnergyTracker},
    floating::{FLOATING_LIFETIME_SECS, FloatingDamage},
    kit::UnitSkills,
    log::{ActionLog, LogEntry},
    preview::{PreviewDamageSummary, query_skill_preview, summarize_preview_damage},
    sp::SpPool,
    state::{CombatPhase, CombatState},
    stun::Stunned,
    team::Team,
    toughness::{DamageKind, Toughness, visible_toughness},
    turn_order::TurnOrder,
    turn_system::ActionIntent,
    types::{Attribute, SkillId, UnitId},
    ultimate::UltimateCharge,
    unit::{Commander, Ko, Unit},
};
#[cfg(feature = "windowed")]
use crate::data::{SkillBookHandle, skills_ron::LegalityReasonCode, skills_ron::SkillBook};

#[cfg(feature = "windowed")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingKind {
    Basic,
    Skill(SkillId),
    Ultimate,
}

#[cfg(feature = "windowed")]
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct PendingAction {
    pub kind: Option<PendingKind>,
}

#[cfg(feature = "windowed")]
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct PreviewDamageCache {
    pub actor_id: Option<UnitId>,
    pub pending_kind: Option<PendingKind>,
    pub skill_id: Option<SkillId>,
    pub target_id: Option<UnitId>,
    pub summary: Option<PreviewDamageSummary>,
}

#[cfg(feature = "windowed")]
impl PreviewDamageCache {
    fn matches_context(
        &self,
        actor_id: Option<UnitId>,
        pending_kind: Option<&PendingKind>,
        target_id: Option<UnitId>,
    ) -> bool {
        self.actor_id == actor_id
            && self.target_id == target_id
            && self.pending_kind.as_ref() == pending_kind
    }

    fn label_for(
        &self,
        actor_id: Option<UnitId>,
        pending_kind: Option<&PendingKind>,
        target_id: Option<UnitId>,
    ) -> Option<String> {
        let summary = self.summary.as_ref()?;
        if !self.matches_context(actor_id, pending_kind, target_id) {
            return None;
        }

        Some(format!(
            "preview: {} dmg across {} hit(s)",
            summary.total_damage, summary.deal_damage_intents
        ))
    }
}

#[cfg(feature = "windowed")]
fn pending_kind_skill_id(kind: &PendingKind, kit: &UnitSkills) -> SkillId {
    match kind {
        PendingKind::Basic => kit.basic.clone(),
        PendingKind::Skill(skill_id) => skill_id.clone(),
        PendingKind::Ultimate => kit.ultimate.clone(),
    }
}

#[cfg(feature = "windowed")]
pub fn refresh_preview_damage_cache(world: &mut World) {
    let Some(active_actor_id) = world
        .get_resource::<TurnOrder>()
        .and_then(|order| order.active_unit)
    else {
        return;
    };

    let Some(pending_kind) = world
        .get_resource::<PendingAction>()
        .and_then(|pending| pending.kind.clone())
    else {
        return;
    };

    let Some((skill_id, target_id, summary)) =
        (|| -> Option<(SkillId, UnitId, PreviewDamageSummary)> {
            let skill_book = world
                .get_resource::<Assets<SkillBook>>()
                .and_then(|assets| {
                    world
                        .get_resource::<SkillBookHandle>()
                        .and_then(|handle| assets.get(&handle.0).cloned())
                })?;
            let Some(mut cast_id_gen) = world.get_resource_mut::<CastIdGen>() else {
                return None;
            };
            let cast_id = cast_id_gen.next();
            let combat_state = world.resource::<CombatState>().clone();
            let order = world.resource::<TurnOrder>().clone();
            let sp_current = world.resource::<SpPool>().current;

            let mut units_data = Vec::new();
            let mut active_kit: Option<UnitSkills> = None;
            let mut units_q = world.query::<(
                &'static Unit,
                &'static Team,
                Option<&'static Toughness>,
                Option<&'static EnemyCounterplayKit>,
                &'static UltimateCharge,
                &'static UnitSkills,
                Option<&'static Ko>,
                Option<&'static Commander>,
                Option<&'static Stunned>,
                Option<&'static Energy>,
                Option<&'static RoundEnergyTracker>,
            )>();
            for (
                unit,
                team,
                tough,
                counterplay,
                ult,
                kit,
                ko,
                commander,
                stunned,
                energy,
                tracker,
            ) in units_q.iter(world)
            {
                if unit.id == active_actor_id {
                    active_kit = Some(kit.clone());
                }
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
            }

            let kit = active_kit?;
            let snapshot = build_snapshot_from_ecs_with_sp(
                &combat_state,
                &order,
                sp_current,
                active_actor_id,
                active_actor_id,
                units_data,
            );

            let affordance = query_pending_action_affordance(
                &snapshot,
                &skill_book,
                active_actor_id,
                &pending_kind,
            );
            if !matches!(affordance.action, ActionStatus::Enabled) {
                return None;
            }

            let target_id = first_enabled_target_id(&affordance)?;
            let skill_id = pending_kind_skill_id(&pending_kind, &kit);
            let preview_pending =
                query_skill_preview(world, &skill_id, cast_id, active_actor_id, target_id);
            let summary = summarize_preview_damage(&preview_pending);
            Some((skill_id, target_id, summary))
        })()
    else {
        return;
    };

    let mut cache = world.resource_mut::<PreviewDamageCache>();
    cache.actor_id = Some(active_actor_id);
    cache.pending_kind = Some(pending_kind);
    cache.skill_id = Some(skill_id);
    cache.target_id = Some(target_id);
    cache.summary = Some(summary);
}

#[cfg(feature = "windowed")]
type CombatPanelUnitsQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Unit,
        &'static Team,
        Option<&'static Toughness>,
        Option<&'static EnemyCounterplayKit>,
        &'static UltimateCharge,
        &'static UnitSkills,
        Option<&'static Ko>,
        Option<&'static Commander>,
        Option<&'static Stunned>,
        Option<&'static Energy>,
        Option<&'static RoundEnergyTracker>,
    ),
>;

#[cfg(feature = "windowed")]
fn pending_label(kind: &PendingKind) -> String {
    match kind {
        PendingKind::Basic => "Basic".to_string(),
        PendingKind::Skill(skill_id) => format!("Skill: {}", skill_id.0),
        PendingKind::Ultimate => "Ultimate".to_string(),
    }
}

#[cfg(feature = "windowed")]
fn skill_name(skill_book: Option<&SkillBook>, skill_id: &SkillId) -> String {
    skill_book
        .and_then(|book| book.0.iter().find(|skill| skill.id == *skill_id))
        .map(|skill| skill.name.clone())
        .unwrap_or_else(|| skill_id.0.clone())
}

#[cfg(feature = "windowed")]
fn action_status_label(status: &ActionStatus) -> String {
    match status {
        ActionStatus::Enabled => "enabled".to_string(),
        ActionStatus::Disabled { reason } => format!("disabled({reason:?})"),
        ActionStatus::Deferred { reason } => format!("deferred({reason:?})"),
        ActionStatus::Hidden { reason } => format!("hidden({reason:?})"),
    }
}

#[cfg(feature = "windowed")]
fn target_status_label(status: &TargetStatus) -> String {
    match status {
        TargetStatus::Enabled => "enabled".to_string(),
        TargetStatus::Disabled { reason } => format!("disabled({reason:?})"),
        TargetStatus::Deferred { reason } => format!("deferred({reason:?})"),
        TargetStatus::Hidden { reason } => format!("hidden({reason:?})"),
    }
}

#[cfg(feature = "windowed")]
fn action_button_label(base: &str, status: &ActionStatus) -> String {
    match status {
        ActionStatus::Enabled => base.to_string(),
        _ => format!("{base} · {}", action_status_label(status)),
    }
}

#[cfg(feature = "windowed")]
fn action_tooltip(base: &str, affordance: &ActionAffordance<'_>, preview: Option<&str>) -> String {
    let targets = if affordance.targets.is_empty() {
        "none".to_string()
    } else {
        affordance
            .targets
            .iter()
            .map(|(id, target)| format!("{:?}: {}", id, target_status_label(&target.status)))
            .collect::<Vec<_>>()
            .join(" | ")
    };

    let mut tooltip = format!(
        "{base}\naction: {}\ntargets: {}",
        action_status_label(&affordance.action),
        targets,
    );
    if let Some(preview) = preview {
        tooltip.push_str("\n");
        tooltip.push_str(preview);
    }
    tooltip
}

#[cfg(feature = "windowed")]
fn attr_color(a: Attribute) -> egui::Color32 {
    match a {
        Attribute::Vaccine => egui::Color32::from_rgb(80, 140, 220),
        Attribute::Data => egui::Color32::from_rgb(220, 200, 60),
        Attribute::Virus => egui::Color32::from_rgb(200, 60, 180),
        Attribute::Free => egui::Color32::from_gray(160),
    }
}

#[cfg(feature = "windowed")]
fn target_button_label(unit_name: &str, affordance: Option<&TargetAffordance>) -> String {
    match affordance {
        Some(affordance) => format!("{unit_name} · {}", target_status_label(&affordance.status)),
        None => unit_name.to_string(),
    }
}

#[cfg(feature = "windowed")]
fn query_pending_action_affordance<'a>(
    snapshot: &'a CombatQuerySnapshot,
    skill_book: &'a SkillBook,
    actor_id: UnitId,
    kind: &'a PendingKind,
) -> ActionAffordance<'a> {
    match kind {
        PendingKind::Basic => {
            query_action_affordance(snapshot, skill_book, actor_id, ActionQueryKind::Basic)
        }
        PendingKind::Skill(skill_id) => query_action_affordance(
            snapshot,
            skill_book,
            actor_id,
            ActionQueryKind::Skill(skill_id),
        ),
        PendingKind::Ultimate => {
            query_action_affordance(snapshot, skill_book, actor_id, ActionQueryKind::Ultimate)
        }
    }
}

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
    mut action_intent: MessageWriter<ActionIntent>,
    units_q: CombatPanelUnitsQuery,
    floating_q: Query<&FloatingDamage>,
    unit_entities: Query<Instance<Unit>>,
    floating_entities: Query<Instance<FloatingDamage>>,
) -> Result {
    #[derive(Clone)]
    struct SkillDisplay {
        id: SkillId,
        label: String,
    }

    #[derive(Clone)]
    struct UnitDisplay {
        id: UnitId,
        team: Team,
        name: String,
        attribute: Attribute,
        hp_cur: i32,
        hp_max: i32,
        ult_cur: i32,
        ult_trigger: i32,
        ult_cap: i32,
        skills: Vec<SkillDisplay>,
        is_ko: bool,
        is_stunned: bool,
        is_commander: bool,
        toughness: Option<crate::combat::toughness::ToughnessView>,
        energy_cur: Option<i32>,
        energy_max: Option<i32>,
        energy_secondary_gained: Option<i32>,
        energy_external_gained: Option<i32>,
    }

    struct FdDisplay {
        target_idx: u32,
        amount: i32,
        kind: DamageKind,
        alpha: u8,
    }

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
    let preview_actor_id = order.future_preview.first().copied();
    let display_active_id = active_actor_id.or(preview_actor_id);
    let active_display =
        display_active_id.and_then(|id| unit_displays.iter().find(|unit| unit.id == id));

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
                    pending_request = Some(Some(basic_pending));
                }

                if let Some(active) = active_display {
                    for skill in &active.skills {
                        let skill_affordance = query_action_affordance(
                            snapshot,
                            skill_book,
                            actor_id,
                            ActionQueryKind::Skill(&skill.id),
                        );
                        let skill_enabled =
                            matches!(skill_affordance.action, ActionStatus::Enabled);
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
                            pending_request = Some(Some(skill_pending));
                        }
                    }
                }

                let ultimate_affordance = query_action_affordance(
                    snapshot,
                    skill_book,
                    actor_id,
                    ActionQueryKind::Ultimate,
                );
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
                        egui::Button::new(action_button_label(
                            "Ultimate",
                            &ultimate_affordance.action,
                        )),
                    )
                    .on_hover_text(action_tooltip(
                        "Ultimate",
                        &ultimate_affordance,
                        ultimate_preview.as_deref(),
                    ));
                if ultimate_response.clicked() {
                    pending_request = Some(Some(ultimate_pending));
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
        });
        ui.separator();

        let pending_targets = selected_action_affordance.as_ref();
        let pending_enabled = pending_targets
            .is_some_and(|affordance| matches!(affordance.action, ActionStatus::Enabled));

        ui.columns(3, |cols| {
            for ally in &allies {
                cols[0].group(|ui| {
                    let bg = egui::Frame::default()
                        .fill(attr_color(ally.attribute))
                        .inner_margin(egui::Margin::symmetric(4, 2));
                    bg.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(&ally.name);
                            if ally.is_commander {
                                ui.label(
                                    egui::RichText::new("COMMANDER").color(egui::Color32::YELLOW),
                                );
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
                        let target_enabled =
                            matches!(target_affordance.status, TargetStatus::Enabled);
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
                            clicked_target = Some(ally.id);
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
            if action_snapshot.is_none() {
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

            for enemy in &enemies {
                cols[2].group(|ui| {
                    let chip = egui::Button::new(
                        egui::RichText::new(&enemy.name).color(egui::Color32::BLACK),
                    )
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
                        let target_enabled =
                            matches!(target_affordance.status, TargetStatus::Enabled);
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
                            clicked_target = Some(enemy.id);
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

        let painter = ui.painter();
        let content_rect = ui.ctx().content_rect();
        let panel_w = content_rect.width();
        for fd in &fd_displays {
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
        for floating in &floating_entities {
            commands.entity(floating.entity()).despawn();
        }
        for unit in &unit_entities {
            commands.entity(unit.entity()).despawn();
        }
        asset_server.reload("data/units.ron");
        info!("restart: roster reloaded");
    }

    Ok(())
}
