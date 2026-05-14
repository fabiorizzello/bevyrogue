//! Gabumon blueprint — exercises **Pattern 3: cross-blueprint synergy**.
//!
//! Twin Core ice (canon `docs/future_design_draft/digimon/gabumon/04_passive_fur_cloak.md`,
//! Path B) is a state-watch passive that arms when **Agumon** applies
//! Heated. The discriminator is **caster identity**: any Heated apply by a
//! non-Agumon caster must NOT arm Twin Core ice.
//!
//! The spike validates this through a predicate that filters on
//! `ctx.identity_of(caster) == "agumon"`. The Heated apply event is modelled
//! as a beat whose predecessor passed through Agumon's blueprint; the
//! predicate reads the listener-style `BeatEvent.caster` and checks identity.
//!
//! In production this is the listener-side filter:
//! ```ignore
//! if status == Heated && ctx.identity_of(event.caster) == "agumon" { arm }
//! ```

use crate::*;

// ---------- Predicates: filter on caster identity ----------

/// True iff the event's caster (whoever applied Heated, in this beat's
/// context the `evt.caster` is the unit being checked) has identity "agumon".
/// The kernel installs `identity` on each unit at spawn time.
pub fn heated_caster_is_agumon(evt: &BeatEvent, ctx: &SkillCtx) -> bool {
    ctx.identity_of(evt.caster) == "agumon"
}

// ---------- Hooks: arm Twin Core ice on event ----------

/// Arm Twin Core ice: self-apply TwinCoreRage status as a stand-in for the
/// `Buff_TwinCoreIceActive` marker. Spike uses the existing StatusKind set.
pub fn on_twin_core_ice_arm(evt: &BeatEvent, ctx: &mut SkillCtx) {
    // `primary_target` here is "self" (Gabumon) — the listener arms its own buff.
    ctx.enqueue(Intent::ApplyStatus {
        target: evt.primary_target,
        kind: StatusKind::TwinCoreRage,
        duration: 99, // round-scoped — spike uses a long duration as a stand-in
        mode: StatusApplyMode::Refresh,
        cast_id: evt.cast_id,
    });
}

// ---------- Registration ----------

pub fn register(reg: &mut ExtRegistries) {
    reg.predicates.register("gabumon::heated_caster_is_agumon", heated_caster_is_agumon);
    reg.hooks.register("gabumon::on_twin_core_ice_arm", on_twin_core_ice_arm);
}
