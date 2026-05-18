use bevyrogue::combat::av::{ActionValue, MAX_AV};
use bevyrogue::combat::resistance::{apply_advance, apply_delay};
use bevyrogue::combat::resolution::{TargetEntry, TargetableSnapshot, resolve_targets};
use bevyrogue::combat::team::Team;
use bevyrogue::combat::types::UnitId;
use bevyrogue::data::skills_ron::TargetShape;

/// Standalone scenario: advance-delay-cap
///
/// Demonstrates AdvanceTurn / DelayTurn cap (≤50%) and AV clamp [0, 2*MAX_AV].
/// Prints a human-readable AV gauge step-by-step and emits JSONL to stdout.
/// Does not start Bevy — exits after printing.
pub fn run_advance_delay_cap_scenario() {
    println!("=== scenario: advance-delay-cap ===");
    println!("MAX_AV = {MAX_AV}");

    struct MockUnit {
        name: &'static str,
        av: ActionValue,
    }

    let mut units = [
        MockUnit {
            name: "Agumon",
            av: ActionValue(0),
        },
        MockUnit {
            name: "Gabumon",
            av: ActionValue(MAX_AV / 2),
        },
    ];

    #[derive(serde::Serialize)]
    struct JsonlEntry {
        kind: &'static str,
        target: &'static str,
        amount_pct_requested: u32,
        amount_pct_capped: u32,
        av_pre: i32,
        av_delta: i32,
        av_post: i32,
    }

    let steps: &[(&str, usize, u32)] = &[
        ("AdvanceTurn", 0, 50),
        ("AdvanceTurn", 0, 50),
        ("DelayTurn", 1, 80),
        ("DelayTurn", 1, 50),
    ];

    for (kind, unit_idx, amount_pct) in steps {
        let unit = &mut units[*unit_idx];
        let av_pre = unit.av.0;
        let capped = (*amount_pct).min(50);

        let delta = if *kind == "AdvanceTurn" {
            apply_advance(&mut unit.av, *amount_pct)
        } else {
            apply_delay(&mut unit.av, *amount_pct, None)
        };

        let av_post = unit.av.0;
        let bar_width = 40usize;
        let fill = ((av_post.max(0) as f64 / (2.0 * MAX_AV as f64)) * bar_width as f64) as usize;
        let bar: String =
            "#".repeat(fill.min(bar_width)) + &".".repeat(bar_width - fill.min(bar_width));

        println!(
            "[{kind}] {name} pct={amount_pct}(cap={capped}) AV {av_pre:>6} → {av_post:>6} (Δ{delta:+}) [{bar}]",
            name = unit.name,
        );

        let entry = JsonlEntry {
            kind,
            target: unit.name,
            amount_pct_requested: *amount_pct,
            amount_pct_capped: capped,
            av_pre,
            av_delta: delta,
            av_post,
        };
        println!("{}", serde_json::to_string(&entry).unwrap());
    }

    println!("=== scenario complete ===");
}

