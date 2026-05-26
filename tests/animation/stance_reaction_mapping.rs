//! Headless contract for the pure event-to-stance-reaction mapping.
//!
//! Links only against the lib crate — no windowed/bevy-render dependency.

use bevyrogue::animation::anim_graph::NodeId;
use bevyrogue::animation::reaction::{
    StanceReaction, resolve_stance_reaction, stance_reaction_for,
};
use bevyrogue::combat::observability::events::CombatEventKind;

#[test]
fn hit_maps_to_hurt_node() {
    let kind = CombatEventKind::OnHitTaken { amount: 12 };
    assert_eq!(stance_reaction_for(&kind), Some(StanceReaction::Hurt));
    assert_eq!(
        StanceReaction::Hurt.stance_node(),
        NodeId("hurt".to_string())
    );
}

#[test]
fn death_maps_to_death_node() {
    let kind = CombatEventKind::UnitDied {
        status_remaining: Vec::new(),
        heated_remaining: 0,
    };
    assert_eq!(stance_reaction_for(&kind), Some(StanceReaction::Death));
    assert_eq!(
        StanceReaction::Death.stance_node(),
        NodeId("death".to_string())
    );
}

#[test]
fn death_takes_precedence_over_hurt_in_batch() {
    let kinds = [
        CombatEventKind::OnHitTaken { amount: 9 },
        CombatEventKind::UnitDied {
            status_remaining: Vec::new(),
            heated_remaining: 0,
        },
    ];
    assert_eq!(resolve_stance_reaction(&kinds), Some(StanceReaction::Death));
}

#[test]
fn non_reaction_kinds_and_empty_batch_map_to_none() {
    let non_reaction = CombatEventKind::OnActionResolved;
    assert_eq!(stance_reaction_for(&non_reaction), None);

    let empty: [CombatEventKind; 0] = [];
    assert_eq!(resolve_stance_reaction(&empty), None);
}
