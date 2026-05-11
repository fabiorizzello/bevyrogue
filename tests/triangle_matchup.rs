use bevyrogue::combat::damage::triangle_modifiers;
use bevyrogue::combat::types::Attribute;

// Parametric test enumerating all 16 (attacker_attr, defender_attr) pairs and
// asserting the expected (dmg_modifier, tough_modifier, status_acc_modifier) triple.
//
// Cycle: Vaccine > Virus > Data > Vaccine. Free neutral to all.
// Attacker wins → dmg=1.11, tough=1.0, status_acc=1.1
// Defender wins → dmg=0.87, tough=1.0, status_acc=0.9
// Neutral       → dmg=1.0,  tough=1.0, status_acc=1.0

#[test]
fn triangle_all_16_pairs() {
    use Attribute::{Data, Free, Vaccine, Virus};

    // (attacker, defender, expected_dmg, expected_tough, expected_status_acc)
    let cases: &[(Attribute, Attribute, f32, f32, f32)] = &[
        // Same-vs-same: neutral
        (Vaccine, Vaccine, 1.0, 1.0, 1.0),
        (Virus, Virus, 1.0, 1.0, 1.0),
        (Data, Data, 1.0, 1.0, 1.0),
        (Free, Free, 1.0, 1.0, 1.0),
        // Attacker wins the cycle
        (Vaccine, Virus, 1.11, 1.0, 1.1),
        (Virus, Data, 1.11, 1.0, 1.1),
        (Data, Vaccine, 1.11, 1.0, 1.1),
        // Defender wins the cycle
        (Virus, Vaccine, 0.87, 1.0, 0.9),
        (Data, Virus, 0.87, 1.0, 0.9),
        (Vaccine, Data, 0.87, 1.0, 0.9),
        // Free attacker: always neutral
        (Free, Vaccine, 1.0, 1.0, 1.0),
        (Free, Virus, 1.0, 1.0, 1.0),
        (Free, Data, 1.0, 1.0, 1.0),
        // Free defender: always neutral
        (Vaccine, Free, 1.0, 1.0, 1.0),
        (Virus, Free, 1.0, 1.0, 1.0),
        (Data, Free, 1.0, 1.0, 1.0),
    ];

    for &(att, def, exp_dmg, exp_tough, exp_status) in cases {
        let mods = triangle_modifiers(att, def);
        assert!(
            (mods.dmg_modifier - exp_dmg).abs() < 1e-5,
            "{att:?} vs {def:?}: dmg_modifier expected {exp_dmg}, got {}",
            mods.dmg_modifier
        );
        assert!(
            (mods.tough_modifier - exp_tough).abs() < 1e-5,
            "{att:?} vs {def:?}: tough_modifier expected {exp_tough}, got {}",
            mods.tough_modifier
        );
        assert!(
            (mods.status_acc_modifier - exp_status).abs() < 1e-5,
            "{att:?} vs {def:?}: status_acc_modifier expected {exp_status}, got {}",
            mods.status_acc_modifier
        );
    }
}
