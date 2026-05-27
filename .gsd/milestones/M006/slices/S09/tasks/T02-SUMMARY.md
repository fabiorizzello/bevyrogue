---
id: T02
parent: S09
milestone: M006
key_files:
  - src/windowed/digimon/agumon/mod.rs
  - src/windowed/digimon/renamon/mod.rs
  - src/windowed/render.rs
key_decisions:
  - Species import the moved types directly from crate::windowed::render::registries (the new canonical path), not through a render.rs re-export
  - Once species repointed, downgrade render.rs's transitional `pub(in crate::windowed) use` re-export to a plain internal `use` and drop EnokiEffect (zero internal uses) to satisfy the -D warnings build
  - Leave windowed/mod.rs unchanged: it was already thin (panels + validation + register_all wiring) and held none of the moved registry code; deleting working wiring would be an out-of-scope behavior change
duration: 
verification_result: passed
completed_at: 2026-05-27T11:34:02.574Z
blocker_discovered: false
---

# T02: Repointed Agumon/Renamon registry imports to crate::windowed::render::registries and trimmed render.rs's now-internal-only re-export to a warnings-clean state

**Repointed Agumon/Renamon registry imports to crate::windowed::render::registries and trimmed render.rs's now-internal-only re-export to a warnings-clean state**

## What Happened

Repointed the per-Digimon windowed modules to import the moved engine-generic types directly from the new submodule path crate::windowed::render::registries instead of crate::windowed::render::*. agumon/mod.rs now imports DetonateEffectRegistry, EnokiEffect, EnokiLifecycle, EnokiVfxRegistry, OnEnterEffectRegistry, SkillReleaseEffectRegistry, SkillStartNodeRegistry, SpritePresentationEntry, SpritePresentationRegistry from registries; renamon/mod.rs imports its seven from registries. Agumon's test-module reference to crate::windowed::render::should_auto_release_unbridged was left untouched — that helper stays in render.rs and is not a registry type. With species no longer routing through render.rs's transitional re-export, that re-export became internal-only; I converted it from `pub(in crate::windowed) use` to a plain `use registries::{...}` and dropped EnokiEffect (which render.rs constructs/names nowhere — 0 internal uses, confirmed by grep), keeping the nine names render.rs's own presentation systems actually use. windowed/mod.rs required no edit: it already contains only validation config/state, the UiPlugin panel wiring, windowed bootstrap/turn-gate systems, and the digimon::register_all(app) call (line 109); it never referenced the moved registry types, so it is already in the target "panels + validation + register_all" shape. Forcing further deletion would change behavior, which the slice forbids (pure structural move). Confirmed the species modules still only populate their own registry entries (no engine-side species branches).

## Verification

RUSTFLAGS='-D warnings' cargo build --features windowed finished clean (exit 0) after trimming the re-export — the intermediate `unused import: EnokiEffect` error under -D warnings was resolved by removing it from render.rs's internal use list. cargo test --features windowed --test windowed_only: 75 passed, 0 failed. Default headless `cargo build` also clean (exit 0), confirming the lib/headless path is unaffected (R002). Did not run the windowed binary (K001).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `RUSTFLAGS='-D warnings' cargo build --features windowed` | 0 | pass | 3350ms |
| 2 | `cargo test --features windowed --test windowed_only` | 0 | pass | 1034ms |
| 3 | `cargo build` | 0 | pass | 5258ms |

## Deviations

windowed/mod.rs (a slice-goal target for "thinning") needed no change because the engine-generic registries lived in render.rs, not mod.rs — mod.rs already wires only panels + validation + digimon::register_all. The re-export trim touched render.rs (a T01 file) because removing species' dependency on the re-export is what exposes EnokiEffect as unused; this is the natural completion of the repointing, not separate scope.

## Known Issues

none

## Files Created/Modified

- `src/windowed/digimon/agumon/mod.rs`
- `src/windowed/digimon/renamon/mod.rs`
- `src/windowed/render.rs`
