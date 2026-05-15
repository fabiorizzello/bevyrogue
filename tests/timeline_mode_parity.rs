//! Integration test — I2/D024 (S03): Mode-parity on a BRANCHED timeline.
//!
//! A single-entry Impact beat fires a hook that enqueues `Intent::DealDamage`.
//! Two outgoing edges leave it:
//!   - Edge A (finisher): gated by a predicate that reads `hp_current` from `ctx.world`.
//!     Fires when target HP is below a threshold (deterministic via spawn).
//!   - Edge B (normal): unconditional fallback.
//!
//! Each finisher / normal beat also fires a hook that enqueues a `DealDamage` intent
//! with a distinct amount so the branch taken is observable in the pending stream.
//!
//! The test asserts:
//!   normalize(execute) == normalize(dryrun) == normalize(preview)
//! and that the stream is non-empty and contains the expected finisher amount.
//!
//! A second test case flips the spawned HP so the predicate does NOT fire, routing
//! through the normal branch — proving the predicate is live, not dead.

use bevy::prelude::*;
use bevyrogue::combat::{
    api::{
        CastId, CastIdGen, Intent, IntentQueue,
        applier::intent_applier,
        registry::ExtRegistries,
        runner::{BeatRunner, StepOutcome},
        skill_ctx::{SkillCtx, SkillCtxMode},
        timeline::{Beat, BeatEdge, BeatEvent, BeatKind, CompiledTimeline},
    },
    team::Team,
    types::{Attribute, DamageTag, EvoStage, UnitId},
    unit::Unit,
};
use std::collections::VecDeque;
use std::sync::Arc;

// ─── Fixture constants ────────────────────────────────────────────────────────

const CASTER_ID: UnitId = UnitId(1);
const TARGET_ID: UnitId = UnitId(2);

/// Damage emitted by the entry Impact beat (every run).
const ENTRY_DAMAGE: i32 = 50;
/// Damage emitted by the finisher beat (branch A — low HP).
const FINISHER_DAMAGE: i32 = 200;
/// Damage emitted by the normal beat (branch B — full HP).
const NORMAL_DAMAGE: i32 = 100;
/// HP threshold: target hp_current strictly below this routes to finisher.
const LOW_HP_THRESHOLD: i32 = 30;

// ─── App scaffolding ──────────────────────────────────────────────────────────

fn setup_app() -> App {
    let mut app = App::new();
    app.init_resource::<IntentQueue>()
        .init_resource::<CastIdGen>()
        .add_systems(Update, intent_applier);
    app
}

fn spawn_unit(app: &mut App, id: UnitId, team: Team, hp_current: i32, hp_max: i32) {
    app.world_mut().spawn((
        Unit {
            id,
            name: format!("unit_{}", id.0),
            hp_max,
            hp_current,
            attribute: Attribute::Data,
            resists: vec![],
            evo_stage: EvoStage::Adult,
        },
        team,
    ));
}

// ─── Hook & predicate functions ───────────────────────────────────────────────

/// Entry beat hook: enqueues a fixed `DealDamage` at `ENTRY_DAMAGE`.
fn entry_hook(_ev: &BeatEvent, ctx: &mut SkillCtx<'_>) {
    ctx.enqueue(Intent::DealDamage {
        source: ctx.caster,
        target: ctx.primary_target,
        amount: ENTRY_DAMAGE,
        tag: DamageTag::Physical,
        cast_id: ctx.cast_id,
    });
}

/// Finisher beat hook: enqueues `DealDamage` at `FINISHER_DAMAGE`.
fn finisher_hook(_ev: &BeatEvent, ctx: &mut SkillCtx<'_>) {
    ctx.enqueue(Intent::DealDamage {
        source: ctx.caster,
        target: ctx.primary_target,
        amount: FINISHER_DAMAGE,
        tag: DamageTag::Physical,
        cast_id: ctx.cast_id,
    });
}

/// Normal beat hook: enqueues `DealDamage` at `NORMAL_DAMAGE`.
fn normal_hook(_ev: &BeatEvent, ctx: &mut SkillCtx<'_>) {
    ctx.enqueue(Intent::DealDamage {
        source: ctx.caster,
        target: ctx.primary_target,
        amount: NORMAL_DAMAGE,
        tag: DamageTag::Physical,
        cast_id: ctx.cast_id,
    });
}

/// Edge-gate predicate: reads `hp_current` of `TARGET_ID` from `ctx.world`.
/// Returns `true` (finisher branch) if hp_current < LOW_HP_THRESHOLD.
///
/// `SkillCtx::world` is a `&World` (immutable borrow). `World::try_query` takes
/// `&self` and returns `Option<QueryState>`, which lets us call `QueryState::iter`
/// with the same `&World` — the canonical read-only world query pattern in Bevy 0.18.
fn target_is_low_hp(_ev: &BeatEvent, ctx: &SkillCtx<'_>) -> bool {
    let world = ctx.world;
    let Some(mut qs) = world.try_query::<&Unit>() else {
        return false;
    };
    qs.iter(world)
        .find(|u| u.id == TARGET_ID)
        .map(|u| u.hp_current < LOW_HP_THRESHOLD)
        .unwrap_or(false)
}

