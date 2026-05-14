//! M021 Timeline-FSM spike — PoC headless, zero deps.
//!
//! Validates the proposal: an `Ability` is a **CompiledTimeline** of beats.
//! Beats carry presentation cues (data) and optional hooks (Rust fn) that
//! `enqueue` Intents into a `SkillCtx`. A beat-runner walks the timeline,
//! advancing on `Auto` or `OnSignal(name)`. Two clocks: `HeadlessAuto`
//! auto-fires signals (deterministic tests, AI lookahead), `Windowed`
//! waits for an external animation system to emit them.
//!
//! **This revision** introduces the `Registry<E: ExtPoint>` pattern: every
//! axis where a blueprint plugs custom logic in (hook, selector, predicate,
//! formula, tick, ai utility, cue resolver) goes through the same
//! id-to-fn-pointer mechanism. Validation at "App::finish()" iterates each
//! registry and fails fast on dangling references.
//!
//! NOT production code. NOT linked to bevyrogue. Lives outside the workspace.

use std::collections::{HashMap, HashSet, VecDeque};
use std::marker::PhantomData;

// ---------- Domain types (subset of M021 Intent design) ----------

pub type UnitId = u32;
pub type CastId = u64;
pub type SignalName = &'static str;
pub type BeatId = &'static str;
pub type HookId = &'static str;
pub type PredicateId = &'static str;
pub type SelectorId = &'static str;
pub type FormulaId = &'static str;
pub type TickId = &'static str;
pub type AiUtilityId = &'static str;
pub type CueId = &'static str;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DamageTag { Physical, Fire, Lightning }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatusKind { Heated, Burn, Stun, TwinCoreRage }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusApplyMode { Stack, Refresh, MaxOf }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceKind { Sp, UltCharge }

/// Subset of the M021 Intent set, sufficient to validate the FSM model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intent {
    DealDamage {
        target: UnitId,
        amount: i32,
        tag: DamageTag,
        cast_id: CastId,
    },
    ApplyStatus {
        target: UnitId,
        kind: StatusKind,
        duration: u32,
        mode: StatusApplyMode,
        cast_id: CastId,
    },
    ModifyResource {
        actor: UnitId,
        kind: ResourceKind,
        delta: i32,
        cast_id: CastId,
    },
    /// Blueprint state write (Dorumon Predator Loop pattern). Per-unit,
    /// per-key, i32-valued. Routed through the Intent stream — never
    /// applied directly by hooks — so the kernel remains single-writer
    /// and replays are reconstructible from the stream alone.
    SetBlueprintState {
        actor: UnitId,
        key: BlueprintStateKey,
        value: i32,
        cast_id: CastId,
    },
}

/// Observable side-channel events. Logic is in `Intent`; this is for UI/log/AI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CombatEvent {
    BeatEntered { beat: BeatId, cast_id: CastId },
    BeatExited  { beat: BeatId, cast_id: CastId },
    PresentationCue {
        beat: BeatId,
        anim: Option<&'static str>,
        vfx:  Option<&'static str>,
        sfx:  Option<&'static str>,
    },
}

// ---------- Mock combat state (replaces the real Bevy world for the spike) ----------
//
// In production these reads go through `SkillCtx` accessors backed by an ECS
// Query. The spike uses a flat map-backed mock so selectors, predicates and
// formulas can be exercised without bringing in Bevy.

/// D033 — Skill tree as immutable runtime context. The component carries
/// the per-unit unlocked talent set + per-talent rank (tier-aware perks).
/// Skill trees are **load-time-immutable** for a combat run (Slay-the-Spire
/// shape): talents change between encounters, never mid-cast.
#[derive(Debug, Clone, Default)]
pub struct SkillTree {
    pub unlocked: HashSet<&'static str>,
    pub ranks:    HashMap<&'static str, u8>,
}

