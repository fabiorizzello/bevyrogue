# §2.2d — VFX positioning model (Locus + Motion)

> **Scope.** Define WHERE a `SpawnParticle` command places its VFX and HOW it moves. Single source of truth for the `origin`/`motion` fields of `SpawnParticle`.
>
> **Supersedes** the `AnchorRef` enum proposal in `digimon/agumon/02_skill_baby_flame.md` §6.4. Rationale: no sprite rig / bone system exists. Atlas frames are flat pre-rendered 2D — body-part anchors (`Mouth`, `Antennae`, `ClawTip`) cannot be resolved to coordinates without per-frame metadata that doesn't exist. Anchoring to "body parts" was fake precision.

## §A — Coordinate sources (what's actually known)

What the runtime actually has at VFX spawn time:

| Source | Available | Type |
|---|---|---|
| Self sprite bbox center | ✅ — sprite is positioned on combat line | `Vec2` world coord |
| Target sprite bbox center | ✅ — target entity has Transform | `Vec2` |
| Adjacent enemies bbox center | ✅ — line topology gives left/right neighbors | `Vec2` |
| Grid cell coord | ✅ — combat line is a grid | `IVec2` → world `Vec2` |
| Sprite bone / body part | ❌ — flat 2D atlas, no rig | — |
| Per-frame anchor metadata | ❌ — atlas json has no anchor data | — |

**Design implication:** VFX positioning uses bbox centers + grid + explicit world coords. No bones, no per-frame data.

## §B — `VfxLocus` enum (origin point)

```rust
pub enum VfxLocus {
    /// Self sprite bbox center (default for "from caster").
    SelfCenter,

    /// Self bbox top edge + small offset (for "above caster" auras, charge glows).
    SelfAbove,

    /// Primary target sprite bbox center (for impact bursts, status locks).
    /// Kept as a shorthand alias for `EntityCenter(EntityRef::Primary)`.
    TargetCenter,

    /// Adjacent enemy by index (-1 = left of primary, +1 = right). Resolved
    /// against the combat line at emission time.
    Adj(i8),

    /// Explicit grid cell. Used for AoE epicenters that are not unit-anchored
    /// (e.g. screen-center for full-board effects).
    WorldGrid(IVec2),

    /// Sprite bbox center of an arbitrary entity selected via `EntityRef`.
    /// Superset of `SelfCenter` / `TargetCenter`; necessary when the entity
    /// is not the primary target (passive marks on tracked unit, partner
    /// links, listener-emitted FX bound to an event participant).
    EntityCenter(EntityRef),
}

pub enum EntityRef {
    /// The blueprint's own unit. Equivalent to `SelfCenter` when used as an
    /// origin; useful for symmetry inside `EntityCenter(...)`.
    Self_,

    /// Primary target resolved by the active skill. Equivalent to
    /// `TargetCenter`. Only meaningful inside an FSM-emitted `SpawnParticle`.
    Primary,

    /// Caster of the kernel event being reacted to. Listener-only context
    /// (e.g. `StatusApplied { caster, .. }`).
    Caster,

    /// Target of the kernel event being reacted to. Listener-only context.
    EventTarget,

    /// Read `blueprint_state[key]` and resolve to an `EntityId`. Mirrors
    /// `ParamRef::BlueprintState` from §2.2b §S-Param. Drives passive marks
    /// whose target is owned by listener state (e.g. Dorumon
    /// `tracked_target`, Twin Core partner link).
    FromBlueprintState(String),

    /// Read a snapshot-stored entity reference. Symmetric to
    /// `ParamRef::Snapshot`; rare, useful only when the FSM has captured an
    /// entity at commit-time and wants to re-fetch it later in the same
    /// resolution cascade.
    FromParamSnapshot(String),
}
```

**Resolution:** `VfxLocus → Vec2` is a pure function of (self transform, target transform, line topology, grid, blueprint_state, snapshot store). No per-unit rig table. Same resolver for every Digimon.

**`EntityRef` failure modes:** if `FromBlueprintState(k)` returns `None` (key missing) or the referenced entity is despawned/dead, the spawn is **dropped silently** (no panic, no fallback). One-shot FX (`Static`/`Radial`) thus simply don't appear; for `Travel` whose `to` becomes invalid mid-flight, see §H.4 (snapshot-once policy unchanged — endpoint Vec2 captured at `on_enter`, particle lands at the snapshotted point).

