/// Direct unit-level test for StatusBag::cleanse_debuffs (no Bevy events).
/// Verifies §H.1 cleanse policy: Debuff-classified kinds removed; Blessed survives.
use bevyrogue::combat::{StatusBag, StatusEffectKind};
use std::collections::HashSet;

#[test]
fn cleanse_removes_all_debuffs_and_keeps_blessed() {
    let mut bag = StatusBag::default();
    bag.apply(StatusEffectKind::Heated, 2);
    bag.apply(StatusEffectKind::Chilled, 3);
    bag.apply(StatusEffectKind::Paralyzed, 1);
    bag.apply(StatusEffectKind::Slowed, 4);
    bag.apply(StatusEffectKind::Blessed, 5);

    let removed = bag.cleanse_debuffs();

    let removed_set: HashSet<_> = removed.iter().collect();
    assert_eq!(
        removed_set.len(),
        4,
        "cleanse must remove exactly 4 debuff kinds"
    );
    assert!(removed_set.contains(&StatusEffectKind::Heated));
    assert!(removed_set.contains(&StatusEffectKind::Chilled));
    assert!(removed_set.contains(&StatusEffectKind::Paralyzed));
    assert!(removed_set.contains(&StatusEffectKind::Slowed));

    assert!(!bag.has(&StatusEffectKind::Heated));
    assert!(!bag.has(&StatusEffectKind::Chilled));
    assert!(!bag.has(&StatusEffectKind::Paralyzed));
    assert!(!bag.has(&StatusEffectKind::Slowed));

    assert!(
        bag.has(&StatusEffectKind::Blessed),
        "Blessed (Buff) must survive cleanse"
    );
    assert_eq!(bag.get_dur(&StatusEffectKind::Blessed), Some(5));
}
