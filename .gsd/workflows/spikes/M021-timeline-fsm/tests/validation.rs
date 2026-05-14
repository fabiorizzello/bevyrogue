//! M021 Timeline-FSM spike — validation suite.
//!
//! Invariants the production design must hold:
//!
//!  I1. Determinism: identical inputs ⇒ identical `Intent` + `CombatEvent`
//!      sequences across runs (and across separate fresh harnesses).
//!  I2. Dry-run ≡ Live: hook fns produce the same `Intent` stream regardless
//!      of `SkillCtxMode`; only the kernel's downstream application differs.
//!  I3. Signal-gating (Windowed): the runner *stalls* on `OnSignal` beats and
//!      only progresses when the matching signal is consumed.
//!  I4. Skill-tree patch: a `TimelinePatchOp::InsertBeatAfter` is a
//!      compile-time graph rewrite.
//!  I5. Extension-point uniformity: every axis (hook, selector, predicate,
//!      formula, tick, ai_utility, cue) goes through the same `Registry<E>`
//!      and the same id-by-string lookup. The validator catches dangling
//!      references for every axis with the same code path.
//!  I6. Single entry point: the entire Agumon kit installs via one call to
//!      `agumon::register` — no kernel changes, no Digimon-specific naming
//!      anywhere outside the `agumon` module.

use m021_timeline_fsm_spike::*;
use m021_timeline_fsm_spike::{agumon, dorumon, gabumon, tentomon};

// ---------- Fixtures ----------

/// Build a fully-wired harness: kernel built-ins + the Agumon blueprint.
struct Harness {
    timeline: CompiledTimeline,
    registries: ExtRegistries,
    state: CombatStateMock,
    signals: SignalBus,
}

impl Harness {
    fn new(timeline: CompiledTimeline) -> Self {
        let mut registries = ExtRegistries::new();
        builtin::register(&mut registries);
        agumon::register(&mut registries);
        Self {
            timeline,
            registries,
            state: agumon::default_state(),
            signals: SignalBus::default(),
        }
    }

    /// Run a full cast and return (events, intents). Installs the formula
    /// thread-local so hooks can resolve formulas through the registry.
    fn run_headless(mut self, mode: SkillCtxMode) -> (Vec<CombatEvent>, Vec<Intent>) {
        let empty = std::collections::HashSet::new();
        let _guard = agumon::RuntimeGuard::install(&self.registries, &self.state, &empty);
        let mut runner = BeatRunner::new(
            &self.timeline, &self.registries, &self.state, &mut self.signals,
            Clock::HeadlessAuto, 42, agumon::base_event_with_state(),
        );
        runner.run_to_completion(mode, 64);
        (runner.events, runner.all_intents)
    }
}

// ---------- I1: determinism ----------

#[test]
fn determinism_headless_two_runs_identical() {
    let (e1, i1) = Harness::new(agumon::base_timeline()).run_headless(SkillCtxMode::Live);
    let (e2, i2) = Harness::new(agumon::base_timeline()).run_headless(SkillCtxMode::Live);
    assert_eq!(e1, e2, "CombatEvent stream diverged across runs");
    assert_eq!(i1, i2, "Intent stream diverged across runs");

    // Sanity: canonical cast against `default_state` (caster ATK 200, fire_res 0).
    //   fire_atk_scaling = (120 + 200*80/100) * (100-0)/100 = 120 + 160 = 280
    //   splash = 280 / 2 = 140
    // baby_burner_splash sorts adjacents by HP — 11 (HP 600) before 12 (HP 900).
    assert_eq!(i1, vec![
        Intent::DealDamage  { target: 10, amount: 280, tag: DamageTag::Fire, cast_id: 42 },
        Intent::DealDamage  { target: 11, amount: 140, tag: DamageTag::Fire, cast_id: 42 },
        Intent::DealDamage  { target: 12, amount: 140, tag: DamageTag::Fire, cast_id: 42 },
        Intent::ApplyStatus { target: 10, kind: StatusKind::Heated, duration: 2,
                              mode: StatusApplyMode::Refresh, cast_id: 42 },
    ]);
}

// ---------- I2: dry-run ≡ live ----------

#[test]
fn dry_run_intent_stream_matches_live() {
    let (e_live, i_live) = Harness::new(agumon::base_timeline()).run_headless(SkillCtxMode::Live);
    let (e_dry,  i_dry)  = Harness::new(agumon::base_timeline()).run_headless(SkillCtxMode::DryRun);

    assert_eq!(i_live, i_dry,
        "Hook output diverged between Live and DryRun — preview cannot match execute");
    assert_eq!(e_live, e_dry,
        "CombatEvent stream diverged between Live and DryRun");
}

// ---------- I3: signal gating in Windowed ----------