**Listener context (`Caster` / `EventTarget`):** only resolvable when a `SpawnParticle` is emitted from a listener via the §2.2e `notify` channel. FSM-emitted `SpawnParticle` cannot use these refs (no kernel-event context) — validator rejects.

**Headless:** `VfxLocus` is never resolved — `SpawnParticle` is cosmetic (no-op headless per §2.2b §G). Resolver only runs under `feature = "windowed"`.

## §B.1 — RON iteration syntax for `EntityRef::FromParamSnapshot`

Skills with AoE per-target loops (Tentomon `petit_thunder` Bounce(3), Renamon `koyosetsu`/`tohakken`, Patamon `sparking_air_shot` hybrid) emit one `SpawnParticle` per iteration of a RON-side loop. To reference the loop-current entity inside a `VfxLocus`, blueprints use a snapshot-key naming convention `<iter:loop_var>`:

```ron
// inside Renamon koyosetsu §2 RON, per-enemy impact:
for enemy_i in enemies_alive {
    SpawnParticle(
        name: "diamond_shard_impact",
        origin: EntityCenter(FromParamSnapshot("<iter:enemy_i>")),
        motion: Static,
    )
}
```

**Resolution rule:** at FSM commit time, the blueprint resolver writes one snapshot entry per loop iteration with key `<iter:<var_name>>` (literal angle brackets) and value = current iter entity. The resolver clears these keys at the end of the loop scope (no leakage to subsequent commands).

**Three manifestations of `EntityRef`** are now formalized (closes the §02-02b N8-extended gap accumulated across Tentomon `hop1_target` / Renamon `paralyzed_target` / per-enemy + per-ally loops):

| Manifestation | Snapshot key form | Lifetime | Example |
|---|---|---|---|
| Fixed (`Primary` / `Self_` / `Caster` / `EventTarget`) | none (resolved live) | per-spawn | Patamon basic impact on Primary |
| `FromBlueprintState("key")` | persistent state, listener-owned | listener lifetime | Dorumon `tracked_target` for predator mark |
| `FromParamSnapshot("key")` | snapshot at FSM commit | one resolution cascade | Tentomon hop targets, Renamon `paralyzed_target` |
| `FromParamSnapshot("<iter:var>")` | per-iteration snapshot | one loop iteration | per-enemy/per-ally AoE spawns |

The `<iter:...>` syntax is **literal in the RON** (angle brackets included) — distinguishes loop-iter snapshots from regular snapshot keys at validator level. Keys with `<iter:` prefix that are not bound by an active loop scope are rejected by the §2.2b §L validator.

## §C — `VfxMotion` enum (movement model)

```rust
pub enum VfxMotion {
    /// Particle spawns at origin and plays its own loop/lifetime in place.
    Static,

    /// Particle travels from origin to `to`, along easing curve, duration ms.
    /// Used for projectiles, chain arcs, beam tracers. Both `origin` (the
    /// outer `SpawnParticle.origin`) and `to` are `VfxLocus` — any pair is
    /// legal, including entity→entity beams via `EntityCenter`.
    Travel { to: VfxLocus, ease: Ease, ms: u32 },

    /// Particle expands radially inward to outward from origin out to
    /// `range_tiles`, duration ms. Used for AoE shock waves whose epicenter
    /// is the origin (full-body discharges, world-pivot detonations).
    Radial { range_tiles: f32, ms: u32 },

    /// Particle expands radially outward from origin out to `range_tiles`,
    /// duration ms. Distinct from `Radial` in motion semantics (§C.2):
    /// `RadialOut` signals **dispersal/grant** (Patamon ult `holy_burst_split`,
    /// Renamon ult `holy_shockwave`), while `Radial` signals **coverage/cover**
    /// (Tentomon ult discharge filling the field). Visually similar; the
    /// distinction is for animator presets to read direction-of-attention.
    RadialOut { range_tiles: f32, ms: u32 },

    /// Particle rises vertically from origin over `height_tiles`, duration ms.
    /// Used for heal motes, blessing absorption, buff-grant uprising (Renamon
    /// `blessed_motes` per-ally, Patamon `holy_heal_burst` post-heal, Patamon
    /// `holy_heal_mist` per-ally hybrid ult).
    RiseUp { height_tiles: f32, ms: u32 },
}

pub enum Ease { Linear, EaseIn, EaseOut, EaseInOut }
```

