use bevy::prelude::*;
use bevyrogue::combat::rng::{
    CombatEntropy, CombatRng, CombatRngSeed, UnitRng, combat_entropy_plugin_from_seed,
    seed_unit_rngs,
};
use bevyrogue::combat::types::{Attribute, DamageTag, EvoStage, UnitId};
use bevyrogue::combat::unit::Unit;
use rand_core::Rng;

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
        false, false, true, true, true, false, true, true, true, true, true, false,
    ];

    assert_eq!(first, expected, "seeded RNG sequence drifted");
    assert_eq!(replay, expected, "same seed must replay the same sequence");
}

#[test]
fn seeded_combat_rng_forks_replayable_entity_streams() {
    let mut first_root = CombatRng::from_seed(0xBEEF);
    let mut replay_root = CombatRng::from_seed(0xBEEF);

    let mut first_streams: Vec<_> = (0..4).map(|_| first_root.fork_rng()).collect();
    let mut replay_streams: Vec<_> = (0..4).map(|_| replay_root.fork_rng()).collect();
    let first_values: Vec<_> = first_streams.iter_mut().map(|rng| rng.next_u64()).collect();
    let replay_values: Vec<_> = replay_streams
        .iter_mut()
        .map(|rng| rng.next_u64())
        .collect();

    assert_eq!(
        first_values, replay_values,
        "forked per-entity streams must replay from the same root seed"
    );
    assert!(
        first_values.windows(2).any(|pair| pair[0] != pair[1]),
        "forked streams should not all collapse to the same first draw"
    );
}

#[test]
fn unit_rng_streams_are_seeded_from_bevy_rand_global_entropy() {
    let mut app = App::new();
    app.add_plugins(combat_entropy_plugin_from_seed(0xA11CE))
        .add_systems(Update, seed_unit_rngs);

    app.world_mut().spawn(test_unit(UnitId(1), "Agumon"));
    app.world_mut().spawn(test_unit(UnitId(2), "Gabumon"));

    app.update();
    app.update();

    let mut seeded = app.world_mut().query_filtered::<Entity, (
        With<Unit>,
        With<UnitRng>,
        With<CombatRngSeed>,
        With<CombatEntropy>,
    )>();
    assert_eq!(seeded.iter(app.world()).count(), 2);
}

fn test_unit(id: UnitId, name: &str) -> Unit {
    Unit {
        id,
        name: name.to_string(),
        hp_max: 100,
        hp_current: 100,
        attribute: Attribute::Vaccine,
        resists: Vec::<DamageTag>::new(),
        evo_stage: EvoStage::Child,
    }
}
