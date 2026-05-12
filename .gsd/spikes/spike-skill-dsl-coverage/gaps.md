---
spike: SP3
related: RESEARCH.md
status: complete
created: 2026-05-12
---

# SP3 — gaps.md (missing `Effect` variants for M017)

Compact list of `Effect` enum variants + ancillary types that the 24-skill round-3 canon requires but `src/data/skills_ron.rs` lacks today. Each entry: pseudocode payload, dependent skills, decision.

## Add-now-in-M017 (load-bearing for round-3 canon)

### 1. `ApplyBuff`

```rust
pub enum Effect {
    // ...
    ApplyBuff {
        id: BuffId,              // e.g. "twin_core_fire_active", "fur_cloak_dr", "holy_aegis", "blessed"
        target: TargetRef,       // Self_, SingleAlly, AoE { side }, Primary, etc.
        mul: Option<f32>,        // damage / DR multiplier (1.15, 0.20, 0.10, ...); None for tag-only buffs
        kind: BuffKind,          // DR | DamageMul | TempoMul | StatusBuff | Aura
        dur: BuffDuration,       // Turns(u8) | UntilRoundEnd | Permanent
    },
}

pub enum BuffDuration { Turns(u8), UntilRoundEnd, Permanent }
pub enum BuffKind { DR, DamageMul, TempoMul, StatusBuff, Aura }
```

**Skills:** Twin Core Fire (Agumon passive), Twin Core Ice + Fur Cloak (Gabumon passive), Predator Loop active marker (Dorumon passive), Holy Aegis (Patamon passive), Blue Cyclone self DR (Gabumon ult), Petit Thunder self DR (Tentomon skill), Tōhakken Blessed buff to allies (Renamon ult).
**Recommendation:** **add-now-in-M017.** Required by 5 of 6 passives and 3 actives. Largest single coverage win.

### 2. `EmitHeal`

```rust
pub enum Effect {
    // ...
    EmitHeal {
        amount: HealAmount,
        target: TargetRef,
    },
}

pub enum HealAmount { Flat(i32), PctMax(u8) }
```

**Skills:** Patapata Hover (Patamon skill, single-ally `PctMax(25)`), Sparking Air Shot (Patamon ult, all-ally `PctMax(20)`).
**Recommendation:** **add-now-in-M017.** Patamon kit has no healing today; current `Effect::Revive(i32)` covers only KO targets. Heal-on-alive is a structural gap.

### 3. `EmitCleanse`

```rust
pub enum Effect {
    // ...
    EmitCleanse {
        target: TargetRef,
        count: u8,                       // how many statuses to strip
        filter: CleanseFilter,           // All | Negative | Positive | ById(StatusId)
        priority: CleansePriority,       // OldestFirst | NewestFirst | RandomSeeded(SeedSource)
    },
}

pub enum CleanseFilter { All, Negative, Positive, ById(StatusId) }
pub enum CleansePriority { OldestFirst, NewestFirst, RandomSeeded(SeedSource) }
```

**Skills:** Patapata Hover (`SingleAlly, count=1, Negative, OldestFirst`), Sparking Air Shot (`AoE(AllyTeam), count=1, Negative, OldestFirst`), Holy Aegis self-death cleanup (`AoE(AllyTeam), filter=ById("holy_aegis")`).
**Recommendation:** **add-now-in-M017.** Coupled with `EmitHeal` (same skills); cleanse-immune buffs (`Blessed`) require the `Negative` filter to exist.

### 4. `EmitSpGrant`

```rust
pub enum Effect {
    // ...
    EmitSpGrant {
        amount: i32,
        target: TargetRef,               // Self_, SingleAlly, Team
    },
}
```

**Note:** existing `GainSP(i32)` is self-only literal; `EmitSpGrant` adds `target_ref` for `Team` grants. Cap-aware on receiver (clamp at `SpPool.max`), does **not** consume `RoundSpTracker.max_non_basic_per_round` (grant ≠ spend, `tentomon/00 §7 D1`).
**Skills:** Blue Cyclone +1 SP team (Gabumon ult), Electrical Discharge +1 SP team (Tentomon ult), Battery Loop SP feedback to self on ally-spent (Tentomon passive Path A).
**Recommendation:** **add-now-in-M017.** Either extend `GainSP` payload or replace with `EmitSpGrant`. Three skills depend on it.

### 5. Status kind expansion (`StatusEffectKind`)