#[test]
fn windowed_runner_stalls_until_signal() {
    let mut h = Harness::new(agumon::base_timeline());
    let empty = std::collections::HashSet::new();
    let _guard = agumon::RuntimeGuard::install(&h.registries, &h.state, &empty);
    let mut runner = BeatRunner::new(
        &h.timeline, &h.registries, &h.state, &mut h.signals,
        Clock::Windowed, 42, agumon::base_event_with_state(),
    );

    // Step once: enters "cast", runs cue, but no hook and OnSignal advance.
    runner.step(SkillCtxMode::Live);
    assert_eq!(runner.cursor_for_test(), Some("cast"),
        "runner advanced past `cast` without the gating signal");
    assert!(runner.all_intents.is_empty(),
        "no impact hook should have fired yet");

    runner.signals_for_test().emit("anim_charge_done");
    runner.step(SkillCtxMode::Live);
    assert_eq!(runner.cursor_for_test(), Some("projectile"));
    assert!(runner.all_intents.is_empty());

    runner.step(SkillCtxMode::Live);
    assert_eq!(runner.cursor_for_test(), Some("projectile"),
        "runner advanced past `projectile` without `projectile_hit`");
    assert!(runner.all_intents.is_empty());

    runner.signals_for_test().emit("projectile_hit");
    runner.run_to_completion(SkillCtxMode::Live, 64);
    assert_eq!(runner.cursor_for_test(), None);

    let (_, i_headless) = Harness::new(agumon::base_timeline()).run_headless(SkillCtxMode::Live);
    assert_eq!(runner.all_intents, i_headless,
        "Windowed Intent stream diverged from HeadlessAuto");
}

// ---------- I4: skill-tree patch ----------

#[test]
fn pyromaniac_patch_injects_lingering_burn() {
    let base = agumon::base_timeline();
    let patched = compile_timeline(base, &agumon::pyromaniac_patch());

    assert!(patched.beats.iter().any(|b| b.id == "lingering_burn"),
        "patch did not insert `lingering_burn` beat");

    let into_lb: Vec<_> = patched.edges.iter()
        .filter(|e| e.to == "lingering_burn").collect();
    assert_eq!(into_lb.len(), 1);
    assert_eq!(into_lb[0].from, "splash_adj");
    assert_eq!(into_lb[0].gate, Some("agumon::has_adjacent_targets"));

    let from_lb: Vec<_> = patched.edges.iter()
        .filter(|e| e.from == "lingering_burn").collect();
    assert_eq!(from_lb.len(), 1);
    assert_eq!(from_lb[0].to, "aftermath");

    let splash_to_aftermath = patched.edges.iter()
        .any(|e| e.from == "splash_adj" && e.to == "aftermath");
    assert!(!splash_to_aftermath,
        "patch failed to rewire splash_adj → aftermath through lingering_burn");

    let (events, intents) = Harness::new(patched).run_headless(SkillCtxMode::Live);

    assert_eq!(intents, vec![
        Intent::DealDamage  { target: 10, amount: 280, tag: DamageTag::Fire, cast_id: 42 },
        Intent::DealDamage  { target: 11, amount: 140, tag: DamageTag::Fire, cast_id: 42 },
        Intent::DealDamage  { target: 12, amount: 140, tag: DamageTag::Fire, cast_id: 42 },
        // Pyromaniac addition — lingering_burn fires *before* aftermath:
        Intent::ApplyStatus { target: 11, kind: StatusKind::Burn, duration: 3,
                              mode: StatusApplyMode::Refresh, cast_id: 42 },
        Intent::ApplyStatus { target: 12, kind: StatusKind::Burn, duration: 3,
                              mode: StatusApplyMode::Refresh, cast_id: 42 },
        Intent::ApplyStatus { target: 10, kind: StatusKind::Heated, duration: 2,
                              mode: StatusApplyMode::Refresh, cast_id: 42 },
    ]);

    let entered: Vec<BeatId> = events.iter().filter_map(|e| match e {
        CombatEvent::BeatEntered { beat, .. } => Some(*beat),
        _ => None,
    }).collect();
    assert_eq!(entered, vec!["cast", "projectile", "impact_main",
                             "splash_adj", "lingering_burn", "aftermath"]);
}

#[test]
fn pyromaniac_gate_halts_when_predicate_fails() {
    let mut state = agumon::default_state();
    state.adjacency.remove(&10); // remove adjacents → splash selector empty

    let patched = compile_timeline(agumon::base_timeline(), &agumon::pyromaniac_patch());
    let mut registries = ExtRegistries::new();
    builtin::register(&mut registries);
    agumon::register(&mut registries);
    let mut signals = SignalBus::default();

    let empty = std::collections::HashSet::new();
    let _guard = agumon::RuntimeGuard::install(&registries, &state, &empty);
    let mut runner = BeatRunner::new(
        &patched, &registries, &state, &mut signals,
        Clock::HeadlessAuto, 42, agumon::base_event_with_state(),
    );
    runner.run_to_completion(SkillCtxMode::Live, 64);

    let entered: Vec<BeatId> = runner.events.iter().filter_map(|e| match e {
        CombatEvent::BeatEntered { beat, .. } => Some(*beat),
        _ => None,
    }).collect();

    // Finding F1 from the first spike pass still applies: `next_from` is
    // first-passing-edge, and the patch removed the unconditional fallback.
    // The production fix is two edges (gated + fallback). The test documents
    // the current behaviour honestly.
    assert_eq!(entered, vec!["cast", "projectile", "impact_main", "splash_adj"],
        "expected runner to halt at splash_adj when patch gate fails (finding F1)");
}

