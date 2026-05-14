//! Agumon blueprint — the canonical "miniature complete kit" the spike uses
//! to prove that **all** custom logic for a Digimon lives in one module and
//! plugs into the kernel exclusively through `ExtRegistries`.
//!
//! Exercised extension points:
//!
//! | Axis        | Concrete usage                              |
//! |-------------|---------------------------------------------|
//! | hook        | `on_impact_main`, `on_splash`, `on_aftermath`, `on_lingering_burn`, `twin_core_on_ally_ko` |
//! | selector    | custom `agumon::baby_burner_splash` (built-in `primary` is used too) |
//! | predicate   | `agumon::has_adjacent_targets`, `agumon::has_two_alive_allies`     |
//! | formula     | `agumon::fire_atk_scaling`, `agumon::heated_dot`                   |
//! | tick        | `agumon::heated_tick`                                              |
//! | ai_utility  | `agumon::burner_utility`                                           |
//! | cue         | `agumon::charge_by_hp` (HP-dependent charge anim)                  |
//!
//! Adding a new Digimon means creating a sibling module like this one with
//! its own `register()` entry point — zero kernel changes.

use crate::*;

// ---------- Hooks (D023: hooks-as-fn, return-by-enqueue) ----------

/// Main impact: fire damage to the primary target via `fire_atk_scaling`.
pub fn on_impact_main(evt: &BeatEvent, ctx: &mut SkillCtx) {
    // The hook does NOT compute damage itself — it delegates to the registered
    // formula so the same scaling can be reused by splash, by status ticks, by
    // AI utility, and so on. This is the key win of the ExtPoint pattern: one
    // axis = one place to change behaviour.
    let dmg = formula_value("agumon::fire_atk_scaling",
                            evt.caster, evt.primary_target);
    ctx.enqueue(Intent::DealDamage {
        target: evt.primary_target,
        amount: dmg,
        tag: DamageTag::Fire,
        cast_id: evt.cast_id,
    });
}

/// Splash AoE: each target in `beat_targets` (populated by the custom
/// `agumon::baby_burner_splash` selector) takes half-scaled fire damage.
pub fn on_splash(evt: &BeatEvent, ctx: &mut SkillCtx) {
    for &t in &evt.beat_targets {
        let dmg = formula_value("agumon::fire_atk_scaling", evt.caster, t) / 2;
        ctx.enqueue(Intent::DealDamage {
            target: t,
            amount: dmg,
            tag: DamageTag::Fire,
            cast_id: evt.cast_id,
        });
    }
}

/// Aftermath: apply Heated(2t) to the primary, Refresh semantics.
pub fn on_aftermath(evt: &BeatEvent, ctx: &mut SkillCtx) {
    ctx.enqueue(Intent::ApplyStatus {
        target: evt.primary_target,
        kind: StatusKind::Heated,
        duration: 2,
        mode: StatusApplyMode::Refresh,
        cast_id: evt.cast_id,
    });
}

/// Pyromaniac branch addon: extra Burn DoT on each splash target.
pub fn on_lingering_burn(evt: &BeatEvent, ctx: &mut SkillCtx) {
    for &t in &evt.beat_targets {
        ctx.enqueue(Intent::ApplyStatus {
            target: t,
            kind: StatusKind::Burn,
            duration: 3,
            mode: StatusApplyMode::Refresh,
            cast_id: evt.cast_id,
        });
    }
}

/// Bouncing Fire hop hook — deals half-scaled fire damage to the selector's
/// chosen target. The Loop runner re-invokes this once per iteration with
/// `evt.hop_index` set to 0, 1, 2, ... and `evt.beat_targets` populated by
/// the `bounce_pick_next` selector.
pub fn on_bounce_hop(evt: &BeatEvent, ctx: &mut SkillCtx) {
    for &t in &evt.beat_targets {
        let dmg = formula_value("agumon::fire_atk_scaling", evt.caster, t) / 2;
        ctx.enqueue(Intent::DealDamage {
            target: t,
            amount: dmg,
            tag: DamageTag::Fire,
            cast_id: evt.cast_id,
        });
    }
}