impl SkillTree {
    pub fn has(&self, talent: &str) -> bool { self.unlocked.contains(talent) }
    pub fn rank(&self, talent: &str) -> Option<u8> { self.ranks.get(talent).copied() }
    pub fn unlock(&mut self, talent: &'static str) { self.unlocked.insert(talent); }
    pub fn set_rank(&mut self, talent: &'static str, rank: u8) {
        self.unlocked.insert(talent);
        self.ranks.insert(talent, rank);
    }
}

/// Predator-Loop-style blueprint state: per-unit, per-key, integer-valued
/// (the spike keeps it i32 for simplicity; production widens to a tagged
/// value enum). Mutated via `Intent::SetBlueprintState`, read by predicates
/// and hooks through `SkillCtx::blueprint_state(...)`.
pub type BlueprintStateKey = &'static str;

#[derive(Debug, Clone, Default)]
pub struct CombatStateMock {
    pub hp:        HashMap<UnitId, i32>,
    pub max_hp:    HashMap<UnitId, i32>,
    pub atk:       HashMap<UnitId, i32>,
    pub fire_res:  HashMap<UnitId, i32>,
    pub adjacency: HashMap<UnitId, Vec<UnitId>>,
    pub enemies:   HashMap<UnitId, Vec<UnitId>>,
    pub allies:    HashMap<UnitId, Vec<UnitId>>,
    pub statuses:  HashMap<UnitId, Vec<StatusKind>>,
    pub ult_charge: HashMap<UnitId, i32>,
    /// Per-unit skill tree (D033). Empty by default — units without unlocks
    /// see baseline behaviour identical to "no skill tree exists".
    pub skill_trees: HashMap<UnitId, SkillTree>,
    /// Per-(unit,key) blueprint state. Predator Loop, Twin Core, Battery
    /// Loop, etc., live here. Read-only from predicates; mutated by hooks
    /// through `Intent::SetBlueprintState`.
    pub blueprint_state: HashMap<(UnitId, BlueprintStateKey), i32>,
    /// Deterministic identity tag per unit (e.g. "agumon", "gabumon"). Used
    /// by Twin-Core-style predicates that filter on "caster is Agumon".
    pub identity: HashMap<UnitId, &'static str>,
    /// Seed for the cast-scope deterministic RNG. Mixed with `cast_id` +
    /// `beat_id` hash at the predicate site to yield a reproducible draw.
    pub rng_seed: u64,
}

impl CombatStateMock {
    pub fn hp(&self, u: UnitId) -> i32 { *self.hp.get(&u).unwrap_or(&0) }
    pub fn max_hp(&self, u: UnitId) -> i32 { *self.max_hp.get(&u).unwrap_or(&1).max(&1) }
    pub fn atk(&self, u: UnitId) -> i32 { *self.atk.get(&u).unwrap_or(&0) }
    pub fn fire_res(&self, u: UnitId) -> i32 { *self.fire_res.get(&u).unwrap_or(&0) }
    pub fn is_alive(&self, u: UnitId) -> bool { self.hp(u) > 0 }
    pub fn adjacent_to(&self, u: UnitId) -> Vec<UnitId> {
        self.adjacency.get(&u).cloned().unwrap_or_default()
    }
    pub fn enemies_of(&self, u: UnitId) -> Vec<UnitId> {
        self.enemies.get(&u).cloned().unwrap_or_default()
    }
    pub fn allies_of(&self, u: UnitId) -> Vec<UnitId> {
        self.allies.get(&u).cloned().unwrap_or_default()
    }
    pub fn is_enemy_of(&self, a: UnitId, b: UnitId) -> bool {
        self.enemies_of(a).contains(&b)
    }
    pub fn has_status(&self, u: UnitId, s: StatusKind) -> bool {
        self.statuses.get(&u).map(|v| v.contains(&s)).unwrap_or(false)
    }
    pub fn hp_pct(&self, u: UnitId) -> f32 {
        self.hp(u) as f32 / self.max_hp(u) as f32
    }
    pub fn ult_charge(&self, u: UnitId) -> i32 {
        *self.ult_charge.get(&u).unwrap_or(&0)
    }
    pub fn skill_tree(&self, u: UnitId) -> SkillTree {
        self.skill_trees.get(&u).cloned().unwrap_or_default()
    }
    pub fn blueprint_state_value(&self, u: UnitId, key: BlueprintStateKey) -> i32 {
        *self.blueprint_state.get(&(u, key)).unwrap_or(&0)
    }
    pub fn identity_of(&self, u: UnitId) -> &'static str {
        self.identity.get(&u).copied().unwrap_or("")
    }
}

// ---------- Hook / Predicate / Selector / Formula / Tick / AI / Cue ctx ----------

#[derive(Debug, Clone)]
pub struct BeatEvent {
    pub caster: UnitId,
    pub primary_target: UnitId,
    /// Populated by the runner when a beat's selector resolves a target set
    /// (`BeatKind::Impact { selector }`). Hooks read this rather than computing
    /// their own targets.
    pub beat_targets: Vec<UnitId>,
    pub cast_id: CastId,
    pub beat: BeatId,
    /// 0 outside a `BeatKind::Loop` body. Inside a Loop, 0..N counting iterations.
    /// Field-on-event keeps every predicate signature uniform: `fn(&BeatEvent, &SkillCtx) -> bool`.
    /// Non-loop predicates simply ignore it (see option A vs B trade-off in spike notes).
    pub hop_index: u32,
}

pub struct SelectorCtx<'a> {
    pub caster: UnitId,
    pub primary_target: UnitId,
    pub state: &'a CombatStateMock,
    pub mode: SkillCtxMode,
}

pub struct FormulaCtx<'a> {
    pub caster: UnitId,
    pub target: UnitId,
    pub state: &'a CombatStateMock,
}

pub struct AiCtx<'a> {
    pub caster: UnitId,
    pub state: &'a CombatStateMock,
}

pub struct CueCtx<'a> {
    pub caster: UnitId,
    pub state: &'a CombatStateMock,
}

#[derive(Debug, Clone)]
pub struct StatusInstance {
    pub target: UnitId,
    pub kind: StatusKind,
    pub stacks: u32,
    pub remaining_turns: u32,
    pub source: UnitId,
    pub cast_id: CastId,
}

// ---------- ExtPoint pattern: one Registry<E> per axis ----------

pub trait ExtPoint: 'static {
    type Fn: Copy;
    const KIND: &'static str;
}

pub struct Registry<E: ExtPoint> {
    fns: HashMap<&'static str, E::Fn>,
    _phantom: PhantomData<E>,
}

impl<E: ExtPoint> Default for Registry<E> {
    fn default() -> Self { Self::new() }
}