**Why five motions, not three:**
- `Static` covers ~60% of cases (charge auras, impact bursts, status locks).
- `Travel` is irreducible for projectile/chain VFX (Bounce skills, projectile basics).
- `Radial` is irreducible for AoE expand-and-cover (Ultimates, full-body discharges).
- `RadialOut` (new) distinguishes "dispersal/grant" semantics from `Radial` "coverage" (§C.2 motion semantics convention).
- `RiseUp` (new) is irreducible for heal/buff-grant verticals — `Travel` with upward `to` doesn't carry the same animator preset hint (heal is *not* a projectile).

No sixth verb in v0. If a skill needs something exotic (e.g. follow path, orbit), document it as deferred gap rather than expanding the enum prematurely.

## §C.1 — Travel sync timing convention (R10)

Skills that pair a `Travel` motion with a `Static` impact on arrival (sharp_claws / dash_metal / metal_cannon / petit_thunder / tohakken phantom_paw / patapata_hover descent / sparking_air_shot bubble) must satisfy a timing invariant for visual coherence: **the impact `SpawnParticle` is emitted on the node whose `frames` start at the Travel arrival frame, not earlier and not later.**

```
Travel.ms ≤ (node.frames_end - node.frames_start) * frame_ms
impact node entry frame ≥ Travel emit frame + Travel.ms / frame_ms
```

**Concrete pattern (canonical):** put the `Travel` `SpawnParticle` in one FSM node (`Throw`/`Wind`/`Charge`), put the impact `Static` `SpawnParticle` in the *next* node (`Impact`/`Strike`/`Land`). The edge `TimeInNode` between them gates on the Travel node frame budget — by the time the FSM transitions into the impact node, the Travel particle has visually arrived.

**What this rules out:**
- Travel + impact in the same `on_enter` block (impact would spawn before particle arrives).
- Travel with `ms` longer than the host node frame budget (particle still in flight when next node fires).

**Validator hook (§2.2b §L extension):** for every FSM node with `on_enter` containing `SpawnParticle` of motion `Travel`, walk outgoing edges and check that the **first time-only successor** node's frame budget allows the Travel to complete before the successor's `on_enter` fires. Warning if violated; error if the successor's `on_enter` references an impact preset that pairs with the Travel preset (preset-pair table maintained in `presentation/vfx_presets.rs`).

**Headless implication:** none — `SpawnParticle` is no-op headless (§G), timing convention is pure presentation-layer rule.

## §C.2 — Motion semantics convention (K6)

`Travel` and `RadialOut` carry semantically meaningful **direction of attention**. The convention exists because the same geometry (point A → point B) can sell two different gameplay intents depending on direction:

| Motion + direction | Semantic | Examples |
|---|---|---|
| `Travel { from: SelfCenter, to: EntityCenter(Primary) }` | **outward: grant / projectile / attack** | Patamon basic charge→slam, Gabumon `ice_bubble_travel`, Renamon `phantom_paw_projectile` |
| `Travel { from: EntityCenter(prev_target), to: EntityCenter(next_target) }` | **chain / link / echo** | Tentomon `lightning_arc` hop-N→hop-N+1, Gabumon `ice_echo_link` |
| `Travel { from: EntityCenter(Caster), to: SelfCenter }` | **inward: steal / absorb / link-back** | Twin Core fire-arm Gabumon→Agumon partner link, Patamon `descent_blessing_trail` apex→ally (downward not outward) |
| `RadialOut` | **dispersal / grant** (the caster emits, the field receives) | Patamon ult `holy_burst_split`, Renamon ult `holy_shockwave` |
| `Radial` | **coverage / cover** (the caster fills the field volumetrically) | Tentomon ult `lightning_storm` full-body discharge, Renamon `diamond_shards_rain` overhead column |

**Why bother codifying:** animator preset choices follow the semantic, not the geometry. A `RadialOut "holy_burst"` preset uses outward radial gradient + sparkle dispersal; a `Radial "lightning_storm"` preset uses inward-and-outward turbulence + sustained presence. Same enum field (`range_tiles`/`ms`), different artistic intent. Mixing presets across semantic = visually muddied combat.

