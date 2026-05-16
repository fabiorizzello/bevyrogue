# S08 Research: Agumon + Gabumon migrated (Twin Core paired)

## Executive Summary

S08 migrates the Agumon and Gabumon blueprints so that both operate fully within the M021 kernel framework (CompiledTimeline + ExtRegistries + PassiveRunner), with their shared Twin Core mechanic extracted into a kernel-independent mini-plugin. The "Bouncing Fire OFF≡baseline" goal requires wiring a Loop-branch edge gate for Agumon's talent-gated skill path.

The key finding: **the mechanics are largely already implemented**. Both blueprints have working `signals.rs` dispatchers and `register_passive_runtime()` functions. What is missing is:
1. Gabumon has no `AgumonPlugin`-equivalent (no Bevy plugin, registered via `register_canonical_passive_runners` only).
2. TwinCore types and state (`TwinCoreState`, `TwinCoreHook`, `TwinCoreSignal`, `TwinCoreTransition`) still live inside `kernel.rs` (violating P001/M021 success criterion).
3. `apply_twin_core_transitions_system` and `TwinCoreState` still live in `src/combat/blueprints/agumon/identity.rs` — accessed correctly from blueprints, but `kernel.rs` still imports `TwinCoreTransition`/`TwinCoreSignal` via `CombatKernelTransition::TwinCore(...)`.
4. Bouncing Fire is a spike-only construct; there is no `BeatKind::Loop` branch for it in the current production `baby_flame` timeline in `skills.ron`.
5. The `CombatKernelTransition` enum in `kernel.rs` has a `TwinCore(TwinCoreTransition)` variant — this is a kernel-level Digimon coupling that must be removed (M021 success criterion: `rg "TwinCore|BatteryLoop|..." src/combat/ --glob '!blueprints/**'` → 0 lines).

---

## 1. Current State of Agumon Blueprint

### Location
`src/combat/blueprints/agumon/` (directory module)

### Files
- `mod.rs` — Plugin registration, passive runtime wiring
- `identity.rs` — TwinCore tags, state, hook, transitions
- `signals.rs` — Custom signal dispatcher (owner: `"agumon"`)

### Active skills registered via skills.ron
| Skill ID | Signal emitted | Timeline present |
|---|---|---|
| `baby_flame` | `agumon::apply_heated` (Amount 3) | Yes — cast→impact_damage→impact_break→impact_signal |
| `agumon_ult` (Baby Burner) | `agumon::apply_thermal_spark` (Amount 3) | No (legacy_ops only) |
| `agumon_follow_up` (Spitfire) | `agumon::apply_heated` (Amount 3) | Yes |

### Passive
- `PassiveRunner` listening on `EventFilter::blueprint("kernel", "ult_used")`
- Timeline: `dormant → proc → resolve` with gate `"agumon/twin_core/passive_trigger"`
- On proc: emits `Intent::SetBlueprintState` (guard) + `Intent::BlueprintSignal { owner: OWNER, name: "apply_heated", payload: Amount(3) }`
- Hook `"agumon/twin_core/passive_proc"` + predicate `"agumon/twin_core/passive_trigger"` registered via `register_passive_hooks()`
- **No Bevy plugin for the passive** — hooks registered via `register_passive_runtime()` which is called from `kernel::register_canonical_passive_runners()`

### Agumon Plugin (`AgumonPlugin`)
- `init_resource::<TwinCoreState>()`
- `add_systems(Update, apply_twin_core_transitions_system)`
- Registers `TwinCoreHook` in `CombatKernelRegistry`
- Added via `register_combat_kernel_runtime()` → `app.add_plugins(AgumonPlugin, ...)` in `kernel.rs`

### Signal dispatcher (`agumon::dispatch`)
Handles: `apply_heated`, `apply_meltdown_crack`, `apply_thermal_spark`
Each maps to `twin_core_added_tag_transition(TwinCoreDesignTag::*, turns_left)` which produces `CombatKernelTransition::Tag(...)`.

---

## 2. Current State of Gabumon Blueprint

### Location
`src/combat/blueprints/gabumon.rs` (single flat file — NOT a directory module yet)