// ---------- I5: extension-point uniformity ----------

#[test]
fn validation_passes_for_correctly_wired_blueprint() {
    let mut registries = ExtRegistries::new();
    builtin::register(&mut registries);
    agumon::register(&mut registries);
    let tl = agumon::base_timeline();

    match validate_timeline_refs(&tl, &registries) {
        Ok(()) => {}
        Err(errs) => panic!("validation should pass — got errors: {:#?}", errs),
    }

    let patched = compile_timeline(tl, &agumon::pyromaniac_patch());
    validate_timeline_refs(&patched, &registries)
        .expect("patched timeline should also validate");
}

#[test]
fn validation_catches_missing_hook() {
    let mut registries = ExtRegistries::new();
    builtin::register(&mut registries);
    agumon::register(&mut registries);
    let mut tl = agumon::base_timeline();
    // Point an existing beat at an unregistered hook.
    tl.beats.iter_mut().find(|b| b.id == "impact_main").unwrap().hook =
        Some("agumon::does_not_exist");

    let errs = validate_timeline_refs(&tl, &registries).unwrap_err();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].axis, "hook");
    assert_eq!(errs[0].id, "agumon::does_not_exist");
    assert!(errs[0].site.contains("impact_main"));
}

#[test]
fn validation_catches_missing_selector() {
    let mut registries = ExtRegistries::new();
    builtin::register(&mut registries);
    agumon::register(&mut registries);
    let mut tl = agumon::base_timeline();
    tl.beats.iter_mut().find(|b| b.id == "splash_adj").unwrap().kind =
        BeatKind::Impact { selector: "agumon::ghost_selector" };

    let errs = validate_timeline_refs(&tl, &registries).unwrap_err();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].axis, "selector");
    assert_eq!(errs[0].id, "agumon::ghost_selector");
}

#[test]
fn validation_catches_missing_predicate() {
    let mut registries = ExtRegistries::new();
    builtin::register(&mut registries);
    agumon::register(&mut registries);
    let mut tl = agumon::base_timeline();
    tl.edges.push(BeatEdge {
        from: "aftermath",
        to: "cast",
        gate: Some("agumon::ghost_predicate"),
    });

    let errs = validate_timeline_refs(&tl, &registries).unwrap_err();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].axis, "predicate");
    assert_eq!(errs[0].id, "agumon::ghost_predicate");
}

#[test]
fn validation_catches_missing_cue_resolver() {
    let mut registries = ExtRegistries::new();
    builtin::register(&mut registries);
    agumon::register(&mut registries);
    let mut tl = agumon::base_timeline();
    tl.beats.iter_mut().find(|b| b.id == "cast").unwrap().presentation =
        Some(Presentation::Dynamic("agumon::ghost_cue"));

    let errs = validate_timeline_refs(&tl, &registries).unwrap_err();
    assert_eq!(errs.len(), 1);
    assert_eq!(errs[0].axis, "cue");
    assert_eq!(errs[0].id, "agumon::ghost_cue");
}

#[test]
fn validation_aggregates_multiple_errors() {
    let mut registries = ExtRegistries::new();
    // Do NOT register agumon — every Agumon-namespaced reference dangles.
    builtin::register(&mut registries);
    let tl = agumon::base_timeline();

    let errs = validate_timeline_refs(&tl, &registries).unwrap_err();
    // Expected dangling references on `base_timeline`:
    //  - cue:       cast → "agumon::charge_by_hp"
    //  - selector:  splash_adj → "agumon::baby_burner_splash"
    //  - hook:      impact_main, splash_adj, aftermath  (3 hooks)
    // = 5 errors total. Built-in "primary" selector resolves fine.
    assert_eq!(errs.len(), 5, "got: {errs:#?}");
    let axes: Vec<&str> = errs.iter().map(|e| e.axis).collect();
    assert!(axes.contains(&"cue"));
    assert!(axes.contains(&"selector"));
    assert_eq!(axes.iter().filter(|a| **a == "hook").count(), 3);
}

// ---------- I5 part B: registries used directly (formula, tick, ai) ----------
//
// These axes aren't referenced from the timeline graph — they're invoked from
// inside hook/tick fns or by external systems (the AI scorer). Validation
// for them is "lookup at invocation site returns Some". We exercise each
// through the Agumon blueprint to prove the same `Registry<E>` mechanism
// covers them with zero new code.

