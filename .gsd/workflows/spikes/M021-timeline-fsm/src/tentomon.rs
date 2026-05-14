//! Tentomon blueprint — exercises **Pattern 4: RNG-gated edge**.
//!
//! Block Reaction (canon `docs/future_design_draft/digimon/tentomon/04_passive_battery_loop.md`)
//! transitions `BlockReady → BlockProc` only when a deterministic RNG roll
//! comes in below the threshold. The RNG is seeded by
//! `(state.rng_seed, cast_id, beat_id, hop_index, salt)` so the same fixture
//! always reproduces the same outcome — the determinism invariant I1 must
//! hold even with RNG-driven gates.
//!
//! Predicate name convention: `tentomon::rng_below_X` where X is the
//! percent threshold × 100 (e.g. `rng_below_30` = 30% chance to pass).

use crate::*;

// ---------- RNG-gated predicates ----------

const SALT_BLOCK_REACTION: u32 = 0x424c_4f43; // 'BLOC'

/// Returns true with `chance_bp` basis-points probability, deterministically
/// keyed off the current `(cast_id, beat, hop_index, salt)`.
fn rng_below(evt: &BeatEvent, ctx: &SkillCtx, chance_bp: u32) -> bool {
    let draw = ctx.rng_u32(evt.cast_id, evt.beat, evt.hop_index, SALT_BLOCK_REACTION);
    (draw % 10_000) < chance_bp
}

pub fn rng_below_30pct(evt: &BeatEvent, ctx: &SkillCtx) -> bool {
    rng_below(evt, ctx, 3_000)
}

pub fn rng_below_70pct(evt: &BeatEvent, ctx: &SkillCtx) -> bool {
    rng_below(evt, ctx, 7_000)
}

// ---------- Hooks ----------

/// Block reaction hook: damage is halved (caller pre-applies a damage_mult of
/// 0.50 in production via a kernel Command; the spike represents the gameplay
/// outcome as a flat "absorbed" intent).
pub fn on_block_proc(evt: &BeatEvent, ctx: &mut SkillCtx) {
    ctx.enqueue(Intent::ApplyStatus {
        target: evt.caster,
        kind: StatusKind::TwinCoreRage, // reuse the existing enum as a marker
        duration: 1,
        mode: StatusApplyMode::Refresh,
        cast_id: evt.cast_id,
    });
}

// ---------- Registration ----------

pub fn register(reg: &mut ExtRegistries) {
    reg.predicates.register("tentomon::rng_below_30pct", rng_below_30pct);
    reg.predicates.register("tentomon::rng_below_70pct", rng_below_70pct);
    reg.hooks.register("tentomon::on_block_proc", on_block_proc);
}
