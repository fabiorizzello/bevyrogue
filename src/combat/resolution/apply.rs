use crate::combat::{
    StatusEffectKind,
    buffs::DrBag,
    damage::{AttackContext, DamageBreakdown, calculate_damage},
    energy::Energy,
    events::CombatEventKind,
    sp::RoundSpTracker,
    state::{ResolvedAction, UltEffect},
    status_effect::StatusBag,
    team::Team,
    toughness::{Toughness, can_apply_toughness_damage, classify},
    types::EvoStage,
    ult_gauge::{UltGaugeMetadata, drain_energy_on_ult_reset},
    ultimate::UltimateCharge,
    unit::{BasicStreak, Unit},
};

use super::types::{ResolutionOutcome, ko_payload};

/// Apply damage to a single defender without consuming attacker resources (SP, ult, streak).
/// Called in the per-target loop of Blast/AllEnemies fan-out; the caller hoists resource
/// consumption before the loop. Returns per-target events only: OnDamageDealt, OnBreak, UnitDied.
pub fn apply_damage_only(
    resolved: &ResolvedAction,
    attacker_unit: &Unit,
    defender_unit: &mut Unit,
    defender_team: Team,
    mut defender_tough: Option<&mut Toughness>,
    defender_is_commander: bool,
    defender_break_sealed: bool,
    defender_status: Option<&StatusBag>,
    attacker_statuses: Option<&StatusBag>,
    defender_dr: Option<&DrBag>,
) -> (ResolutionOutcome, Vec<CombatEventKind>) {
    if defender_is_commander {
        return (
            ResolutionOutcome::default(),
            vec![CombatEventKind::OnActionFailed {
                reason: "Target is a Commander".to_string(),
            }],
        );
    }
    if attacker_unit.is_ko() {
        return (
            ResolutionOutcome::default(),
            vec![CombatEventKind::OnActionFailed {
                reason: "Attacker is KO".to_string(),
            }],
        );
    }
    // KO'd adjacents are omitted by resolve_targets; guard here for stale-snapshot edge cases.
    if defender_unit.is_ko() {
        return (ResolutionOutcome::default(), vec![]);
    }

    let mut events = Vec::new();
    let mut outcome = ResolutionOutcome::default();

    if resolved.base_damage > 0 || resolved.toughness_damage > 0 {
        let toughness_weaknesses = defender_tough
            .as_deref()
            .map(|t| t.weaknesses.clone())
            .unwrap_or_default();
        let attack = AttackContext {
            damage_tag: resolved.damage_tag,
            base_damage: resolved.base_damage,
            is_break: false,
        };
        let attacker_dmg_mult = attacker_statuses
            .map(|bag| {
                if bag.has(&StatusEffectKind::Blessed) {
                    1.15_f32
                } else {
                    1.0_f32
                }
            })
            .unwrap_or(1.0_f32);
        let DamageBreakdown {
            final_damage: amount,
            tag_mod_pct,
            triangle_mod_pct,
            ..
        } = calculate_damage(
            attacker_unit,
            &attack,
            defender_unit,
            &toughness_weaknesses,
            defender_status,
            attacker_dmg_mult,
            defender_dr,
        );
        defender_unit.hp_current -= amount;
        let broke = if can_apply_toughness_damage(defender_team, defender_tough.as_deref()) {
            defender_tough
                .as_deref_mut()
                .map(|t| {
                    t.apply_hit(
                        resolved.damage_tag,
                        resolved.toughness_damage,
                        defender_break_sealed,
                    )
                })
                .unwrap_or(false)
        } else {
            false
        };
        let kind = classify(
            resolved.damage_tag,
            &toughness_weaknesses,
            &defender_unit.resists,
            broke,
        );
        let ko = defender_unit.hp_current <= 0;

        outcome.amount = amount;
        outcome.kind = kind;
        outcome.broke = broke;
        outcome.ko = ko;

        events.push(CombatEventKind::OnDamageDealt {
            amount,
            kind,
            tag_mod_pct,
            triangle_mod_pct,
            damage_tag: resolved.damage_tag,
        });
        if broke {
            events.push(CombatEventKind::OnBreak {
                damage_tag: resolved.damage_tag,
            });
        }
        if ko {
            let (status_remaining, heated_remaining) = ko_payload(defender_status);
            events.push(CombatEventKind::UnitDied {
                status_remaining,
                heated_remaining,
            });
        }
    }

    outcome.sp_ok = true;
    outcome.succeeded = true;
    (outcome, events)
}