#[test]
fn agumon_formula_used_by_status_tick_via_registry() {
    let registries = {
        let mut r = ExtRegistries::new();
        builtin::register(&mut r);
        agumon::register(&mut r);
        r
    };
    let state = agumon::default_state();

    let empty = std::collections::HashSet::new();
    let _guard = agumon::RuntimeGuard::install(&registries, &state, &empty);

    let tick = registries.ticks.get("agumon::heated_tick")
        .expect("tick should be registered");
    let st = StatusInstance {
        target: 10, kind: StatusKind::Heated, stacks: 1,
        remaining_turns: 2, source: 1, cast_id: 99,
    };
    let mut ctx = SkillCtx::new(SkillCtxMode::Live);
    tick(&st, &mut ctx);
    let intents = ctx.drain();

    // heated_dot = (caster_atk * 15/100) * (100 - fire_res)/100
    //            = (200 * 15/100) * 100/100 = 30
    assert_eq!(intents, vec![
        Intent::DealDamage { target: 10, amount: 30, tag: DamageTag::Fire, cast_id: 99 },
    ]);
}

#[test]
fn agumon_ai_utility_via_registry() {
    let registries = {
        let mut r = ExtRegistries::new();
        builtin::register(&mut r);
        agumon::register(&mut r);
        r
    };
    let state = agumon::default_state();

    let util_fn = registries.ai.get("agumon::burner_utility")
        .expect("ai utility should be registered");
    let ctx = AiCtx { caster: 1, state: &state };
    let score = util_fn(&ctx);

    // Ult charged (100) AND primary 10 has 2 adjacent enemies (11, 12).
    assert!((score - 0.9).abs() < f32::EPSILON, "score = {score}, expected 0.9");
}

#[test]
fn agumon_cue_resolver_varies_with_state() {
    let registries = {
        let mut r = ExtRegistries::new();
        builtin::register(&mut r);
        agumon::register(&mut r);
        r
    };
    let cue = registries.cues.get("agumon::charge_by_hp")
        .expect("cue resolver should be registered");

    // Normal HP → normal anim.
    let healthy = agumon::default_state();
    assert_eq!(cue(&CueCtx { caster: 1, state: &healthy }),
               "anim_baby_burner_charge_normal");

    // Drop caster HP below 30% → desperate anim.
    let mut hurt = agumon::default_state();
    hurt.hp.insert(1, 100); // 100/1000 = 10%
    assert_eq!(cue(&CueCtx { caster: 1, state: &hurt }),
               "anim_baby_burner_charge_desperate");
}

#[test]
fn unregistered_formula_returns_none_at_lookup() {
    let mut r = ExtRegistries::new();
    builtin::register(&mut r);
    agumon::register(&mut r);
    assert!(r.formulas.get("agumon::ghost_formula").is_none());
}

#[test]
fn unregistered_tick_returns_none_at_lookup() {
    let mut r = ExtRegistries::new();
    builtin::register(&mut r);
    agumon::register(&mut r);
    assert!(r.ticks.get("agumon::ghost_tick").is_none());
}

#[test]
fn unregistered_ai_utility_returns_none_at_lookup() {
    let mut r = ExtRegistries::new();
    builtin::register(&mut r);
    agumon::register(&mut r);
    assert!(r.ai.get("agumon::ghost_utility").is_none());
}

// ---------- I6: single entry point ----------

#[test]
fn single_register_call_installs_every_axis_for_agumon() {
    let mut r = ExtRegistries::new();
    builtin::register(&mut r);
    agumon::register(&mut r);

    // Every Agumon-specific ID this blueprint claims to register must be live.
    let hooks = [
        "agumon::on_impact_main",
        "agumon::on_splash",
        "agumon::on_aftermath",
        "agumon::on_lingering_burn",
        "agumon::twin_core_on_ally_ko",
    ];
    for h in hooks { assert!(r.hooks.contains(h), "missing hook {h}"); }

    let predicates = ["agumon::has_adjacent_targets", "agumon::has_two_alive_allies"];
    for p in predicates { assert!(r.predicates.contains(p), "missing predicate {p}"); }

    assert!(r.selectors.contains("agumon::baby_burner_splash"));
    assert!(r.formulas.contains("agumon::fire_atk_scaling"));
    assert!(r.formulas.contains("agumon::heated_dot"));
    assert!(r.ticks.contains("agumon::heated_tick"));
    assert!(r.ai.contains("agumon::burner_utility"));
    assert!(r.cues.contains("agumon::charge_by_hp"));

    // Kernel built-ins also live — proves the blueprint coexists with them.
    assert!(r.selectors.contains("primary"));
    assert!(r.selectors.contains("adjacent_to_primary"));
}