/// Passive `Twin Core Fire`: when an ally goes down and the caster ends up
/// with exactly 2 alive allies, gain +1 SP and self-buff TwinCoreRage.
pub fn twin_core_on_ally_ko(evt: &BeatEvent, ctx: &mut SkillCtx) {
    ctx.enqueue(Intent::ModifyResource {
        actor: evt.caster,
        kind: ResourceKind::Sp,
        delta: 1,
        cast_id: evt.cast_id,
    });
    ctx.enqueue(Intent::ApplyStatus {
        target: evt.caster,
        kind: StatusKind::TwinCoreRage,
        duration: 2,
        mode: StatusApplyMode::Refresh,
        cast_id: evt.cast_id,
    });
}

// ---------- Predicates ----------

/// True iff the splash selector found at least one target. Pyromaniac patch
/// uses this so we don't insert `lingering_burn` when there's no AoE pool.
pub fn has_adjacent_targets(evt: &BeatEvent, _ctx: &SkillCtx) -> bool {
    // The gate is evaluated *after* the splash beat exits, at which point
    // `beat_targets` still reflects the splash selector's last resolution.
    !evt.beat_targets.is_empty()
}

/// Used by the Twin Core passive: only triggers when the caster has exactly
/// 2 alive allies (mock state lookup; in production this is an ECS query).
pub fn has_two_alive_allies(_evt: &BeatEvent, _ctx: &SkillCtx) -> bool {
    // The spike can't read `&CombatStateMock` from a `PredicateFn` because
    // PredicateFn is `(&BeatEvent, &SkillCtx) -> bool` — the state isn't
    // threaded through. This is a real finding (see FINDINGS F6); for the
    // spike we hard-code true. Production: extend PredicateCtx to carry state.
    true
}

/// D033 — Skilltree-gate predicate. Edge `aftermath → bounce_loop` only
/// passes when the caster has the `bouncing_fire` talent unlocked AND its
/// rank is ≥1 (rank 0 ≡ off).
pub fn has_bouncing_fire(evt: &BeatEvent, ctx: &SkillCtx) -> bool {
    ctx.skill_tree(evt.caster)
        .rank("agumon::bouncing_fire")
        .map(|r| r > 0)
        .unwrap_or(false)
}

/// Loop `exit_when` predicate for the bounce sub-timeline. Returns true when
/// the current iteration (`evt.hop_index`) has reached the talent rank OR
/// the bounce target pool is exhausted.
///
/// Three exit conditions, OR'd together:
/// - **Tier cap**: `hop_index + 1 >= rank` (we've completed `rank` iterations).
/// - **Pool exhaustion**: `cast_hit_set` covers every alive enemy of caster
///   (no further bounce target left). Mirrors the "Bounce pool exhaustion
///   breaks the hop loop silently" gotcha already documented in K-memory.
/// - **Talent missing**: skilltree rank `None` ⇒ exit immediately (defence
///   in depth: if the gate edge was somehow taken without the talent, the
///   Loop body executes zero iterations).
pub fn bounce_should_stop(evt: &BeatEvent, ctx: &SkillCtx) -> bool {
    let rank = ctx.skill_tree(evt.caster).rank("agumon::bouncing_fire").unwrap_or(0) as u32;
    if rank == 0 { return true; }
    if evt.hop_index + 1 >= rank { return true; }
    // Pool exhaustion: every enemy is in the hit set.
    let pool = ctx.enemies_alive(evt.caster);
    let hit  = ctx.cast_hit_set();
    pool.iter().all(|e| hit.contains(e))
}

// ---------- Selectors ----------

/// Custom selector that the production version of Baby Burner needs:
/// adjacents to primary, alive, enemies of caster, up to 2, sorted by
/// lowest HP first (flavor: the flame "seeks" the weak).
pub fn baby_burner_splash(ctx: &SelectorCtx) -> Vec<UnitId> {
    let mut adj: Vec<UnitId> = ctx.state.adjacent_to(ctx.primary_target).into_iter()
        .filter(|e| *e != ctx.primary_target
                 && ctx.state.is_alive(*e)
                 && ctx.state.is_enemy_of(ctx.caster, *e))
        .collect();
    adj.sort_by_key(|e| ctx.state.hp(*e));
    adj.truncate(2);
    adj
}