```rust
pub enum StatusEffectKind {
    // existing: Burn, Freeze, Shock, DeepFreeze
    Heated   { stacks: u8 },             // Agumon — +Fire vuln per stack, cap 6, decay -1/turn
    Chilled  { stacks: u8 },             // Gabumon — +Ice vuln per stack, cap 6
    Slowed   { speed_pct: i32 },         // Gabumon ult — gauge slow
    Paralyzed { skip_chance_pct: u8 },   // Tentomon — roll skip on turn start
    Blessed  { dmg_pct: u8, ult_gen: u8 },  // Renamon ult — buff, cleanse-immune
}
```

**Skills:** Heated → Agumon basic+skill+ult; Chilled → Gabumon basic+skill; Slowed → Gabumon ult; Paralyzed → Tentomon skill+ult; Blessed → Renamon ult.
**Recommendation:** **add-now-in-M017.** Round-3 explicitly drops `Confused`; these 5 are the closed status set. Backwards-compat keep `Burn/Freeze/Shock/DeepFreeze` as compat shims for existing legacy assets.

### 6. `ApplyStatus.stacks` (extension)

```rust
pub struct ApplyStatusArgs {
    pub kind: StatusEffectKind,
    pub duration: u32,
    pub stacks: u8,                      // NEW; default 1 if absent
}
```

**Skills:** Sharp Claws (+1), Baby Flame (+2), Claw Attack (+1), Gabumon Shot (+2 + echo +1).
**Recommendation:** **add-now-in-M017.** Trivial schema extension; current `custom_signals` shim is the legacy hack being retired.

### 7. `TargetShape` expansion

```rust
pub enum TargetShape {
    // existing: Single, Row, AllEnemies, SelfOnly
    Blast,                                       // Agumon ult — primary + 2 adj
    AoE { side: TargetSide },                    // Renamon skill/ult, Patamon ult, Tentomon ult
    Bounce { hits: u8, selector: BounceSelector }, // Tentomon skill
    AdjLowest { metric: AdjMetric, side: TargetSide }, // Gabumon echo
    LowestHpPctAlive { scope: TargetScope },     // Dorumon chain
    RandomEnemyAlive { seed: SeedSource },       // Tentomon ult paralyze target
    SingleAlly,                                  // Patamon heal/cleanse
}

pub enum BounceSelector { NextAliveAdj { scan: Direction }, RandomEnemyAlive { seed: SeedSource } }
pub enum AdjMetric { HpPctMin, HpMin, RawHpMin }
pub enum TargetScope { EnemyTeam, AllyTeam, BothTeams }
pub enum SeedSource { TurnRng, CombatRng }
```

**Also: loosen `validate_skill_book`** — current code rejects non-`Single` shapes for `Implemented` skills. M017 must permit all of the above; only retain rejection for `Deferred(reason: UnimplementedTargetShape)` legacy assets.

**Skills:** every AoE/Blast/Bounce/Adj skill in the roster (15 of 24).
**Recommendation:** **add-now-in-M017.** Largest schema impact, but unavoidable.

### 8. `AdvanceTurn` / `DelayTurn` (replace `TurnAdvance` + `SelfAdvance`)

```rust
pub enum Effect {
    // ...
    AdvanceTurn { actor: TargetRef, pct: i8 },   // pos = advance (gauge -), neg = alias DelayTurn
    DelayTurn   { target: TargetRef, pct: i8 },  // alias of AdvanceTurn with sign flip + multi-target shape
}
```

Clamp `±50% per call`, gauge `[0, 200]` per `renamon/00 §8 D1` + `02-02b §C2.1`.
**Skills:** Kōyōsetsu (self +25%), Tōhakken (all enemies -30%), Kitsune Grace (self +10%), Kyubimon Onibidama Storm (legacy, all enemies +20%).
**Recommendation:** **add-now-in-M017.** Current `Effect::TurnAdvance(i32)` is single-target int with ambiguous semantics; replace with the canon shape. Keep `TurnAdvance(i32)` as a deprecated alias if existing tests depend on it.

### 9. `BlockReaction`

```rust
pub enum Effect {
    // ...
    BlockReaction {
        kind: ReactionKind,              // FollowUp | Counter | All
        target: TargetRef,
        damage_mult: f32,                // typically 0.50
    },
}

pub enum ReactionKind { FollowUp, Counter, All }
```

