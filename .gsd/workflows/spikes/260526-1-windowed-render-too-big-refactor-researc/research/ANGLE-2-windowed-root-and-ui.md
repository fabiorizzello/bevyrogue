# Angle 2 — Split `src/windowed/mod.rs` and clarify the app-wiring seam

## Question

Should `src/windowed/mod.rs` stay as a mixed implementation file, or become a thin composition root for the windowed app?

## Evidence

`src/windowed/mod.rs` is **512 lines** and currently mixes these responsibilities:

- plugin/resource wiring (`UiPlugin`) — lines **67-112**
- validation env parsing/config — lines **113-168**
- app registration/composition root — lines **169-196**
- demo/bootstrap flow — lines **197-250**
- combat-system registration — lines **251-271**
- validation tick runtime — lines **272-337**
- egui panel rendering/helpers
  - `roster_panel` — lines **338-372**
  - `turn_order_panel` — lines **379-436**
  - `unit_chip` — lines **437-512**

There is already one successful extraction here:

- `src/windowed/demo.rs` now owns the windowed demo composition registry and builder.

There is also an existing stable external seam:

- `src/windowed/digimon/mod.rs::register_all(app)`

That suggests `mod.rs` is already moving toward a composition-root role, but has not finished the transition.

## Options considered

### Option A — Make `mod.rs` a thin composition root  **(recommended)**

Keep `mod.rs` as the public entrypoint for the feature, but move implementation details into smaller siblings.

Suggested shape:

- `src/windowed/mod.rs`
  - `pub fn config_from_env`
  - `pub fn register`
  - `pub fn register_combat_systems`
  - module declarations only
- `src/windowed/validation.rs`
  - `WindowedValidationConfig`
  - parsing helpers
  - `WindowedValidationState`
  - `windowed_validation_tick`
- `src/windowed/ui_shell.rs` or `ui_panels.rs`
  - `UiPlugin`
  - `roster_panel`
  - `turn_order_panel`
  - `unit_chip`
  - any view-only helpers
- `src/windowed/bootstrap.rs`
  - `windowed_bootstrap_system`
  - maybe the combat-system registration if you want one “runtime wiring” home
- keep `demo.rs` and `digimon/` as they are

#### Pros
- Fits the direction the codebase is already taking.
- Keeps the public seam stable while shrinking the internal file.
- Easier to review UI, validation, and bootstrap changes independently.
- Prevents `mod.rs` from becoming the next monolith after `render.rs` is split.

#### Cons
- Slightly more module hopping for small edits.
- May feel “over-structured” if the windowed feature remains very small — but at 500+ lines it is already past that point.

### Option B — Only move the egui panels out

Keep config/validation/bootstrap in `mod.rs`, extract only UI helper functions.

#### Pros
- Lowest churn.
- Good immediate payoff because `unit_chip` + panel functions are isolated UI code.

#### Cons
- Leaves `mod.rs` as a mixed lifecycle/config/runtime file.
- Likely postpones, rather than avoids, the validation/bootstrap split.

### Option C — Keep everything in `mod.rs`

#### Pros
- No churn.

#### Cons
- Weakens the architectural story: the “root module” is also a large implementation file.
- Makes future windowed additions accumulate in the same place.

## Findings

1. **`mod.rs` is already acting like an app root.**
   It coordinates plugins, demo bootstrap, combat systems, and validation. That is a strong sign it should become a thin orchestrator rather than keep implementation-heavy helpers.

2. **The egui panels are the cleanest first extraction.**
   `roster_panel`, `turn_order_panel`, and `unit_chip` are highly local, view-only helpers with little coupling to the rest of the file.

3. **Validation is the second clean extraction.**
   `WindowedValidationConfig`, parsing, runtime state, and `windowed_validation_tick` form a coherent subdomain that does not need to live beside UI helpers.

4. **`register_combat_systems` should stay easy to find.**
   Whether it remains in `mod.rs` or moves to `bootstrap.rs`, it should still be re-exported from `mod.rs` so callers do not need to know the internal file map.

## Recommendation for this angle

Use **Option A**: keep `src/windowed/mod.rs` as the stable feature entrypoint, but move UI panel code and validation/bootstrap internals into dedicated sibling files.

If the production refactor wants the smallest possible first step, the order should be:

1. extract egui panels
2. extract validation/config
3. extract bootstrap/runtime wiring helpers
4. leave `register`, `config_from_env`, and public re-exports in `mod.rs`

## Confidence

**High.** This follows the module’s current direction, respects the existing public seam, and complements the larger `render.rs` decomposition rather than competing with it.
