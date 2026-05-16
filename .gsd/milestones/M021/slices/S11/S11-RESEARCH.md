# S11 Research: UI and AI consumers via SkillCtx Preview

## Executive Summary

S11 is low-risk and well-scaffolded. The three key building blocks are already in place:

1. **`SkillCtxMode::Preview` exists** — defined in `src/combat/api/skill_ctx.rs`, passed through `BeatRunner::step` and `run_to_completion`, and already tested for stream parity with Execute/DryRun in `tests/timeline_mode_parity.rs`.
2. **`query_skill_preview` does not exist** — needs to be created as a standalone free function in `src/combat/api/` (or a new `src/combat/preview.rs`). It would call `BeatRunner::run_to_completion` with `SkillCtxMode::Preview` and return the collected `VecDeque<Intent>`.
3. **`predict_damage` does not exist in combat_panel.rs** — the combat panel (`src/ui/combat_panel.rs`) currently shows no damage preview at all; it only shows HP bars, floating damage numbers (post-fact), and action affordance (enabled/disabled). Nothing to remove.

The AI scorer (`src/combat/enemy_ai.rs`) is a pure function that picks targets by toughness ratio with no timeline involvement. It does not need or use the Intent stream at all yet.

---

## Detailed Findings

### 1. SkillCtxMode — Current State

**File:** `src/combat/api/skill_ctx.rs`

`SkillCtxMode` is a `#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]` enum with three variants:
- `Execute` (default) — normal in-game path
- `DryRun` — simulation; intents resolved but not applied
- `Preview` — UI/AI affordance path

The mode is forwarded verbatim to `BeatRunner::step` and `BeatRunner::run_to_completion` via the `mode: SkillCtxMode` parameter, and then to `SkillCtx` via `make_params`. Hooks receive the mode via `ctx.mode()` but currently no builtin hook branches on it — all three modes produce identical Intent streams (this is the I2 invariant, already verified green).

The `intent_applier` system (`src/combat/api/applier.rs`) is the only consumer that mutates world state. It drains `IntentQueue` unconditionally. **Preview mode produces intents in the pending `VecDeque` but nothing calls `intent_applier` with them** — that's precisely the mechanism for "preview without applying."

### 2. Mode Parity Tests — Already Green

**File:** `tests/timeline_mode_parity.rs`

Proves `normalize(Execute) == normalize(DryRun) == normalize(Preview)` on a branched two-path timeline with a live world predicate. Both finisher-branch and normal-branch paths are covered. This is the I2 invariant.

The `run_mode` helper used there (lines 208–227) is essentially the skeleton of `query_skill_preview`:

```rust
fn run_mode(world, regs, timeline, cast_id, mode) -> VecDeque<Intent> {
    let mut pending = VecDeque::new();
    let mut runner = BeatRunner::new(timeline, cast_id, caster, target);
    runner.run_to_completion(world, regs, mode, &mut pending, 64);
    pending
}
```

### 3. `query_skill_preview` — Does Not Exist

No function named `query_skill_preview` exists anywhere in `src/`. The planner must create it. The natural home is either:
- A new file `src/combat/api/preview.rs` (pub fn, re-exported from `src/combat/api/mod.rs`), or
- Inline in `src/combat/preview.rs` at the `src/combat/` level.

The function signature should look roughly like:

```rust
pub fn query_skill_preview(
    world: &mut World,
    regs: &ExtRegistries,
    timeline: Arc<CompiledTimeline>,
    cast_id: CastId,
    caster: UnitId,
    target: UnitId,
) -> VecDeque<Intent>
```

It calls `BeatRunner::run_to_completion` with `SkillCtxMode::Preview` and does NOT call `intent_applier` afterward. The returned queue is the "Intent stream" for downstream consumers.

### 4. `combat_panel.rs` — Current State and What Changes

**File:** `src/ui/combat_panel.rs` (864 lines, entire file under `#[cfg(feature = "windowed")]`)

The panel currently:
- Renders HP bars, ultimate charge, toughness, energy, floating damage numbers
- Shows action affordance (enabled/disabled per action type and target via `query_action_affordance`)
- Has no damage preview calculation whatsoever — no `predict_damage`, no call to any combat engine function for preview

The S11 target ("UI preview damage via stream") means adding a tooltip or label that shows expected damage when hovering a skill button, before casting. Implementation pattern: on hover/selection of a skill, call `query_skill_preview` with `SkillCtxMode::Preview`, fold the resulting `DealDamage` intents to sum expected damage, display in egui tooltip.

**What needs to change in `combat_panel.rs`:**
- Add a system parameter to access `ExtRegistries`, `TimelineLibrary`, `CastIdGen` (all Bevy Resources)
- On skill hover or pending selection, call `query_skill_preview` via `World` access
- Fold `Intent::DealDamage` entries from the stream to produce a preview damage number
- Display in `on_hover_text` or a label below the skill button

The `combat_panel` function signature is already large (`#[allow(clippy::too_many_arguments)]`). Adding world access via `ExclusiveSystemParam` or restructuring into a system that takes `&mut World` is the path of least resistance, given `query_skill_preview` needs `&mut World`.