/// Apply heal to a single target. KO targets are silently skipped (no event emitted).
/// Returns per-target events only: OnHealed. Caller hoists resource consumption.
pub fn apply_heal_only(
    resolved: &ResolvedAction,
    defender_unit: &mut Unit,
) -> (ResolutionOutcome, Vec<CombatEventKind>) {
    if defender_unit.is_ko() {
        return (
            ResolutionOutcome {
                sp_ok: true,
                ..ResolutionOutcome::default()
            },
            vec![],
        );
    }

    let hp_max = defender_unit.hp_max as i64;
    let hp_current = defender_unit.hp_current as i64;
    let pct = resolved.heal_pct as i64;
    // Floor division: (hp_max * pct) / 100; capped so hp_current does not exceed hp_max.
    let raw = (hp_max * pct) / 100;
    let healed = raw.min(hp_max - hp_current).max(0) as i32;
    defender_unit.hp_current += healed;
    let hp_after = defender_unit.hp_current;

    let mut outcome = ResolutionOutcome::default();
    outcome.amount = healed;
    outcome.sp_ok = true;
    outcome.succeeded = true;
    (
        outcome,
        vec![CombatEventKind::OnHealed {
            amount: healed,
            hp_after,
        }],
    )
}

/// Apply cleanse to a single target. KO targets are silently skipped (no event emitted).
/// Caller must ensure `action.cleanse_count` is `Some(_)` before calling this.
pub fn apply_cleanse_only(
    action: &ResolvedAction,
    bag: &mut StatusBag,
    defender_alive: bool,
) -> (ResolutionOutcome, Vec<CombatEventKind>) {
    if !defender_alive {
        return (
            ResolutionOutcome {
                sp_ok: true,
                ..ResolutionOutcome::default()
            },
            vec![],
        );
    }
    let inner_count = action
        .cleanse_count
        .expect("apply_cleanse_only called on action without cleanse_count");
    let kinds = bag.cleanse_n(inner_count);
    let outcome = ResolutionOutcome {
        sp_ok: true,
        succeeded: true,
        ..ResolutionOutcome::default()
    };
    (outcome, vec![CombatEventKind::OnCleansed { kinds }])
}

