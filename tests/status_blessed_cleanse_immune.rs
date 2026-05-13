/// Regression guard: Blessed is immune to cleanse (§H.1).
/// Blessed is classified BuffKind::Buff — cleanse_debuffs() must not remove it.
use bevyrogue::combat::{StatusBag, StatusEffectKind};

#[test]
fn blessed_survives_cleanse_when_alone() {
    let mut bag = StatusBag::default();
    bag.apply(StatusEffectKind::Blessed, 3);

    let removed = bag.cleanse_debuffs();

    assert!(removed.is_empty(), "cleanse must remove nothing when only Blessed is present");
    assert!(bag.has(&StatusEffectKind::Blessed), "Blessed must still be present after cleanse");
    assert_eq!(bag.get_dur(&StatusEffectKind::Blessed), Some(3), "duration must be unchanged");
}

#[test]
fn blessed_survives_cleanse_alongside_debuffs() {
    let mut bag = StatusBag::default();
    bag.apply(StatusEffectKind::Heated, 2);
    bag.apply(StatusEffectKind::Paralyzed, 1);
    bag.apply(StatusEffectKind::Blessed, 5);

    let removed = bag.cleanse_debuffs();

    assert_eq!(removed.len(), 2, "only debuffs removed");
    assert!(!bag.has(&StatusEffectKind::Heated));
    assert!(!bag.has(&StatusEffectKind::Paralyzed));
    assert!(bag.has(&StatusEffectKind::Blessed), "Blessed must survive cleanse of debuffs");
    assert_eq!(bag.get_dur(&StatusEffectKind::Blessed), Some(5));
}
