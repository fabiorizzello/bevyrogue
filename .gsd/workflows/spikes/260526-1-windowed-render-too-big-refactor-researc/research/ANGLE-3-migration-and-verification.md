# Angle 3 — Refactor strategy and verification plan

## Question

What is the safest sequence for executing the future refactor, and how should it be verified given the project’s windowed/testing constraints?

## Evidence

### Existing proof surfaces

There are **15** files under `tests/windowed_only/`.

Static/source-contract coverage is already substantial:
- **8** windowed-only tests mention `render.rs`
- **4** mention `mod.rs`
- **8** use `include_str!`-style source inspection

Key examples:
- `tests/windowed_only/agumon_module_extraction.rs`
- `tests/windowed_only/renamon_extension_contract.rs`
- `tests/windowed_only/digimon_sprite_cue_dispatch.rs`
- `tests/windowed_only/enoki_impact_render.rs`
- `tests/windowed_only/vfx_windowed_contracts.rs`

### Operational constraint

`docs/agent-testing.md` explicitly says:
- headless is the default for CI/agent loops
- visual `windowed` verification is for humans
- `cargo winx` / `cargo run --features windowed` is a manual path

This aligns with the existing test comments citing **K001**: auto-mode should not rely on launching the real windowed app.

## Options considered

### Option A — Incremental extraction with proof preserved at each step  **(recommended)**

Refactor in small, reviewable steps while keeping existing tests green and updating source-contracts only when the seam genuinely moves.

Suggested order:

1. **Extract shared registry/types first**
   - Move registry types out of `render.rs`
   - Update Digimon module imports
   - Keep behavior unchanged
2. **Extract `render` subsystems next**
   - spawn/atlas code
   - effect lifecycle code
   - feedback/reaction code
   - leave `advance_digimon_presentation` last or near-last
3. **Thin `windowed/mod.rs` in parallel or afterward**
   - egui panels
   - validation/config
   - bootstrap helper(s)
4. **Only then split the large playback system internally**
   - extract helper fns around mode sync, release handling, VFX-on-enter, fade/idle return

#### Pros
- Lowest risk.
- Keeps diffs understandable.
- Lets existing tests continue acting as tripwires.
- Avoids combining structural and behavioral change in the same patch.

#### Cons
- More commits/PR steps.
- Temporary transitional re-exports may be needed.

### Option B — Big-bang restructure in one patch

#### Pros
- Fastest path to the final target structure.

#### Cons
- Harder review.
- Source-contract failures become noisier to diagnose.
- Easier to accidentally change visibility/order/behavior while moving files.

### Option C — Add new tests first, then refactor

Add dedicated source-contract tests for the intended future modules before moving code.

#### Pros
- Stronger preconditions.
- Makes the target architecture explicit.

#### Cons
- Risk of prematurely locking in file names/layout details that the research spike should leave flexible.
- Can create churn if the chosen module map changes during implementation.

## Findings

1. **The project already prefers source-contract proof for binary-crate-only windowed code.**
   That is the right verification mode to preserve. The refactor should lean into it, not fight it.

2. **The current tests protect architecture more than file names.**
   Many assertions are token-based (“engine stays species-agnostic”, “register_all seam exists”, “render consults registry”). That is good: it gives room to move functions as long as the real seam is preserved.

3. **The highest-risk move is changing import/ownership surfaces, not moving pure helpers.**
   Especially:
   - registry type ownership
   - visibility scopes
   - plugin ordering / system ordering
   - any helper called by source-contract-tested code paths

4. **A small amount of additional source-contract coverage may be worth adding during implementation, not before.**
   Example candidates:
   - `windowed/mod.rs` stays a thin entrypoint after extraction
   - `render/mod.rs` owns ordering while registries live elsewhere
   - Digimon modules import registries from the new shared module rather than from the runtime system file

## Recommended migration plan

### Phase A — make the type hub explicit
- Create shared registry/type module(s)
- Re-point Agumon/Renamon imports
- Keep runtime systems functionally untouched

### Phase B — peel off low-risk subsystems
- extract UI panel helpers from `windowed/mod.rs`
- extract validation helpers/state
- extract atlas/spawn helpers from `render`
- extract effect lifecycle helpers from `render`

### Phase C — isolate the complex playback core
- move `DigimonSprite` + playback helpers to dedicated module
- split `advance_digimon_presentation` with private helper functions
- keep plugin ordering in one obvious place

## Verification plan for the future implementation

After each extraction step:

1. Run targeted compile/tests first
   - `cargo test`
   - if the implementation touches `windowed`-gated tests directly, also run the relevant targeted test(s)
2. Re-check source-contract tests that mention the moved seam
3. Avoid depending on live windowed execution for proof
4. If a seam meaningfully changes, update token-based tests to assert the new architectural truth, not the old file path by habit

## Recommendation for this angle

Use **Option A**: incremental, proof-preserving extraction, with source-contract tests as the primary safety net and manual `windowed` runs treated as optional human follow-up, not required agent proof.

## Confidence

**High.** The project already contains the exact test style needed for this refactor, and the operational constraint against agent-run windowed sessions strongly favors this approach.
