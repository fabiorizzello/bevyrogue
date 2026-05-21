#![cfg(feature = "windowed")]

use bevy::prelude::*;
use bevyrogue::combat::{
    floating::{FLOATING_LIFETIME_SECS, FloatingDamage},
    toughness::DamageKind,
    types::UnitId,
    unit::Unit,
};
use bevyrogue::ui::combat_panel::{
    FloatingDamageView, HpBarView, compute_floating_damage_view, compute_hp_bar_view,
};

fn hp_bar_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<HpBarView>()
        .add_systems(Update, compute_hp_bar_view);
    app
}

fn fd_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<FloatingDamageView>()
        .add_systems(Update, compute_floating_damage_view);
    app
}

fn spawn_unit(app: &mut App, id: UnitId, hp_cur: i32, hp_max: i32) {
    use bevyrogue::combat::types::{Attribute, EvoStage};
    app.world_mut().spawn(Unit {
        id,
        name: format!("unit_{}", id.0),
        hp_max,
        hp_current: hp_cur,
        attribute: Attribute::Vaccine,
        resists: vec![],
        evo_stage: EvoStage::Adult,
    });
}

#[test]
fn hp_bar_view_computes_pct_from_unit_without_mutating_unit() {
    let mut app = hp_bar_app();
    let a = UnitId(1);
    let b = UnitId(2);
    spawn_unit(&mut app, a, 75, 100);
    spawn_unit(&mut app, b, 0, 80);

    app.update();

    let view = app.world().resource::<HpBarView>().clone();
    let entry_a = view.bars.iter().find(|e| e.unit_id == a).expect("unit A missing");
    let entry_b = view.bars.iter().find(|e| e.unit_id == b).expect("unit B missing");

    assert_eq!(entry_a.cur, 75);
    assert_eq!(entry_a.max, 100);
    assert!((entry_a.pct - 0.75).abs() < 1e-5, "pct should be 0.75");

    assert_eq!(entry_b.cur, 0);
    assert_eq!(entry_b.pct, 0.0, "KO unit pct should be 0");

    // Unit components must remain unmodified.
    let hp_a: Vec<i32> = app
        .world_mut()
        .query::<&Unit>()
        .iter(app.world())
        .filter(|u| u.id == a)
        .map(|u| u.hp_current)
        .collect();
    assert_eq!(hp_a, vec![75]);
}

#[test]
fn hp_bar_view_pct_clamps_at_one_for_overheal() {
    let mut app = hp_bar_app();
    spawn_unit(&mut app, UnitId(5), 120, 100);
    app.update();

    let view = app.world().resource::<HpBarView>().clone();
    let entry = view.bars.first().expect("entry missing");
    assert!(entry.pct <= 1.0, "pct must not exceed 1.0 for overheal");
}

#[test]
fn hp_bar_view_zero_max_hp_yields_zero_pct() {
    let mut app = hp_bar_app();
    spawn_unit(&mut app, UnitId(6), 0, 0);
    app.update();

    let view = app.world().resource::<HpBarView>().clone();
    let entry = view.bars.first().expect("entry missing");
    assert_eq!(entry.pct, 0.0);
}

#[test]
fn floating_damage_view_text_and_anchor_unit_id_normal_hit() {
    let mut app = fd_app();
    let target = UnitId(10);
    // spawn_time = 0; Time starts at 0 so elapsed = 0 → still alive, alpha ≈ 1.
    app.world_mut().spawn(FloatingDamage {
        target,
        amount: 42,
        kind: DamageKind::Normal,
        spawn_time: 0.0,
    });

    app.update();

    let view = app.world().resource::<FloatingDamageView>().clone();
    assert_eq!(view.entries.len(), 1);
    let entry = &view.entries[0];
    assert_eq!(entry.unit_id, target);
    assert_eq!(entry.text, "42");
    assert!(entry.alpha > 0.9, "alpha should be near 1 at spawn");
}

#[test]
fn floating_damage_view_text_includes_damage_kind_prefix() {
    let mut app = fd_app();
    app.world_mut().spawn(FloatingDamage {
        target: UnitId(11),
        amount: 30,
        kind: DamageKind::Weak,
        spawn_time: 0.0,
    });
    app.world_mut().spawn(FloatingDamage {
        target: UnitId(12),
        amount: 15,
        kind: DamageKind::Resist,
        spawn_time: 0.0,
    });
    app.world_mut().spawn(FloatingDamage {
        target: UnitId(13),
        amount: 50,
        kind: DamageKind::Break,
        spawn_time: 0.0,
    });

    app.update();

    let view = app.world().resource::<FloatingDamageView>().clone();
    assert_eq!(view.entries.len(), 3);
    let weak = view.entries.iter().find(|e| e.unit_id == UnitId(11)).unwrap();
    let resist = view.entries.iter().find(|e| e.unit_id == UnitId(12)).unwrap();
    let brk = view.entries.iter().find(|e| e.unit_id == UnitId(13)).unwrap();
    assert_eq!(weak.text, "WEAK 30");
    assert_eq!(resist.text, "RES 15");
    assert_eq!(brk.text, "BRK 50");
}

#[test]
fn floating_damage_view_excludes_expired_entries() {
    let mut app = fd_app();
    // spawn_time = -(FLOATING_LIFETIME_SECS + 1) simulates an old entry when elapsed = 0.
    app.world_mut().spawn(FloatingDamage {
        target: UnitId(20),
        amount: 99,
        kind: DamageKind::Normal,
        spawn_time: -(FLOATING_LIFETIME_SECS + 1.0),
    });

    app.update();

    let view = app.world().resource::<FloatingDamageView>().clone();
    assert!(
        view.entries.is_empty(),
        "expired FloatingDamage must not appear in the view"
    );
}