### 5. AI Scoring — Current State and What Changes

**File:** `src/combat/enemy_ai.rs`

`pick_enemy_action` is a pure function operating on `EnemyTurnContext` (a snapshot struct with no world reference). It picks a target by toughness ratio and picks an action type by availability (ult > skill > basic). It has no concept of expected damage and no timeline involvement.

The S11 target ("AI score ottimale via stream") means creating a scoring helper that:
1. For each candidate (skill, target) pair, calls `query_skill_preview`
2. Folds the Intent stream to sum `DealDamage.amount` (and possibly other factors)
3. Returns a score used to rank candidate actions

This replaces/augments the current heuristic (toughness ratio) with a timeline-derived damage estimate. The `AiUtilityExt` axis in `ExtRegistries` (`src/combat/api/registry.rs:100`) has a placeholder `type Fn = fn()` signature — S11 may or may not refine this; the simplest approach is a standalone `score_action_by_preview` free function rather than touching the registry.

### 6. PassiveRunner — Not In Scope for S11

The `PassiveRunner` (`src/combat/api/passive_runner.rs`) always calls with `SkillCtxMode::Execute` (line 137 in `passive_dispatch_system`). S11 does not touch passive runners — they are reactive to in-game events, not preview contexts.

### 7. Key Constraint: `&mut World` in `combat_panel`

`BeatRunner::step` (and thus `run_to_completion`) takes `world: &mut World`. The current `combat_panel` system is a normal Bevy system (not exclusive), using typed system params. To call `query_skill_preview` from the UI panel:

**Option A:** Make `combat_panel` an exclusive system (`fn combat_panel(world: &mut World)`). This is the simplest but requires restructuring the system to extract params manually.

**Option B:** Run `query_skill_preview` in a separate exclusive system that caches the result into a Bevy `Resource` (e.g. `PreviewCache { skill_id, target, damage: Option<i32> }`), and have the `combat_panel` system read the cache. This keeps the panel as a normal system.

Option B is preferable for separation of concerns and avoids a large refactor of `combat_panel`.

---

## Natural Task Seams

| Task | Scope | Files |
|------|-------|-------|
| T01 | Create `query_skill_preview` free function + unit test | `src/combat/api/preview.rs` (new), `src/combat/api/mod.rs` (re-export), test inline or `tests/preview_stream.rs` |
| T02 | Add `PreviewCache` resource + exclusive system to compute it | `src/combat/preview.rs` or `src/combat/api/preview.rs`, `src/combat/plugin.rs` (register system) |
| T03 | Wire `combat_panel.rs` to read `PreviewCache` and show damage tooltip | `src/ui/combat_panel.rs` |
| T04 | Add AI scoring helper `score_action_by_preview` using the stream; wire into `pick_enemy_action` | `src/combat/enemy_ai.rs`, optionally a new `src/combat/ai_preview.rs` |

T01 and T02 are sequential (T02 depends on T01). T03 and T04 are independent of each other but both depend on T01/T02 being done.

---

## Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| `combat_panel` needs `&mut World` for preview call | Low | Use PreviewCache resource pattern (Option B above); panel reads cached value |
| Preview run mutates world inadvertently | Low | BeatRunner + SkillCtxMode::Preview does not call `intent_applier`; intents go only to the local `VecDeque` |
| AI scoring makes the enemy turn noticeably slower | Low | Preview runs are headless, fast; number of candidates is small (≤4 skills × ≤4 targets) |
| RNG in preview changes AI behavior | Low | Preview mode shares the same RNG path but does not consume global `CombatRng` — the runner has no RNG call path today; only `intent_applier` does |
| `AiUtilityExt` registry axis has wrong signature for S11 | Low | The registry axis placeholder `fn()` does not need to be changed; use a free function instead, as D024 doesn't mandate registry wiring for S11 |

---

## Verification Approach

1. **Green test: `query_skill_preview` parity** — assert that calling `query_skill_preview(SkillCtxMode::Preview)` on a fixture timeline produces the same normalized intent stream as `run_mode(SkillCtxMode::Execute)`. Mirrors `timeline_mode_parity.rs` pattern.
2. **Green test: AI scoring picks higher-damage option** — two candidates with different damage payloads; assert the scorer picks the one with the higher summed `DealDamage.amount`.
3. **Windowed smoke** (`cargo run --features windowed`) — skill button tooltip shows a numeric damage estimate.
4. **No world mutation regression** — after calling `query_skill_preview`, assert `Unit.hp_current` is unchanged (preview did not apply).

---

## Implementation Recommendation

Start with T01 (`query_skill_preview` as a thin wrapper around `BeatRunner::run_to_completion` with `Preview` mode). The function can live in a new `src/combat/api/preview.rs` and be re-exported from `mod.rs`. Write the parity test immediately as part of T01.

T02 (PreviewCache) can be deferred or skipped if the AI scorer (T04) calls `query_skill_preview` directly in an exclusive system — the cache only matters for the UI panel (T03) which runs every frame.

The slice is genuinely low-risk: no new language features, no new dependencies, no ECS topology changes. The main work is plumbing the existing `BeatRunner` + `SkillCtxMode::Preview` path to new callers.