impl<E: ExtPoint> Registry<E> {
    pub fn new() -> Self {
        Self { fns: HashMap::new(), _phantom: PhantomData }
    }
    pub fn register(&mut self, id: &'static str, f: E::Fn) -> &mut Self {
        self.fns.insert(id, f);
        self
    }
    pub fn get(&self, id: &str) -> Option<E::Fn> { self.fns.get(id).copied() }
    pub fn contains(&self, id: &str) -> bool { self.fns.contains_key(id) }
    pub fn ids(&self) -> impl Iterator<Item = &&'static str> { self.fns.keys() }
    pub fn kind(&self) -> &'static str { E::KIND }
}

// Axis markers — each is a 3-line declaration. Production code adds new axes
// the same way: define a type, impl `ExtPoint`, slot it into `ExtRegistries`.

pub struct HookExt;
impl ExtPoint for HookExt {
    type Fn = fn(&BeatEvent, &mut SkillCtx);
    const KIND: &'static str = "hook";
}

pub struct PredicateExt;
impl ExtPoint for PredicateExt {
    type Fn = fn(&BeatEvent, &SkillCtx) -> bool;
    const KIND: &'static str = "predicate";
}

pub struct SelectorExt;
impl ExtPoint for SelectorExt {
    type Fn = for<'a> fn(&SelectorCtx<'a>) -> Vec<UnitId>;
    const KIND: &'static str = "selector";
}

pub struct FormulaExt;
impl ExtPoint for FormulaExt {
    type Fn = for<'a> fn(&FormulaCtx<'a>) -> i32;
    const KIND: &'static str = "formula";
}

pub struct TickExt;
impl ExtPoint for TickExt {
    type Fn = fn(&StatusInstance, &mut SkillCtx);
    const KIND: &'static str = "tick";
}

pub struct AiUtilityExt;
impl ExtPoint for AiUtilityExt {
    type Fn = for<'a> fn(&AiCtx<'a>) -> f32;
    const KIND: &'static str = "ai_utility";
}

pub struct CueExt;
impl ExtPoint for CueExt {
    type Fn = for<'a> fn(&CueCtx<'a>) -> CueId;
    const KIND: &'static str = "cue";
}

/// Aggregate of every per-axis registry. In production this lives as a
/// `Resource` on the Bevy `App` and is populated by blueprint plugins.
#[derive(Default)]
pub struct ExtRegistries {
    pub hooks:      Registry<HookExt>,
    pub predicates: Registry<PredicateExt>,
    pub selectors:  Registry<SelectorExt>,
    pub formulas:   Registry<FormulaExt>,
    pub ticks:      Registry<TickExt>,
    pub ai:         Registry<AiUtilityExt>,
    pub cues:       Registry<CueExt>,
}

impl ExtRegistries {
    pub fn new() -> Self { Self::default() }
}

// ---------- Built-in selectors (kernel-provided primitives) ----------

pub mod builtin {
    use super::*;

    pub fn sel_primary(ctx: &SelectorCtx) -> Vec<UnitId> {
        if ctx.state.is_alive(ctx.primary_target) { vec![ctx.primary_target] } else { vec![] }
    }

    pub fn sel_all_enemies(ctx: &SelectorCtx) -> Vec<UnitId> {
        ctx.state.enemies_of(ctx.caster).into_iter()
            .filter(|e| ctx.state.is_alive(*e))
            .collect()
    }

    pub fn sel_all_allies(ctx: &SelectorCtx) -> Vec<UnitId> {
        ctx.state.allies_of(ctx.caster).into_iter()
            .filter(|e| ctx.state.is_alive(*e))
            .collect()
    }

    pub fn sel_adjacent_to_primary(ctx: &SelectorCtx) -> Vec<UnitId> {
        ctx.state.adjacent_to(ctx.primary_target).into_iter()
            .filter(|e| *e != ctx.primary_target && ctx.state.is_alive(*e))
            .collect()
    }

    pub fn sel_self_only(ctx: &SelectorCtx) -> Vec<UnitId> {
        vec![ctx.caster]
    }

    pub fn register(reg: &mut ExtRegistries) {
        reg.selectors.register("primary",              sel_primary);
        reg.selectors.register("all_enemies",          sel_all_enemies);
        reg.selectors.register("all_allies",           sel_all_allies);
        reg.selectors.register("adjacent_to_primary",  sel_adjacent_to_primary);
        reg.selectors.register("self_only",            sel_self_only);
    }
}

// ---------- SkillCtx (D024: Mode { Live, DryRun }) ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillCtxMode { Live, DryRun }

pub struct SkillCtx {
    pub mode: SkillCtxMode,
    intents: Vec<Intent>,
}

impl SkillCtx {
    pub fn new(mode: SkillCtxMode) -> Self {
        Self { mode, intents: Vec::new() }
    }
    /// Hook produces effects via this single channel — no direct state mutation.
    pub fn enqueue(&mut self, i: Intent) { self.intents.push(i); }
    pub fn drain(&mut self) -> Vec<Intent> { std::mem::take(&mut self.intents) }

