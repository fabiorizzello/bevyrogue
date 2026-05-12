---
spike: SP1
name: Combat kernel primitives & contracts audit
status: complete
created: 2026-05-12
inputs:
  - docs/future_design_draft/02-02b_animation_fsm.md (turn manipulation §C2/§C2.1, target shapes §C3, reactive vocab §C4, kernel events §R)
  - docs/future_design_draft/02-08_effect_cascade.md (damage reduction taxonomy §H, status §H.1/§H.2, DR §H.3)
  - docs/future_design_draft/08_roster_minimal.md (reactive signature vocab §8.1, target shape vocab §8.2, roster §8.3)
  - src/combat/ (current implementation)
outputs:
  - gap table (design → kernel coverage matrix)
  - decision list (new event variants vs new bus vs generalize existing)
  - input to M017 slices S01/S02/S03 scope
---

# SP1 — Combat kernel primitives & contracts

## Goal

Verify which primitives required by the round-3 design canon are **present**, **partial**, or **missing** in `src/combat/`. Output a gap table that drives M017 slice scoping.

## Primitives to audit

### Turn manipulation (§02-02b §C2.1)
- [Partial] `AdvanceTurn(±%)` event / mechanic — `CombatEventKind::TurnAdvance { target, amount_pct }` exists (`src/combat/events.rs:94`); applied by `apply_turn_advance_system` (`src/combat/turn_system/mod.rs:633`) via `resistance::apply_av_change` over `ActionValue` (`av.rs`). **Sign convention drift**: canon uses positive=delay/positive=advance distinction by Command identity (`AdvanceTurn`/`DelayTurn`), code uses signed `amount_pct` (+=advance, −=delay). Canon further states cap **±50% per call** and clamp gauge `[0, 200]` of next-action gauge — code instead uses `MAX_AV=10000` accumulator and `MIN_ACTION_THRESHOLD_AV=-15000` floor (`resistance.rs:7`), plus `TempoResistance` diminishing-returns multiplier (1.0/0.5/0.25). The semantics are HSR-AV-like, not §C2.1 "% of next-action gauge with hard cap".
  - **Proposal:** `src/combat/resistance.rs::apply_av_change` add canonical clamp helper `clamp_turn_manip_pct(±50)` (per-call cap) and rework `MAX_AV` model to map to "gauge in `[0, 200]`" or expose a wrapper that normalises AV→gauge for §C2.1 conformance. Split `CombatEventKind::TurnAdvance` into `AdvanceTurn { target, pct }` + `DelayTurn { target, pct }` (or keep single variant but enforce abs(pct) ≤ 50 at emit).

- [Missing] `DelayTurn(±%)` event / mechanic as **separate Command/event** — only encoded as negative `TurnAdvance` amount.
  - **Proposal:** add `CombatEventKind::DelayTurn { target: UnitId, pct: u8 }` (or keep unified `TimelineDelta` enum). Resolver in `resistance.rs` should branch by variant rather than sign for clearer log/UI.

- [Missing] Cap enforcement: ±50% per call.
  - **Proposal:** add `const TURN_MANIP_CAP_PCT: i32 = 50;` in `resistance.rs`, clamp at event-emit site (`resolution.rs` where `TurnAdvance` is pushed) AND at apply site (defense-in-depth).

- [Missing] Clamp: timeline position ∈ `[0, 200]`.
  - **Proposal:** introduce `TurnGauge(u8)` component (range 0..=200) alongside or replacing the `ActionValue(0..=MAX_AV)` model. Alternatively, keep AV internal but expose a derived gauge view + clamp it post-mutation. Canon (`renamon/00 §8 D1`) explicitly says gauge clamp, not speed-stat clamp — current `Speed` component is untouched, which is good.

- Audit point: `src/combat/turn_order.rs`, `speed.rs`, `av.rs`, `resistance.rs`, `turn_system/mod.rs::apply_turn_advance_system`.

