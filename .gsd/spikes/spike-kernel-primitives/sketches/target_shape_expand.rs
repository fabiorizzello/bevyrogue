// SKETCH — non-compiled, illustrative only.
// Proposed expansion of `TargetShape` to cover canon §C3 vocabulary.
// Lives logically in `src/data/skills_ron.rs` (enum) + `src/combat/resolution.rs` (resolver).

use crate::combat::types::UnitId;

// --- 1. Enum surface (src/data/skills_ron.rs) ---

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Side {
    AllyTeam,
    EnemyTeam,
    BothTeams,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetShape {
    // === Single-target ===
    Single,                          // canon "Primary"
    SelfOnly,                        // unchanged
    AdjLeft,
    AdjRight,

    // === Multi-target ===
    /// Primary + 2 adjacent (3 hits, decreasing damage to adj — caller multiplies).
    Blast,
    /// Whole side; dead exclusion governed by `exclude_dead`.
    AoE { side: Side, exclude_dead: bool },
    /// Chain N hits; each hop re-resolves `selector` against live world.
    Bounce { hits: u8, selector: Box<TargetShape> },
    // (Row kept as deprecated alias of AoE { side: EnemyTeam, exclude_dead: true } during migration.)
}

// --- 2. Resolver contract (src/combat/resolution.rs) ---

pub struct ResolveCtx<'a> {
    pub primary: UnitId,
    pub caster: UnitId,
    pub world_view: &'a WorldView, // slim, blueprint-side view: alive units, slots, teams
}

/// Pure: snapshot-once at commit, except `Bounce` which re-resolves per hop.
/// Returns Vec, never None. Empty Vec ⇒ silent no-op (canon §C3 rule 3).
pub fn resolve_shape(shape: &TargetShape, ctx: &ResolveCtx) -> Vec<UnitId> {
    match shape {
        TargetShape::Single | TargetShape::SelfOnly => vec![ctx.primary],
        TargetShape::AdjLeft => ctx.world_view.adj(ctx.primary, -1).into_iter().collect(),
        TargetShape::AdjRight => ctx.world_view.adj(ctx.primary, 1).into_iter().collect(),
        TargetShape::Blast => {
            let mut out = vec![ctx.primary];
            out.extend(ctx.world_view.adj(ctx.primary, -1));
            out.extend(ctx.world_view.adj(ctx.primary, 1));
            out
        }
        TargetShape::AoE { side, exclude_dead } => {
            let mut targets: Vec<UnitId> = ctx
                .world_view
                .units_on_side(*side, *exclude_dead)
                .collect();
            // Deterministic order: ascending UnitId (mirrors follow_up.rs tie-break).
            targets.sort_by_key(|u| u.0);
            targets
        }
        TargetShape::Bounce { hits, selector } => {
            // Bounce is special: caller (apply loop) re-invokes per hop with updated world.
            // Here we resolve only the first hop; the loop in `execute_action_intent` handles
            // hop_n = 2..hits by re-calling `resolve_shape(selector, ctx_at_hop_n)`.
            resolve_shape(selector, ctx).into_iter().take(1).collect()
        }
    }
}

// --- 3. Apply loop (sketch, src/combat/turn_system/pipeline.rs) ---

pub fn fanout_effects(
    shape: &TargetShape,
    base_effect: &Effect,
    ctx_init: &ResolveCtx,
    world: &mut WorldMut,
) {
    match shape {
        TargetShape::Bounce { hits, selector } => {
            for hop_idx in 0..*hits {
                // Re-snapshot world per hop (canon §C3 rule 4)
                let ctx = ctx_init.refresh_against(world);
                let next = resolve_shape(selector, &ctx);
                let Some(target) = next.first().copied() else {
                    // chain interrupted (no alive target) — silent break, no panic
                    break;
                };
                apply_single(base_effect, target, world, hop_idx);
            }
        }
        _ => {
            for target in resolve_shape(shape, ctx_init) {
                apply_single(base_effect, target, world, 0);
            }
        }
    }
}

// --- 4. Migration path ---
//
// LegalityReasonCode::UnimplementedTargetShape gate in src/data/skills_ron.rs:269
// and src/combat/resolution.rs:174 must be removed; the resolver now accepts all
// canon shapes. RON `Effect::Damage { target: TargetShape::Row, ... }` already
// parses — only the execution side was unimplemented. Existing tests that assert
// `UnimplementedTargetShape` legality become positive-execution tests.

// --- 5. Test sketch (tests/target_shape_fanout.rs) ---
//
//   #[test]
//   fn blast_hits_primary_and_two_adj() { ... }
//
//   #[test]
//   fn aoe_excludes_dead_when_flag_set() { ... }
//
//   #[test]
//   fn bounce_re_resolves_per_hop_and_breaks_on_no_target() { ... }
//
//   #[test]
//   fn bounce_tie_break_is_unitid_ascending() { ... }
