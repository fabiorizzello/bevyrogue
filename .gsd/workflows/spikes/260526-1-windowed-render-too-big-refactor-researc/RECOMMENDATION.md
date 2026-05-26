# Recommendation — windowed/render too big, refactor research to split it in multiple files

## Executive summary

The best next step is a **responsibility-first refactor** that turns the current oversized windowed implementation into a small set of focused modules, while preserving the existing species-agnostic and headless-proof seams.

The strongest conclusion from the research is that `src/windowed/render.rs` is not only too large; it is also carrying two different architectural roles at once: **runtime systems** and **shared registration types consumed by per-Digimon modules**. Because of that, the safest refactor is not “move a few big functions around” — it is to first extract the shared registries/types, then peel off coherent subsystems, and only then split the largest playback system internally.

## Comparison matrix

| Angle / option | What it optimizes for | Benefits | Risks / downsides | Verdict |
|---|---|---|---|---|
| **A1: Responsibility-first split of `render.rs`** | Long-term maintainability | Clean ownership, better reviews, better future extensibility | Highest initial file churn | **Best option** |
| A1: Coarse two-way split (`core` + `effects`) | Lower short-term churn | Faster initial shrink | Leaves mixed responsibilities in place | Acceptable fallback |
| A1: Keep file flat | Lowest immediate risk | No import churn | Does not solve the real problem | Reject |
| **A2: Thin `windowed/mod.rs` composition root** | Clear app wiring | Stable public seam, smaller focused files | More module hopping | **Best option** |
| A2: Move only egui panels | Minimal churn | Good quick win | Leaves config/bootstrap/validation mixed | Partial improvement |
| A2: Keep `mod.rs` mixed | No churn | None beyond status quo | Future monolith risk | Reject |
| **A3: Incremental proof-preserving migration** | Safe execution | Reviewable steps, existing tests remain useful | More commits/steps | **Best option** |
| A3: Big-bang restructure | Speed | Reaches target layout quickly | High review and breakage risk | Reject |
| A3: Add many new tests before moving code | Extra preconditions | Strong upfront assertions | Can over-lock file layout too early | Use sparingly |

## Recommended structure

### 1) Refactor `src/windowed/render.rs` into a module tree

Suggested target:

- `src/windowed/render/mod.rs`
  - `RenderPlugin`
  - system ordering / public exports only
- `src/windowed/render/registries.rs`
  - all shared registry/resource types
  - `SpritePresentationEntry`
  - atlas registry/resource types
- `src/windowed/render/playback.rs`
  - `DigimonSprite`
  - `advance_digimon_presentation`
  - `sync_digimon_mode`
  - barrier/release helpers
- `src/windowed/render/spawn.rs`
  - atlas build
  - sprite spawn
- `src/windowed/render/effects.rs`
  - effect spawn
  - projectile advancement
  - detonate handling
  - anchor helpers / enoki lifecycle types
- `src/windowed/render/feedback.rs`
  - camera shake
  - damage numbers
  - hurt/death/fade systems
- `src/windowed/render/clock.rs` or `state.rs`
  - `AnimationClock`
  - `PendingAnimationTicks`
  - render-local constants/marker types as appropriate

### 2) Turn `src/windowed/mod.rs` into a thin feature entrypoint

Suggested target:

- `src/windowed/mod.rs`
  - `pub fn config_from_env`
  - `pub fn register`
  - `pub fn register_combat_systems`
  - module declarations / re-exports only
- `src/windowed/validation.rs`
  - validation config + parsing + soak tick
- `src/windowed/ui_panels.rs` or `ui_shell.rs`
  - `UiPlugin`
  - roster/turn-order/unit-chip egui code
- `src/windowed/bootstrap.rs`
  - demo/bootstrap flow and related runtime wiring
- keep existing:
  - `src/windowed/demo.rs`
  - `src/windowed/digimon/`

## Why this is the primary recommendation

1. **It fixes the real coupling, not just the line count.**
   The per-Digimon modules currently import shared registry types from `render.rs`. Extracting those types first creates a cleaner architecture for every later move.

2. **It matches the seams already visible in the codebase.**
   The project already has good examples of this style:
   - `demo.rs` extracted from `windowed/mod.rs`
   - `digimon/` extracted for species-specific presentation ownership

3. **It preserves the current architectural contracts.**
   The recommendation keeps:
   - engine files species-agnostic
   - headless/source-contract proof as the primary verification mode
   - the stable `register_all(app)` Digimon aggregation seam
   - `windowed` feature isolation

## Fallback recommendation

If the first production pass needs to minimize churn, do a **two-stage partial refactor**:

1. extract `registries.rs` first
2. extract `ui_panels.rs` and `validation.rs`
3. stop there temporarily if needed

That would already remove the most awkward cross-module coupling and shrink the two biggest files without committing to the full render subsystem breakup immediately.

## What would change this recommendation

The recommendation would change only if one of these turns out to be true during implementation:

- `advance_digimon_presentation` has hidden circular dependencies that make subsystem extraction much harder than the current read suggests
- source-contract tests are more path-coupled than they appear and need more widespread rewrites than expected
- a near-term feature needs rapid iteration inside windowed code and would be blocked by an in-progress large restructure

If that happens, the fallback is to stop after `registries.rs` + `ui/validation` extraction and defer the deeper playback split.

## Recommended execution order

1. **Extract shared registry/type ownership from `render.rs`**
2. Update Agumon/Renamon imports to the new shared module
3. Extract `windowed/mod.rs` panel helpers
4. Extract `windowed/mod.rs` validation helpers/state
5. Extract `render` spawn/atlas helpers
6. Extract `render` effects/projectile helpers
7. Extract playback core and internally break up `advance_digimon_presentation`
8. Keep plugin/system ordering centralized and obvious in `mod.rs` / `render/mod.rs`

## Verification approach for the future implementation

Use the project’s existing proof model:

- keep running headless/source-contract tests as the main proof surface
- do **not** depend on agent-launched live windowed execution
- update contract tests only when the architectural truth changes
- prefer several small refactor steps over one giant patch

## Next steps if accepted

1. Create a follow-up implementation task/plan for the refactor itself.
2. Start with `render` shared registry extraction.
3. Immediately after that, thin `windowed/mod.rs` by moving egui panels and validation.
4. Only then split the heavy playback/effects systems.
5. Preserve or adjust source-contract tests as each seam moves.
