use bevyrogue::combat::action_query::{
    ActionAffordance, ActionQueryKind, ActionStatus, CombatQuerySnapshot, TargetAffordance,
    TargetStatus, query_action_affordance,
};
use bevyrogue::combat::kit::UnitSkills;
use bevyrogue::combat::team::Team;
use bevyrogue::combat::types::UnitId;
use bevyrogue::combat::unit::Unit;
use bevyrogue::data::skills_ron::SkillBook;

/// Interactive action selection: presents inquire menus when it is an ally's
/// turn and stdin is a terminal.  In non-interactive mode it emits a default
/// BasicAttack so the CI verification loop can still run.
///
/// State machine driven by `PlayerActed`:
///   false → prompt (or auto-act) then set true
///   true  → action was resolved; advance turn queue and reset to false
#[derive(Clone)]
pub struct ActionMenuEntry<'a> {
    pub kind: ActionQueryKind<'a>,
    pub label: String,
    pub affordance: ActionAffordance<'a>,
}

pub fn action_kind_label(kind: ActionQueryKind<'_>) -> String {
    match kind {
        ActionQueryKind::Basic => "Basic Attack".to_string(),
        ActionQueryKind::Skill(skill_id) => format!("Skill: {}", skill_id.0),
        ActionQueryKind::Ultimate => "Ultimate".to_string(),
    }
}

pub fn action_status_label(status: &ActionStatus) -> String {
    match status {
        ActionStatus::Enabled => "enabled".to_string(),
        ActionStatus::Disabled { reason } => format!("disabled({reason:?})"),
        ActionStatus::Deferred { reason } => format!("deferred({reason:?})"),
        ActionStatus::Hidden { reason } => format!("hidden({reason:?})"),
    }
}

pub fn target_status_label(status: &TargetStatus) -> String {
    match status {
        TargetStatus::Enabled => "enabled".to_string(),
        TargetStatus::Disabled { reason } => format!("disabled({reason:?})"),
        TargetStatus::Deferred { reason } => format!("deferred({reason:?})"),
        TargetStatus::Hidden { reason } => format!("hidden({reason:?})"),
    }
}

pub fn action_entry_label(entry: &ActionMenuEntry<'_>) -> String {
    format!(
        "{} [{}]",
        action_kind_label(entry.kind),
        action_status_label(&entry.affordance.action),
    )
}

pub fn target_entry_label(unit: &Unit, team: &Team, affordance: &TargetAffordance) -> String {
    let side = match team {
        Team::Ally => "ALLY",
        Team::Enemy => "ENEM",
    };

    format!(
        "[{side}] {} HP:{}/{} [{}]",
        unit.name,
        unit.hp_current,
        unit.hp_max,
        target_status_label(&affordance.status)
    )
}

pub fn build_action_entries<'a>(
    snapshot: &'a CombatQuerySnapshot,
    skill_book: &'a SkillBook,
    actor_id: UnitId,
    actor_skills: Option<&'a UnitSkills>,
) -> Vec<ActionMenuEntry<'a>> {
    let mut entries = Vec::new();

    entries.push(ActionMenuEntry {
        kind: ActionQueryKind::Basic,
        label: String::new(),
        affordance: query_action_affordance(snapshot, skill_book, actor_id, ActionQueryKind::Basic),
    });

    if let Some(skills) = actor_skills {
        for skill_id in &skills.skills {
            let kind = ActionQueryKind::Skill(skill_id);
            entries.push(ActionMenuEntry {
                kind,
                label: String::new(),
                affordance: query_action_affordance(snapshot, skill_book, actor_id, kind),
            });
        }
    }

    entries.push(ActionMenuEntry {
        kind: ActionQueryKind::Ultimate,
        label: String::new(),
        affordance: query_action_affordance(
            snapshot,
            skill_book,
            actor_id,
            ActionQueryKind::Ultimate,
        ),
    });

    for entry in &mut entries {
        entry.label = action_entry_label(entry);
    }

    entries
}

pub fn print_action_entries(entries: &[ActionMenuEntry<'_>]) {
    println!("\n  Action affordances:");
    for entry in entries {
        println!("    - {}", entry.label);
    }
}

pub fn print_target_entries(target_entries: &[(UnitId, String)]) {
    println!("\n  Target affordances:");
    for (_, label) in target_entries {
        println!("    - {}", label);
    }
}
