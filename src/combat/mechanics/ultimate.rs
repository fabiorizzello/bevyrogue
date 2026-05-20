
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
pub fn matches_trigger(
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

