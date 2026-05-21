use bevyrogue::combat::resolution::resolve_targets;
use bevyrogue::combat::{team::Team, types::UnitId};
use bevyrogue::data::skills_ron::TargetShape;
use crate::common::resolution_helpers::snap;

// ── resolve_targets table-driven tests ──────────────────────────────────

#[test]
fn resolve_targets_single_returns_primary() {
    let s = snap(vec![
        (UnitId(1), Team::Ally, 0, true),
        (UnitId(2), Team::Enemy, 0, true),
    ]);
    assert_eq!(
        resolve_targets(&TargetShape::Single, UnitId(2), &s),
        vec![UnitId(2)]
    );
}

#[test]
fn resolve_targets_blast_edge_slot_zero_returns_only_0_and_1() {
    // primary at slot 0 → slot -1 absent → only slots 0 and 1
    let s = snap(vec![
        (UnitId(10), Team::Enemy, 0, true),
        (UnitId(11), Team::Enemy, 1, true),
        (UnitId(12), Team::Enemy, 2, true),
    ]);
    assert_eq!(
        resolve_targets(&TargetShape::Blast, UnitId(10), &s),
        vec![UnitId(10), UnitId(11)],
    );
}

#[test]
fn resolve_targets_blast_ko_adjacent_omitted() {
    // primary at slot 1, slot 0 KO'd → only [slot1, slot2]
    let s = snap(vec![
        (UnitId(10), Team::Enemy, 0, false),
        (UnitId(11), Team::Enemy, 1, true),
        (UnitId(12), Team::Enemy, 2, true),
    ]);
    assert_eq!(
        resolve_targets(&TargetShape::Blast, UnitId(11), &s),
        vec![UnitId(11), UnitId(12)],
    );
}

#[test]
fn resolve_targets_blast_all_three_alive_sorted_asc() {
    // Inserted out of order → sorted by slot_index
    let s = snap(vec![
        (UnitId(12), Team::Enemy, 2, true),
        (UnitId(10), Team::Enemy, 0, true),
        (UnitId(11), Team::Enemy, 1, true),
    ]);
    assert_eq!(
        resolve_targets(&TargetShape::Blast, UnitId(11), &s),
        vec![UnitId(10), UnitId(11), UnitId(12)],
    );
}

#[test]
fn resolve_targets_all_enemies_omits_dead() {
    let s = snap(vec![
        (UnitId(1), Team::Ally, 0, true),
        (UnitId(10), Team::Enemy, 0, true),
        (UnitId(11), Team::Enemy, 1, false),
        (UnitId(12), Team::Enemy, 2, true),
    ]);
    assert_eq!(
        resolve_targets(&TargetShape::AllEnemies, UnitId(10), &s),
        vec![UnitId(10), UnitId(12)],
    );
}

#[test]
fn resolve_targets_all_enemies_sorted_slot_asc() {
    let s = snap(vec![
        (UnitId(12), Team::Enemy, 2, true),
        (UnitId(10), Team::Enemy, 0, true),
        (UnitId(11), Team::Enemy, 1, true),
    ]);
    assert_eq!(
        resolve_targets(&TargetShape::AllEnemies, UnitId(12), &s),
        vec![UnitId(10), UnitId(11), UnitId(12)],
    );
}