// ─── Timeline builder ─────────────────────────────────────────────────────────

/// Build the branched CompiledTimeline:
///
///   entry (Impact/hook=entry_hook)
///     ├─ [gate: target_is_low_hp] ──► finisher (Impact/hook=finisher_hook)
///     └─ [unconditional]           ──► normal   (Impact/hook=normal_hook)
fn build_branched_timeline() -> Arc<CompiledTimeline> {
    Arc::new(CompiledTimeline {
        id: "mode_parity_branched",
        entry: "entry",
        beats: vec![
            Beat {
                id: "entry",
                kind: BeatKind::Impact,
                hook: Some("parity/entry_hook"),
                selector: None,
                presentation: None,
                payload: None,
            },
            Beat {
                id: "finisher",
                kind: BeatKind::Impact,
                hook: Some("parity/finisher_hook"),
                selector: None,
                presentation: None,
                payload: None,
            },
            Beat {
                id: "normal",
                kind: BeatKind::Impact,
                hook: Some("parity/normal_hook"),
                selector: None,
                presentation: None,
                payload: None,
            },
        ],
        edges: vec![
            BeatEdge {
                from: "entry",
                to: "finisher",
                gate: Some("parity/target_is_low_hp"),
            },
            BeatEdge {
                from: "entry",
                to: "normal",
                gate: None, // unconditional fallback
            },
        ],
    })
}

fn build_regs() -> ExtRegistries {
    let mut regs = ExtRegistries::default();
    regs.hooks.register("parity/entry_hook", entry_hook);
    regs.hooks.register("parity/finisher_hook", finisher_hook);
    regs.hooks.register("parity/normal_hook", normal_hook);
    regs.predicates.register("parity/target_is_low_hp", target_is_low_hp);
    regs
}

// ─── Normalization helper ─────────────────────────────────────────────────────

/// Normalize an Intent stream to a structural, cast-id-independent representation.
///
/// `cast_id` is intentionally stripped so that Execute/DryRun/Preview runs — each
/// issued a distinct `CastId` — can be compared for structural equality.
/// Format: `"DealDamage(src=N,tgt=N,amt=N,tag=…)"`.
fn normalize(p: &VecDeque<Intent>) -> Vec<String> {
    p.iter()
        .map(|i| match i {
            Intent::DealDamage { source, target, amount, tag, .. } => {
                format!("DealDamage(src={:?},tgt={:?},amt={},tag={:?})", source, target, amount, tag)
            }
            other => format!("{:?}", other),
        })
        .collect()
}

// ─── Shared runner helper ─────────────────────────────────────────────────────

/// Run the branched timeline in the given mode on `world`, return the pending queue.
fn run_mode(
    world: &mut World,
    regs: &ExtRegistries,
    timeline: Arc<CompiledTimeline>,
    cast_id: CastId,
    mode: SkillCtxMode,
) -> VecDeque<Intent> {
    let mut pending = VecDeque::new();
    let mut runner = BeatRunner::new(timeline, cast_id, CASTER_ID, TARGET_ID);
    let outcome =
        runner.run_to_completion(world, regs, mode, &mut pending, 64);
    assert_eq!(
        outcome,
        StepOutcome::Done,
        "timeline must finish normally (mode={:?})",
        mode
    );
    pending
}

// ─── Test case 1: LOW HP — routes through finisher branch ────────────────────