    /// Read accessors backed by the thread-local runtime context the
    /// `BeatRunner` installs for the duration of a step. Production replaces
    /// these with proper borrows on `SkillCtx<'a>` (see F7) — the spike
    /// keeps the function signatures flat so `Registry<E::Fn>` stays Copy.
    pub fn skill_tree(&self, u: UnitId) -> SkillTree { runtime::with_state(|s| s.skill_tree(u)) }
    pub fn blueprint_state(&self, u: UnitId, k: BlueprintStateKey) -> i32 {
        runtime::with_state(|s| s.blueprint_state_value(u, k))
    }
    pub fn identity_of(&self, u: UnitId) -> &'static str {
        runtime::with_state(|s| s.identity_of(u))
    }
    pub fn cast_hit_set(&self) -> HashSet<UnitId> { runtime::with_hit_set(|h| h.clone()) }
    pub fn cast_hit_set_len(&self) -> usize { runtime::with_hit_set(|h| h.len()) }
    /// Targetable-alive enemies of the caster (used to compute pool exhaustion).
    pub fn enemies_alive(&self, caster: UnitId) -> Vec<UnitId> {
        runtime::with_state(|s| s.enemies_of(caster).into_iter()
            .filter(|e| s.is_alive(*e)).collect())
    }
    /// Deterministic RNG draw seeded by `(rng_seed, cast_id, beat_id, hop_index, salt)`.
    /// Same inputs ⇒ same output. Salt lets a hook draw multiple times
    /// within one beat without collision (`ctx.rng_u32(0)`, `ctx.rng_u32(1)`, ...).
    pub fn rng_u32(&self, cast_id: CastId, beat: BeatId, hop_index: u32, salt: u32) -> u32 {
        let seed = runtime::with_state(|s| s.rng_seed);
        // SplitMix64-style step — small, deterministic, no external deps.
        let mut z = seed
            .wrapping_add(cast_id.wrapping_mul(0x9E37_79B9_7F4A_7C15))
            .wrapping_add(hash_str(beat).wrapping_mul(0xBF58_476D_1CE4_E5B9))
            .wrapping_add((hop_index as u64).wrapping_mul(0x94D0_49BB_1331_11EB))
            .wrapping_add((salt as u64).wrapping_mul(0xC2B2_AE35_94CF_5BAD));
        z ^= z >> 30; z = z.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z ^= z >> 27; z = z.wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        z as u32
    }
}

fn hash_str(s: &str) -> u64 {
    // FNV-1a 64. Cheap, deterministic, good enough for keying RNG draws.
    let mut h: u64 = 0xCBF2_9CE4_8422_2325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01B3);
    }
    h
}

// ---------- Runtime context (thread-local borrows for the spike) ----------
//
// Production replaces this with proper `SkillCtx<'a>` borrows; the spike
// uses thread-local raw pointers to keep `Registry<E>::Fn` Copy and the
// hook signature flat. Lives for the duration of `BeatRunner::step`.

pub mod runtime {
    use super::*;
    use std::cell::RefCell;

    thread_local! {
        static REGISTRIES: RefCell<Option<*const ExtRegistries>> = const { RefCell::new(None) };
        static STATE:      RefCell<Option<*const CombatStateMock>> = const { RefCell::new(None) };
        static HIT_SET:    RefCell<Option<*const HashSet<UnitId>>> = const { RefCell::new(None) };
    }

    pub struct RuntimeGuard {
        // Stack-safe: each guard remembers what the slot held when it was
        // installed and restores that on drop. Tests can install an outer
        // guard while `BeatRunner` installs nested ones during steps.
        prev_reg: Option<*const ExtRegistries>,
        prev_state: Option<*const CombatStateMock>,
        prev_hit_set: Option<*const HashSet<UnitId>>,
    }

    impl RuntimeGuard {
        pub fn install(
            reg: &ExtRegistries,
            state: &CombatStateMock,
            hit_set: &HashSet<UnitId>,
        ) -> Self {
            let prev_reg = REGISTRIES.with(|c| c.replace(Some(reg as *const _)));
            let prev_state = STATE.with(|c| c.replace(Some(state as *const _)));
            let prev_hit_set = HIT_SET.with(|c| c.replace(Some(hit_set as *const _)));
            Self { prev_reg, prev_state, prev_hit_set }
        }
    }

    impl Drop for RuntimeGuard {
        fn drop(&mut self) {
            REGISTRIES.with(|c| *c.borrow_mut() = self.prev_reg);
            STATE.with(|c| *c.borrow_mut() = self.prev_state);
            HIT_SET.with(|c| *c.borrow_mut() = self.prev_hit_set);
        }
    }

    pub(crate) fn with_state<R>(f: impl FnOnce(&CombatStateMock) -> R) -> R {
        let ptr = STATE.with(|c| *c.borrow())
            .expect("RuntimeGuard not installed before invoking state accessor");
        // SAFETY: pointer comes from a guard alive for the entire step,
        // dropped before the harness returns. Single-threaded by design.
        let state = unsafe { &*ptr };
        f(state)
    }

    pub(crate) fn with_hit_set<R>(f: impl FnOnce(&HashSet<UnitId>) -> R) -> R {
        let ptr = HIT_SET.with(|c| *c.borrow())
            .expect("RuntimeGuard not installed before invoking hit_set accessor");
        let hs = unsafe { &*ptr };
        f(hs)
    }

    pub fn with_registries<R>(f: impl FnOnce(&ExtRegistries) -> R) -> R {
        let ptr = REGISTRIES.with(|c| *c.borrow())
            .expect("RuntimeGuard not installed before invoking registries accessor");
        let reg = unsafe { &*ptr };
        f(reg)
    }
}

