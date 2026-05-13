use crate::combat::{
    StatusEffectKind,
    damage::{AttackContext, DamageBreakdown, calculate_damage},
    status_effect::StatusBag,
    events::CombatEventKind,
    kit::UnitSkills,
    sp::RoundSpTracker,
    state::{ResolvedAction, UltEffect},
    team::Team,
    toughness::{DamageKind, Toughness, can_apply_toughness_damage, classify},
    turn_system::ActionIntent,
    types::{EvoStage, SkillId},
    ultimate::UltimateCharge,
    unit::{BasicStreak, Unit},
};
use crate::data::skills_ron::{Effect, SkillBook, TargetShape};

/// Emit one `OnSkillCast` per granted free-skill slot, using the provided ally basic skill ids.
/// Callers (e.g. `execute_action_intent`) collect the ally basics and call this; the function is
/// extracted here so resolution unit-tests can exercise it without a Bevy world.
pub fn grant_free_skill_events(count: usize, ally_basics: &[SkillId]) -> Vec<CombatEventKind> {
    ally_basics
        .iter()
        .take(count)
        .map(|skill_id| CombatEventKind::OnSkillCast {
            skill_id: skill_id.clone(),
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolutionOutcome {
    pub amount: i32,
    pub kind: DamageKind,
    pub broke: bool,
    pub ko: bool,
    pub sp_ok: bool,
    /// True only when the action actually executed (past all guard checks).
    /// False on early returns (KO'd target, commander guard, etc.).
    pub succeeded: bool,
}

impl Default for ResolutionOutcome {
    fn default() -> Self {
        Self {
            amount: 0,
            kind: DamageKind::Normal,
            broke: false,
            ko: false,
            sp_ok: true,
            succeeded: false,
        }
    }
}

pub fn resolve_action(
    intent: &ActionIntent,
    kit: &UnitSkills,
    book: Option<&SkillBook>,
) -> Option<ResolvedAction> {
    let skill_id = match intent {
        ActionIntent::Basic { .. } => &kit.basic,
        ActionIntent::Skill { skill_id, .. } => skill_id,
        ActionIntent::Ultimate { .. } => &kit.ultimate,
    };
    let skill = book?.0.iter().find(|skill| &skill.id == skill_id)?;

    let (source, target, ult_effect) = match intent {
        ActionIntent::Basic { attacker, target } => (*attacker, *target, UltEffect::GainFromBasic),
        ActionIntent::Skill {
            attacker, target, ..
        } => (*attacker, *target, UltEffect::None),
        ActionIntent::Ultimate { attacker, target } => (*attacker, *target, UltEffect::Reset),
    };

    Some(ResolvedAction {
        source,
        target,
        skill_id: skill.id.clone(),
        damage_tag: skill.damage_tag,
        base_damage: skill_base_damage(&skill.effects),
        toughness_damage: skill_toughness_hit(&skill.effects),
        revive_pct: skill_revive_pct(&skill.effects),
        sp_cost: skill.sp_cost,
        ult_effect,
        grant_free_skill_count: skill_grant_free_count(&skill.effects),
        status_to_apply: skill_apply_status(&skill.effects),
        turn_advance_pct: skill_turn_advance(&skill.effects),
        energy_grant: skill_grant_energy(&skill.effects),
        self_advance_pct: skill_self_advance(&skill.effects),
        target_shape: skill.targeting.shape,
        custom_signals: skill.custom_signals.clone(),
    })
}

fn skill_base_damage(effects: &[Effect]) -> i32 {
    effects
        .iter()
        .find_map(|effect| match effect {
            Effect::Damage { amount, .. } => Some(*amount),
            _ => None,
        })
        .unwrap_or(0)
}

fn skill_toughness_hit(effects: &[Effect]) -> i32 {
    effects
        .iter()
        .find_map(|effect| match effect {
            Effect::ToughnessHit(amount) => Some(*amount),
            _ => None,
        })
        .unwrap_or(0)
}

fn skill_revive_pct(effects: &[Effect]) -> i32 {
    effects
        .iter()
        .find_map(|effect| match effect {
            Effect::Revive(pct) => Some(*pct),
            _ => None,
        })
        .unwrap_or(0)
}

fn skill_grant_free_count(effects: &[Effect]) -> usize {
    effects
        .iter()
        .find_map(|effect| match effect {
            Effect::GrantFreeSkill { count } => Some(*count),
            _ => None,
        })
        .unwrap_or(0)
}

/// First ApplyStatus effect in the skill's effect list; first match wins.
fn skill_apply_status(effects: &[Effect]) -> Option<(StatusEffectKind, u32)> {
    effects.iter().find_map(|effect| match effect {
        Effect::ApplyStatus { kind, duration } => Some((kind.clone(), *duration)),
        _ => None,
    })
}

fn skill_turn_advance(effects: &[Effect]) -> i32 {
    effects
        .iter()
        .find_map(|effect| match effect {
            Effect::TurnAdvance(amount) => Some(*amount),
            _ => None,
        })
        .unwrap_or(0)
}

fn skill_grant_energy(effects: &[Effect]) -> i32 {
    effects
        .iter()
        .find_map(|effect| match effect {
            Effect::GrantEnergy(amount) => Some(*amount),
            _ => None,
        })
        .unwrap_or(0)
}

fn skill_self_advance(effects: &[Effect]) -> i32 {
    effects
        .iter()
        .find_map(|effect| match effect {
            Effect::SelfAdvance(amount) => Some(*amount),
            _ => None,
        })
        .unwrap_or(0)
}

pub fn target_shape_is_executable_now(shape: TargetShape) -> bool {
    matches!(shape, TargetShape::Single)
}

pub fn target_shape_rejection_reason(shape: TargetShape) -> Option<String> {
    if target_shape_is_executable_now(shape) {
        None
    } else {
        Some(format!("UnimplementedTargetShape:{shape:?}"))
    }
}

pub fn apply_effects(
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

    if resolved.revive_pct > 0 {
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

    if resolved.revive_pct > 0 {
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
                .map(|bag| if bag.has(&StatusEffectKind::Blessed) { 1.15_f32 } else { 1.0_f32 })
                .unwrap_or(1.0_f32);
            let DamageBreakdown {
                final_damage: amount,
                tag_mod_pct,
                triangle_mod_pct,
                status_amp_pct: _status_amp_pct,
            } = calculate_damage(
                attacker_unit,
                &attack,
                defender_unit,
                &toughness_weaknesses,
                defender_status,
                attacker_dmg_mult,
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
                events.push(CombatEventKind::OnKO);
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
        }
    }

    events.push(CombatEventKind::OnSkillCast {
        skill_id: resolved.skill_id.clone(),
    });

    if resolved.turn_advance_pct != 0 {
        events.push(CombatEventKind::TurnAdvance {
            target: resolved.target,
            amount_pct: resolved.turn_advance_pct,
        });
    }

    if resolved.self_advance_pct != 0 {
        events.push(CombatEventKind::TurnAdvance {
            target: resolved.source,
            amount_pct: resolved.self_advance_pct,
        });
    }

    outcome.sp_ok = true;
    outcome.succeeded = true;
    (outcome, events)
}

#[cfg(test)]
#[path = "resolution_tests.rs"]
mod resolution_tests;