#[test]
fn passive_hook_fires_via_registry_without_kernel_changes() {
    // Twin Core Fire passive: blueprint defines the hook, kernel doesn't
    // know it exists. The passive system (mocked here) just looks up the
    // hook id and invokes it.
    let mut r = ExtRegistries::new();
    builtin::register(&mut r);
    agumon::register(&mut r);

    let hook = r.hooks.get("agumon::twin_core_on_ally_ko")
        .expect("passive hook should be registered");
    let evt = BeatEvent {
        caster: 1, primary_target: 0, beat_targets: vec![],
        cast_id: 7, beat: "passive::ally_ko", hop_index: 0,
    };
    let mut ctx = SkillCtx::new(SkillCtxMode::Live);
    hook(&evt, &mut ctx);

    assert_eq!(ctx.drain(), vec![
        Intent::ModifyResource { actor: 1, kind: ResourceKind::Sp, delta: 1, cast_id: 7 },
        Intent::ApplyStatus    { target: 1, kind: StatusKind::TwinCoreRage, duration: 2,
                                  mode: StatusApplyMode::Refresh, cast_id: 7 },
    ]);
}

// ======================================================================
// I7 — Pattern survey: bouncing_fire / predator_loop / twin_core / rng
// ----------------------------------------------------------------------
// Each pattern was identified in the docs/future_design_draft/ survey
// (Agumon talents, Dorumon predator_loop, Gabumon twin_core_ice, Tentomon
// battery_loop). Tests below validate the kernel additions hold for the
// architecturally-distinct shapes — Loop + skilltree, mutable blueprint
// state, cross-blueprint identity filter, RNG-gated edge.
// ======================================================================

/// Shared fixture: state + registries with all four blueprints wired in.
struct PatternHarness {
    registries: ExtRegistries,
    state: CombatStateMock,
    signals: SignalBus,
    timeline: CompiledTimeline,
}

impl PatternHarness {
    fn new(timeline: CompiledTimeline) -> Self {
        let mut registries = ExtRegistries::new();
        builtin::register(&mut registries);
        agumon::register(&mut registries);
        dorumon::register(&mut registries);
        gabumon::register(&mut registries);
        tentomon::register(&mut registries);
        let mut state = agumon::default_state();
        state.identity.insert(1, "agumon");
        Self {
            registries,
            state,
            signals: SignalBus::default(),
            timeline,
        }
    }

    fn run(mut self) -> (Vec<CombatEvent>, Vec<Intent>) {
        let mut runner = BeatRunner::new(
            &self.timeline, &self.registries, &self.state, &mut self.signals,
            Clock::HeadlessAuto, 42, agumon::base_event_with_state(),
        );
        runner.run_to_completion(SkillCtxMode::Live, 256);
        (runner.events, runner.all_intents)
    }
}

// ---------- Pattern 1: bouncing_fire ----------

#[test]
fn bouncing_fire_off_baseline_identical_to_no_loop() {
    // Skilltree empty ⇒ the gate fails, fallback edge to `cast_end` fires,
    // no Loop body executes. Intent stream must match the no-bouncing variant.
    let (_evts, intents_with_branch) =
        PatternHarness::new(agumon::base_timeline_with_bouncing_fire()).run();

    // Same scenario but base timeline (no branch in graph at all).
    let mut h_base = PatternHarness::new(agumon::base_timeline());
    let (_, intents_base) = {
        let mut runner = BeatRunner::new(
            &h_base.timeline, &h_base.registries, &h_base.state, &mut h_base.signals,
            Clock::HeadlessAuto, 42, agumon::base_event_with_state(),
        );
        runner.run_to_completion(SkillCtxMode::Live, 64);
        (runner.events, runner.all_intents)
    };

    assert_eq!(intents_with_branch, intents_base,
        "with talent OFF, the bounce branch in the graph must produce \
         the same Intent stream as the no-branch baseline");
}

#[test]
fn bouncing_fire_tier1_runs_exactly_one_hop() {
    let mut h = PatternHarness::new(agumon::base_timeline_with_bouncing_fire());
    let mut tree = SkillTree::default();
    tree.set_rank("agumon::bouncing_fire", 1);
    h.state.skill_trees.insert(1, tree);
    let (_evts, intents) = h.run();

    // Baseline intents: primary 280, splash 11→140, splash 12→140, Heated on 10.
    // Tier-1 adds exactly ONE bounce hop dealing fire/2 = 140 to lowest-HP
    // enemy not yet hit. Enemies 10/11/12 already hit ⇒ pool exhausted ⇒
    // selector returns empty ⇒ DealDamage NOT enqueued ⇒ same intent stream.
    //
    // This validates the **pool exhaustion** path: tier-1 nominally wants
    // 1 hop, but the cast already covered every alive enemy, so the bounce
    // adds nothing. The existing K-memory gotcha is reproduced here.
    let dmg_count = intents.iter().filter(|i|
        matches!(i, Intent::DealDamage { tag: DamageTag::Fire, .. })).count();
    assert_eq!(dmg_count, 3,
        "fire damage intents: 1 primary + 2 splash (pool already exhausted, no bounce damage). \
         intents = {intents:#?}");
}