### Target shapes (§08 vocab)
- [Partial] `Single` — `TargetShape::Single` present (`src/data/skills_ron.rs:11`).
- [Missing] `Blast` (primary + adjacent) — not in enum; canon `§C3` calls it `Blast(TargetRef)`.
  - **Proposal:** add `TargetShape::Blast` variant in `src/data/skills_ron.rs`; resolver (currently single-target) in `resolution.rs::target_shape_is_executable_now` must accept it; resolution path must expand into 3 hits (primary + 2 adj). Adj geometry needs a new helper `adj_units(primary, world) -> [Option<UnitId>; 2]` (slot-based — line composition).
- [Generalize-needed] `AoE(All)` — code has `TargetShape::AllEnemies` and `Row`, but both are gated to `Single` only in `action_query.rs:485` and `resolution.rs:174` with `LegalityReasonCode::UnimplementedTargetShape`. The enum exists, the **resolver does not**.
  - **Proposal:** generalize `target_shape_is_executable_now` + execute loop to fan-out N hits per shape; rename `AllEnemies` → `AoE { side, exclude_dead }` to match `§C3`.
- [Missing] `Bounce(N)` with deterministic tie-break (slot_index ascending).
  - **Proposal:** add `TargetShape::Bounce { hits: u8, selector: Box<TargetShape> }` (canon §C3). Resolver must `re-resolve` the inner selector per hop (canon rule 4). Deterministic tie-break: stable sort by `UnitId.0` ascending (already the pattern used by `select_follow_up_target`).
- Audit point: `src/data/skills_ron.rs::TargetShape`, `src/combat/resolution.rs::target_shape_is_executable_now`, `src/combat/action_query.rs:485`.

### Damage reduction (§02-08 §H)
- [Missing] Intra-unit max-replace semantics — no DR concept in code. `damage.rs::calculate_damage` formula is `base × tag_mod × tri_mod × break_mod`; no defender-side mitigation pipeline beyond tag/triangle. No `Toughness`-style mitigation field.
- [Missing] Cross-unit additive with clamp 0.5 — same gap.
- [Missing] Layer ordering deterministic (canon `§2.8 §H.3` algorithm: per-source max → sum across sources → clamp 0.5).
  - **Proposal:** new module `src/combat/damage_reduction.rs` with:
    - `pub struct DrInstance { source_blueprint_id: BlueprintId, value: f32, kind: BuffKind }`
    - registry resource `DrRegistry` storing `Vec<DrInstance>` per target
    - `compute_dr_for_target(target, registry) -> f32` implementing the §H.3 algorithm (see canon code block lines 297-310)
    - Hook into `damage.rs::calculate_damage` post-tri_mod, pre-break_mod, as `final = round(base × tag × tri × (1.0 - dr) × break_mod)`.
- Audit point: `src/combat/damage.rs`, `status_effect.rs` (needs `BuffKind` + `Aura` + DR source tracking).