**Doc requirement:** when a passive/skill doc lists VFX, the motion verb selected (`Travel`/`RadialOut`/`Radial`/`RiseUp`/`Static`) must match the semantic intent. Validator does not enforce this (it's design-side); the doc-review pass does.

## §D — `SpawnParticle` command shape

```rust
pub struct SpawnParticle {
    /// Particle preset name (registered in presentation layer).
    pub name: String,
    /// Where it starts.
    pub origin: VfxLocus,
    /// How it moves (or doesn't).
    pub motion: VfxMotion,
}
```

RON form in blueprints:

```ron
SpawnParticle(
    name: "petit_arc",
    origin: SelfCenter,
    motion: Travel(to: TargetCenter, ease: EaseOut, ms: 100),
)
```

## §E — Common patterns

| Intent | `origin` | `motion` | Example |
|---|---|---|---|
| Charge glow on caster | `SelfCenter` or `SelfAbove` | `Static` | Tentomon ult `BuildUp`: charge aura |
| Impact burst on target | `TargetCenter` | `Static` | Agumon basic Strike: claw burst on target |
| Projectile / beam | `SelfCenter` | `Travel { to: TargetCenter, ... }` | Patamon basic Boom Bubble: projectile |
| Bounce chain segment | `prev TargetCenter` | `Travel { to: next TargetCenter, ... }` | Tentomon skill: arc from hop N to hop N+1 |
| Status lock visual | `TargetCenter` | `Static` | Tentomon skill: paralysis lock on tgt_3 |
| AoE shockwave | `SelfCenter` | `Radial { range, ... }` | Tentomon ult `Discharge`: full-body radial |
| AoE world-pivot | `WorldGrid(epicenter)` | `Radial { range, ... }` | Renamon ult: world-pivot detonation |
| Splash on adj | `Adj(-1)` / `Adj(+1)` | `Static` | Agumon ult Detonate: splash burst |
| **Mark on listener-tracked unit** | `EntityCenter(FromBlueprintState("tracked_target"))` | `Static` | Dorumon passive `predator_mark` on lowest-HP enemy |
| **Listener slash on event target** | `SelfCenter` | `Travel { to: EntityCenter(EventTarget), ... }` | Dorumon passive `predator_lock` entry flash |
| **Partner link beam** | `EntityCenter(FromBlueprintState("partner"))` | `Travel { to: SelfCenter, ... }` | Twin Core fire-arm link Gabumon→Agumon |
| **Event-caster origin** (listener) | `EntityCenter(Caster)` | `Static` | Listener-emitted FX on whoever triggered the kernel event |

## §F — Migration from old `anchor: String` model

Old (deprecated):

```ron
SpawnParticle(name: "petit_arc", anchor: "antennae")
SpawnParticle(name: "fire_breath_cone", anchor: "mouth")
SpawnParticle(name: "celestial_pillar", anchor: "center_pivot")
```

Conversion table for legacy anchor strings used across roster docs:

| Old anchor string | New `origin` | New `motion` |
|---|---|---|
| `"mouth"`, `"antennae"`, `"horn"`, `"tail"`, `"claw"`, `"claws"`, `"weapon"`, `"wings"`, `"feet"`, `"tails"` | `SelfCenter` | `Static` (or `Travel` if projectile) |
| `"self_pivot"`, `"body_center"` | `SelfCenter` | `Static` |
| `"sky_pivot"`, `"head_above"` | `SelfAbove` | `Static` |
| `"primary_pivot"`, `"target"`, `"target_ally_pivot"`, `"tgt_N"` | `TargetCenter` | `Static` |
| `"adj_pivot"`, `"adj_left_pivot"`, `"adj_right_pivot"` | `Adj(-1)` / `Adj(+1)` | `Static` |
| `"center_pivot"`, `"ground"` | `WorldGrid(combat_center)` | `Static` |
| `"new_target_pivot"` (Dorumon chain) | `TargetCenter` (target re-resolved post-chain) | `Static` |

Body-part anchors collapse to `SelfCenter` because there is no rig to distinguish them. Flavor is preserved via the particle `name` (e.g. `"fire_breath_cone"` reads as mouth-emitted purely by its preset; no positional precision needed at the pixel-art scale of this game).

## §G — Determinism + headless

- `VfxLocus` resolution depends only on combat state (transforms, line topology). Deterministic.
- `VfxMotion` is presentation-layer animation. Headless drops the whole `SpawnParticle` (cosmetic command per §2.2b §G). No determinism risk.
- Resolver lives in `src/presentation/` (or wherever `PresentationBus` lives) under `#[cfg(feature = "windowed")]`. Headless builds don't even compile the resolver.

## §H — Open gaps

1. **Adj-N for N > 1.** `VfxLocus::Adj(i8)` supports arbitrary N, but combat line topology in v0 has only 4-wide. Decision: clamp out-of-range Adj to no-op (no panic). Documented in resolver.
2. **`WorldGrid` epicenter API.** How does a blueprint pick the AoE epicenter? Proposal: blueprint computes `IVec2` at `on_enter` time, embeds in the command. Decision deferred until first Renamon-style world-pivot Ult lands.
3. **`Ease` set extension.** v0 has 4 curves. If a skill needs a custom curve (spring, bounce), document as deferred.
4. **Motion `Travel` collision with target death.** If `to: TargetCenter` (or any `EntityCenter(...)`) and the resolved entity dies mid-travel, where does the particle land? Decision: VFX snapshots **the resolved `Vec2`** at `on_enter` (snapshot-once, aligns with cascade snapshot model §2.8 G5). For `EntityRef::FromBlueprintState` whose key changes mid-flight, the same snapshot rule applies — the in-flight particle does not retarget.
5. **`EntityRef` validator coverage.** §2.2b §L validator must reject `EntityRef::Caster` / `EntityRef::EventTarget` inside FSM-emitted `SpawnParticle` (no kernel-event context). Must also warn when `EntityRef::FromBlueprintState(k)` references a key not declared in the owning blueprint's state schema.

## §I — Cross-refs

- §2.2b `02-02b_animation_fsm.md` §C: `SpawnParticle` command in the closed vocabulary. This doc defines its `origin` + `motion` fields.
- §2.2b §G: cosmetic-vs-gameplay tagging — `SpawnParticle` is cosmetic, no-op headless.
- §2.2b §S-Param: `ParamRef::BlueprintState` — `EntityRef::FromBlueprintState` is its entity-typed counterpart, same resolver backing store.
- §2.2e `02-02e_passive_presentation.md`: presentation channel for listeners (`ListenerCtx::notify`) + persistent state-driven emitter pattern that consumes the `EntityCenter` extension defined here.
- §2.8 `02-08_effect_cascade.md` G5: snapshot model for resolution (motion `Travel` follows the same snapshot rule).
- `digimon/agumon/02_skill_baby_flame.md` §6.4: superseded by this doc.

## §J — Persistent state-driven emitters

A single `SpawnParticle` has **preset-driven lifecycle**: the named preset specifies its own loop count and total duration. That is the right shape for one-shot bursts (impact, arm flash, dissipate poof) and short looping FX whose lifetime is intrinsic.

It is **not** the right shape for FX that must live as long as a gameplay condition holds — e.g. an aura that lasts `UntilRoundEnd`, a tracking mark that follows a listener-owned target until cleared, a partner-link beam that fades only when the partner dies. These require **state-driven lifecycle**, not preset-driven.

### Pattern: presentation `VfxEmitter` component

```rust
#[cfg(feature = "windowed")]
#[derive(Component)]
pub struct VfxEmitter {
    pub preset:   String,        // particle preset to re-emit
    pub origin:   VfxLocus,      // re-resolved each emission tick
    pub motion:   VfxMotion,     // typically `Static`; `Travel` is rare for persistent
    pub cadence:  EmitCadence,   // EveryNFrames(n) | Continuous | OneShotOnSpawn
}

pub enum EmitCadence {
    OneShotOnSpawn,              // emits once at spawn, then idles (degenerate; rarely useful)
    EveryNFrames(u32),           // re-spawn the preset every N frames
    Continuous,                  // delegate to preset's own loop, manager only owns lifetime
}
```

The **manager entity** carrying `VfxEmitter` is spawned by a presentation observer (§2.2e §B) when a gameplay condition becomes true, and despawned when it becomes false. The emitter does not know about the condition; it only knows it is alive. **Lifecycle is entirely owned by the observer that spawned it.**

### What this is *not*

- **Not** a new gameplay command. `VfxEmitter` is a presentation component; the gameplay layer never references it.
- **Not** a new RON entry. `clipmontage.ron` / `skills.ron` are not extended. Persistent FX mappings live in Rust (`presentation/passive_observers.rs`, §2.2e §B), one function per buff/state that needs persistent FX.
- **Not** snapshot-once. `VfxEmitter.origin` is **re-resolved every emission tick**: a `predator_mark` whose `tracked_target` migrates mid-round follows the migration (the emitter manager is the same entity; only the resolved `Vec2` changes per tick). For one-shot snapshot semantics, use `SpawnParticle` instead.

### Cross-ref

Full pattern, ownership rules, and the listener↔observer split are in §2.2e. This section only formalizes the **component contract** that the observer layer consumes.