### Active skills registered via skills.ron
| Skill ID | Signal emitted | Timeline present |
|---|---|---|
| `bubble_blast` | `gabumon::apply_chilled` (Amount 3) | Yes — same pattern as baby_flame |
| `gabumon_ult` (Arctic Torrent) | `gabumon::apply_thermal_spark` (Amount 3) | No (legacy_ops only) |
| `gabumon_follow_up` | `gabumon::apply_chilled` (Amount 3) | Yes |

### Passive
- `PassiveRunner` listening on `EventFilter::blueprint("kernel", "ult_used")`
- Same dormant→proc→resolve structure as Agumon, keys `"gabumon/twin_core/passive_trigger"` and `"gabumon/twin_core/passive_proc"`
- On proc: `Intent::SetBlueprintState` + `Intent::BlueprintSignal { owner: "gabumon", name: "apply_chilled", payload: Amount(3) }`
- `PASSIVE_OWNER: UnitId = UnitId(2)` (hardcoded, mirrors Agumon's `UnitId(1)`)

### Signal dispatcher (`gabumon::dispatch`)
Handles: `apply_chilled`, `apply_deep_crack`, `apply_thermal_spark`
All mapped via re-exported `twin_core_added_tag_transition` from `blueprints::agumon::identity`.

### Key issue
Gabumon imports directly from `blueprints::agumon::identity`:
```rust
use crate::combat::blueprints::agumon::{TwinCoreDesignTag, twin_core_added_tag_transition};
```
This is the expected cross-blueprint coupling point (Twin Core shared mechanic), but per D005 this logic should live in a dedicated `twin_core` mini-plugin module, not under `agumon/`.

### No Plugin for Gabumon
Unlike Agumon, Gabumon has NO `GabumonPlugin` struct. It only has `register_passive_runtime()`. The passive hooks are registered there.

---

## 3. Twin Core Mechanic — Full Detail

### What it is
Twin Core is the shared Fire/Ice paired mechanic between Agumon (Fire half) and Gabumon (Ice half). It maintains:
- `cross_resonance: u8` (max 2) — built by Heated/Chilled/Primed tags
- `active_thermal_spark_targets: Vec<UnitId>` — targets marked by ThermalSpark
- `fire_spend_markers` / `ice_spend_markers` — per-cycle spending counters
- `twin_burst_used_this_cycle` / `shatter_used_this_cycle` — one-shot guards per cycle

### Six design tags
`Heated`, `Chilled`, `ThermalSpark`, `Primed`, `MeltdownCrack`, `DeepCrack`

### TwinCoreHook (currently in `kernel.rs` via `CombatKernelTransition::TwinCore`)
The hook reacts to:
- `Tag(Added)` → Heated/Chilled/Primed: `BuildCrossResonance(1)`, ThermalSpark: `thermal_spark(1)`, MeltdownCrack: `fire_spend_marker(1)`, DeepCrack: `ice_spend_marker(1)`
- `Tag(Consumed|Expired)` → ThermalSpark: `shatter(1)`
- `Beat(Damage)` → `twin_burst(1)`
- `TacticalCycle(wrapped_cycle=true)` → `cycle_reset()`

### Kernel coupling (the violation of P001)
`kernel.rs` contains:
1. `enum TwinCoreSignal` (8 variants)
2. `struct TwinCoreTransition { signal, amount }` with 8 constructor fns
3. `CombatKernelTransition::TwinCore(TwinCoreTransition)` variant in the main enum

These must move to `blueprints/twin_core/` for M021 success criterion compliance. `CombatKernelTransition::TwinCore(...)` must be replaced by `CombatKernelTransition::Blueprint { owner: "twin_core", name: "...", payload: ... }`.

### Existing tests
- `tests/twin_core_integration.rs` — full integration test using `TwinCoreHook`, `TwinCoreState`, `apply_twin_core_transitions_system`. Currently imports from `blueprints::agumon::{TwinCoreDesignTag, TwinCoreHook, TwinCoreState, ...}`.
- `tests/twin_core_mechanics.rs` — unit-level state machine test; same imports.

Both tests will need import path updates if TwinCore moves to its own mini-plugin module.

---

## 4. Bouncing Fire

### What it is
Bouncing Fire is Agumon's talent-gated Loop skill branch. When the talent `agumon::bouncing_fire` is at rank ≥ 1, after the normal `baby_flame` impact sequence, additional bounce hops fire against subsequent targets (up to `rank` hops, pool-exhaustion stops early).

### Current state (PRODUCTION)
**Bouncing Fire does not exist in production code.** The current `baby_flame` timeline in `skills.ron` is a simple linear 4-beat sequence (cast → impact_damage → impact_break → impact_signal). There is no Loop branch, no `agumon::has_bouncing_fire` predicate, no bounce selector.

### Spike state (reference implementation)
The spike at `.gsd/workflows/spikes/M021-timeline-fsm/src/agumon.rs` has a complete reference:
- `base_timeline_with_bouncing_fire()` — creates the timeline with the Loop branch always present
- `has_bouncing_fire()` — predicate gating the branch: `ctx.skill_tree(evt.caster).rank("agumon::bouncing_fire").unwrap_or(0) >= 1`
- `bounce_pick_next_selector()` — picks the next bounce target (excludes already-hit targets)
- `on_bounce_hop()` — half-scaled fire damage hook for each hop

### S08 requirement
"Bouncing Fire OFF≡baseline" means: with the talent at rank 0 (default), the Intent stream from a baby_flame cast must be **identical** to the baseline (no loop branch executed). This is validated by a test analogous to `bouncing_fire_off_baseline_identical_to_no_loop` from the spike.

### Implementation approach
1. Add `BeatKind::Loop` branch to `baby_flame` timeline (always present in graph)
2. Register `"agumon/has_bouncing_fire"` predicate in ExtRegistries
3. Register `"agumon/bounce_pick_next"` selector in ExtRegistries
4. Register `"agumon/on_bounce_hop"` hook in ExtRegistries
5. The edge `impact_signal → bounce_loop` has `gate: Some("agumon/has_bouncing_fire")`; fallback edge `impact_signal → cast_end` unconditional (D029)
6. Test: rank=0 → same intent stream as no-loop; rank=1 → exactly 1 hop

---

## 5. What Needs to Change

### 5.1 Extract Twin Core to mini-plugin (D005)

**Create `src/combat/blueprints/twin_core/mod.rs`** containing:
- `TwinCoreDesignTag`, `TwinCoreState` (Resource), tag constants
- `twin_core_design_tag()`, `classify_twin_core_tag()`, `twin_core_added_tag_transition()` helpers
- `TwinCoreHook` (implementing `CombatKernelHook`)
- `apply_twin_core_transitions_system`
- No Digimon names in this module

**Move from `kernel.rs`**:
- `TwinCoreSignal` enum
- `TwinCoreTransition` struct + constructors
- Remove `CombatKernelTransition::TwinCore(TwinCoreTransition)` variant
- Replace all `CombatKernelTransition::TwinCore(...)` dispatch with `CombatKernelTransition::Blueprint { owner: "twin_core", name: "...", payload: ... }` or keep as kernel-side primitive depending on final D008 decision

**Update `kernel.rs` `register_combat_kernel_runtime`**:
- Remove `app.add_plugins(AgumonPlugin, ...)` — replace with `app.add_plugins(TwinCorePlugin)`
- `TwinCorePlugin` owns: `init_resource::<TwinCoreState>()`, `add_systems(Update, apply_twin_core_transitions_system)`, `registry.register(TwinCoreHook)`

### 5.2 Convert Gabumon to directory module

- Create `src/combat/blueprints/gabumon/` directory
- Move `gabumon.rs` → `gabumon/mod.rs`
- Move signals into `gabumon/signals.rs` mirroring Agumon structure
- Update `gabumon/mod.rs` imports to reference `twin_core::` instead of `blueprints::agumon::`

### 5.3 Add Bouncing Fire to skills.ron + register hooks

- Extend `baby_flame` timeline with Loop branch (always in graph, gate-blocked at rank 0)
- Register `"agumon/has_bouncing_fire"`, `"agumon/bounce_pick_next"`, `"agumon/on_bounce_hop"` in Agumon's `register()` fn

### 5.4 Update all import paths

Files referencing `blueprints::agumon::TwinCore*` that will need updating:
- `tests/twin_core_integration.rs`
- `tests/twin_core_mechanics.rs`
- `src/combat/blueprints/gabumon.rs` (→ `twin_core::`)
- Any other test importing from `blueprints::agumon::identity`

### 5.5 Kernel P001 compliance check

After changes, `rg "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/ --glob '!blueprints/**'` must → 0 lines. The `CombatKernelTransition::TwinCore(...)` variant removal is the key change needed for this.

**Decision required**: Does `CombatKernelTransition::TwinCore` disappear entirely (replaced by `Blueprint { owner: "twin_core", ... }`) or does it become a generic `CombatKernelTransition::Blueprint` variant? Per M021 CONTEXT M5: "Rimozione delle 5 variant Digimon-specific da `CombatKernelTransition` ... Sostituiti da `CombatKernelTransition::Blueprint { owner, payload }` generico." → Use Blueprint variant.

---

## 6. Files Affected

| File | Change Type | Reason |
|---|---|---|
| `src/combat/blueprints/agumon/identity.rs` | Shrink / move | TwinCore types move to `twin_core/` |
| `src/combat/blueprints/agumon/mod.rs` | Update | AgumonPlugin reduced; passive hooks stay, TwinCorePlugin separate |
| `src/combat/blueprints/gabumon.rs` | Restructure to dir | Needs `signals.rs` split and `twin_core::` import |
| `src/combat/blueprints/mod.rs` | Add `twin_core` module | Export twin_core mini-plugin |
| `src/combat/blueprints/twin_core/mod.rs` | Create new | Canonical home for TwinCore mechanic |
| `src/combat/kernel.rs` | Modify | Remove `TwinCoreSignal`, `TwinCoreTransition`, `TwinCore` variant from enum |
| `src/combat/api/applier.rs` | Possibly minor | If TwinCore CombatEvent handling changes |
| `src/combat/observability.rs` | Check | May reference `TwinCoreState` in snapshots |
| `assets/data/skills.ron` | Extend | Add Loop branch to `baby_flame` |
| `tests/twin_core_integration.rs` | Update imports | `blueprints::twin_core::` instead of `blueprints::agumon::` |
| `tests/twin_core_mechanics.rs` | Update imports | Same |

---

## 7. Natural Task Seams

### T01: Extract TwinCore to mini-plugin module
Move `TwinCoreSignal`, `TwinCoreTransition`, `TwinCoreState`, `TwinCoreHook`, `apply_twin_core_transitions_system`, tag types + helpers from `agumon/identity.rs` and `kernel.rs` into `src/combat/blueprints/twin_core/mod.rs`. Remove `CombatKernelTransition::TwinCore(...)` variant, replace with `Blueprint { owner: "twin_core", ... }`. Create `TwinCorePlugin`. Update `kernel.rs` to add `TwinCorePlugin` instead of `AgumonPlugin` for the TwinCore wiring. Update test imports. Verify: `cargo test twin_core` passes; P001 grep passes for TwinCore.

**Risk**: Removing `CombatKernelTransition::TwinCore` is a breaking change across all match arms in the kernel transition dispatch. Need to audit every match on this variant in `identity.rs`, `observability.rs`, and any test harnesses. The `apply_twin_core_transitions_system` currently pattern-matches on `CombatKernelTransition::TwinCore(...)` — this must be adapted to `Blueprint { owner: "twin_core", ... }` or the TwinCore state machine must use a different event path.

### T02: Convert Gabumon to directory module + update coupling
Create `src/combat/blueprints/gabumon/` directory. Move `gabumon.rs` to `gabumon/mod.rs`. Extract signal logic to `gabumon/signals.rs`. Replace `use blueprints::agumon::{TwinCoreDesignTag, twin_core_added_tag_transition}` with `use blueprints::twin_core::`. Verify: `cargo test` still passes. `blueprints::mod.rs` continues to work.

**Risk**: Low. This is pure restructuring; no behavior changes. Import path fix is mechanical.

### T03: Add Bouncing Fire (Loop branch) to baby_flame + register hooks
Register `"agumon/has_bouncing_fire"` predicate, `"agumon/bounce_pick_next"` selector, `"agumon/on_bounce_hop"` hook in Agumon's `register_passive_hooks` or a new `register_active_hooks` fn. Extend `baby_flame` timeline in `skills.ron` with `BeatKind::Loop` branch behind the predicate gate. Write a deterministic test: with rank=0, Intent stream identical to current baseline; with rank=1, one extra hop fires.

**Risk**: Medium. This is the first production use of `BeatKind::Loop` in a Digimon blueprint (not a unit test). Need to verify the loop runner in `src/combat/api/runner.rs` handles the `hop_index` correctly for non-zero hop counts and that `validate_timeline_refs` covers Loop body nodes. Also need a `SkillTree` component on the Agumon entity for the predicate to read from.

### T04: Canonical end-to-end test — Twin Core fire+ice cycle
Write (or adapt existing `twin_core_integration.rs`) a canonical test that exercises the full Twin Core loop through the new `Blueprint { owner: "twin_core" }` event path: Heated tag added → cross_resonance=1 → Chilled tag added → cross_resonance=2 → Damage beat → TwinBurst fires → cycle reset. Verify `TwinCoreState` after each step. This replaces the existing tests which pump `CombatKernelTransition::TwinCore(...)` directly.

**Risk**: Medium. The existing tests are a good baseline but they test the old event path. The new path must produce identical state transitions through a different event type.

---

## 8. Risks and Mitigations

| Risk | Severity | Mitigation |
|---|---|---|
| `CombatKernelTransition::TwinCore` removal breaks many match arms | High | Audit with `rg "TwinCore"` across entire `src/`; fix all arms before removing |
| `observability.rs` snapshots include TwinCore snapshot section | Medium | Check `src/combat/observability.rs` for TwinCoreState reads; update to read from `blueprints::twin_core::` |
| Loop runner not battle-tested in production | Medium | Use spike reference implementation directly; start with rank=0 (no-op) test |
| SkillTree component not yet on Agumon in all test harnesses | Medium | Add `SkillTree` spawn in the Bouncing Fire test; default to empty (rank 0) |
| UnitId hardcoding in Gabumon passive (`PASSIVE_OWNER: UnitId(2)`) | Low | Acceptable for v0; D032 says one module + one register, not that UnitId must be dynamic |

---

## 9. Verification Approach

1. `cargo check` — headless, no new warnings
2. `cargo test twin_core` — both integration and mechanics tests pass with new import paths
3. `cargo test passive` — `passive_reactive_canon.rs` + `passive_canon_support.rs` unaffected
4. `rg "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame" src/combat/ --glob '!blueprints/**'` → 0 lines (P001)
5. New test: `bouncing_fire_off_equals_baseline` — rank=0 Intent stream ≡ current baby_flame intent stream
6. New test: `twin_core_end_to_end_via_blueprint_event` — full fire/ice cycle through Blueprint event path

---

## 10. Implementation Recommendation

**Ordering: T01 → T02 → T04 → T03**

Do T01 (TwinCore extraction) first as it is the prerequisite for everything: it unblocks T02 (Gabumon can import from `twin_core::`) and T04 (test the new event path). T03 (Bouncing Fire) is independent of T04 and can go last — it requires Loop infrastructure which should be separately verified against the existing runner before wiring into production.

The Bouncing Fire work (T03) is scoped to: predicate + selector + hook registration, skill.ron extension, and one deterministic test. It explicitly does NOT require implementing a real `SkillTree` progression system — just read `0` from a `SkillTree` with no ranks set, which already satisfies "OFF≡baseline."

The total scope is moderate: 2-3 new/moved files, 2 updated blueprint files, 1 skills.ron extension, 2 updated test files, 1 kernel.rs modification (removing 3 types + 1 enum variant).

---

## 11. Key Reference Paths

- `src/combat/blueprints/agumon/mod.rs` — AgumonPlugin + passive runtime
- `src/combat/blueprints/agumon/identity.rs` — TwinCore types (to be moved)
- `src/combat/blueprints/agumon/signals.rs` — active signal dispatcher
- `src/combat/blueprints/gabumon.rs` — full Gabumon blueprint (to be restructured)
- `src/combat/kernel.rs` lines 432–506 — `TwinCoreSignal`, `TwinCoreTransition`, and `CombatKernelTransition::TwinCore` (to be removed)
- `src/combat/kernel.rs` lines 1094–1128 — `register_combat_kernel_runtime` (add TwinCorePlugin, remove AgumonPlugin TwinCore wiring)
- `assets/data/skills.ron` lines 1–31 — `baby_flame` timeline (to be extended with Loop branch)
- `tests/twin_core_integration.rs` — integration tests (import path updates)
- `tests/twin_core_mechanics.rs` — state machine tests (import path updates)
- `.gsd/workflows/spikes/M021-timeline-fsm/src/agumon.rs` — spike Bouncing Fire reference
- `.gsd/workflows/spikes/M021-timeline-fsm/tests/validation.rs` lines 527–638 — spike Bouncing Fire test patterns
