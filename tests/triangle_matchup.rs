//! Parametric coverage of the 16-cell attribute-triangle modifier table.
//!
//! Cycle: Vaccine > Virus > Data > Vaccine. Free is neutral to all.
//!   * attacker wins → dmg=1.11, tough=1.0, status_acc=1.1
//!   * defender wins → dmg=0.87, tough=1.0, status_acc=0.9
//!   * neutral       → dmg=1.0,  tough=1.0, status_acc=1.0
//!
//! Each case is a named `#[rstest]` test (one per cell).

use bevyrogue::combat::damage::triangle_modifiers;
use bevyrogue::combat::types::Attribute::{self, Data, Free, Vaccine, Virus};
use rstest::rstest;

#[rstest]
// Same-vs-same: neutral
#[case::vac_vac(Vaccine, Vaccine, 1.0_f32, 1.0_f32, 1.0_f32)]
#[case::vir_vir(Virus,   Virus,   1.0_f32, 1.0_f32, 1.0_f32)]
#[case::dat_dat(Data,    Data,    1.0_f32, 1.0_f32, 1.0_f32)]
#[case::free_free(Free,  Free,    1.0_f32, 1.0_f32, 1.0_f32)]
// Attacker wins the cycle
#[case::vac_beats_vir(Vaccine, Virus,   1.11_f32, 1.0_f32, 1.1_f32)]
#[case::vir_beats_dat(Virus,   Data,    1.11_f32, 1.0_f32, 1.1_f32)]
#[case::dat_beats_vac(Data,    Vaccine, 1.11_f32, 1.0_f32, 1.1_f32)]
// Defender wins the cycle
#[case::vir_loses_vac(Virus,   Vaccine, 0.87_f32, 1.0_f32, 0.9_f32)]
#[case::dat_loses_vir(Data,    Virus,   0.87_f32, 1.0_f32, 0.9_f32)]
#[case::vac_loses_dat(Vaccine, Data,    0.87_f32, 1.0_f32, 0.9_f32)]
// Free attacker: always neutral
#[case::free_vac(Free, Vaccine, 1.0_f32, 1.0_f32, 1.0_f32)]
#[case::free_vir(Free, Virus,   1.0_f32, 1.0_f32, 1.0_f32)]
#[case::free_dat(Free, Data,    1.0_f32, 1.0_f32, 1.0_f32)]
// Free defender: always neutral
#[case::vac_free(Vaccine, Free, 1.0_f32, 1.0_f32, 1.0_f32)]
#[case::vir_free(Virus,   Free, 1.0_f32, 1.0_f32, 1.0_f32)]
#[case::dat_free(Data,    Free, 1.0_f32, 1.0_f32, 1.0_f32)]
fn triangle_modifier_cell(
    #[case] attacker: Attribute,
    #[case] defender: Attribute,
    #[case] expected_dmg: f32,
    #[case] expected_tough: f32,
    #[case] expected_status_acc: f32,
) {
    let mods = triangle_modifiers(attacker, defender);
    assert!(
        (mods.dmg_modifier - expected_dmg).abs() < 1e-5,
        "{attacker:?} vs {defender:?}: dmg_modifier expected {expected_dmg}, got {}",
        mods.dmg_modifier
    );
    assert!(
        (mods.tough_modifier - expected_tough).abs() < 1e-5,
        "{attacker:?} vs {defender:?}: tough_modifier expected {expected_tough}, got {}",
        mods.tough_modifier
    );
    assert!(
        (mods.status_acc_modifier - expected_status_acc).abs() < 1e-5,
        "{attacker:?} vs {defender:?}: status_acc_modifier expected {expected_status_acc}, got {}",
        mods.status_acc_modifier
    );
}