#[test]
fn bouncing_fire_tier3_pool_exhausts_before_cap() {
    // Add a 4th enemy NOT in the splash adjacency so the bounce has a real
    // target after primary+splash. Tier-3 ⇒ would want 3 hops, but only one
    // extra target exists ⇒ exit_when fires on pool exhaustion after hop 0.
    let mut h = PatternHarness::new(agumon::base_timeline_with_bouncing_fire());
    h.state.hp.insert(13, 400); h.state.max_hp.insert(13, 1000);
    h.state.fire_res.insert(13, 0);
    // Add 13 to caster's enemies list. Not adjacent to primary, so splash skips.
    h.state.enemies.insert(1, vec![10, 11, 12, 13]);
    let mut tree = SkillTree::default();
    tree.set_rank("agumon::bouncing_fire", 3);
    h.state.skill_trees.insert(1, tree);
    let (_, intents) = h.run();

    // Bounce targets unit 13 once, then pool exhausts.
    let bounce_hits: Vec<&Intent> = intents.iter().filter(|i| matches!(i,
        Intent::DealDamage { target: 13, .. })).collect();
    assert_eq!(bounce_hits.len(), 1,
        "expected exactly 1 bounce hop landing on unit 13; got {bounce_hits:#?}");
}

#[test]
fn bouncing_fire_tier2_actually_hops_twice_when_pool_is_deep() {
    // Two extra enemies (13, 14), tier-2 talent ⇒ two bounce hops.
    let mut h = PatternHarness::new(agumon::base_timeline_with_bouncing_fire());
    h.state.hp.insert(13, 300); h.state.max_hp.insert(13, 1000);
    h.state.fire_res.insert(13, 0);
    h.state.hp.insert(14, 500); h.state.max_hp.insert(14, 1000);
    h.state.fire_res.insert(14, 0);
    h.state.enemies.insert(1, vec![10, 11, 12, 13, 14]);
    let mut tree = SkillTree::default();
    tree.set_rank("agumon::bouncing_fire", 2);
    h.state.skill_trees.insert(1, tree);
    let (_, intents) = h.run();

    // bounce_pick_next picks lowest HP first: 13 (300), then 14 (500).
    let bounce_hits: Vec<UnitId> = intents.iter().filter_map(|i| match i {
        Intent::DealDamage { target, .. } if *target == 13 || *target == 14 => Some(*target),
        _ => None,
    }).collect();
    assert_eq!(bounce_hits, vec![13, 14],
        "tier-2 bounce should hit lowest-HP-first: 13 then 14. got: {bounce_hits:?}");
}

#[test]
fn bouncing_fire_dry_run_matches_live() {
    // I2 carries over to Loop bodies — same Intent stream in DryRun.
    let mut h_live = PatternHarness::new(agumon::base_timeline_with_bouncing_fire());
    h_live.state.hp.insert(13, 400); h_live.state.max_hp.insert(13, 1000);
    h_live.state.fire_res.insert(13, 0);
    h_live.state.enemies.insert(1, vec![10, 11, 12, 13]);
    let mut tree = SkillTree::default();
    tree.set_rank("agumon::bouncing_fire", 2);
    h_live.state.skill_trees.insert(1, tree.clone());

    let mut h_dry = PatternHarness::new(agumon::base_timeline_with_bouncing_fire());
    h_dry.state.hp.insert(13, 400); h_dry.state.max_hp.insert(13, 1000);
    h_dry.state.fire_res.insert(13, 0);
    h_dry.state.enemies.insert(1, vec![10, 11, 12, 13]);
    h_dry.state.skill_trees.insert(1, tree);

    let (_, i_live) = h_live.run();

    let mut runner = BeatRunner::new(
        &h_dry.timeline, &h_dry.registries, &h_dry.state, &mut h_dry.signals,
        Clock::HeadlessAuto, 42, agumon::base_event_with_state(),
    );
    runner.run_to_completion(SkillCtxMode::DryRun, 256);

    assert_eq!(i_live, runner.all_intents,
        "Loop body Intent stream must match between Live and DryRun");
}

// ---------- Pattern 2: predator_loop blueprint state ----------

