//! Relocated from `src/combat/runtime/registry.rs` (R003 — no inline `mod tests` in src/).
//! Pure relocate: every symbol the tests touch is already `pub`.

use bevyrogue::combat::runtime::registry::{ExtPoint, ExtRegistries, Registry};

/// Minimal test axis: fn pointer returning u32.
struct NumExt;
impl ExtPoint for NumExt {
    type Fn = fn() -> u32;
}

#[test]
fn registry_hit() {
    let mut reg: Registry<NumExt> = Registry::new();
    reg.register("answer", || 42u32);
    let f = reg.get("answer").expect("registered id must resolve");
    assert_eq!(f(), 42);
}

#[test]
fn registry_miss() {
    let reg: Registry<NumExt> = Registry::new();
    assert!(reg.get("nonexistent").is_none());
}

#[test]
fn registry_overwrite() {
    let mut reg: Registry<NumExt> = Registry::new();
    reg.register("v", || 1u32);
    reg.register("v", || 2u32);
    assert_eq!(reg.get("v").unwrap()(), 2);
}

#[test]
fn ext_registries_default_empty() {
    let r = ExtRegistries::default();
    assert!(r.hooks.is_empty());
    assert!(r.selectors.is_empty());
    assert!(r.predicates.is_empty());
    assert!(r.formulas.is_empty());
    assert!(r.ticks.is_empty());
    assert!(r.ai_utilities.is_empty());
    assert!(r.cues.is_empty());
    assert!(r.pre_damage_reactions.is_empty());
    assert!(r.validation.is_empty());
}