// ---------- Beat / Timeline ----------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdvanceMode {
    /// Move to the next beat as soon as entry side-effects fire.
    Auto,
    /// Wait for `signal_bus.consume(name)` (Windowed) — auto-pass in HeadlessAuto.
    OnSignal(SignalName),
}

#[derive(Debug, Clone)]
pub enum BeatKind {
    Cast,
    Phase,
    /// `selector` is a string ID resolved against `Registry<SelectorExt>`.
    /// Kernel-provided primitives are registered by `builtin::register`;
    /// skill-specific selectors are registered by their blueprint.
    Impact { selector: SelectorId },
    Aftermath,
    /// A bounded iteration over a sub-timeline. The runner enters `body`
    /// from its first beat, walks each contained beat once, then evaluates
    /// `exit_when` against the current `BeatEvent` (carrying `hop_index`).
    /// If false, body restarts with `hop_index += 1`. If true, the Loop beat
    /// exits and the runner takes outgoing edges from the enclosing beat.
    ///
    /// `exit_when` is the canonical termination predicate. Pool exhaustion,
    /// tier-aware caps, target-pred conditions all express through it.
    Loop { body: Vec<Beat>, exit_when: PredicateId },
}

// `BeatKind` no longer derives `PartialEq`/`Eq` because `Loop` contains a
// `Vec<Beat>` and structural equality on a beat is not needed anywhere — the
// kernel matches on shape only (`matches!`, `if let`).

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Cue {
    pub anim: Option<&'static str>,
    pub vfx:  Option<&'static str>,
    pub sfx:  Option<&'static str>,
}

/// A beat's presentation is either a static cue (pure data, RON-authored)
/// or a dynamic one resolved by a `CueExt` fn (e.g. anim varies with HP%).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Presentation {
    Static(Cue),
    Dynamic(CueId),
}

#[derive(Debug, Clone)]
pub struct Beat {
    pub id: BeatId,
    pub kind: BeatKind,
    pub presentation: Option<Presentation>,
    pub hook: Option<HookId>,
    pub advance: AdvanceMode,
}

#[derive(Debug, Clone)]
pub struct BeatEdge {
    pub from: BeatId,
    pub to: BeatId,
    /// `None` ⇒ always passes. Otherwise a registered `PredicateFn`.
    pub gate: Option<PredicateId>,
}

#[derive(Debug, Clone)]
pub struct CompiledTimeline {
    pub id: &'static str,
    pub entry: BeatId,
    pub beats: Vec<Beat>,
    pub edges: Vec<BeatEdge>,
}

impl CompiledTimeline {
    pub fn beat(&self, id: BeatId) -> &Beat {
        self.beats.iter().find(|b| b.id == id)
            .unwrap_or_else(|| panic!("beat {id:?} not found in timeline"))
    }

    /// First edge from `id` whose gate passes (or unconditional).
    pub fn next_from(
        &self,
        id: BeatId,
        ctx: &SkillCtx,
        evt: &BeatEvent,
        predicates: &Registry<PredicateExt>,
    ) -> Option<BeatId> {
        for e in &self.edges {
            if e.from != id { continue; }
            let pass = match &e.gate {
                None => true,
                Some(p) => predicates.get(p).is_some_and(|f| f(evt, ctx)),
            };
            if pass { return Some(e.to); }
        }
        None
    }
}

// ---------- Skill-tree patches (compile-time timeline rewriting) ----------

#[derive(Debug, Clone)]
pub enum TimelinePatchOp {
    /// Splice `beat` between `anchor` and whatever currently follows it.
    /// All edges `anchor → X` become `anchor → beat → X` (gates preserved).
    InsertBeatAfter {
        anchor: BeatId,
        beat: Beat,
        edge_gate: Option<PredicateId>,
    },
    /// Add a gate to every unconditional edge entering `id`.
    GateBeat { id: BeatId, gate: PredicateId },
    /// Replace the hook bound to `beat`.
    ReplaceHook { beat: BeatId, hook: HookId },
}

pub fn compile_timeline(base: CompiledTimeline, patches: &[TimelinePatchOp]) -> CompiledTimeline {
    let mut t = base;
    for op in patches {
        match op {
            TimelinePatchOp::InsertBeatAfter { anchor, beat, edge_gate } => {
                let new_id: BeatId = beat.id;
                let outgoing: Vec<BeatEdge> = t.edges.iter()
                    .filter(|e| e.from == *anchor).cloned().collect();
                t.edges.retain(|e| e.from != *anchor);
                t.edges.push(BeatEdge { from: *anchor, to: new_id, gate: *edge_gate });
                for e in outgoing {
                    t.edges.push(BeatEdge { from: new_id, to: e.to, gate: e.gate });
                }
                t.beats.push(beat.clone());
            }
            TimelinePatchOp::GateBeat { id, gate } => {
                for e in t.edges.iter_mut() {
                    if e.to == *id && e.gate.is_none() {
                        e.gate = Some(*gate);
                    }
                }
            }
            TimelinePatchOp::ReplaceHook { beat, hook } => {
                if let Some(b) = t.beats.iter_mut().find(|b| b.id == *beat) {
                    b.hook = Some(*hook);
                }
            }
        }
    }
    t
}

