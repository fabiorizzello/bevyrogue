use bevyrogue::combat::mechanics::buffs::{DrBag, sum_dr};

#[test]
fn sum_dr_none_is_zero() {
    assert_eq!(sum_dr(None), 0.0);
}

#[test]
fn sum_dr_sums_unclamped() {
    let mut bag = DrBag::default();
    bag.apply(0.3, 2);
    bag.apply(0.5, 1);
    bag.apply(0.4, 3);
    assert!((sum_dr(Some(&bag)) - 1.2).abs() < f32::EPSILON);
}

#[test]
fn tick_all_drops_expired() {
    let mut bag = DrBag::default();
    bag.apply(0.2, 1);
    bag.apply(0.3, 2);
    let dropped = bag.tick_all();
    assert_eq!(dropped, 1);
    assert_eq!(bag.instances().len(), 1);
    assert_eq!(bag.instances()[0].duration, 1);
}