/// Bouncing Fire selector — picks the next bounce target. Reads the
/// cast-scope `cast_hit_set` (via runtime thread-local) so the same unit
/// is never bounced onto twice. Sort key is lowest-HP-first; ties broken
/// by stable insertion order from `enemies_of`.
///
/// Returns 0 or 1 target. The Loop's `exit_when` predicate is the canonical
/// place that surfaces "pool exhausted" (empty selector ⇒ no Intent::DealDamage
/// ⇒ cast_hit_set unchanged ⇒ next iteration exits via the same check).
pub fn bounce_pick_next(ctx: &SelectorCtx) -> Vec<UnitId> {
    let hit_set = runtime::with_hit_set(|h| h.clone());
    let mut candidates: Vec<UnitId> = ctx.state.enemies_of(ctx.caster).into_iter()
        .filter(|e| ctx.state.is_alive(*e) && !hit_set.contains(e))
        .collect();
    candidates.sort_by_key(|e| ctx.state.hp(*e));
    candidates.truncate(1);
    candidates
}

// ---------- Formulas ----------

/// Fire damage scaling — used by impact, splash, and (via heated_dot) by the
/// Heated status tick.
pub fn fire_atk_scaling(ctx: &FormulaCtx) -> i32 {
    let base = 120;
    let atk_bonus = ctx.state.atk(ctx.caster) * 80 / 100;
    let resist = ctx.state.fire_res(ctx.target);
    ((base + atk_bonus) * (100 - resist) / 100).max(1)
}

/// Heated tick damage = 15% of caster ATK, post-resist.
pub fn heated_dot(ctx: &FormulaCtx) -> i32 {
    let raw = ctx.state.atk(ctx.caster) * 15 / 100;
    let resist = ctx.state.fire_res(ctx.target);
    (raw * (100 - resist) / 100).max(1)
}

// ---------- Status tick ----------

/// Each turn while Heated is active, the source deals `heated_dot` to the
/// target. The tick fn doesn't *compute* damage — it asks the formula
/// registry, same pattern as hooks.
pub fn heated_tick(st: &StatusInstance, ctx: &mut SkillCtx) {
    let dmg = formula_value("agumon::heated_dot", st.source, st.target);
    ctx.enqueue(Intent::DealDamage {
        target: st.target,
        amount: dmg,
        tag: DamageTag::Fire,
        cast_id: st.cast_id,
    });
}

// ---------- AI utility ----------

/// Score in [0, 1]. The AI prefers Baby Burner when at least 2 enemies are
/// adjacent to a viable primary AND the ult is charged. In production we'd
/// thread a `simulate_selector` capability through `AiCtx`; the spike approximates
/// by counting enemy adjacency directly on the mock state.
pub fn burner_utility(ctx: &AiCtx) -> f32 {
    let enemies = ctx.state.enemies_of(ctx.caster);
    let candidates: usize = enemies.iter().filter_map(|e| {
        if !ctx.state.is_alive(*e) { return None; }
        let adj_count = ctx.state.adjacent_to(*e).into_iter()
            .filter(|a| ctx.state.is_alive(*a))
            .count();
        Some(adj_count)
    }).max().unwrap_or(0);

    let charged = ctx.state.ult_charge(ctx.caster) >= 100;
    if charged && candidates >= 2 { 0.9 }
    else if charged                { 0.5 }
    else                           { 0.1 }
}

// ---------- Cue resolver ----------

/// Charge anim varies with HP: desperate (<30%) vs. normal. The result
/// is a static cue id the kernel then emits as the beat's PresentationCue.
pub fn charge_by_hp(ctx: &CueCtx) -> CueId {
    if ctx.state.hp_pct(ctx.caster) < 0.30 { "anim_baby_burner_charge_desperate" }
    else                                    { "anim_baby_burner_charge_normal" }
}

// ---------- Helper: read a formula via the registry from inside a hook ----------
//
// In production this lives on `SkillCtx` as `ctx.formula(id)(&fctx)`, but the
// spike keeps `SkillCtx` flat (no registry borrow) to mirror the simplest
// possible runtime API. The helper exists so blueprint code reads naturally;
// the `BeatRunner` installs the runtime context (registries + state +
// hit_set) for the duration of each step via `runtime::RuntimeGuard`.

/// Re-export for tests that drive blueprint fns outside a `BeatRunner`.
pub use crate::runtime::RuntimeGuard;

pub fn formula_value(id: FormulaId, caster: UnitId, target: UnitId) -> i32 {
    runtime::with_registries(|reg| {
        runtime::with_state(|state| {
            let f = reg.formulas.get(id).unwrap_or_else(|| {
                panic!("formula `{id}` not registered")
            });
            let fctx = FormulaCtx { caster, target, state };
            f(&fctx)
        })
    })
}

// ---------- Base timeline ----------