// ---------- Validation (kernel-side `App::finish` equivalent) ----------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub timeline: &'static str,
    pub axis: &'static str,
    pub id: String,
    pub site: String,
}

/// Verifies every string ID referenced by a `CompiledTimeline` exists in the
/// corresponding registry. Mirrors the production `App::finish()` validator.
pub fn validate_timeline_refs(
    tl: &CompiledTimeline,
    reg: &ExtRegistries,
) -> Result<(), Vec<ValidationError>> {
    let mut errs = Vec::new();
    for b in &tl.beats {
        validate_beat(b, tl.id, reg, &mut errs);
    }
    for e in &tl.edges {
        if let Some(p) = e.gate {
            if !reg.predicates.contains(p) {
                errs.push(ValidationError {
                    timeline: tl.id,
                    axis: PredicateExt::KIND,
                    id: p.to_string(),
                    site: format!("edge `{} → {}`", e.from, e.to),
                });
            }
        }
    }
    if errs.is_empty() { Ok(()) } else { Err(errs) }
}

/// Recursive helper so `BeatKind::Loop` body beats are validated just like
/// top-level beats. Validation reaches every string ID referenced by the
/// timeline, including loop-body axes.
fn validate_beat(
    b: &Beat,
    timeline_id: &'static str,
    reg: &ExtRegistries,
    errs: &mut Vec<ValidationError>,
) {
    match &b.kind {
        BeatKind::Impact { selector } => {
            if !reg.selectors.contains(selector) {
                errs.push(ValidationError {
                    timeline: timeline_id,
                    axis: SelectorExt::KIND,
                    id: selector.to_string(),
                    site: format!("beat `{}`", b.id),
                });
            }
        }
        BeatKind::Loop { body, exit_when } => {
            if !reg.predicates.contains(exit_when) {
                errs.push(ValidationError {
                    timeline: timeline_id,
                    axis: PredicateExt::KIND,
                    id: exit_when.to_string(),
                    site: format!("beat `{}` (loop exit_when)", b.id),
                });
            }
            for body_beat in body {
                validate_beat(body_beat, timeline_id, reg, errs);
            }
        }
        _ => {}
    }
    if let Some(hid) = b.hook {
        if !reg.hooks.contains(hid) {
            errs.push(ValidationError {
                timeline: timeline_id,
                axis: HookExt::KIND,
                id: hid.to_string(),
                site: format!("beat `{}`", b.id),
            });
        }
    }
    if let Some(Presentation::Dynamic(cid)) = b.presentation {
        if !reg.cues.contains(cid) {
            errs.push(ValidationError {
                timeline: timeline_id,
                axis: CueExt::KIND,
                id: cid.to_string(),
                site: format!("beat `{}` (dynamic cue)", b.id),
            });
        }
    }
}

// ---------- Signal bus + clock model ----------

#[derive(Debug, Default)]
pub struct SignalBus { pending: VecDeque<SignalName> }

impl SignalBus {
    pub fn emit(&mut self, s: SignalName) { self.pending.push_back(s); }
    pub fn consume(&mut self, s: SignalName) -> bool {
        if let Some(idx) = self.pending.iter().position(|p| *p == s) {
            self.pending.remove(idx);
            true
        } else {
            false
        }
    }
    pub fn pending_count(&self) -> usize { self.pending.len() }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clock {
    /// Headless: signals auto-pass on the same step. Deterministic.
    HeadlessAuto,
    /// Windowed: signals must arrive externally.
    Windowed,
}

// ---------- Beat runner ----------

/// Loop iteration state held while the runner is walking a `BeatKind::Loop`
/// body. Single-level only — the spike doesn't need nested loops.
struct LoopFrame {
    /// The Loop beat in the outer timeline that contains this body.
    enclosing_loop_beat: BeatId,
    /// Index into `body[..]` of the beat we are currently executing.
    body_cursor: usize,
    /// `0..N` counting iterations through the body. Surfaces to predicates
    /// via `BeatEvent.hop_index`.
    hop_index: u32,
    /// Predicate that, when true after a body iteration, exits the loop.
    exit_when: PredicateId,
}

pub struct BeatRunner<'a> {
    timeline: &'a CompiledTimeline,
    registries: &'a ExtRegistries,
    state: &'a CombatStateMock,
    signals: &'a mut SignalBus,
    clock: Clock,
    base_event: BeatEvent,
    cast_id: CastId,
    cursor: Option<BeatId>,
    /// True between BeatEntered (entry side-effects fired) and BeatExited.
    entered_current: bool,
    /// Targets resolved by the most recent `BeatKind::Impact` selector.
    /// Gates evaluated by `next_from` see these via `evt.beat_targets`, so
    /// a predicate like `has_adjacent_targets` can decide based on what the
    /// previous beat actually targeted instead of recomputing.
    last_beat_targets: Vec<UnitId>,
    /// Loop iteration frame. `None` when the runner is in the linear
    /// outer timeline; `Some` while walking a `BeatKind::Loop` body.
    loop_frame: Option<LoopFrame>,
    /// Set of unit IDs hit by `Intent::DealDamage` during this cast.
    /// Bounce selectors read this through `SkillCtx::cast_hit_set()` to
    /// avoid retargeting the same unit twice. Cleared on cast start.
    cast_hit_set: HashSet<UnitId>,
    pub events: Vec<CombatEvent>,
    pub all_intents: Vec<Intent>,
}

