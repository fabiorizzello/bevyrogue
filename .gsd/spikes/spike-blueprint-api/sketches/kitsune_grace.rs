// THROWAWAY SKETCH — NON-COMPILED, ILLUSTRATIVE ONLY.
// Spike SP2 — proves that Option B's `Blueprint` trait can express
// Renamon's Kitsune Grace passive (round-3 canon) cleanly.
//
// Canon source: docs/future_design_draft/digimon/renamon/04_passive_kitsune_grace.md
//   Trigger: an ALLY (not self, same team) uses an Ultimate.
//   Effect:  the Renamon owning the passive gains +10% AV (advance turn).
//
// Required SP1 primitives:
//   * CombatEventKind::UltimateUsed { actor: UnitId }   (SP1 §"Reactive signature bus")
//
// Required SP3 effects:
//   * Effect::AdvanceTurn { actor: TargetRef, pct: i8 } (SP3 add-now §8)
//   * TargetRef::Self_                                  (SP3 add-now §7)
//
// Verdict: cleanly expressible. Zero commit-time signals, zero state,
// only an `on_event` override. Trait defaults handle commit_signals and snapshot.

use super::blueprint_trait::{Blueprint, BlueprintCtx, BlueprintId};

/// Renamon's Kitsune Grace passive — listener-only blueprint.
///
/// Stateless: no resource, no fields. The struct is a marker that
/// gets registered with the BlueprintRegistry and dispatched on every
/// `CombatEvent::UltimateUsed` drain.
pub struct KitsuneGraceBlueprint;

impl Blueprint for KitsuneGraceBlueprint {
    fn id(&self) -> BlueprintId {
        BlueprintId("kitsune_grace")
    }

    // commit_signals: default empty — Kitsune Grace has no commit-time custom_signals
    // (it is a passive triggered by events, not a skill effect).

    fn on_event(
        &self,
        event: &super::CombatEvent,
        ctx: &BlueprintCtx,
    ) -> Vec<super::Effect> {
        // Filter shape: react only to `UltimateUsed { actor }`.
        let actor = match &event.kind {
            super::CombatEventKind::UltimateUsed { actor } => *actor,
            _ => return Vec::new(),
        };

        // Resolve the unit that owns this passive. In the current single-instance
        // state-resource model (RESEARCH.md §"Surprises" #2), this is a unique
        // lookup. Post-M018 multi-instance refactor, ctx.find_unit_id_for_blueprint
        // returns the specific UnitId that has the kitsune_grace component.
        let self_id = match ctx.find_unit_id_for_blueprint(self.id()) {
            Some(id) => id,
            None => return Vec::new(), // No Renamon with this passive in combat — silent no-op.
        };

        // Guard 1: ignore self-cast ultimates (the canon says "ally", not "self").
        if actor == self_id {
            return Vec::new();
        }

        // Guard 2: must be same team. An enemy Renamon's "ally" is the enemy team;
        // an enemy ult by an enemy ally would still trigger the listener on the
        // owner's *enemy team* Renamon. The team check rejects cross-team triggers.
        let actor_team = ctx.team_of(actor);
        let self_team = ctx.team_of(self_id);
        if actor_team != self_team || actor_team.is_none() {
            return Vec::new();
        }

        // Guard 3: only living owners react. (Dead Renamon does not press into
        // the next turn — defensive even though dead units shouldn't be in any
        // turn queue.)
        if !ctx.is_alive(self_id) {
            return Vec::new();
        }

        // Effect: advance self's turn gauge by 10%. Per SP1 D-M017-TIMEMANIP-SPLIT,
        // `AdvanceTurn` is the canonical positive-direction variant; pct clamped
        // to [0, 50] at emit-site (M017 S01 work).
        vec![super::Effect::AdvanceTurn {
            actor: super::TargetRef::Self_,
            pct: 10,
        }]
    }

    // snapshot: default empty — no state to surface to ValidationSnapshot.
}

// Note on test coverage (M017 Slice F):
//   * ally non-self ult → +10% AV gain (1 emitted Effect)
//   * self ult         → no Effect emitted
//   * enemy ult        → no Effect emitted
//   * dead owner       → no Effect emitted
//   * multiple ally ults in one event drain → multiple +10% emissions (stacks linearly,
//     since the canon does not specify once-per-round; revisit with canon if needed)
