use bevyrogue::combat::rng::CombatRng;

fn roll_sequence(seed: u64) -> Vec<bool> {
    let mut rng = CombatRng::from_seed(seed);
    [0, 1, 25, 50, 75, 99, 100, 10, 90, 33, 66, 42]
        .into_iter()
        .map(|threshold| rng.roll_pct(threshold))
        .collect()
}

#[test]
fn seeded_combat_rng_replays_same_roll_sequence() {
    let first = roll_sequence(0xC0FFEE);
    let replay = roll_sequence(0xC0FFEE);
    let expected = vec![
        false, false, false, false, true, true, true, false, true, false, true, true,
    ];

    assert_eq!(first, expected, "seeded RNG sequence drifted");
    assert_eq!(replay, expected, "same seed must replay the same sequence");
}