pub fn base_timeline() -> CompiledTimeline {
    CompiledTimeline {
        id: "agumon::baby_burner",
        entry: "cast",
        beats: vec![
            Beat {
                id: "cast",
                kind: BeatKind::Cast,
                // Dynamic cue: the charge anim depends on caster HP%.
                presentation: Some(Presentation::Dynamic("agumon::charge_by_hp")),
                hook: None,
                advance: AdvanceMode::OnSignal("anim_charge_done"),
            },
            Beat {
                id: "projectile",
                kind: BeatKind::Phase,
                presentation: Some(Presentation::Static(Cue {
                    anim: None,
                    vfx: Some("vfx_projectile_fireball"),
                    sfx: None,
                })),
                hook: None,
                advance: AdvanceMode::OnSignal("projectile_hit"),
            },
            Beat {
                id: "impact_main",
                // Kernel-provided built-in selector — most beats use these.
                kind: BeatKind::Impact { selector: "primary" },
                presentation: Some(Presentation::Static(Cue {
                    anim: None,
                    vfx: Some("vfx_impact_main"),
                    sfx: Some("sfx_hit_heavy"),
                })),
                hook: Some("agumon::on_impact_main"),
                advance: AdvanceMode::Auto,
            },
            Beat {
                id: "splash_adj",
                // Skill-specific selector registered by this blueprint.
                kind: BeatKind::Impact { selector: "agumon::baby_burner_splash" },
                presentation: Some(Presentation::Static(Cue {
                    anim: None,
                    vfx: Some("vfx_splash_aoe"),
                    sfx: Some("sfx_hit_aoe"),
                })),
                hook: Some("agumon::on_splash"),
                advance: AdvanceMode::Auto,
            },
            Beat {
                id: "aftermath",
                kind: BeatKind::Aftermath,
                presentation: None,
                hook: Some("agumon::on_aftermath"),
                advance: AdvanceMode::Auto,
            },
        ],
        edges: vec![
            BeatEdge { from: "cast",        to: "projectile",  gate: None },
            BeatEdge { from: "projectile",  to: "impact_main", gate: None },
            BeatEdge { from: "impact_main", to: "splash_adj",  gate: None },
            BeatEdge { from: "splash_adj",  to: "aftermath",   gate: None },
        ],
    }
}

/// Timeline variant with the Bouncing Fire branch pre-wired. Identical to
/// `base_timeline()` up through `aftermath`, then adds a `BeatKind::Loop`
/// (`bounce_loop`) that the runner only enters when the skilltree-gate
/// predicate `agumon::has_bouncing_fire` passes. A fallback edge `aftermath
/// → end` (with the "end" beat acting as terminal) catches the talent-off
/// case so the cast doesn't halt at `aftermath` like the F1 finding.
///
/// This is the canonical shape of a **runtime-gated talent branch** (D033):
/// the bounce sub-timeline lives permanently in the graph; the skilltree
/// state decides at edge-evaluation time whether to take it.
pub fn base_timeline_with_bouncing_fire() -> CompiledTimeline {
    let mut tl = base_timeline();
    tl.id = "agumon::baby_burner_with_bouncing_fire";

    // Loop body: one "hop" — select next non-hit enemy, deal half damage.
    let bounce_loop = Beat {
        id: "bounce_loop",
        kind: BeatKind::Loop {
            body: vec![Beat {
                id: "bounce_hop",
                kind: BeatKind::Impact { selector: "agumon::bounce_pick_next" },
                presentation: Some(Presentation::Static(Cue {
                    anim: None,
                    vfx: Some("vfx_bounce_arc"),
                    sfx: Some("sfx_hit_light"),
                })),
                hook: Some("agumon::on_bounce_hop"),
                advance: AdvanceMode::Auto,
            }],
            exit_when: "agumon::bounce_should_stop",
        },
        presentation: None,
        hook: None,
        advance: AdvanceMode::Auto,
    };

    // Inert terminal beat so the fallback edge has a target. Production has
    // a proper "end" sentinel; the spike represents it as a trivial Phase.
    let cast_end = Beat {
        id: "cast_end",
        kind: BeatKind::Phase,
        presentation: None,
        hook: None,
        advance: AdvanceMode::Auto,
    };

    tl.beats.push(bounce_loop);
    tl.beats.push(cast_end);
    // First-passing edge: if the talent is unlocked, walk the Loop;
    // otherwise the fallback (unconditional) edge ends the cast cleanly.
    tl.edges.push(BeatEdge {
        from: "aftermath",
        to: "bounce_loop",
        gate: Some("agumon::has_bouncing_fire"),
    });
    tl.edges.push(BeatEdge {
        from: "aftermath",
        to: "cast_end",
        gate: None,
    });
    tl
}