#[test]
fn mode_parity_execute_dryrun_preview_match_on_finisher_branch() {
    // Spawn target with hp_current=10, which is < LOW_HP_THRESHOLD(30) → finisher branch.
    let mut app = setup_app();
    spawn_unit(&mut app, CASTER_ID, Team::Ally, 500, 500);
    spawn_unit(&mut app, TARGET_ID, Team::Enemy, 10, 100);

    let regs = build_regs();
    let timeline = build_branched_timeline();

    // Allocate a fresh cast_id per run so modes don't share identity.
    let cast_id_exec = app.world_mut().resource_mut::<CastIdGen>().next();
    let cast_id_dry = app.world_mut().resource_mut::<CastIdGen>().next();
    let cast_id_prev = app.world_mut().resource_mut::<CastIdGen>().next();

    let pending_exec = run_mode(
        app.world_mut(),
        &regs,
        Arc::clone(&timeline),
        cast_id_exec,
        SkillCtxMode::Execute,
    );
    let pending_dry = run_mode(
        app.world_mut(),
        &regs,
        Arc::clone(&timeline),
        cast_id_dry,
        SkillCtxMode::DryRun,
    );
    let pending_prev = run_mode(
        app.world_mut(),
        &regs,
        Arc::clone(&timeline),
        cast_id_prev,
        SkillCtxMode::Preview,
    );

    // Primary parity assertion: full structural stream (cast_id stripped by normalize).
    let norm_exec = normalize(&pending_exec);
    let norm_dry = normalize(&pending_dry);
    let norm_prev = normalize(&pending_prev);
    assert_eq!(
        norm_exec, norm_dry,
        "Execute and DryRun must produce identical normalized Intent streams"
    );
    assert_eq!(
        norm_exec, norm_prev,
        "Execute and Preview must produce identical normalized Intent streams"
    );

    // Branch verification: extract damage amounts from the Execute stream.
    fn norm_amounts(p: &VecDeque<Intent>) -> Vec<i32> {
        p.iter()
            .filter_map(|i| match i {
                Intent::DealDamage { amount, .. } => Some(*amount),
                _ => None,
            })
            .collect()
    }
    let exec_amounts = norm_amounts(&pending_exec);

    // Stream must be non-empty and contain the finisher amount (not normal amount).
    assert!(
        !exec_amounts.is_empty(),
        "intent stream must be non-empty"
    );
    assert!(
        exec_amounts.contains(&FINISHER_DAMAGE),
        "finisher branch must be taken (hp=10 < threshold={}); got: {:?}",
        LOW_HP_THRESHOLD,
        exec_amounts
    );
    assert!(
        !exec_amounts.contains(&NORMAL_DAMAGE),
        "normal branch must NOT be taken when finisher branch fires; got: {:?}",
        exec_amounts
    );
    // Entry damage must always be present.
    assert!(
        exec_amounts.contains(&ENTRY_DAMAGE),
        "entry beat damage must always appear; got: {:?}",
        exec_amounts
    );
    // Expected stream: [ENTRY_DAMAGE, FINISHER_DAMAGE]
    assert_eq!(
        exec_amounts,
        vec![ENTRY_DAMAGE, FINISHER_DAMAGE],
        "expected exactly [entry, finisher] damage stream; got: {:?}",
        exec_amounts
    );
}

// ─── Test case 2: FULL HP — routes through normal branch ──────────────────────

#[test]
fn mode_parity_execute_dryrun_preview_match_on_normal_branch() {
    // Spawn target with hp_current=100 = hp_max, which is >= LOW_HP_THRESHOLD(30) → normal branch.
    let mut app = setup_app();
    spawn_unit(&mut app, CASTER_ID, Team::Ally, 500, 500);
    spawn_unit(&mut app, TARGET_ID, Team::Enemy, 100, 100);

    let regs = build_regs();
    let timeline = build_branched_timeline();

    let cast_id_exec = app.world_mut().resource_mut::<CastIdGen>().next();
    let cast_id_dry = app.world_mut().resource_mut::<CastIdGen>().next();
    let cast_id_prev = app.world_mut().resource_mut::<CastIdGen>().next();

    let pending_exec = run_mode(
        app.world_mut(),
        &regs,
        Arc::clone(&timeline),
        cast_id_exec,
        SkillCtxMode::Execute,
    );
    let pending_dry = run_mode(
        app.world_mut(),
        &regs,
        Arc::clone(&timeline),
        cast_id_dry,
        SkillCtxMode::DryRun,
    );
    let pending_prev = run_mode(
        app.world_mut(),
        &regs,
        Arc::clone(&timeline),
        cast_id_prev,
        SkillCtxMode::Preview,
    );

    // Primary parity assertion.
    let norm_exec = normalize(&pending_exec);
    let norm_dry = normalize(&pending_dry);
    let norm_prev = normalize(&pending_prev);
    assert_eq!(
        norm_exec, norm_dry,
        "Execute and DryRun must produce identical normalized Intent streams on normal branch"
    );
    assert_eq!(
        norm_exec, norm_prev,
        "Execute and Preview must produce identical normalized Intent streams on normal branch"
    );

    // Branch verification.
    fn norm_amounts(p: &VecDeque<Intent>) -> Vec<i32> {
        p.iter()
            .filter_map(|i| match i {
                Intent::DealDamage { amount, .. } => Some(*amount),
                _ => None,
            })
            .collect()
    }
    let exec_amounts = norm_amounts(&pending_exec);

    assert!(
        !exec_amounts.is_empty(),
        "intent stream must be non-empty"
    );
    assert!(
        exec_amounts.contains(&NORMAL_DAMAGE),
        "normal branch must be taken (hp=100 >= threshold={}); got: {:?}",
        LOW_HP_THRESHOLD,
        exec_amounts
    );
    assert!(
        !exec_amounts.contains(&FINISHER_DAMAGE),
        "finisher branch must NOT be taken when target is at full HP; got: {:?}",
        exec_amounts
    );
    assert!(
        exec_amounts.contains(&ENTRY_DAMAGE),
        "entry beat damage must always appear; got: {:?}",
        exec_amounts
    );
    // Expected stream: [ENTRY_DAMAGE, NORMAL_DAMAGE]
    assert_eq!(
        exec_amounts,
        vec![ENTRY_DAMAGE, NORMAL_DAMAGE],
        "expected exactly [entry, normal] damage stream; got: {:?}",
        exec_amounts
    );
}
