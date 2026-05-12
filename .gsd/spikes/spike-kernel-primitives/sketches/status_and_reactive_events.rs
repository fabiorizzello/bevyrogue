// SKETCH — non-compiled, illustrative only.
// Twin proposal:
//   (A) Rewrite `src/combat/status_effect.rs` to canon §H.1/§H.2 taxonomy.
//   (B) Extend `src/combat/events.rs` with the round-3 reactive events from §R.

// =====================================================================
// (A) src/combat/status_effect.rs — canon StatusKind + BuffKind
// =====================================================================

use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

use crate::combat::types::UnitId;

/// Canon §H.1 — closed status enum. Validator (load-time RON) rejects unknown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatusKind {
    // === M017 active set ===
    Heated,     // Agumon fire skills; +15% fire/holy taken; DoT 4/turn
    Chilled,    // Gabumon ice; speed −20% turn; +15% ice taken
    Paralyzed,  // Tentomon electric; skip turn 30%, cleared next turn-start
    Slowed,     // Gabumon Ult; turn delay +30% gauge; non-stacking
    Blessed,    // Renamon Ult ally buff; +15% dmg dealt; +1 Ult charge/action; cleanse-immune

    // === gas-era reserved (no M017 source) ===
    Burn,       // legacy proto, soppiantato da Heated
    Shock,      // legacy proto, soppiantato da Paralyzed
}

/// Canon §H.2 — BuffKind taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuffKind {
    Buff,
    Debuff,
    DR,
    Aura,
    Mark,
}

/// Canon §H.2 — BuffDur. Permanent allowed ONLY for kind:Aura (validator).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuffDur {
    Turns(u8),
    UntilRoundEnd,
    Permanent,
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusEffect {
    pub kind: StatusKind,
    pub buff_kind: BuffKind,
    pub dur: BuffDur,
    pub source_unit: UnitId,
    pub source_blueprint: &'static str, // for DR registry grouping (§H.3)
}

impl StatusEffect {
    /// Returns true when the component should be removed.
    pub fn tick_turn_end(&mut self) -> bool {
        match &mut self.dur {
            BuffDur::Turns(n) => {
                *n = n.saturating_sub(1);
                *n == 0
            }
            BuffDur::UntilRoundEnd => false, // cleared elsewhere on round_end
            BuffDur::Permanent => false,
        }
    }

    pub fn is_cleansable_by_default(&self) -> bool {
        // §H.2 rule 1: EmitCleanse default removes only Debuff.
        matches!(self.buff_kind, BuffKind::Debuff)
    }
}

// Per-status numeric effect lookup (called from damage.rs and turn_system tick).
pub fn heated_amp_pct(_: &StatusEffect) -> i32 { 15 } // +15% fire/holy taken
pub fn chilled_amp_pct(_: &StatusEffect) -> i32 { 15 }
pub fn heated_dot_per_turn(_: &StatusEffect) -> i32 { 4 }
pub fn chilled_speed_red_pct(_: &StatusEffect) -> i32 { 20 }
pub fn slowed_delay_pct(_: &StatusEffect) -> i32 { 30 }
pub fn paralyzed_skip_pct(_: &StatusEffect) -> i32 { 30 }
pub fn blessed_dmg_amp_pct(_: &StatusEffect) -> i32 { 15 }

// Stack policy (§H.1 — single-instance, refresh_max_dur):
pub fn refresh_or_insert(
    existing: Option<&mut StatusEffect>,
    new: StatusEffect,
) -> Option<StatusEffect> {
    match existing {
        None => Some(new),
        Some(cur) => {
            // replace-max-dur
            cur.dur = max_dur(cur.dur, new.dur);
            None // do not create a second instance
        }
    }
}

fn max_dur(a: BuffDur, b: BuffDur) -> BuffDur {
    use BuffDur::*;
    match (a, b) {
        (Permanent, _) | (_, Permanent) => Permanent,
        (UntilRoundEnd, _) | (_, UntilRoundEnd) => UntilRoundEnd,
        (Turns(x), Turns(y)) => Turns(x.max(y)),
    }
}

// =====================================================================
// (B) src/combat/events.rs — canon §R-Events additions
// =====================================================================

use crate::combat::toughness::DamageKind;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum CombatEventKindAdditions {
    // --- already in code (reference) ---
    // OnStatusApplied { kind: StatusKind }, OnSkillCast, OnEnemyKill, OnKO ...

    // --- NEW per §R-Events ---

    /// Renames the OnEnemyKill/OnKO pair to a canonical reactive primitive.
    UnitDied { unit: UnitId, killer: Option<UnitId> },

    /// Renamon `kitsune_grace` listener. Emitted post-Strike of an Ult, pre-cleanup.
    UltimateUsed { actor: UnitId },

    /// Patamon ult-charge listener (+25/heal event).
    Healed { target: UnitId, amount: u32, source: UnitId },

    /// Gabumon/Tentomon team-grant chain. cap-aware on receiver side.
    SpGranted { recipient_unit: UnitId, amount: u8 },

    /// Pre-cascade hook for block-reaction pipeline (§02-08 §A pre-step).
    /// Listeners (e.g. Tentomon battery_loop) may emit `KernelEffect::BlockReaction`.
    IncomingDamage {
        attacker: UnitId,
        defender: UnitId,
        raw_amount: u32,
        kind: DamageKind,
    },

    /// Emitted post-mitigation when a BlockReaction has applied damage_mult.
    /// Presentation listener consumes it for VFX/sfx.
    BlockReactionTriggered {
        defender: UnitId,
        attacker: UnitId,
        mitigated_pct: u8,
    },

    // --- Time manipulation: split current signed TurnAdvance ---
    /// pct ∈ 0..=50 (enforced at emit and apply).
    AdvanceTurn { target: UnitId, pct: u8 },
    DelayTurn { target: UnitId, pct: u8 },
}

// --- Emit constants (kept central) ---
pub const TURN_MANIP_CAP_PCT: u8 = 50;
pub const TURN_GAUGE_MIN: u8 = 0;
pub const TURN_GAUGE_MAX: u16 = 200;

// --- Migration of OnEnemyKill/OnKO ---
//
// Add `UnitDied` alongside; emit BOTH during M017 transitional period.
// New listeners subscribe to `UnitDied` only; old listeners (follow_up.rs)
// keep reading `OnEnemyKill` until S03e subsume of FollowUpIntent.

// --- UltimateCharge.trigger_type addition (src/combat/ultimate.rs) ---
//
//   pub enum UltAccumulationTrigger {
//       OnBasicAttack,
//       OnHitTaken,
//       OnAllyFollowUp,
//       OnKill,
//       OnOffensivePartyEvent,
//       OnHealEvent,      // NEW — Patamon +25/heal event listener
//       OnAllyUltimate,   // NEW — Renamon kitsune_grace listens, but charge_gain is 0; emits AdvanceTurn(self, 10%) side-effect via blueprint
//   }

// --- Test sketch (tests/reactive_events_round3.rs) ---
//
//   #[test] fn unit_died_emitted_with_killer() { ... }
//   #[test] fn ultimate_used_emitted_post_commit() { ... }
//   #[test] fn healed_event_drives_patamon_ult_charge() { ... }
//   #[test] fn sp_granted_does_not_pass_through_round_sp_tracker() { ... }
//   #[test] fn incoming_damage_pre_step_allows_block_mitigation() { ... }
//   #[test] fn advance_turn_clamps_pct_at_50() { ... }
//   #[test] fn delay_turn_clamps_pct_at_50() { ... }