impl<'a> BeatRunner<'a> {
    pub fn new(
        timeline: &'a CompiledTimeline,
        registries: &'a ExtRegistries,
        state: &'a CombatStateMock,
        signals: &'a mut SignalBus,
        clock: Clock,
        cast_id: CastId,
        base_event: BeatEvent,
    ) -> Self {
        Self {
            cursor: Some(timeline.entry),
            timeline, registries, state, signals, clock,
            base_event, cast_id,
            entered_current: false,
            last_beat_targets: Vec::new(),
            loop_frame: None,
            cast_hit_set: HashSet::new(),
            events: Vec::new(),
            all_intents: Vec::new(),
        }
    }

    /// Test-only accessors so the validation suite can drive Windowed flows
    /// without exposing the bus to production callers.
    pub fn cursor_for_test(&self) -> Option<BeatId> { self.cursor }
    pub fn signals_for_test(&mut self) -> &mut SignalBus { self.signals }
    pub fn cast_hit_set_for_test(&self) -> &HashSet<UnitId> { &self.cast_hit_set }
    pub fn loop_hop_index_for_test(&self) -> Option<u32> {
        self.loop_frame.as_ref().map(|f| f.hop_index)
    }

    /// Helper: install RuntimeGuard then invoke `f` with a fresh closure-scoped
    /// access to the thread-local context. Used everywhere a predicate, hook,
    /// selector, or cue needs to reach back into state / hit_set.
    fn with_runtime<R>(&self, f: impl FnOnce() -> R) -> R {
        let _g = runtime::RuntimeGuard::install(
            self.registries, self.state, &self.cast_hit_set);
        f()
    }

    /// Execute a single beat (selector + presentation + hook). Returns the
    /// targets the selector resolved, so the caller can use them to feed
    /// `last_beat_targets` (linear path) or to expose them to the loop
    /// `exit_when` evaluation (loop body path).
    fn execute_beat(
        &mut self,
        beat: &Beat,
        mode: SkillCtxMode,
        hop_index: u32,
    ) -> Vec<UnitId> {
        self.events.push(CombatEvent::BeatEntered {
            beat: beat.id, cast_id: self.cast_id,
        });

        // 1. Resolve selector for Impact beats.
        let beat_targets = match &beat.kind {
            BeatKind::Impact { selector } => {
                let sel = self.registries.selectors.get(selector).unwrap_or_else(|| {
                    panic!("selector `{selector}` not registered \
                           (validate_timeline_refs would have caught this at App::finish)")
                });
                let sctx = SelectorCtx {
                    caster: self.base_event.caster,
                    primary_target: self.base_event.primary_target,
                    state: self.state,
                    mode,
                };
                self.with_runtime(|| sel(&sctx))
            }
            _ => Vec::new(),
        };

        // 2. Presentation cue — static OR dynamic-resolved.
        if let Some(p) = &beat.presentation {
            let cue = match p {
                Presentation::Static(c) => c.clone(),
                Presentation::Dynamic(id) => {
                    let resolver = self.registries.cues.get(id).unwrap_or_else(|| {
                        panic!("cue resolver `{id}` not registered")
                    });
                    let cctx = CueCtx { caster: self.base_event.caster, state: self.state };
                    let resolved_id = self.with_runtime(|| resolver(&cctx));
                    Cue { anim: Some(resolved_id), vfx: None, sfx: None }
                }
            };
            self.events.push(CombatEvent::PresentationCue {
                beat: beat.id, anim: cue.anim, vfx: cue.vfx, sfx: cue.sfx,
            });
        }

        // 3. Hook fn — produces Intents via SkillCtx.enqueue.
        if let Some(hid) = beat.hook {
            let f = self.registries.hooks.get(&hid).unwrap_or_else(|| {
                panic!("hook `{hid}` not registered")
            });
            let mut ctx = SkillCtx::new(mode);
            let mut evt = self.base_event.clone();
            evt.beat = beat.id;
            evt.beat_targets = beat_targets.clone();
            evt.hop_index = hop_index;
            self.with_runtime(|| f(&evt, &mut ctx));
            let new_intents = ctx.drain();
            // After hooks fire, fold any DealDamage targets into the cast
            // hit set so subsequent bounce selectors can skip them.
            for i in &new_intents {
                if let Intent::DealDamage { target, .. } = i {
                    self.cast_hit_set.insert(*target);
                }
            }
            self.all_intents.extend(new_intents);
        }

        beat_targets
    }

    /// Evaluate a predicate id with the runtime guard installed.
    fn eval_predicate(
        &self,
        pred: PredicateId,
        evt: &BeatEvent,
        mode: SkillCtxMode,
    ) -> bool {
        let f = self.registries.predicates.get(pred).unwrap_or_else(|| {
            panic!("predicate `{pred}` not registered")
        });
        let ctx = SkillCtx::new(mode);
        self.with_runtime(|| f(evt, &ctx))
    }

