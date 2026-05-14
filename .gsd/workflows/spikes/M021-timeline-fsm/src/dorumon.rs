//! Dorumon blueprint — exercises **Pattern 2: mutable blueprint state**.
//!
//! Predator Loop (canon `docs/future_design_draft/digimon/dorumon/04_passive_predator_loop.md`)
//! holds per-unit state (`tracked_target`, `predator_active`, `expires_in`)
//! that:
//!
//! - is mutated by an external listener correlating multiple events
//!   (`DamageDealt` → recompute `tracked_target = lowest_hp%`),
//! - is read by **predicates** to gate edges (`BlueprintState("predator_active") == true`),
//! - is mutated by **hooks** through `Intent::SetBlueprintState` (the
//!   `metal_cannon` Ult force-entry: skill writes the state, no listener),
//! - is tick-decremented at `TurnEnded` (`expires_in -= 1`).
//!
//! The spike validates the **read** path end-to-end through a predicate and
//! the **write** path end-to-end through a hook emitting
//! `Intent::SetBlueprintState`. The kernel-side application of the intent is
//! orthogonal (covered by D008 transition stream).
//!
//! State keys (per-unit, i32-valued):
//! - `dorumon.predator_active` — 0 = off, 1 = on
//! - `dorumon.tracked_target` — UnitId of lowest-HP enemy, 0 if none
//! - `dorumon.expires_in` — turns remaining

use crate::*;

// ---------- Predicates: read blueprint state ----------

/// Edge gate for the chain branch of `dash_metal`: only fires when
/// `predator_active` is set on the caster.
pub fn predator_active(evt: &BeatEvent, ctx: &SkillCtx) -> bool {
    ctx.blueprint_state(evt.caster, "dorumon.predator_active") != 0
}

// ---------- Hooks: read AND write blueprint state ----------

/// Chain consume hook (runs inside the chain branch after dash_metal lands
/// a kill). Reads `tracked_target` from blueprint state and queues a
/// chain damage on that target, then resets `predator_active` to 0 via
/// `Intent::SetBlueprintState` — the canonical write path.
pub fn on_chain_consume(evt: &BeatEvent, ctx: &mut SkillCtx) {
    let chained = ctx.blueprint_state(evt.caster, "dorumon.tracked_target") as UnitId;
    if chained != 0 {
        ctx.enqueue(Intent::DealDamage {
            target: chained,
            amount: 240, // flat chain damage in the spike; production scales by formula
            tag: DamageTag::Physical,
            cast_id: evt.cast_id,
        });
    }
    // Reset state — chain has been consumed.
    ctx.enqueue(Intent::SetBlueprintState {
        actor: evt.caster,
        key: "dorumon.predator_active",
        value: 0,
        cast_id: evt.cast_id,
    });
}

/// Ult force-entry hook (`metal_cannon`): forces `predator_active = 1` on
/// the caster regardless of HP threshold (canon: ult bypasses the listener
/// recompute path). Pure write — no read.
pub fn on_metal_cannon_force_predator(evt: &BeatEvent, ctx: &mut SkillCtx) {
    ctx.enqueue(Intent::SetBlueprintState {
        actor: evt.caster,
        key: "dorumon.predator_active",
        value: 1,
        cast_id: evt.cast_id,
    });
    ctx.enqueue(Intent::SetBlueprintState {
        actor: evt.caster,
        key: "dorumon.expires_in",
        value: 3,
        cast_id: evt.cast_id,
    });
}

// ---------- Registration ----------

pub fn register(reg: &mut ExtRegistries) {
    reg.predicates.register("dorumon::predator_active", predator_active);
    reg.hooks.register("dorumon::on_chain_consume", on_chain_consume);
    reg.hooks.register("dorumon::on_metal_cannon_force_predator",
                       on_metal_cannon_force_predator);
}
