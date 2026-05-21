#[cfg(feature = "windowed")]
use bevy_egui::egui;

#[cfg(feature = "windowed")]
use crate::combat::{
    action_query::{
        ActionAffordance, ActionQueryKind, ActionStatus, CombatQuerySnapshot, TargetAffordance,
        TargetStatus, query_action_affordance,
    },
    runtime::CueBarrierStatus,
    types::Attribute,
    types::{SkillId, UnitId},
};
#[cfg(feature = "windowed")]
use crate::data::skills_ron::SkillBook;

#[cfg(feature = "windowed")]
#[derive(Debug, Clone)]
pub struct TelegraphChip {
    pub label: String,
    pub tooltip: String,
}

#[cfg(feature = "windowed")]
use super::{BabyBurnerFlashDisplay, PendingKind};

#[cfg(feature = "windowed")]
pub(crate) fn pending_label(kind: &PendingKind) -> String {
    match kind {
        PendingKind::Basic => "Basic".to_string(),
        PendingKind::Skill(skill_id) => format!("Skill: {}", skill_id.0),
        PendingKind::Ultimate => "Ultimate".to_string(),
    }
}

#[cfg(feature = "windowed")]
pub(crate) fn skill_name(skill_book: Option<&SkillBook>, skill_id: &SkillId) -> String {
    skill_book
        .and_then(|book| book.0.iter().find(|skill| skill.id == *skill_id))
        .map(|skill| skill.name.clone())
        .unwrap_or_else(|| skill_id.0.clone())
}

#[cfg(feature = "windowed")]
pub(crate) fn action_status_label(status: &ActionStatus) -> String {
    match status {
        ActionStatus::Enabled => "enabled".to_string(),
        ActionStatus::Disabled { reason } => format!("disabled({reason:?})"),
        ActionStatus::Deferred { reason } => format!("deferred({reason:?})"),
        ActionStatus::Hidden { reason } => format!("hidden({reason:?})"),
    }
}

#[cfg(feature = "windowed")]
pub(crate) fn target_status_label(status: &TargetStatus) -> String {
    match status {
        TargetStatus::Enabled => "enabled".to_string(),
        TargetStatus::Disabled { reason } => format!("disabled({reason:?})"),
        TargetStatus::Deferred { reason } => format!("deferred({reason:?})"),
        TargetStatus::Hidden { reason } => format!("hidden({reason:?})"),
    }
}

#[cfg(feature = "windowed")]
pub(crate) fn action_button_label(base: &str, status: &ActionStatus) -> String {
    match status {
        ActionStatus::Enabled => base.to_string(),
        _ => format!("{base} · {}", action_status_label(status)),
    }
}

#[cfg(feature = "windowed")]
pub(crate) fn action_tooltip(
    base: &str,
    affordance: &ActionAffordance<'_>,
    preview: Option<&str>,
) -> String {
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
pub fn telegraph_chip_text(skill_label: &str) -> String {
    format!("Telegraph: {skill_label} · impact pending")
}

#[cfg(feature = "windowed")]
pub fn telegraph_chip_tooltip(status: &CueBarrierStatus) -> String {
    let animation_node = status.animation_node.as_deref().unwrap_or("unbound");
    let animation_frame = status
        .animation_frame
        .map(|frame| frame.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    format!(
        "cast={:?}\nbeat={}\ncue={}\nnode={}\nframe={}",
        status.cast_id, status.beat_id, status.cue_id, animation_node, animation_frame
    )
}

#[cfg(feature = "windowed")]
pub fn cue_barrier_chip(
    status: Option<&CueBarrierStatus>,
    skill_book: Option<&SkillBook>,
) -> Option<TelegraphChip> {
    let status = status?;
    if !status.awaiting_release {
        return None;
    }

    let skill_label = skill_name(skill_book, &status.skill_id);

    Some(TelegraphChip {
        label: telegraph_chip_text(&skill_label),
        tooltip: telegraph_chip_tooltip(status),
    })
}

#[cfg(feature = "windowed")]
pub fn baby_burner_flash_text(display: &BabyBurnerFlashDisplay) -> String {
    format!(
        "Detonate flash · {} target(s) · {}/{}f",
        display.targets.len(),
        display.remaining_frames,
        display.total_frames
    )
}

#[cfg(feature = "windowed")]
pub fn baby_burner_flash_tooltip(display: &BabyBurnerFlashDisplay) -> String {
    let targets = display
        .targets
        .iter()
        .map(|target| format!("{:?}", target))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "source={:?}\ncast={:?}\nsignal={}/{}\ntargets=[{}]\nframes={}/{}",
        display.source,
        display.cast_id,
        display.signal_owner,
        display.signal_name,
        targets,
        display.remaining_frames,
        display.total_frames,
    )
}

#[cfg(feature = "windowed")]
pub fn baby_burner_flash_chip(display: Option<&BabyBurnerFlashDisplay>) -> Option<TelegraphChip> {
    let display = display?;

    Some(TelegraphChip {
        label: baby_burner_flash_text(display),
        tooltip: baby_burner_flash_tooltip(display),
    })
}

#[cfg(feature = "windowed")]
pub fn attr_color(a: Attribute) -> egui::Color32 {
    match a {
        Attribute::Vaccine => egui::Color32::from_rgb(80, 140, 220),
        Attribute::Data => egui::Color32::from_rgb(220, 200, 60),
        Attribute::Virus => egui::Color32::from_rgb(200, 60, 180),
        Attribute::Free => egui::Color32::from_gray(160),
    }
}

#[cfg(feature = "windowed")]
pub(crate) fn target_button_label(
    unit_name: &str,
    affordance: Option<&TargetAffordance>,
) -> String {
    match affordance {
        Some(affordance) => format!("{unit_name} · {}", target_status_label(&affordance.status)),
        None => unit_name.to_string(),
    }
}

#[cfg(feature = "windowed")]
pub fn query_pending_action_affordance<'a>(
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