**Skills:** Battery Loop block proc (Tentomon passive Path B).
**Recommendation:** **add-now-in-M017.** Single dependent skill but the canon pattern is closed (`tentomon/04 §1 X10`), and the kernel cascade ordering ("pre-DR, post-base") needs the variant to land before the damage pipeline can wire it.

### 10. `SetBlueprintState`

```rust
pub enum Effect {
    // ...
    SetBlueprintState {
        state_key: String,               // "predator_active", "battery_charge", ...
        value: ParamValue,               // Bool(true), Int(N), Float(f), Str(s)
        dur: Option<BuffDuration>,       // None = permanent until cleared
    },
}

pub enum ParamValue { Int(i64), Float(f64), Bool(bool), Str(String) }
```

**Skills:** Metal Cannon force-Predator (Dorumon ult). Generic state mutation surface for future passives.
**Recommendation:** **add-now-in-M017.** Without it, force-Predator from Ult is unrepresentable in data; would force Rust hardcode (anti-pattern per CLAUDE.md).

### 11. `DamageTag::Electric`

```rust
pub enum DamageTag {
    Physical, Fire, Ice, Holy, Dark,
    Electric,                            // NEW — Tentomon kit
}
```

**Skills:** Hard Claw, Petit Thunder, Electrical Discharge, Kabuterimon line (existing, currently using `Electric`).
**Recommendation:** **add-now-in-M017.** Likely already present (legacy `tentomon_basic` uses `Electric` in `skills.ron`); verify and codify.

## Defer-until-runtime-evidence

### 12. `StartQTE`

```rust
pub enum Effect {
    // ...
    StartQTE {
        kind: QteKind,                   // HitCheck v1; PowerCharge/Mash deferred
        window_ms: u32,
        headless_default: QteResult,     // Success | Fail (deterministic)
    },
}
```

**Skills:** Baby Burner (Agumon ult) — only dependent skill in canon.
**Recommendation:** **defer-until-runtime-evidence.** QTE v1 (`HitCheck` single press, binary success/fail) spec is closed in `agumon/03 §8 G8`, but the windup-suspend frame-counter pause mechanism is tightly coupled to the FSM scheduler that isn't part of M017 schema work. The Ult can ship in M017 without QTE; QTE adds purely on top of `splash_mul_selected` blueprint state which is itself blueprint-Rust. No skill is gated by the variant existing today.

### 13. `multiplier_chain: Vec<ParamRef>` (replace single `mul_param`)

```rust
pub struct EmitDamageArgs {
    // current: amount: i32
    pub multiplier_chain: Vec<ParamRef>,
    pub tag: DamageTag,
    pub target: TargetRef,
    pub tough_break: Option<ParamRef>,
}

pub enum ParamRef {
    Snapshot(String),                    // skill_def.params[key], commit-time
    EventPayload(String),                // last KernelEvent payload field, live
    BlueprintState(String),              // blueprint_state[key], live
}
```

**Skills:** Baby Burner detonate (Agumon ult, only consumer of `EventPayload`). Dash Metal threshold (resolved blueprint-side path C, no param plumbing needed).
**Recommendation:** **defer-until-runtime-evidence.** Single `amount: i32` (current) or single `mul_param: String` covers 23 of 24 skills. Detonate uses `heated_remaining × per_stack` which the blueprint can pre-compute (path C, no schema gap). Revisit when a second skill needs cross-source multipliers.

### 14. `ParamRef::EventPayload` / `ParamRef::BlueprintState` (deferred with #13)

Same use case as #13. Detonate is the only canon consumer; conditional damage scaling (Dash Metal) is solved blueprint-side without parametric multipliers.
**Recommendation:** **defer-until-runtime-evidence.** Couple with #13.

## Summary

**Add now (10 items):** `ApplyBuff`, `EmitHeal`, `EmitCleanse`, `EmitSpGrant`, status-kind expansion (Heated/Chilled/Slowed/Paralyzed/Blessed), `ApplyStatus.stacks`, `TargetShape` expansion + validator loosening, `AdvanceTurn/DelayTurn` rewrite, `BlockReaction`, `SetBlueprintState`, `DamageTag::Electric`.

**Defer (3 items):** `StartQTE`, `multiplier_chain`, `ParamRef::{EventPayload,BlueprintState}`.

**Status:** every active+passive in the round-3 canon is *representable* in `Effect` post-add-now. No skill requires Rust hardcode; reactive triggers (FSM edges + listener filters) legitimately live in blueprint code per `02-02b §C4`.
