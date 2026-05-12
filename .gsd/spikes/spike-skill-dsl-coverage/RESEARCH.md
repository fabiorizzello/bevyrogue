---
spike: SP3
name: Skill DSL coverage audit
status: complete
created: 2026-05-12
parallel_with: SP1
inputs:
  - src/data/skills_ron.rs (current Effect enum)
  - assets/data/skills.ron (current data)
  - docs/future_design_draft/digimon/*/01_basic_*.md, 02_skill_*.md, 03_ult_*.md, 04_passive_*.md
outputs:
  - skill × Effect coverage table
  - gaps.md (missing Effect variants + decision: add now / defer)
---

# SP3 — Skill DSL coverage audit

## Goal

Map all 24 skills (6 basic + 6 skill + 6 ult + 6 passive) onto `src/data/skills_ron.rs::Effect`. Identify variants that must exist before M017 implementation.

## DSL inventory (current canon in `skills_ron.rs`)

`Effect` enum (current state):
- `Damage { amount: i32, target: TargetShape }`
- `ToughnessHit(i32)`
- `GainSP(i32)`
- `UltGain(i32)`
- `Stun`
- `Revive(i32)`
- `GrantFreeSkill { count: usize }`
- `ApplyStatus { kind: StatusEffectKind, duration: u32 }` with `StatusEffectKind ∈ { Burn, Freeze, Shock, DeepFreeze }`
- `TurnAdvance(i32)` (defender-side)
- `GrantEnergy(i32)` (form identity, round-gated)
- `SelfAdvance(i32)` (attacker-side AV)

`TargetShape ∈ { Single, Row, AllEnemies, SelfOnly }`. `validate_skill_book` enforces `TargetShape::Single` for `Implemented` skills; anything else must be `Deferred(reason: UnimplementedTargetShape)`.

`SkillDef.custom_signals: Vec<SkillCustomSignal>` — legacy escape hatch (string-based; round-3 design canon explicitly retires this in favor of typed `Effect` + blueprint listeners reading typed `CombatEvent`).

## Canon design vocabulary (round-3, from `agumon/04 §9` + `08_roster_minimal.md`)

The design canon stabilizes a richer **Command** vocabulary that the M017 schema is expected to map onto. Source: `agumon/04 §9 G-Verbs`, `02-02b §C2/§C3`, `08_roster_minimal §8.1/§8.2`.

**Commands (gameplay):** `EmitDamage`, `EmitStatus`, `EmitHeal`, `EmitCleanse`, `EmitSpGrant`, `ApplyBuff`, `AdvanceTurn`, `DelayTurn`, `BlockReaction`, `SetBlueprintState`, `StartQTE`.
**TargetShape canon (v0, `02-02b §C3`):** `Single`, `Blast`, `AoE(All|EnemyTeam|AllyTeam)`, `Bounce { hits, selector }`, `AdjLowest { metric, side }`, `LowestHpPctAlive { scope }`, `RandomEnemyAlive { seed }`, `SingleAlly`, `Self_`. (Plus `EntityRef` resolution kinds: `Primary | FromParamSnapshot | EventTarget | Caster | Self | FromBlueprintState | <iter:loop_var>`.)
**Reactive signatures (closed v0, `08 §8.1`):** `OnKill→Detonate(status)`, `OnStatusApplied→Echo(status)`, `OnKill→Chain`, `OnHitN→Apply(status)`. **All four are blueprint-mediated** (FSM edge + Command), not runtime Commands. Their *consequences* are Effect-expressible; their *triggers* are not.
**Status set canon (v0):** `Heated`, `Chilled`, `Slowed`, `Paralyzed`, `Blessed` (Confused dropped).

## Coverage table — 24 entries

Legend:
- **Existing?**: `yes` = current `Effect` enum suffices; `partial` = some effects covered but skill needs new variants/extensions; `no` = none of the skill's required effects are expressible today.
- **Required effects** uses canon Command names (M017 target schema).

| # | Digimon | Slot | Skill | Required effects (canon) | Existing? | Gap |
|---|---|---|---|---|---|---|
| 1 | Agumon | Basic | Sharp Claws | `EmitDamage(Single, Fire) + EmitStatus(Heated, stacks=1)` + +1 SP gen (kernel hook) | partial | `Heated` status kind missing in `StatusEffectKind`; `stacks` parameter missing on `ApplyStatus` |
| 2 | Agumon | Skill | Baby Flame | `EmitDamage(Single, Fire) + ToughnessHit(20) + EmitStatus(Heated, stacks=2)` | partial | Same as #1 (Heated kind + stacks). Current ron uses `custom_signals` string shim |
| 3 | Agumon | Ult | Baby Burner | `EmitDamage(Blast→3× Single, Fire) + ToughnessHit(30) + StartQTE(HitCheck)` + reactive `OnKill→Detonate(Heated)` (FSM edge → `EmitDamage` per adj, multiplier_chain `[EventPayload(heated_remaining), Snapshot(detonate_per_stack)]`) | partial | `TargetShape::Blast` missing; `StartQTE` Command missing; `multiplier_chain` / `EventPayload` ParamRef missing. Reactive trigger half is blueprint-side (out of Effect scope), but consequence is `EmitDamage` per adj |
| 4 | Agumon | Passive | Twin Core (Fire) | `ApplyBuff(Self_, fire_boost_mul ×1.15, UntilRoundEnd)` triggered by listener on `StatusApplied(Chilled, caster=gabumon)` | no | `ApplyBuff` variant missing entirely; `BuffDuration::UntilRoundEnd` missing. Trigger half (listener filter) is blueprint-bound. **However**: damage-pipeline multiplier read pattern is canon (G9) — pure Effect-expressible once `ApplyBuff` exists |
| 5 | Gabumon | Basic | Claw Attack | `EmitDamage(Single, Ice) + EmitStatus(Chilled, stacks=1)` | partial | `Chilled` status kind missing; `stacks` param missing |
| 6 | Gabumon | Skill | Gabumon Shot | `EmitDamage(Single, Ice) + ToughnessHit(8) + EmitStatus(Chilled, stacks=2)` + reactive `OnStatusApplied→Echo(Chilled)` (FSM edge → `EmitStatus(Chilled, stacks=1, target=AdjLowest{HpPct})`) | partial | `Chilled` kind; `stacks_param`; `TargetShape::AdjLowest { metric, side }` missing; echo-target snapshot scope is blueprint-side |
| 7 | Gabumon | Ult | Blue Cyclone | `EmitDamage(Single, Ice) + ToughnessHit(25) + EmitStatus(Slowed, dur=2) + EmitSpGrant(amount=1, target=Team) + ApplyBuff(Self_, dr=0.30, Turns(1))` | partial | `Slowed` kind; `EmitSpGrant` variant missing (current `GainSP` is self-only literal, lacks `target_ref`); `ApplyBuff` missing |
| 8 | Gabumon | Passive | Fur Cloak (+ Twin Core Ice) | `ApplyBuff(Self_, dr=0.20, Turns(1))` on outgoing `StatusApplied(Chilled, caster=self)`; specular: `ApplyBuff(Self_, ice_boost_mul ×1.15, UntilRoundEnd)` on `StatusApplied(Heated, caster=agumon)` | no | `ApplyBuff` + `BuffDuration::Turns/UntilRoundEnd` missing. Trigger filtering blueprint-side |
| 9 | Dorumon | Basic | Bite | `EmitDamage(Single, Dark)` — no status | yes | None. `Damage` works; `Dark` tag already exists in `DamageTag` |
| 10 | Dorumon | Skill | Dash Metal | `EmitDamage(Single, Dark, multiplier_chain=[Snapshot(skill_mul_threshold)|Snapshot(skill_mul_base)])` + reactive `OnKill→Chain` gated by `BlueprintState(predator_active)` (FSM edge → 2nd `EmitDamage(LowestHpPctAlive)`) | partial | Conditional mul (HP-threshold) resolved blueprint-side (G5 path C, no schema gap). `TargetShape::LowestHpPctAlive` missing. `multiplier_chain` ParamRef missing. Reactive trigger half blueprint-side |
| 11 | Dorumon | Ult | Metal Cannon | `EmitDamage(Single, Dark, multiplier_chain) + ToughnessHit + SetBlueprintState(predator_active=true, Turns(N))` | partial | `SetBlueprintState` Command missing. Threshold scaling blueprint-side. Force-Predator entry is data → state mutation, currently unrepresentable in `Effect` |
| 12 | Dorumon | Passive | Predator Loop | `ApplyBuff(Self_, predator_active, Turns(N))` driven by listener tracking `DamageDealt`/`UnitDied`, `HpPctBelow` predicate eval; chain consume on `UnitDied` during `dash_metal` | no | `ApplyBuff` missing. State machine, hp-threshold predicate, chain consume — all blueprint-bound. The *Buff component* is Effect-expressible (once `ApplyBuff` exists); the tracking/threshold/consume logic is not |
| 13 | Renamon | Basic | Kōkaishū | `EmitDamage(Single, Holy)` — no status | yes | None (modulo `Holy` damage tag — already exists) |
| 14 | Renamon | Skill | Kōyōsetsu | `EmitDamage(AoE(EnemyTeam), Holy) + AdvanceTurn(Self_, +25%)` | partial | `AoE(All)`/`AoE(EnemyTeam)` shape works via existing `AllEnemies` but `Implemented` validator rejects non-`Single`; `AdvanceTurn` exists for **defender** (current `TurnAdvance(i32)`) but **semantics conflict**: design wants self-AV advance via `AdvanceTurn(actor=Self_, pct=+25)`. Existing `SelfAdvance(i32)` partially matches but lacks structured `actor_ref` |
| 15 | Renamon | Ult | Tōhakken | `EmitDamage(AoE(EnemyTeam), Holy) + ToughnessHit + DelayTurn(AoE(EnemyTeam), -30%) + ApplyBuff(AoE(AllyTeam), Blessed, Turns(2))` | partial | `DelayTurn` with multi-target shape missing (current `TurnAdvance(i32)` is single-target int, no shape). `Blessed` buff missing (`ApplyBuff` + `BuffKind::Buff` cleanse-immune). AoE shape blocked by validator |
| 16 | Renamon | Passive | Kitsune Grace | `AdvanceTurn(Self_, +10%)` triggered by listener on `UltimateUsed(actor=ally && !is_self)` | partial | `AdvanceTurn` self-side exists as `SelfAdvance(i32)`; gap is the listener-driven invocation half (blueprint). Effect itself representable |
| 17 | Patamon | Basic | Tai Atari | `EmitDamage(Single, Holy)` — no status | yes | None |
| 18 | Patamon | Skill | Patapata Hover | `EmitHeal(SingleAlly, pct_max=25) + EmitCleanse(SingleAlly, count=1, oldest_first, filter=DebuffOnly)` | no | `EmitHeal` Command missing (Effect has no heal variant at all — `Revive` is KO-only). `EmitCleanse` missing. `TargetShape::SingleAlly` missing |
| 19 | Patamon | Ult | Sparking Air Shot | `EmitDamage(AoE(EnemyTeam), Holy) + ToughnessHit + EmitHeal(AoE(AllyTeam), pct_max=20) + EmitCleanse(AoE(AllyTeam), count=1)` | partial | `EmitHeal`, `EmitCleanse`, AoE allies all missing. Damage AoE blocked by validator |
| 20 | Patamon | Passive | Holy Aegis | `ApplyBuff(AoE(AllyTeam-incl-self), dr=0.10, Permanent)` on `CombatStarted`; `EmitCleanse(AoE(AllyTeam), filter=ById("holy_aegis"))` on `UnitDied(self)` | no | `ApplyBuff` + `BuffDuration::Permanent` + `AoE(AllyTeam)` + `EmitCleanse(ById)` all missing. Trigger half blueprint-side |
| 21 | Tentomon | Basic | Hard Claw | `EmitDamage(Single, Electric) + GainSP(2)` (override +2 vs default +1) | partial | `Electric` damage tag — need to verify, may be missing in `DamageTag`. SP override is data-side (`units.ron.sp_gen_per_basic: 2`), not Effect-bound. Existing `GainSP(i32)` covers grant, but the +2 is kernel-hook driven |
| 22 | Tentomon | Skill | Petit Thunder | `EmitDamage(Bounce(3), Electric)` × 3 hops + `EmitStatus(Paralyzed, dur)` on hop3 (`OnHitN(3)→Apply`) + `ApplyBuff(Self_, dr=0.25, Turns(1))` | partial | `TargetShape::Bounce { hits, selector }` missing; `Paralyzed` status kind missing; `ApplyBuff` missing. `OnHitN→Apply` is dichiarative (last-node `on_enter`), no runtime trigger gap |
| 23 | Tentomon | Ult | Electrical Discharge | `EmitDamage(AoE(EnemyTeam), Electric) + EmitStatus(Paralyzed, target=RandomEnemyAlive{seed}, dur) + EmitSpGrant(amount=1, target=Team)` | partial | `TargetShape::RandomEnemyAlive { seed }` missing; `EmitSpGrant` with `target_ref=Team` missing; `Paralyzed` kind missing; AoE blocked by validator |
| 24 | Tentomon | Passive | Battery Loop | Path A (SP-grant side-channel): `EmitSpGrant(Self_, amount)` on `SpEarned(ally)`; Path B (Block-react FSM): `BlockReaction(Self_, damage_mult=0.50)` Command when SP≥3 + `IncomingDamage(self)` + RNG roll | no | `BlockReaction` Command missing. `EmitSpGrant` missing. SP-threshold predicate, RNG roll, pre-DR cascade — blueprint+kernel-bound. The *Commands themselves* (BlockReaction, EmitSpGrant) are Effect-expressible |

### Tally

- **yes**: 3 (Bite, Kōkaishū, Tai Atari — all "plain damage basics")
- **partial**: 14 (most skills/ults — current Effect covers damage+toughness, but status names + buff + heal + cleanse + new shapes + tempo verbs all missing)
- **no**: 7 (Twin Core Fire, Fur Cloak, Predator Loop, Patapata Hover, Holy Aegis, Battery Loop, and effectively a hard-no on the passive cluster — the passives need entire new effect families: ApplyBuff, EmitHeal, EmitCleanse, BlockReaction)

## Decisions

### Which gap variants ship in M017?

**Add now (load-bearing for round-3 design canon, every skill in roster touches at least one):**
- `ApplyBuff { id, target_ref, mul: Option<f32>, dur: BuffDuration }` with `BuffDuration ∈ { Turns(u8), UntilRoundEnd, Permanent }` — required by all 6 passives (Twin Core, Fur Cloak, Predator Loop, Holy Aegis, Kitsune Grace trigger context, Battery Loop) and 2 actives (Blue Cyclone DR self, Petit Thunder DR self).
- `EmitHeal { amount: HealAmount, target_ref: TargetRef }` with `HealAmount ∈ { Flat(i32), PctMax(u8) }` — Patapata Hover + Sparking Air Shot.
- `EmitCleanse { target_ref, count, filter: CleanseFilter, priority: CleansePriority }` — Patapata Hover + Sparking Air Shot + Holy Aegis exit.
- `EmitSpGrant { amount, target_ref }` (extends current `GainSP(i32)`) — Blue Cyclone, Electrical Discharge, Battery Loop side-channel.
- Status kind additions in `StatusEffectKind`: `Heated { stacks: u8 }`, `Chilled { stacks: u8 }`, `Slowed { speed_pct: i32 }`, `Paralyzed { skip_chance_pct: u8 }`, `Blessed { dmg_pct: u8, ult_gen: u8 }`. (`Confused` explicitly dropped per round-3.)
- `ApplyStatus` extension: `stacks` field (u8, default 1) — required for Heated/Chilled which stack 1/2 per Basic/Skill cast.
- `TargetShape` extension: `Blast`, `AoE { side: Enemy | Ally }`, `Bounce { hits: u8, selector: BounceSelector }`, `AdjLowest { metric, side }`, `LowestHpPctAlive { scope }`, `RandomEnemyAlive { seed: SeedSource }`, `SingleAlly`, `Self_`. Loosen `validate_skill_book` to accept these for `Implemented`.
- `AdvanceTurn { actor_ref, pct: i8 }` + `DelayTurn { target_ref, pct: i8 }` (replace `TurnAdvance(i32)`/`SelfAdvance(i32)` — design canon clamps ±50% per call, gauge `[0, 200]`). Renamon skill+ult+passive all depend on this.
- `BlockReaction { kind: ReactionKind, target_ref, damage_mult: f32 }` — Battery Loop block proc.
- `SetBlueprintState { state_key, value, dur: Option<BuffDuration> }` — Metal Cannon force-Predator, plus generic state-machine passives (Predator Loop, Battery Loop FSM gating).
- `DamageTag::Electric` — Tentomon kit (verify in `src/combat/types.rs`).

**Defer until runtime evidence (no current design dependency):**
- `StartQTE` Command (Baby Burner only; QTE v1 spec is closed but the windup-suspend mechanism touches FSM scheduling — defer until FSM lands).
- `multiplier_chain: Vec<ParamRef>` on damage (the `EventPayload("heated_remaining") × Snapshot("detonate_per_stack")` use case for Baby Burner detonate). Single `mul_param` works for everything else; detonate is the only multi-source today.
- `ParamRef::EventPayload` + `ParamRef::BlueprintState` (kicked out with `multiplier_chain` — same use case).

### Which gaps don't need Effect schema changes?

These are **blueprint-side** (listener trigger filtering, FSM edges, kernel events), so the `Effect` enum is not the blocker:

- All 4 reactive signatures (`OnKill→Detonate`, `OnStatusApplied→Echo`, `OnKill→Chain`, `OnHitN→Apply`) — their *consequences* are `Effect` calls; their *triggers* are FSM edge predicates in blueprint code.
- Twin Core "trigger on partner status" — listener filter, not Effect.
- Predator Loop "track lowest HP, threshold entry" — listener state machine, not Effect.
- Holy Aegis "apply on combat start, remove on self death" — listener filter, not Effect.
- Battery Loop "block reaction roll" — listener + RNG, not Effect.
- Kitsune Grace "react to ally ult" — listener filter, not Effect (the `AdvanceTurn` consequence is Effect-expressible).

### Hardcoded fallback in Rust = anti-pattern (CLAUDE.md)

**Audit confirms all 24 active+passive *effects* are Effect-expressible** once the variants above land. Reactive *triggers* legitimately live in blueprint code per `02-02b §C4` (FSM-edge shorthand). This is **not** a hardcode anti-pattern: the design canon explicitly partitions "what happens" (data, in `Effect`) from "when it happens" (FSM edges + listener filters, in blueprint Rust). No skill requires a Rust-only effect.

## Coordination with SP2 — Passive Effect-expressibility signal

**Verdict for SP2: passive *effects* are Effect-expressible; passive *triggers* are inherently blueprint-bound.**

Concretely:
- **Effects on every passive** are `ApplyBuff`, `EmitCleanse`, `EmitSpGrant`, `AdvanceTurn`, `BlockReaction`, `SetBlueprintState`. Once those variants exist in `Effect` (M017 schema work), the *what* of a passive lives in `skills.ron` / `passives.ron`.
- **Triggers on every passive** are: listener filtering on `CombatEvent` (`StatusApplied { caster, status }`, `UltimateUsed { actor }`, `UnitDied { unit }`, `CombatStarted`, `SpEarned`, `IncomingDamage`), plus state machines (`PredatorLoopState`, `BatteryLoopState`, FSM nodes). These require typed Rust code regardless of how the *effects* are stored.
- **Implication for SP2:** if SP2 picks **option C (RON-driven blueprints)**, the RON would carry the trigger predicate spec (event kind + filter expression) and reference a closed Command vocab — but Rust code still needs to *evaluate* predicates and *dispatch* to the Effect interpreter. If SP2 keeps **blueprints in Rust**, the listener filter + state machine stays in pure Rust and the blueprint *invokes* Effect-expressed Commands via a kernel API.
- **Recommendation for SP2:** option B (hybrid: Effects/Commands in RON, listener filter + state in Rust) is the natural fit. Twin Core, Predator Loop, Battery Loop already implement this pattern via `custom_signals` + Rust blueprints. The round-3 design doesn't require RON-driven listener predicates; it requires Effect-driven *consequences*.

So **SP3 scope grows to cover all 24 entries** regardless of SP2's choice (the *effects* are the same), but **the trigger half stays in Rust** under either option B or option C-minus.

## Out of scope

- Modifying `skills_ron.rs`.
- Editing `assets/data/skills.ron`.
- M017 implementation. This spike is pre-planning only.

## See also

- `gaps.md` — compact missing-variant proposal with payload pseudocode + ship/defer recommendation.