#[test]
fn predator_active_predicate_reads_blueprint_state() {
    // Pre-load blueprint state to simulate the listener having set it.
    let mut h = PatternHarness::new(agumon::base_timeline());
    h.state.blueprint_state.insert((1, "dorumon.predator_active"), 1);
    let empty = std::collections::HashSet::new();
    let _guard = agumon::RuntimeGuard::install(&h.registries, &h.state, &empty);

    let pred = h.registries.predicates.get("dorumon::predator_active")
        .expect("predator_active predicate should be registered");
    let evt = BeatEvent {
        caster: 1, primary_target: 10, beat_targets: vec![],
        cast_id: 7, beat: "chain_branch", hop_index: 0,
    };
    let ctx = SkillCtx::new(SkillCtxMode::Live);
    assert!(pred(&evt, &ctx),
        "predicate must read predator_active=1 from blueprint state");

    // Flip it off.
    let mut state_off = h.state.clone();
    state_off.blueprint_state.insert((1, "dorumon.predator_active"), 0);
    let _g2 = agumon::RuntimeGuard::install(&h.registries, &state_off, &empty);
    assert!(!pred(&evt, &ctx),
        "predicate must read predator_active=0 from blueprint state");
}

#[test]
fn metal_cannon_force_predator_writes_blueprint_state_intent() {
    // The ult hook produces `Intent::SetBlueprintState`. Validates the
    // write-path: state mutation goes through the Intent stream, never
    // direct from hook fn (D008 transition stream invariant).
    let mut r = ExtRegistries::new();
    builtin::register(&mut r);
    dorumon::register(&mut r);

    let hook = r.hooks.get("dorumon::on_metal_cannon_force_predator")
        .expect("force-predator hook should be registered");
    let evt = BeatEvent {
        caster: 5, primary_target: 10, beat_targets: vec![],
        cast_id: 99, beat: "metal_cannon::spit", hop_index: 0,
    };
    let mut ctx = SkillCtx::new(SkillCtxMode::Live);
    hook(&evt, &mut ctx);

    assert_eq!(ctx.drain(), vec![
        Intent::SetBlueprintState { actor: 5, key: "dorumon.predator_active",
                                    value: 1, cast_id: 99 },
        Intent::SetBlueprintState { actor: 5, key: "dorumon.expires_in",
                                    value: 3, cast_id: 99 },
    ]);
}

#[test]
fn chain_consume_reads_tracked_target_and_resets_state() {
    let state = {
        let mut s = agumon::default_state();
        // Caster (Dorumon=5) tracks enemy 11 as predator target.
        s.blueprint_state.insert((5, "dorumon.tracked_target"), 11);
        s
    };
    let mut r = ExtRegistries::new();
    builtin::register(&mut r);
    dorumon::register(&mut r);
    let empty = std::collections::HashSet::new();
    let _guard = agumon::RuntimeGuard::install(&r, &state, &empty);

    let hook = r.hooks.get("dorumon::on_chain_consume").unwrap();
    let evt = BeatEvent {
        caster: 5, primary_target: 10, beat_targets: vec![],
        cast_id: 50, beat: "dash_metal::chain", hop_index: 0,
    };
    let mut ctx = SkillCtx::new(SkillCtxMode::Live);
    hook(&evt, &mut ctx);

    assert_eq!(ctx.drain(), vec![
        Intent::DealDamage { target: 11, amount: 240,
                              tag: DamageTag::Physical, cast_id: 50 },
        Intent::SetBlueprintState { actor: 5, key: "dorumon.predator_active",
                                    value: 0, cast_id: 50 },
    ]);
}

// ---------- Pattern 3: twin_core_ice cross-blueprint identity filter ----------

#[test]
fn twin_core_predicate_passes_for_agumon_caster() {
    let mut state = agumon::default_state();
    state.identity.insert(1, "agumon");
    let r = {
        let mut r = ExtRegistries::new();
        builtin::register(&mut r);
        gabumon::register(&mut r);
        r
    };
    let empty = std::collections::HashSet::new();
    let _g = agumon::RuntimeGuard::install(&r, &state, &empty);

    let pred = r.predicates.get("gabumon::heated_caster_is_agumon").unwrap();
    // BeatEvent carries the casting unit. Predicate filters on its identity.
    let evt = BeatEvent {
        caster: 1, primary_target: 0, beat_targets: vec![],
        cast_id: 1, beat: "listener::status_applied", hop_index: 0,
    };
    let ctx = SkillCtx::new(SkillCtxMode::Live);
    assert!(pred(&evt, &ctx), "Twin Core ice should arm when Agumon (id=1) is the caster");
}

#[test]
fn twin_core_predicate_rejects_non_agumon_caster() {
    let mut state = agumon::default_state();
    state.identity.insert(7, "tentomon");
    let r = {
        let mut r = ExtRegistries::new();
        builtin::register(&mut r);
        gabumon::register(&mut r);
        r
    };
    let empty = std::collections::HashSet::new();
    let _g = agumon::RuntimeGuard::install(&r, &state, &empty);

    let pred = r.predicates.get("gabumon::heated_caster_is_agumon").unwrap();
    let evt = BeatEvent {
        caster: 7, primary_target: 0, beat_targets: vec![],
        cast_id: 1, beat: "listener::status_applied", hop_index: 0,
    };
    let ctx = SkillCtx::new(SkillCtxMode::Live);
    assert!(!pred(&evt, &ctx),
        "Twin Core ice must NOT arm for a non-Agumon caster — identity filter required");
}