    /// Process the current beat one phase: entry (if not yet) or advance check.
    /// Returns `true` while the runner is still active.
    pub fn step(&mut self, mode: SkillCtxMode) -> bool {
        // Loop body path: if we're inside a loop frame, walk its body.
        if let Some(frame) = self.loop_frame.as_ref() {
            let enclosing = frame.enclosing_loop_beat;
            let body_cursor = frame.body_cursor;
            let hop_index = frame.hop_index;
            let exit_when = frame.exit_when;

            // Look up the Loop beat to access its body. Clone the current
            // body[cursor] beat so we don't keep an immutable borrow on
            // self.timeline while we mutate self.cast_hit_set etc.
            let cur_beat: Beat = {
                let lb = self.timeline.beat(enclosing);
                let BeatKind::Loop { body, .. } = &lb.kind else {
                    panic!("loop_frame points at non-Loop beat `{enclosing}`")
                };
                body[body_cursor].clone()
            };

            let beat_targets = self.execute_beat(&cur_beat, mode, hop_index);
            if matches!(cur_beat.kind, BeatKind::Impact { .. }) {
                self.last_beat_targets = beat_targets;
            }
            self.events.push(CombatEvent::BeatExited {
                beat: cur_beat.id, cast_id: self.cast_id,
            });

            // Decide what comes next inside the loop.
            let body_len = {
                let lb = self.timeline.beat(enclosing);
                let BeatKind::Loop { body, .. } = &lb.kind else { unreachable!() };
                body.len()
            };
            if body_cursor + 1 < body_len {
                // More beats left in this iteration.
                self.loop_frame.as_mut().unwrap().body_cursor = body_cursor + 1;
            } else {
                // End of body iteration. Evaluate exit_when.
                let evt_for_exit = BeatEvent {
                    caster: self.base_event.caster,
                    primary_target: self.base_event.primary_target,
                    beat_targets: self.last_beat_targets.clone(),
                    cast_id: self.cast_id,
                    beat: enclosing,
                    hop_index,
                };
                let should_exit = self.eval_predicate(exit_when, &evt_for_exit, mode);
                if should_exit {
                    // Pop frame and continue the outer timeline.
                    self.loop_frame = None;
                    let probe_ctx = SkillCtx::new(mode);
                    self.cursor = self.with_runtime(|| {
                        self.timeline.next_from(
                            enclosing, &probe_ctx, &evt_for_exit, &self.registries.predicates)
                    });
                    self.events.push(CombatEvent::BeatExited {
                        beat: enclosing, cast_id: self.cast_id,
                    });
                    self.entered_current = false;
                    return self.cursor.is_some();
                } else {
                    // Restart body, increment hop_index.
                    let f = self.loop_frame.as_mut().unwrap();
                    f.body_cursor = 0;
                    f.hop_index += 1;
                }
            }
            return true;
        }

        // Linear path: walk outer timeline beat-by-beat.
        let Some(beat_id) = self.cursor else { return false; };
        let beat = self.timeline.beat(beat_id).clone();

        if !self.entered_current {
            match &beat.kind {
                BeatKind::Loop { body, exit_when } => {
                    // Loop beat entry: emit BeatEntered for the enclosing
                    // Loop, then install a frame and let subsequent steps
                    // walk the body. The Loop beat itself produces no
                    // selector/hook/cue effects — those concepts belong to
                    // body beats.
                    self.events.push(CombatEvent::BeatEntered {
                        beat: beat_id, cast_id: self.cast_id,
                    });
                    if body.is_empty() {
                        panic!("Loop beat `{beat_id}` has empty body — invalid timeline");
                    }
                    self.loop_frame = Some(LoopFrame {
                        enclosing_loop_beat: beat_id,
                        body_cursor: 0,
                        hop_index: 0,
                        exit_when: *exit_when,
                    });
                    self.entered_current = true;
                    return true;
                }
                _ => {
                    let beat_targets = self.execute_beat(&beat, mode, 0);
                    if matches!(beat.kind, BeatKind::Impact { .. }) {
                        self.last_beat_targets = beat_targets;
                    }
                    self.entered_current = true;
                }
            }
        }

        let can_advance = match &beat.advance {
            AdvanceMode::Auto => true,
            AdvanceMode::OnSignal(sig) => match self.clock {
                Clock::HeadlessAuto => true,
                Clock::Windowed => self.signals.consume(*sig),
            },
        };
        if !can_advance { return true; }

        self.events.push(CombatEvent::BeatExited {
            beat: beat_id, cast_id: self.cast_id,
        });
        let evt_for_gate = BeatEvent {
            caster: self.base_event.caster,
            primary_target: self.base_event.primary_target,
            beat_targets: self.last_beat_targets.clone(),
            cast_id: self.cast_id,
            beat: beat_id,
            hop_index: 0,
        };
        let probe_ctx = SkillCtx::new(mode);
        self.cursor = self.with_runtime(|| self.timeline.next_from(
            beat_id, &probe_ctx, &evt_for_gate, &self.registries.predicates));
        self.entered_current = false;
        self.cursor.is_some()
    }

    pub fn run_to_completion(&mut self, mode: SkillCtxMode, max_steps: usize) {
        for _ in 0..max_steps {
            if !self.step(mode) { return; }
        }
        panic!("BeatRunner exceeded max_steps={max_steps}; possible infinite loop");
    }
}

pub mod agumon;
pub mod dorumon;
pub mod gabumon;
pub mod tentomon;