/// Standalone scenario: aoe-blast
///
/// Builds a deterministic 3-enemy encounter, casts one Blast skill (slot-1 primary)
/// then one AllEnemies skill. Prints resolved target list + per-target damage,
/// final HP per enemy, and emits one JSONL line per OnDamageDealt event.
pub fn run_aoe_blast_scenario() {
    println!("=== scenario: aoe-blast ===");

    #[derive(Clone)]
    struct MockUnit {
        id: UnitId,
        name: &'static str,
        slot_index: u8,
        hp: i32,
    }

    #[derive(serde::Serialize)]
    struct JsonlDamageEvent {
        event: &'static str,
        source_id: u32,
        target_id: u32,
        target_slot: u8,
        amount: i32,
        skill_id: &'static str,
    }

    let attacker_id = UnitId(0);
    let mut enemies = vec![
        MockUnit {
            id: UnitId(1),
            name: "GobA",
            slot_index: 0,
            hp: 60,
        },
        MockUnit {
            id: UnitId(2),
            name: "GobB",
            slot_index: 1,
            hp: 60,
        },
        MockUnit {
            id: UnitId(3),
            name: "GobC",
            slot_index: 2,
            hp: 60,
        },
    ];

    let build_snapshot = |units: &[MockUnit]| TargetableSnapshot {
        entries: units
            .iter()
            .map(|u| TargetEntry {
                id: u.id,
                team: Team::Enemy,
                slot_index: u.slot_index,
                alive: u.hp > 0,
                hp_per_mille: 1000, // mock: full HP for shape-only scenarios
            })
            .collect(),
    };

    // --- Cast 1: Blast on slot-1 primary (GobB) ---
    println!("\n--- Cast 1: blast_demo (Blast, primary=GobB slot-1) ---");
    let primary_blast = UnitId(2);
    let snapshot1 = build_snapshot(&enemies);
    let blast_targets = resolve_targets(&TargetShape::Blast, primary_blast, &snapshot1);

    let blast_damage_per_target = 20i32;
    let blast_toughness_per_target = 10i32;
    println!(
        "Resolved targets (slot_index asc): {:?}",
        blast_targets
            .iter()
            .filter_map(|id| enemies.iter().find(|u| u.id == *id))
            .map(|u| format!("{}(slot{})", u.name, u.slot_index))
            .collect::<Vec<_>>()
    );

    for &target_id in &blast_targets {
        let enemy = enemies.iter_mut().find(|u| u.id == target_id).unwrap();
        enemy.hp -= blast_damage_per_target;
        println!(
            "  {} (slot {}) takes {} dmg → HP {}",
            enemy.name, enemy.slot_index, blast_damage_per_target, enemy.hp
        );
        let entry = JsonlDamageEvent {
            event: "OnDamageDealt",
            source_id: attacker_id.0,
            target_id: enemy.id.0,
            target_slot: enemy.slot_index,
            amount: blast_damage_per_target,
            skill_id: "blast_demo",
        };
        println!("{}", serde_json::to_string(&entry).unwrap());
        let _ = blast_toughness_per_target;
    }

    // --- Cast 2: AllEnemies (aoe_demo) ---
    println!("\n--- Cast 2: aoe_demo (AllEnemies) ---");
    let primary_all = UnitId(1);
    let snapshot2 = build_snapshot(&enemies);
    let all_targets = resolve_targets(&TargetShape::AllEnemies, primary_all, &snapshot2);

    let all_damage_per_target = 15i32;
    let all_toughness_per_target = 8i32;
    println!(
        "Resolved targets (slot_index asc): {:?}",
        all_targets
            .iter()
            .filter_map(|id| enemies.iter().find(|u| u.id == *id))
            .map(|u| format!("{}(slot{})", u.name, u.slot_index))
            .collect::<Vec<_>>()
    );

    for &target_id in &all_targets {
        let enemy = enemies.iter_mut().find(|u| u.id == target_id).unwrap();
        enemy.hp -= all_damage_per_target;
        println!(
            "  {} (slot {}) takes {} dmg → HP {}",
            enemy.name, enemy.slot_index, all_damage_per_target, enemy.hp
        );
        let entry = JsonlDamageEvent {
            event: "OnDamageDealt",
            source_id: attacker_id.0,
            target_id: enemy.id.0,
            target_slot: enemy.slot_index,
            amount: all_damage_per_target,
            skill_id: "aoe_demo",
        };
        println!("{}", serde_json::to_string(&entry).unwrap());
        let _ = all_toughness_per_target;
    }

    // --- Final HP gauge ---
    println!("\n--- Final HP ---");
    for enemy in &enemies {
        println!(
            "  {} (slot {}): HP {}",
            enemy.name, enemy.slot_index, enemy.hp
        );
    }
    println!("=== scenario complete ===");
}