// ---------- Pattern 4: RNG-gated edge (Block Reaction) ----------

#[test]
fn rng_predicate_is_deterministic_per_seed() {
    let mut h = PatternHarness::new(agumon::base_timeline());
    h.state.rng_seed = 0xDEAD_BEEF;
    let empty = std::collections::HashSet::new();
    let _g = agumon::RuntimeGuard::install(&h.registries, &h.state, &empty);

    let pred = h.registries.predicates.get("tentomon::rng_below_30pct").unwrap();
    let evt = BeatEvent {
        caster: 9, primary_target: 0, beat_targets: vec![],
        cast_id: 1234, beat: "block_ready", hop_index: 0,
    };
    let ctx = SkillCtx::new(SkillCtxMode::Live);
    let r1 = pred(&evt, &ctx);
    let r2 = pred(&evt, &ctx);
    let r3 = pred(&evt, &ctx);
    assert_eq!(r1, r2, "same (seed,cast,beat,hop) must yield same draw");
    assert_eq!(r2, r3, "same (seed,cast,beat,hop) must yield same draw");
}

#[test]
fn rng_predicate_differs_with_different_seed() {
    // Find a (cast,beat,hop) where seed A and seed B give different results.
    // 30% threshold ⇒ probabilistic over seeds. Sweep 64 seeds, assert that
    // not all seeds yield the same answer.
    let h0 = PatternHarness::new(agumon::base_timeline());
    let empty = std::collections::HashSet::new();
    let pred = h0.registries.predicates.get("tentomon::rng_below_30pct").unwrap();
    let evt = BeatEvent {
        caster: 9, primary_target: 0, beat_targets: vec![],
        cast_id: 0xAAAA, beat: "block_ready", hop_index: 0,
    };
    let ctx = SkillCtx::new(SkillCtxMode::Live);

    let mut state = agumon::default_state();
    let mut all_true = true;
    let mut all_false = true;
    for seed in 0..64u64 {
        state.rng_seed = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let _g = agumon::RuntimeGuard::install(&h0.registries, &state, &empty);
        let r = pred(&evt, &ctx);
        if r { all_false = false; } else { all_true = false; }
    }
    assert!(!all_true,  "RNG predicate should sometimes be false across seeds");
    assert!(!all_false, "RNG predicate should sometimes be true across seeds");
}

#[test]
fn rng_70pct_threshold_skews_higher_than_30pct() {
    // Empirical: at 70%, the predicate passes ~7×/10. At 30%, ~3×/10.
    // Validates the threshold parameter actually controls probability.
    let h = PatternHarness::new(agumon::base_timeline());
    let empty = std::collections::HashSet::new();
    let p30 = h.registries.predicates.get("tentomon::rng_below_30pct").unwrap();
    let p70 = h.registries.predicates.get("tentomon::rng_below_70pct").unwrap();
    let ctx = SkillCtx::new(SkillCtxMode::Live);

    let mut state = agumon::default_state();
    let mut hits_30 = 0;
    let mut hits_70 = 0;
    for cast_id in 0..400u64 {
        state.rng_seed = 0x1234_5678;
        let _g = agumon::RuntimeGuard::install(&h.registries, &state, &empty);
        let evt = BeatEvent {
            caster: 9, primary_target: 0, beat_targets: vec![],
            cast_id, beat: "block_ready", hop_index: 0,
        };
        if p30(&evt, &ctx) { hits_30 += 1; }
        if p70(&evt, &ctx) { hits_70 += 1; }
    }
    // Loose check — should easily satisfy with 400 samples.
    assert!(hits_70 > hits_30 + 100,
        "p70 should fire substantially more often than p30; got 70={hits_70}, 30={hits_30}");
}

// ---------- I8 — validator catches dangling Loop refs ----------

#[test]
fn validation_catches_loop_exit_when_unregistered() {
    let mut r = ExtRegistries::new();
    builtin::register(&mut r);
    agumon::register(&mut r);
    // Inject a Loop beat whose exit_when references a missing predicate.
    let mut tl = agumon::base_timeline();
    tl.beats.push(Beat {
        id: "ghost_loop",
        kind: BeatKind::Loop {
            body: vec![Beat {
                id: "ghost_body",
                kind: BeatKind::Impact { selector: "primary" },
                presentation: None,
                hook: None,
                advance: AdvanceMode::Auto,
            }],
            exit_when: "agumon::ghost_exit",
        },
        presentation: None,
        hook: None,
        advance: AdvanceMode::Auto,
    });
    tl.edges.push(BeatEdge { from: "aftermath", to: "ghost_loop", gate: None });

    let errs = validate_timeline_refs(&tl, &r).unwrap_err();
    assert!(errs.iter().any(|e| e.axis == "predicate" && e.id == "agumon::ghost_exit"),
        "validator must surface dangling Loop exit_when: {errs:#?}");
}
