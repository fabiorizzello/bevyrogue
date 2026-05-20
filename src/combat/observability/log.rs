use std::collections::VecDeque;

use bevy::prelude::Resource;

use crate::combat::toughness::DamageKind;
use crate::combat::types::{DamageTag, UnitId};

#[derive(Debug, Clone)]
pub enum LogEntry {
    BasicHit {
        attacker: UnitId,
        target: UnitId,
        amount: i32,
        kind: DamageKind,
    },
    Break {
        target: UnitId,
        damage_tag: DamageTag,
    },
    Ko {
        target: UnitId,
    },
    Revive {
        target: UnitId,
        hp_after: i32,
    },
    ActionFailed {
        reason: String,
    },
    AdvanceTurn {
        target: UnitId,
        amount_pct: u32,
    },
    DelayTurn {
        target: UnitId,
        amount_pct: u32,
    },
}

#[derive(Resource, Default, Debug)]
pub struct ActionLog {
    pub events: VecDeque<LogEntry>,
}

impl ActionLog {
    pub const CAP: usize = 5;

    pub fn push(&mut self, ev: LogEntry) {
        self.events.push_back(ev);
        while self.events.len() > Self::CAP {
            self.events.pop_front();
        }
    }
}
