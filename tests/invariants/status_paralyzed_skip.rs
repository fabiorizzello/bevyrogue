/// Property-based invariant for §H.1 Paralyzed semantics (S04/T01).
///
/// Replaces the previous fixed-duration scenario (`paralyzed_enemy_skips_all_100_turns`)
/// with a proptest that varies paralysis duration and RNG seed: a paralyzed unit
/// must emit exactly one `OnActionFailed{reason:"paralyzed"}` per turn and zero
/// `ActionIntent::Skill` across the entire window, regardless of randomness.
///
/// Asserting the behavior (intent blocked + event emitted) rather than a state
/// latch keeps the test alive across kernel refactors.
use bevy::prelude::*;
use bevyrogue::combat::{
    StatusBag, StatusEffectKind,
    av::ActionValueUpdated,
    events::{CombatEvent, CombatEventKind},
    log::ActionLog,
    rng::CombatRng,
    state::CombatState,
    team::Team,
    toughness::Toughness,
    turn_order::{TurnAdvanced, TurnOrder},
    turn_system::{ActionIntent, advance_turn_system},
    types::{Attribute, EvoStage, UnitId},
    unit::Unit,
};
use proptest::prelude::*;

fn setup_app(duration: u32, seed: u64) -> (App, Entity, Entity) {
    let mut app = App::new();
    app.init_resource::<CombatState>()
        .init_resource::<TurnOrder>()
        .init_resource::<ActionLog>()
        .init_resource::<Time>()
        .insert_resource(CombatRng::from_seed(seed))
        .add_message::<TurnAdvanced>()
        .add_message::<ActionIntent>()
        .add_message::<CombatEvent>()
        .add_message::<ActionValueUpdated>()
        .add_systems(Update, advance_turn_system);

    let ally = app
        .world_mut()
        .spawn((
            Unit {
                id: UnitId(1),
                name: "Ally".into(),
                hp_max: 500,
                hp_current: 500,
                attribute: Attribute::Vaccine,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            Team::Ally,
            Toughness::new(100, vec![]),
            StatusBag::default(),
        ))
        .id();

    let enemy = app
        .world_mut()
        .spawn((
            Unit {
                id: UnitId(2),
                name: "Enemy".into(),
                hp_max: 500,
                hp_current: 500,
                attribute: Attribute::Vaccine,
                resists: vec![],
                evo_stage: EvoStage::Adult,
            },
            Team::Enemy,
            Toughness::new(100, vec![]),
            {
                let mut bag = StatusBag::default();
                bag.apply(StatusEffectKind::Paralyzed, duration);
                bag
            },
        ))
        .id();

    (app, ally, enemy)
}

proptest! {
    /// For any paralysis duration `T > 0` and any RNG seed, a paralyzed unit
    /// produces exactly `T` `OnActionFailed{reason:"paralyzed"}` events and
    /// zero `ActionIntent::Skill` across `T` consecutive turns.
    #[test]
    fn paralyzed_unit_skips_every_turn_until_expiry(
        duration in 1u32..=50,
        seed in proptest::num::u64::ANY,
    ) {
        let (mut app, _ally, enemy) = setup_app(duration, seed);
        let enemy_id = app.world().get::<Unit>(enemy).unwrap().id;

        let mut event_cursor = app
            .world()
            .resource::<Messages<CombatEvent>>()
            .get_cursor_current();
        let mut intent_cursor = app
            .world()
            .resource::<Messages<ActionIntent>>()
            .get_cursor_current();

        let mut skip_count = 0usize;
        let mut enemy_intent_count = 0usize;

        for _ in 0..duration {
            app.world_mut().write_message(TurnAdvanced::of(enemy_id));
            app.update();

            let frame_events: Vec<CombatEvent> = {
                let msgs = app.world().resource::<Messages<CombatEvent>>();
                event_cursor.read(msgs).cloned().collect()
            };
            for ev in &frame_events {
                if matches!(&ev.kind, CombatEventKind::OnActionFailed { reason } if reason == "paralyzed")
                {
                    skip_count += 1;
                }
            }

            let frame_intents: Vec<ActionIntent> = {
                let msgs = app.world().resource::<Messages<ActionIntent>>();
                intent_cursor.read(msgs).cloned().collect()
            };
            for intent in &frame_intents {
                if let ActionIntent::Skill { attacker, .. } = intent {
                    if *attacker == enemy_id {
                        enemy_intent_count += 1;
                    }
                }
            }
        }

        prop_assert_eq!(skip_count, duration as usize);
        prop_assert_eq!(enemy_intent_count, 0);
    }
}