### Status caps (§round-3 revisions, also §H.1)
- [Missing] `Heated` cap 6 + decay -1 per turn-end — note canon `§H.1` actually documents `Heated` as **single-instance, default dur 2 turns, DoT 4 dmg/turn**; the "cap 6 + decay" model is **not** in current canon. Audit caveat: skeleton bullet may be stale. Current code has `StatusEffectKind::Burn { damage_per_turn }`, no `Heated`.
  - **Proposal:** treat skeleton bullet as superseded by `§H.1`. Replace `Burn` with `Heated` (cap-stacks model deferred until skill needs it; §H.5 #2 explicitly defers stack-aware status). Rename gas-era `Burn`/`Shock` to reserved tags per `§H.1`.
- [Missing] `Chilled` cap 6 + decay -1 per turn-end — same caveat; canon has `Chilled` as single-instance speed −20%, +15% ice taken.
  - **Proposal:** add `StatusEffectKind::Chilled { ice_amp_pct: i32, speed_red_pct: i32 }`. Speed reduction maps to existing `SpeedModifier` component (`speed.rs`); ice amp needs new pipeline in damage formula.
- [Missing] Status set v0: Heated, Chilled, Slowed, Paralyzed, Blessed (Confused dropped). Code has `Burn`, `Freeze`, `Shock`, `DeepFreeze` — entirely different vocabulary.
  - **Proposal:** Rewrite `StatusEffectKind` enum in `src/combat/status_effect.rs` to canon `StatusKind`: `Heated | Chilled | Paralyzed | Slowed | Blessed` (+ reserved `Burn | Shock`). Add `BuffKind` enum (`Buff | Debuff | DR | Aura | Mark` per `§H.2`). Add `BuffDur` enum. Update validator (load-time RON) to reject unknown ids — RON `Effect::ApplyStatus { kind, duration }` already keys by `StatusEffectKind`, so flipping the enum surface bubbles through.
- Audit point: `src/combat/status_effect.rs`, plus damage pipeline integration for amp%.

### Reactive signature bus (§08 vocab)
- [Generalize-needed] `OnKill` event — code has `CombatEventKind::OnEnemyKill` and `OnKO` (`events.rs:42, 46`) which fire kill-context. Canon `§8.1` table requires `KernelEvent::UnitDied { target }`. The current naming is asymmetric ("OnEnemyKill" attacker-perspective; "OnKO" defender-perspective).
  - **Proposal:** add `CombatEventKind::UnitDied { unit: UnitId, killer: Option<UnitId> }` per canon `§R-Events`. Keep `OnEnemyKill` as alias if needed for back-compat, but `UnitDied` is the canonical reactive primitive.
- [Missing] `OnHitN(threshold)` event (cumulative hit count) — code has `OnHitTaken { amount }` (per-event, no counter). Reactive signature `OnHitN→Apply` (Tentomon `petit_thunder`) requires a **per-FSM hit counter** on the `Hop{N}` node, not a kernel-level event (canon `§C4` row 4: "fired on `TimeInNode` of `Hop{N}` node, after `EmitDamage`").
  - **Proposal:** No new kernel event needed. Mark as **resolved-by-FSM**: M017 S03f (AnimGraph FSM) introduces `Hop{N}` nodes; counter is implicit in FSM topology. Reactive shape pattern lives in `§C4` not in `events.rs`.
- [Present] `OnStatusApplied(status)` — `CombatEventKind::OnStatusApplied { kind }` (`events.rs:84`). Matches canon `§R-Events`.
- [Missing] `OnUltimateUsed` event — canon `§R-Events` requires `UltimateUsed { actor }` for Renamon `kitsune_grace` listener. Code has `UltGain { unit_id, amount }` (charge gain) and `OnSkillCast { skill_id }` (any skill including Ult) but no dedicated post-commit-ult event.
  - **Proposal:** add `CombatEventKind::UltimateUsed { actor: UnitId }`. Emit from `turn_system::pipeline` immediately after Ult commit consumes the bar (post-Strike, pre-cleanup per `§R-Events` row 1).
- [Missing] `Healed`, `SpGranted`, `IncomingDamage` (pre-step), `BlockReactionTriggered` — canon `§R-Events` lists these for M017 listeners (Patamon heal chain, Tentomon block reaction, SP grant chain).
  - **Proposal:** add all four to `CombatEventKind`. `IncomingDamage` is special: emitted **pre-cascade** in `damage.rs::calculate_damage` callsite to allow `BlockReaction { damage_mult }` mitigation pre-step (canon §2.8 §A inset).
- Audit point: `src/combat/events.rs::CombatEventKind`.

### Other primitives flagged in design
- [Present] SP pool — `SpPool { current: 3, max: 5 }` default (`sp.rs:48`). Matches canon §8.0 baseline. `+1 basic` is handled in `apply_effects` (per ult.rs comment line 78), `+2 Tentomon basic` is canon §8.3 row Tentomon but **not yet data-driven**: no `sp_gen_per_basic` field on `Unit`. Code currently appears to hardcode +1.
  - **Proposal (partial):** add `Unit.sp_gen_per_basic: u8` (default 1) sourced from `units.ron`. Tentomon entry sets it to 2.
- [Present] Ult charge off-turn fire — `UltimateCharge` + `ult_accumulation_system` (`ultimate.rs`) handles event-driven charge accrual including off-turn (`OnHitTaken`, `OnAllyFollowUp`, `OnOffensivePartyEvent`). Mature subsystem; M017 just needs to add `Healed` listener variant (Patamon ult charges on heal events).
- [Present] Toughness/break + weakness element — `Toughness` (`toughness.rs`) with `Standard/Armored/Shielded` categories, `apply_hit` + break transition. `DamageKind` classification. Solid.
- [Present] Follow-up FIFO — `FollowUpIntent` message + `follow_up_listener_system` + `resolve_follow_up_action_system` (`follow_up.rs`). Reads `CombatEvent`, fires triggers `OnEnemyBreak/OnAllyLowHp/OnEnemyKill`, schedules action. **Canon-divergent** in that `§2.8 §E` calls for `FollowUpIntent` to be **subsumed** by generic `KernelEffect::EnqueueAction` cascade (M017 S03e explicit refactor). M017 should plan that subsume.

## Method (executed)

1. Read each file under audit. ✅
2. Built matrix below. ✅
3. Wrote proposals inline. ✅
4. Cross-checked with `docs/combat_current.md`. Drift noted: combat_current.md describes M016 status (6 blueprints migrated, `holy_support`/`predator_loop`/`battery_loop`/`precision_mind_game`/`twin_core` primitives) — these are blueprint-level **implementations** of round-2 designs (e.g. `BatteryLoop`, `PredatorLoop`). Round-3 canon (§02-02b/§02-08/§08) is a **vocabulary normalization** (Commands, TargetShape, StatusKind, BuffKind, DR pipeline, reactive events). The blueprints stay; the primitives below are the substrate they will rest on.

## Decision points

- **Should new events join `CombatEventKind` or form a separate `ReactiveEvent` bus?**
  **Recommendation: extend `CombatEventKind`.** Rationale: the existing bus already carries `OnStatusApplied`, `OnEnemyKill`, `OnHitTaken`, `OnSkillCast`, `UltGain`, etc. — these are already reactive primitives by another name. A second bus would force every listener (FSM, blueprint, log, UI) to subscribe twice and reorder. `§2.8 §A` is explicit: "kernel-owned cascade as **extension of the `CombatEvent` bus**". One bus, one drain loop, deterministic FIFO.

- **Are `AdvanceTurn` / `DelayTurn` first-class events or sub-effects under a generic `TimelineDelta`?**
  **Recommendation: keep them as two distinct `CombatEventKind` variants** (`AdvanceTurn { target, pct }`, `DelayTurn { target, pct }`). Rationale: canon `§C2` lists them as **separate Commands** (renamon/00 §8 D1 explicit), the UI/turn-tracker needs to distinguish direction for player feedback, and the `±50%` cap is per-direction not per-magnitude. The current single `TurnAdvance { amount_pct: i32 }` with signed amount obscures intent in logs.

- **Tie-break for `Bounce(N)` and Echo/Chain: enforce at event emit or at resolution?**
  **Recommendation: enforce at resolution.** Rationale: target selection for chain/echo (canon `§C3` rule 4) requires live world state (who is alive at hop N?). Pre-computing at emit-time would mean recording all N targets up front and skipping dead ones, which mismatches "re-resolve the selector at every hop". Resolution-time selection (with deterministic `UnitId.0` ascending tie-break as the canonical rule, copied from `follow_up.rs:151`) keeps logic in one place.

## Gap summary

| Primitive | Status | Proposed action | Target file |
|---|---|---|---|
| `AdvanceTurn(±%)` event | Partial | Split variant + add cap | `src/combat/events.rs`, `resistance.rs` |
| `DelayTurn(±%)` event | Missing | New `CombatEventKind` variant | `src/combat/events.rs` |
| Cap ±50% per call | Missing | Const + clamp at emit and apply | `src/combat/resistance.rs` |
| Gauge clamp `[0, 200]` | Missing | New `TurnGauge` or derived view | `src/combat/av.rs` (+ new `turn_gauge.rs`) |
| `TargetShape::Single` | Present | — | `src/data/skills_ron.rs` |
| `TargetShape::Blast` | Missing | Add variant + adj resolver | `src/data/skills_ron.rs`, `src/combat/resolution.rs` |
| `TargetShape::AoE` | Generalize-needed | Lift `UnimplementedTargetShape` gate; rename `AllEnemies → AoE { side, exclude_dead }` | `src/combat/resolution.rs`, `src/combat/action_query.rs` |
| `TargetShape::Bounce` | Missing | New variant + per-hop resolver | `src/data/skills_ron.rs`, `src/combat/resolution.rs` |
| DR intra-unit max-replace | Missing | New `damage_reduction.rs` + DR registry | `src/combat/damage_reduction.rs` (new), `damage.rs` |
| DR cross-unit additive + clamp 0.5 | Missing | Same module, §H.3 algorithm | `src/combat/damage_reduction.rs` (new) |
| `StatusKind` enum (Heated/Chilled/Paralyzed/Slowed/Blessed) | Missing | Rewrite enum | `src/combat/status_effect.rs` |
| `BuffKind` taxonomy | Missing | New enum (Buff/Debuff/DR/Aura/Mark) | `src/combat/status_effect.rs` |
| `BuffDur::Permanent` for Aura | Missing | New enum | `src/combat/status_effect.rs` |
| Cleanse policy (Debuff-only by default) | Missing | New `Effect::Cleanse { count, selector }` + apply rule | `src/data/skills_ron.rs`, `src/combat/resolution.rs` |
| `OnKill / UnitDied` event | Generalize-needed | Add `UnitDied { unit, killer }`; alias from `OnEnemyKill+OnKO` | `src/combat/events.rs` |
| `OnHitN` reactive | Missing | Resolved by FSM topology — no kernel change | (M017 S03f FSM) |
| `OnStatusApplied` | Present | — | `src/combat/events.rs` |
| `OnUltimateUsed / UltimateUsed` | Missing | New variant + emit post-commit | `src/combat/events.rs`, `turn_system/pipeline.rs` |
| `Healed` event | Missing | New variant + emit post-Heal effect | `src/combat/events.rs`, `resolution.rs` |
| `SpGranted` event | Missing | New variant + emit post-GainSP | `src/combat/events.rs`, `resolution.rs` |
| `IncomingDamage` pre-step | Missing | New variant + pre-cascade emit | `src/combat/events.rs`, `damage.rs` |
| `BlockReactionTriggered` event | Missing | New variant + emit post-mitigation | `src/combat/events.rs` |
| `Effect::Heal` | Missing | New variant (canon `EmitHeal`) | `src/data/skills_ron.rs` |
| `Effect::AdvanceTurn` / `Effect::DelayTurn` | Partial | `Effect::TurnAdvance(i32)` exists; split or rename | `src/data/skills_ron.rs` |
| SP pool 3/5 | Present | — | `src/combat/sp.rs` |
| Per-unit SP gen rate | Missing | `Unit.sp_gen_per_basic: u8` | `src/combat/unit.rs`, `assets/data/units.ron` |
| Ult charge off-turn | Present | Add `Healed` listener variant | `src/combat/ultimate.rs` (`UltAccumulationTrigger::OnHealEvent`) |
| Toughness/break/weakness | Present | — | `src/combat/toughness.rs` |
| Follow-up FIFO | Present | Plan subsume into `KernelEffect::EnqueueAction` cascade (M017 S03e per `§2.8 §E`) | `src/combat/follow_up.rs` |

## Proposed `gsd_save_decision` calls

The following architectural decisions are ready to be persisted to `.gsd/DECISIONS.md` once the human reviews this spike:

1. **D-M017-EVENTS-BUS**: "Extend `CombatEventKind` rather than introducing a parallel `ReactiveEvent` bus. All round-3 reactive primitives (`UnitDied`, `UltimateUsed`, `Healed`, `SpGranted`, `IncomingDamage`, `BlockReactionTriggered`, `AdvanceTurn`, `DelayTurn`) join the existing `CombatEvent` message stream." Rationale: canon `§2.8 §A` explicit single-bus model; existing listeners (follow-up, ultimate, blueprints, log, UI) avoid double-subscription drift.

2. **D-M017-TIMEMANIP-SPLIT**: "Split the current signed `TurnAdvance { amount_pct: i32 }` event into two variants `AdvanceTurn { target, pct: u8 }` and `DelayTurn { target, pct: u8 }`. Enforce `pct ≤ 50` at both emit-site (`resolution.rs`) and apply-site (`resistance.rs`). Introduce a derived `TurnGauge` view clamped to `[0, 200]` per canon `§C2.1`; keep `ActionValue/MAX_AV` as internal accumulator." Rationale: identity match with `§5` design, log/UI readability, defensive double-clamp.

3. **D-M017-STATUS-REWRITE**: "Replace `StatusEffectKind::{Burn, Freeze, Shock, DeepFreeze}` with the canon `StatusKind::{Heated, Chilled, Paralyzed, Slowed, Blessed}` enum (+ reserved `Burn, Shock`). Add `BuffKind::{Buff, Debuff, DR, Aura, Mark}` and `BuffDur::{Turns, UntilRoundEnd, Permanent}` taxonomy from `§H.1/§H.2`. RON validator rejects unknown ids at load-time. Per-status numerical effect (e.g. `Heated +15% fire/holy taken`) wires into `damage.rs` via new `amp%` lookup." Rationale: round-3 canon is the lock; renaming now (before more blueprints land) is cheaper than aliasing forever.

4. **D-M017-DR-MODULE**: "Add a new `src/combat/damage_reduction.rs` module owning the §H.3 algorithm (intra-unit max-replace per source, cross-unit additive across sources, clamp 0.5). Hook into `damage.rs::calculate_damage` between `tri_mod` and `break_mod` as `(1.0 - dr)` multiplier. DR sources are tracked as `BuffKind::DR` instances in the status registry (Gabumon `fur_cloak`, `blue_cyclone` Ult buff; Patamon `holy_aegis` Aura)." Rationale: clear new module avoids polluting `damage.rs` with mitigation pipeline; aligns with `§H.3` pseudocode.

5. **D-M017-TARGETSHAPE-EXPAND**: "Expand `TargetShape` enum to `{ Single, Blast(Primary), AoE { side, exclude_dead }, Bounce { hits, selector } }` per canon `§C3`. Resolver in `src/combat/resolution.rs` lifts the `UnimplementedTargetShape` gate and fans out N effects per shape. `Bounce` re-resolves selector per hop with deterministic `UnitId.0` ASC tie-break. Empty target Vec ⇒ silent no-op (canon §C3 rule 3, matches existing `select_follow_up_target` behavior)." Rationale: M017 roster (§8.3) requires Blast (Agumon ult), AoE (Renamon, Patamon ult, Tentomon ult), Bounce (Tentomon skill) — without these the kit cannot be implemented.

## Out of scope (confirmed)

- Implementation (belongs in M017 slices S01/S02/S03 + S03e/S03f).
- Touching `docs/future_design_draft/` (canon lock-in).
- Modifying `src/` (sketches only, throwaway, in `.gsd/spikes/spike-kernel-primitives/sketches/`).
- AnimGraph FSM parser/interpreter — M017 S03f scope (mentioned only insofar as it absorbs `OnHitN` reactive shape).
- `skill_tree.ron` resolver — canon `§2.2b §I` defers to M018+.
- `cost_effect`/`cooldown_effect` catalog — canon `§2.2b §J` deferred.
