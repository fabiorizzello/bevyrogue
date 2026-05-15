use std::collections::HashMap;

use bevy::prelude::Resource;

use crate::combat::types::UnitId;
use super::skill_ctx::SkillCtx;
use super::timeline::{BeatEvent, CueCtx, SelectorCtx};

/// Marker trait for an extension axis.
///
/// Each axis defines the concrete function type stored in its `Registry`.
/// Implementing this for a unit struct takes ~3 lines; `ExtRegistries` then
/// holds one `Registry<E>` per axis (D031).
pub trait ExtPoint: 'static {
    /// Concrete callable stored per ID on this axis.
    ///
    /// Must be `Send + Sync + 'static` so `Registry<E>` satisfies `Resource`.
    /// Signature is axis-specific and refined in S02+; placeholders use `fn()`.
    type Fn: Send + Sync + 'static;
}

/// ID → function map for one extension axis.
///
/// Keys are `&'static str` (typically `"namespace/name"`). Lookup is O(1).
/// All entries are registered at startup; no runtime mutation after `App::finish()`.
pub struct Registry<E: ExtPoint> {
    entries: HashMap<&'static str, E::Fn>,
}

impl<E: ExtPoint> Registry<E> {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Register `f` under `id`. Overwrites any previous entry with the same ID.
    pub fn register(&mut self, id: &'static str, f: E::Fn) {
        self.entries.insert(id, f);
    }

    /// Look up a function by ID. Returns `None` if unregistered.
    pub fn get(&self, id: &str) -> Option<&E::Fn> {
        self.entries.get(id)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl<E: ExtPoint> Default for Registry<E> {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Seven extension axes (F3 / D031) ────────────────────────────────────────
// Each axis is a unit struct + `ExtPoint` impl. Concrete `Fn` signatures are
// placeholders (`fn()`) for S01 and refined per axis in S02+.

/// Lifecycle hook: `OnTurnStart`, `OnDamageDealt`, etc.
pub struct HookExt;
impl ExtPoint for HookExt {
    type Fn = for<'a> fn(&BeatEvent, &mut SkillCtx<'a>);
}

/// Target selector: `primary`, `all_enemies`, `lowest_hp_pct`, etc.
pub struct SelectorExt;
impl ExtPoint for SelectorExt {
    type Fn = for<'a> fn(&SelectorCtx<'a>) -> Vec<UnitId>;
}

/// Edge-gate predicate: `has_target_alive`, `skilltree_unlocked`, etc.
pub struct PredicateExt;
impl ExtPoint for PredicateExt {
    type Fn = for<'a> fn(&BeatEvent, &SkillCtx<'a>) -> bool;
}

/// Damage / heal formula: `atk_scaling`, `fixed_value`, etc.
/// Signature placeholder — S05 will refine to `for<'a> fn(&FormulaCtx<'a>) -> i32`.
pub struct FormulaExt;
impl ExtPoint for FormulaExt {
    type Fn = fn();
}

/// Per-turn status tick: `dot_tick`, `regen_tick`, etc.
/// Signature placeholder — S07 will refine.
pub struct TickExt;
impl ExtPoint for TickExt {
    type Fn = fn();
}

/// AI utility scorer for action selection.
/// Signature placeholder — S07 will refine.
pub struct AiUtilityExt;
impl ExtPoint for AiUtilityExt {
    type Fn = fn();
}

/// Presentation cue ID → animation handle for `Clock::Windowed` (I3 / D026).
pub struct CueExt;
impl ExtPoint for CueExt {
    type Fn = for<'a> fn(&CueCtx<'a>) -> &'static str;
}

/// Aggregated resource holding all seven extension registries (D031).
///
/// Inserted as a Bevy `Resource` at startup. Built-in fns are registered in S05;
/// blueprint-specific fns are registered by each `register(reg)` call in S08+.
#[derive(Resource, Default)]
pub struct ExtRegistries {
    pub hooks: Registry<HookExt>,
    pub selectors: Registry<SelectorExt>,
    pub predicates: Registry<PredicateExt>,
    pub formulas: Registry<FormulaExt>,
    pub ticks: Registry<TickExt>,
    pub ai_utilities: Registry<AiUtilityExt>,
    pub cues: Registry<CueExt>,
}

// ─── Tests ────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

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
    }
}
