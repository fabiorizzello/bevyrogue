#![allow(dead_code)]

use bevy::prelude::*;

use crate::combat::{
    runtime::intent::CastId,
    events::{CombatEvent, CombatEventKind},
    team::Team,
    unit::Unit,
};

/// Staging buffer: UltGain events are accumulated here by ult_accumulation_system
/// and flushed into the CombatEvent bus by flush_ult_gain_system each frame.
/// Required because Bevy forbids reading and writing the same event type in one system.
#[derive(Resource, Default)]
pub struct UltGainQueue(pub Vec<CombatEvent>);

/// Determines when a unit's ultimate meter gains charge.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Default,
)]
pub enum UltAccumulationTrigger {
    #[default]
    OnBasicAttack,
    OnHitTaken,
    OnAllyFollowUp,
    OnKill,
    OnOffensivePartyEvent,
}

#[derive(Component, Debug, Clone)]
pub struct UltimateCharge {
    pub current: i32,
    pub trigger: i32,
    pub cap: i32,
    pub trigger_type: UltAccumulationTrigger,
    pub charge_per_event: i32,
}

impl UltimateCharge {
    pub fn new(
        trigger: i32,
        cap: i32,
        trigger_type: UltAccumulationTrigger,
        charge_per_event: i32,
    ) -> Self {
        Self {
            current: 0,
            trigger,
            cap,
            trigger_type,
            charge_per_event,
        }
    }

    /// Returns true when this addition crosses the trigger threshold from below (first ready).
    pub fn try_add(&mut self, amount: i32) -> bool {
        let was_ready = self.current >= self.trigger;
        self.current = (self.current + amount).clamp(0, self.cap);
        let now_ready = self.current >= self.trigger;
        !was_ready && now_ready
    }

    pub fn ready(&self) -> bool {
        self.current >= self.trigger
    }
}

/// Pure predicate: does `event` trigger charge gain for a unit with the given trigger_type?
/// `OnBasicAttack` always returns false — that path is handled in `apply_effects`.
pub(crate) fn matches_trigger(
    event: &CombatEvent,
    unit_id: crate::combat::types::UnitId,
    unit_team: Team,
    trigger_type: UltAccumulationTrigger,
    source_team: Option<Team>,
) -> bool {
    match trigger_type {
        UltAccumulationTrigger::OnBasicAttack => false,
        UltAccumulationTrigger::OnHitTaken => {
            matches!(event.kind, CombatEventKind::OnHitTaken { .. }) && event.target == unit_id
        }
        UltAccumulationTrigger::OnAllyFollowUp => {
            matches!(event.kind, CombatEventKind::OnSkillCast { .. })
                && event.follow_up_depth >= 1
                && source_team == Some(unit_team)
        }
        UltAccumulationTrigger::OnKill => {
            matches!(event.kind, CombatEventKind::OnEnemyKill) && event.source == unit_id
        }
        UltAccumulationTrigger::OnOffensivePartyEvent => {
            matches!(
                event.kind,
                CombatEventKind::OnDamageDealt { .. }
                    | CombatEventKind::OnBreak { .. }
                    | CombatEventKind::OnEnemyKill
            ) && source_team == Some(Team::Ally)
        }
    }
}

/// Per-unit ultimate accumulation system. Replaces `offensive_momentum_system`.
/// Dispatches charge gains based on each unit's `trigger_type` and `charge_per_event`.
/// `OnBasicAttack` is excluded here — handled exclusively in `apply_effects` to avoid double-count.
/// UltGain events are staged in `UltGainQueue` (MEM003-safe pattern) and flushed by
/// `flush_ult_gain_system`.
pub fn ult_accumulation_system(
    mut events: MessageReader<CombatEvent>,
    all_units: Query<(&Unit, &Team)>,
    mut ult_units: Query<(&Unit, &Team, &mut UltimateCharge)>,
    mut queue: ResMut<UltGainQueue>,
) {
    let team_of: std::collections::HashMap<_, _> = all_units
        .iter()
        .map(|(unit, team)| (unit.id, *team))
        .collect();

    let combat_events: Vec<CombatEvent> = events.read().cloned().collect();

    for event in &combat_events {
        let source_team = team_of.get(&event.source).copied();

        for (unit, unit_team, mut charge) in ult_units.iter_mut() {
            if !matches_trigger(event, unit.id, *unit_team, charge.trigger_type, source_team) {
                continue;
            }
            let before = charge.current;
            let cpe = charge.charge_per_event;
            charge.try_add(cpe);
            let delta = charge.current - before;
            if delta > 0 {
                queue.0.push(CombatEvent {
                    kind: CombatEventKind::UltGain {
                        unit_id: unit.id,
                        amount: delta,
                    },
                    source: unit.id,
                    target: unit.id,
                    follow_up_depth: 0,
                    cast_id: CastId::ROOT,
                });
            }
        }
    }
}