/// Skill-tree node "Pyromaniac": inserts a `lingering_burn` beat after splash,
/// only when adjacent targets actually exist.
pub fn pyromaniac_patch() -> Vec<TimelinePatchOp> {
    vec![TimelinePatchOp::InsertBeatAfter {
        anchor: "splash_adj",
        beat: Beat {
            id: "lingering_burn",
            // Reuses the same splash selector — keeps targeting consistent.
            kind: BeatKind::Impact { selector: "agumon::baby_burner_splash" },
            presentation: Some(Presentation::Static(Cue {
                anim: None,
                vfx: Some("vfx_floor_fire"),
                sfx: None,
            })),
            hook: Some("agumon::on_lingering_burn"),
            advance: AdvanceMode::Auto,
        },
        edge_gate: Some("agumon::has_adjacent_targets"),
    }]
}

// ---------- Blueprint registration — SINGLE entry point ----------

/// The one and only function the App builder needs to call for Agumon.
/// Everything else lives inside this module.
pub fn register(reg: &mut ExtRegistries) {
    // hooks
    reg.hooks.register("agumon::on_impact_main",     on_impact_main);
    reg.hooks.register("agumon::on_splash",          on_splash);
    reg.hooks.register("agumon::on_aftermath",       on_aftermath);
    reg.hooks.register("agumon::on_lingering_burn",  on_lingering_burn);
    reg.hooks.register("agumon::on_bounce_hop",      on_bounce_hop);
    reg.hooks.register("agumon::twin_core_on_ally_ko", twin_core_on_ally_ko);

    // predicates
    reg.predicates.register("agumon::has_adjacent_targets", has_adjacent_targets);
    reg.predicates.register("agumon::has_two_alive_allies", has_two_alive_allies);
    reg.predicates.register("agumon::has_bouncing_fire",    has_bouncing_fire);
    reg.predicates.register("agumon::bounce_should_stop",   bounce_should_stop);

    // selectors (custom — built-ins come from `builtin::register`)
    reg.selectors.register("agumon::baby_burner_splash", baby_burner_splash);
    reg.selectors.register("agumon::bounce_pick_next",   bounce_pick_next);

    // formulas
    reg.formulas.register("agumon::fire_atk_scaling", fire_atk_scaling);
    reg.formulas.register("agumon::heated_dot",       heated_dot);

    // status ticks
    reg.ticks.register("agumon::heated_tick", heated_tick);

    // AI utility
    reg.ai.register("agumon::burner_utility", burner_utility);

    // cue resolvers
    reg.cues.register("agumon::charge_by_hp", charge_by_hp);
}

// ---------- Default state builder (used by tests) ----------
//
// Encapsulates the "Agumon cast against a primary with 2 adjacent enemies"
// fixture so tests don't repeat the wiring.

pub fn default_state() -> CombatStateMock {
    let mut s = CombatStateMock::default();
    // Caster
    s.hp.insert(1, 800);
    s.max_hp.insert(1, 1000);
    s.atk.insert(1, 200);
    s.ult_charge.insert(1, 100);
    s.enemies.insert(1, vec![10, 11, 12]);
    s.allies.insert(1, vec![2, 3]);
    // Allies
    s.hp.insert(2, 700); s.max_hp.insert(2, 1000);
    s.hp.insert(3, 700); s.max_hp.insert(3, 1000);
    // Enemies — primary + two adjacents to it
    s.hp.insert(10, 1000); s.max_hp.insert(10, 1000); s.fire_res.insert(10, 0);
    s.hp.insert(11, 600);  s.max_hp.insert(11, 1000); s.fire_res.insert(11, 0);
    s.hp.insert(12, 900);  s.max_hp.insert(12, 1000); s.fire_res.insert(12, 0);
    s.adjacency.insert(10, vec![11, 12]);
    s.enemies.insert(10, vec![1, 2, 3]);
    s.enemies.insert(11, vec![1, 2, 3]);
    s.enemies.insert(12, vec![1, 2, 3]);
    s
}

pub fn base_event_with_state() -> BeatEvent {
    BeatEvent {
        caster: 1,
        primary_target: 10,
        beat_targets: Vec::new(),
        cast_id: 42,
        beat: "cast",
        hop_index: 0,
    }
}
