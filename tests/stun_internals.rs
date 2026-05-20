use bevyrogue::combat::mechanics::stun::Stunned;

#[test]
fn tick_keeps_component_while_turns_remain() {
    let mut stunned = Stunned { turns_left: 2 };

    assert!(!stunned.tick());
    assert_eq!(stunned.turns_left, 1);
}

#[test]
fn tick_down_to_zero() {
    let mut stunned = Stunned { turns_left: 1 };

    assert!(stunned.tick());
    assert_eq!(stunned.turns_left, 0);
}

#[test]
fn tick_no_op_at_zero() {
    let mut stunned = Stunned { turns_left: 0 };

    assert!(stunned.tick());
    assert_eq!(stunned.turns_left, 0);
}