/// Drains `UltGainQueue` into the CombatEvent message bus.
/// Must run after `ult_accumulation_system` each frame.
pub fn flush_ult_gain_system(
    mut queue: ResMut<UltGainQueue>,
    mut writer: MessageWriter<CombatEvent>,
) {
    for event in queue.0.drain(..) {
        writer.write(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::{
        toughness::DamageKind,
        types::{SkillId, UnitId},
    };

    #[test]
    fn try_add_crosses_trigger_returns_true() {
        let mut ult = UltimateCharge::new(100, 150, UltAccumulationTrigger::OnBasicAttack, 25);
        let result = ult.try_add(100);
        assert!(result);
        assert!(ult.ready());
    }

    #[test]
    fn try_add_caps_at_cap() {
        let mut ult = UltimateCharge::new(100, 150, UltAccumulationTrigger::OnBasicAttack, 25);
        ult.try_add(200);
        assert_eq!(ult.current, 150);
    }

    #[test]
    fn try_add_already_ready_no_new_cross() {
        let mut ult = UltimateCharge {
            current: 100,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        };
        let result = ult.try_add(10);
        assert!(!result);
    }

    #[test]
    fn ult_gain_delta_is_actual_increase() {
        let mut ult = UltimateCharge::new(100, 150, UltAccumulationTrigger::OnBasicAttack, 25);
        let before = ult.current;
        ult.try_add(10);
        let delta = ult.current - before;
        assert_eq!(delta, 10);
    }

    #[test]
    fn ult_gain_delta_clamps_at_cap() {
        let mut ult = UltimateCharge {
            current: 145,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        };
        let before = ult.current;
        ult.try_add(10);
        let delta = ult.current - before;
        assert_eq!(delta, 5);
    }

    #[test]
    fn ult_gain_delta_zero_when_already_at_cap() {
        let mut ult = UltimateCharge {
            current: 150,
            trigger: 100,
            cap: 150,
            trigger_type: UltAccumulationTrigger::OnBasicAttack,
            charge_per_event: 25,
        };
        let before = ult.current;
        ult.try_add(10);
        let delta = ult.current - before;
        assert_eq!(delta, 0);
    }

    // --- matches_trigger dispatch tests ---

    fn damage_event(source: UnitId, target: UnitId) -> CombatEvent {
        CombatEvent {
            kind: CombatEventKind::OnDamageDealt {
                amount: 50,
                kind: DamageKind::Normal,
                tag_mod_pct: 100,
                triangle_mod_pct: 100,
                damage_tag: crate::combat::types::DamageTag::Fire,
            },
            source,
            target,
            follow_up_depth: 0,
            cast_id: CastId::ROOT,
        }
    }

    fn hit_taken_event(source: UnitId, target: UnitId, amount: i32) -> CombatEvent {
        CombatEvent {
            kind: CombatEventKind::OnHitTaken { amount },
            source,
            target,
            follow_up_depth: 0,
            cast_id: CastId::ROOT,
        }
    }

    fn skill_cast_event(source: UnitId, depth: u8) -> CombatEvent {
        CombatEvent {
            kind: CombatEventKind::OnSkillCast {
                skill_id: SkillId("s".into()),
            },
            source,
            target: source,
            follow_up_depth: depth,
            cast_id: CastId::ROOT,
        }
    }

    fn kill_event(source: UnitId, target: UnitId) -> CombatEvent {
        CombatEvent {
            kind: CombatEventKind::OnEnemyKill,
            source,
            target,
            follow_up_depth: 0,
            cast_id: CastId::ROOT,
        }
    }

    #[test]
    fn trigger_on_basic_attack_never_matches() {
        let event = damage_event(UnitId(1), UnitId(2));
        assert!(!matches_trigger(
            &event,
            UnitId(1),
            Team::Ally,
            UltAccumulationTrigger::OnBasicAttack,
            Some(Team::Ally)
        ));
    }

    #[test]
    fn trigger_on_hit_taken_matches_correct_target() {
        let event = hit_taken_event(UnitId(1), UnitId(2), 30);
        assert!(matches_trigger(
            &event,
            UnitId(2),
            Team::Ally,
            UltAccumulationTrigger::OnHitTaken,
            Some(Team::Ally)
        ));
        assert!(!matches_trigger(
            &event,
            UnitId(99),
            Team::Ally,
            UltAccumulationTrigger::OnHitTaken,
            Some(Team::Ally)
        ));
    }

    #[test]
    fn trigger_on_ally_follow_up_matches_depth_and_team() {
        let event = skill_cast_event(UnitId(1), 1);
        assert!(matches_trigger(
            &event,
            UnitId(5),
            Team::Ally,
            UltAccumulationTrigger::OnAllyFollowUp,
            Some(Team::Ally)
        ));
        // depth 0 → not a follow-up
        let root_event = skill_cast_event(UnitId(1), 0);
        assert!(!matches_trigger(
            &root_event,
            UnitId(5),
            Team::Ally,
            UltAccumulationTrigger::OnAllyFollowUp,
            Some(Team::Ally)
        ));
    }

    #[test]
    fn trigger_on_kill_matches_source_unit_only() {
        let event = kill_event(UnitId(3), UnitId(7));
        assert!(matches_trigger(
            &event,
            UnitId(3),
            Team::Ally,
            UltAccumulationTrigger::OnKill,
            Some(Team::Ally)
        ));
        assert!(!matches_trigger(
            &event,
            UnitId(7),
            Team::Ally,
            UltAccumulationTrigger::OnKill,
            Some(Team::Ally)
        ));
    }

    #[test]
    fn trigger_on_offensive_party_event_matches_ally_source_only() {
        let event = damage_event(UnitId(1), UnitId(5));
        assert!(matches_trigger(
            &event,
            UnitId(99),
            Team::Ally,
            UltAccumulationTrigger::OnOffensivePartyEvent,
            Some(Team::Ally)
        ));
        assert!(!matches_trigger(
            &event,
            UnitId(99),
            Team::Ally,
            UltAccumulationTrigger::OnOffensivePartyEvent,
            Some(Team::Enemy)
        ));
    }

    /// Q7 negative test: ally-vs-ally damage — Taichi charges on OnDamageDealt, not on
    /// OnHitTaken; Hackmon charges on OnHitTaken when it is the target.
    #[test]
    fn ally_vs_ally_semantics_taichi_and_hackmon() {
        let attacker = UnitId(1);
        let defender = UnitId(2); // Hackmon's id

        let damage_evt = damage_event(attacker, defender);
        let hit_evt = hit_taken_event(attacker, defender, 30);

        // Taichi (OnOffensivePartyEvent) charges on OnDamageDealt from ally source
        assert!(matches_trigger(
            &damage_evt,
            UnitId(99),
            Team::Ally,
            UltAccumulationTrigger::OnOffensivePartyEvent,
            Some(Team::Ally)
        ));
        // Taichi does NOT charge on OnHitTaken (wrong kind for OnOffensivePartyEvent)
        assert!(!matches_trigger(
            &hit_evt,
            UnitId(99),
            Team::Ally,
            UltAccumulationTrigger::OnOffensivePartyEvent,
            Some(Team::Ally)
        ));
        // Hackmon charges on OnHitTaken when it's the target
        assert!(matches_trigger(
            &hit_evt,
            defender,
            Team::Ally,
            UltAccumulationTrigger::OnHitTaken,
            Some(Team::Ally)
        ));
        // Hackmon does NOT charge if it's not the target
        assert!(!matches_trigger(
            &hit_evt,
            UnitId(99),
            Team::Ally,
            UltAccumulationTrigger::OnHitTaken,
            Some(Team::Ally)
        ));
    }
}
