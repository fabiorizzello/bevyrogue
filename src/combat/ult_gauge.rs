use bevy::prelude::Component;

use crate::combat::{energy::Energy, ultimate::UltimateCharge};
use crate::data::units_ron::BlueprintRoster;

#[derive(Component, Debug, Clone, Default, PartialEq, Eq)]
pub struct UltGaugeMetadata(pub BlueprintRoster);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveUltGauge {
    pub current: i32,
    pub trigger: i32,
    pub cap: i32,
    pub ready: bool,
    pub backing: &'static str,
}

pub fn effective_ult_gauge(
    metadata: Option<&UltGaugeMetadata>,
    energy: Option<&Energy>,
    legacy: Option<&UltimateCharge>,
) -> EffectiveUltGauge {
    if is_energy_backed(metadata) {
        if let Some(energy) = energy {
            return EffectiveUltGauge {
                current: energy.current,
                trigger: energy.max,
                cap: energy.max,
                ready: energy.current >= energy.max,
                backing: "energy",
            };
        }
    }

    let current = legacy.map(|ult| ult.current).unwrap_or(0);
    let trigger = legacy.map(|ult| ult.trigger).unwrap_or(100);
    let cap = legacy.map(|ult| ult.cap).unwrap_or(trigger);
    EffectiveUltGauge {
        current,
        trigger,
        cap,
        ready: legacy.is_some_and(UltimateCharge::ready),
        backing: "legacy_ultimate_charge",
    }
}

pub fn is_energy_backed(metadata: Option<&UltGaugeMetadata>) -> bool {
    metadata
        .and_then(|metadata| metadata.0 .0.get("agumon"))
        .and_then(|payload| payload.0.get("ult_gauge"))
        .is_some_and(|value| value == "energy")
}

/// S07/T03: drain `Energy.current` to 0 when the attacker is energy-backed.
/// Invoked at every `UltEffect::Reset` finalize site alongside the legacy
/// `UltimateCharge.current = 0` reset. Legacy ult charge is kept at 0 for
/// back-compat until the old gauge is fully smantellato.
pub fn drain_energy_on_ult_reset(metadata: Option<&UltGaugeMetadata>, energy: Option<&mut Energy>) {
    if is_energy_backed(metadata) {
        if let Some(energy) = energy {
            energy.current = 0;
        }
    }
}

/// S07/T03: world-level convenience used at finalize sites that have
/// deferred access to the World (via `Commands::queue`). Drains
/// `Energy.current` for `attacker_entity` if it is energy-backed.
pub fn drain_energy_on_ult_reset_for_entity(
    world: &mut bevy::prelude::World,
    attacker_entity: bevy::prelude::Entity,
) {
    let is_backed = world
        .get::<UltGaugeMetadata>(attacker_entity)
        .map(|meta| is_energy_backed(Some(meta)))
        .unwrap_or(false);
    if !is_backed {
        return;
    }
    if let Some(mut energy) = world.get_mut::<Energy>(attacker_entity) {
        energy.current = 0;
    }
}
