//! Relocated from `src/combat/runtime/registry.rs` (R003 — no inline `mod tests` in src/).
//! Pure relocate: every symbol the tests touch is already `pub`.

use bevyrogue::combat::{
    StatusEffectKind,
    events::CombatKernelTransition,
    runtime::registry::{ExtPoint, ExtRegistries, Registry},
    runtime::{
        CastId, Intent, PostActionContext, PostActionQueue, PostActionUnitDied,
        PostActionUnitSnapshot, SignalPayload, dispatch_post_action_reactions,
    },
    team::Team,
    types::{SkillId, UnitId},
};

/// Minimal test axis: fn pointer returning u32.
struct NumExt;
impl ExtPoint for NumExt {
    type Fn = fn() -> u32;
}

#[test]
fn registry_hit() {
    let mut reg: Registry<NumExt> = Registry::new();
    reg.register("answer", || 42u32);
    let f = reg.get("answer").expect("registered id must resolve");
    assert_eq!(f(), 42);
}

#[test]
fn registry_miss() {
    let reg: Registry<NumExt> = Registry::new();
    assert!(reg.get("nonexistent").is_none());
}

#[test]
fn registry_overwrite() {
    let mut reg: Registry<NumExt> = Registry::new();
    reg.register("v", || 1u32);
    reg.register("v", || 2u32);
    assert_eq!(reg.get("v").unwrap()(), 2);
}

#[test]
fn ext_registries_default_empty() {
    let r = ExtRegistries::default();
    assert!(r.hooks.is_empty());
    assert!(r.selectors.is_empty());
    assert!(r.predicates.is_empty());
    assert!(r.formulas.is_empty());
    assert!(r.ticks.is_empty());
    assert!(r.ai_utilities.is_empty());
    assert!(r.cues.is_empty());
    assert!(r.pre_damage_reactions.is_empty());
    assert!(r.post_action_reactions.is_empty());
    assert!(r.validation.is_empty());
}

fn queue_damage_and_flash(ctx: &PostActionContext, out: &mut PostActionQueue) {
    if ctx.skill_id != SkillId("agumon_ult".into()) {
        return;
    }
    let Some(unit_died) = &ctx.unit_died else {
        return;
    };
    if unit_died.heated_remaining == 0 {
        return;
    }

    out.push_intent(Intent::DealDamage {
        source: ctx.source,
        target: UnitId(3),
        amount: 12,
        tag: bevyrogue::combat::types::DamageTag::Fire,
        cast_id: ctx.cast_id,
    });
    out.push_transition(CombatKernelTransition::Blueprint {
        owner: "agumon".to_string(),
        name: "baby_burner_detonate".to_string(),
        payload: SignalPayload::Amount(i64::from(unit_died.heated_remaining)),
    });
}

#[test]
fn post_action_dispatch_collects_registered_outputs() {
    let mut regs = ExtRegistries::default();
    regs.post_action_reactions
        .register("test/post_action", queue_damage_and_flash);

    let ctx = PostActionContext::new(
        SkillId("agumon_ult".into()),
        UnitId(1),
        UnitId(2),
        CastId::ROOT,
        0,
        Some(PostActionUnitDied::new(
            vec![StatusEffectKind::Heated, StatusEffectKind::Slowed],
            2,
        )),
        vec![
            PostActionUnitSnapshot::new(UnitId(1), Team::Ally, Some(0), 120, 120, true),
            PostActionUnitSnapshot::new(UnitId(2), Team::Enemy, Some(1), 0, 100, false),
            PostActionUnitSnapshot::new(UnitId(3), Team::Enemy, Some(2), 80, 100, true),
        ],
    );

    let out = dispatch_post_action_reactions(&regs, &ctx);
    assert_eq!(out.intents.len(), 1);
    assert_eq!(out.transitions.len(), 1);
    assert!(matches!(
        out.intents.first(),
        Some(Intent::DealDamage {
            source,
            target,
            amount: 12,
            tag: bevyrogue::combat::types::DamageTag::Fire,
            cast_id,
        }) if *source == UnitId(1) && *target == UnitId(3) && *cast_id == CastId::ROOT
    ));
    assert!(matches!(
        out.transitions.first(),
        Some(CombatKernelTransition::Blueprint { owner, name, payload })
            if owner == "agumon"
                && name == "baby_burner_detonate"
                && *payload == SignalPayload::Amount(2)
    ));
}

#[test]
fn post_action_dispatch_is_noop_when_registry_empty() {
    let ctx = PostActionContext::new(
        SkillId("agumon_ult".into()),
        UnitId(1),
        UnitId(2),
        CastId::ROOT,
        1,
        Some(PostActionUnitDied::new(vec![StatusEffectKind::Heated], 1)),
        vec![
            PostActionUnitSnapshot::new(UnitId(1), Team::Ally, Some(0), 120, 120, true),
            PostActionUnitSnapshot::new(UnitId(2), Team::Enemy, Some(1), 0, 100, false),
        ],
    );

    let out = dispatch_post_action_reactions(&ExtRegistries::default(), &ctx);
    assert!(out.is_empty());
}