pub fn apply_legacy_ops(
    resolved: &ResolvedAction,
    attacker_unit: &Unit,
    defender_unit: &mut Unit,
    defender_team: Team,
    mut defender_tough: Option<&mut Toughness>,
    attacker_ult: &mut UltimateCharge,
    sp: &mut crate::combat::sp::SpPool,
    _sp_tracker: &mut RoundSpTracker,
    basic_streak: &mut BasicStreak,
    defender_is_commander: bool,
    defender_break_sealed: bool,
    defender_status: Option<&StatusBag>,
    attacker_statuses: Option<&StatusBag>,
    defender_dr: Option<&DrBag>,
    // S07/T03: optional energy/metadata so energy-backed Ult casts also drain
    // `Energy.current` alongside the legacy `UltimateCharge.current = 0`.
    attacker_energy: Option<&mut Energy>,
    attacker_gauge_meta: Option<&UltGaugeMetadata>,
) -> (ResolutionOutcome, Vec<CombatEventKind>) {
    let mut events = Vec::new();

    // 1. Validation (Does NOT consume SP on failure)
    if defender_is_commander {
        events.push(CombatEventKind::OnActionFailed {
            reason: "Target is a Commander".to_string(),
        });
        return (ResolutionOutcome::default(), events);
    }

    if attacker_unit.is_ko() {
        events.push(CombatEventKind::OnActionFailed {
            reason: "Attacker is KO".to_string(),
        });
        return (ResolutionOutcome::default(), events);
    }

    // Heal no-op on KO: sp_ok=true, no event, no SP consumed (resources not yet spent here).
    if resolved.heal_pct > 0 {
        if defender_unit.is_ko() {
            return (
                ResolutionOutcome {
                    sp_ok: true,
                    ..ResolutionOutcome::default()
                },
                events,
            );
        }
    } else if resolved.revive_pct > 0 {
        if !defender_unit.is_ko() {
            events.push(CombatEventKind::OnActionFailed {
                reason: "Target is not KO".to_string(),
            });
            return (ResolutionOutcome::default(), events);
        }
    } else if defender_unit.is_ko() {
        events.push(CombatEventKind::OnActionFailed {
            reason: "Target is KO".to_string(),
        });
        return (ResolutionOutcome::default(), events);
    }

    // 2. Resource Consumption
    // Child discount: -1 SP on next Skill after 2+ consecutive Basics
    let effective_sp_cost = if matches!(resolved.ult_effect, UltEffect::None)
        && resolved.sp_cost > 0
        && attacker_unit.evo_stage == EvoStage::Child
        && basic_streak.qualifies_for_discount()
    {
        basic_streak.reset();
        (resolved.sp_cost - 1).max(0)
    } else {
        resolved.sp_cost
    };

    if effective_sp_cost > 0 && !sp.spend(effective_sp_cost) {
        return (
            ResolutionOutcome {
                sp_ok: false,
                ..ResolutionOutcome::default()
            },
            Vec::new(),
        );
    }

    if matches!(resolved.ult_effect, UltEffect::Reset) && !attacker_ult.ready() {
        return (
            ResolutionOutcome {
                sp_ok: false,
                ..ResolutionOutcome::default()
            },
            Vec::new(),
        );
    }

    let mut outcome = ResolutionOutcome::default();

    if resolved.heal_pct > 0 {
        let (heal_outcome, heal_events) = apply_heal_only(resolved, defender_unit);
        outcome.amount = heal_outcome.amount;
        events.extend(heal_events);
    } else if resolved.revive_pct > 0 {
        defender_unit.revive(resolved.revive_pct);
        let hp_after = defender_unit.hp_current;
        events.push(CombatEventKind::OnRevive { hp_after });
        outcome.amount = hp_after;
    } else {
        // Short-circuit damage path for modifier-only skills (e.g. GrantEnergy with no hit).
        if resolved.base_damage > 0 || resolved.toughness_damage > 0 {
            let toughness_weaknesses = defender_tough
                .as_deref()
                .map(|t| t.weaknesses.clone())
                .unwrap_or_default();
            let attack = AttackContext {
                damage_tag: resolved.damage_tag,
                base_damage: resolved.base_damage,
                is_break: false,
            };
            let attacker_dmg_mult = attacker_statuses
                .map(|bag| {
                    if bag.has(&StatusEffectKind::Blessed) {
                        1.15_f32
                    } else {
                        1.0_f32
                    }
                })
                .unwrap_or(1.0_f32);
            let DamageBreakdown {
                final_damage: amount,
                tag_mod_pct,
                triangle_mod_pct,
                status_amp_pct: _status_amp_pct,
                ..
            } = calculate_damage(
                attacker_unit,
                &attack,
                defender_unit,
                &toughness_weaknesses,
                defender_status,
                attacker_dmg_mult,
                defender_dr,
            );
            defender_unit.hp_current -= amount;
            let broke = if can_apply_toughness_damage(defender_team, defender_tough.as_deref()) {
                defender_tough
                    .as_deref_mut()
                    .map(|t| {
                        t.apply_hit(
                            resolved.damage_tag,
                            resolved.toughness_damage,
                            defender_break_sealed,
                        )
                    })
                    .unwrap_or(false)
            } else {
                false
            };
            let kind = classify(
                resolved.damage_tag,
                &toughness_weaknesses,
                &defender_unit.resists,
                broke,
            );
            let ko = defender_unit.hp_current <= 0;

            outcome.amount = amount;
            outcome.kind = kind;
            outcome.broke = broke;
            outcome.ko = ko;

            events.push(CombatEventKind::OnDamageDealt {
                amount,
                kind,
                tag_mod_pct,
                triangle_mod_pct,
                damage_tag: resolved.damage_tag,
            });
            if broke {
                events.push(CombatEventKind::OnBreak {
                    damage_tag: resolved.damage_tag,
                });
            }
            if ko {
                let (status_remaining, heated_remaining) = ko_payload(defender_status);
                events.push(CombatEventKind::UnitDied {
                    status_remaining,
                    heated_remaining,
                });
            }
        }
    }

    match resolved.ult_effect {
        UltEffect::GainFromBasic => {
            sp.gain(1);
            let cpe = attacker_ult.charge_per_event;
            attacker_ult.try_add(cpe);
            basic_streak.increment();
        }
        UltEffect::None => {}
        UltEffect::Reset => {
            attacker_ult.current = 0;
            // S07/T03: drain energy alongside legacy charge for energy-backed
            // attackers (e.g. Agumon). Legacy units (None metadata or non-energy
            // backing) are unaffected.
            drain_energy_on_ult_reset(attacker_gauge_meta, attacker_energy);
        }
    }

    events.push(CombatEventKind::OnSkillCast {
        skill_id: resolved.skill_id.clone(),
    });

    if resolved.advance_pct != 0 {
        events.push(CombatEventKind::AdvanceTurn {
            target: resolved.target,
            amount_pct: resolved.advance_pct,
        });
    }

    if resolved.delay_pct != 0 {
        events.push(CombatEventKind::DelayTurn {
            target: resolved.target,
            amount_pct: resolved.delay_pct,
        });
    }

    if resolved.self_advance_pct != 0 {
        let capped = (resolved.self_advance_pct.max(0) as u32).min(50);
        if capped != 0 {
            events.push(CombatEventKind::AdvanceTurn {
                target: resolved.source,
                amount_pct: capped,
            });
        }
    }

    outcome.sp_ok = true;
    outcome.succeeded = true;

    // §H.1: Blessed grants +1 Ult charge per action, but not when the action is
    // an Ultimate cast (Reset branch) — skipping avoids self-feeding the firing Ult.
    if resolved.ult_effect != UltEffect::Reset {
        if let Some(bag) = attacker_statuses {
            if bag.has(&StatusEffectKind::Blessed) {
                attacker_ult.try_add(1);
            }
        }
    }

    (outcome, events)
}
