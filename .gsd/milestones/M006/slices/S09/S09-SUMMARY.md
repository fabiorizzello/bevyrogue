---
id: S09
parent: M006
milestone: M006
provides:
  - render/registries.rs as the canonical location for engine-generic presentation registry types
  - WarnOnce<K> shared lib util reusable by S10/S11/S12/S13/S14 windowed consumers
requires:
  []
affects:
  []
key_files: []
key_decisions:
  - render.rs uses a transitional pub(in crate::windowed) use re-export in T01 so species call sites keep compiling before T02 repointing; T02 then trims it to a plain internal use
  - Species import directly from crate::windowed::render::registries (canonical), not through the render.rs re-export layer
  - WarnOnce<K> placed at lib top level (src/warn_once.rs) rather than under animation/ so windowed consumers (S08/S11–S14) can reuse it engine-generically
  - windowed/mod.rs left unchanged — it already held only panels + validation + register_all wiring and never referenced the moved registry types
patterns_established:
  - Engine-generic presentation registries live in src/windowed/render/registries.rs; species modules import from that canonical path directly
  - Shared warn-once dedup pattern is WarnOnce<K: Eq + Hash> at src/warn_once.rs — do not re-inline HashSet dedup in future consumers
observability_surfaces:
  - none
drill_down_paths:
  []
duration: ""
verification_result: passed
completed_at: 2026-05-27T11:50:03.262Z
blocker_discovered: false
---

# S09: Extract shared registries and types out of render.rs into render/registries.rs

**Extracted engine-generic registries into render/registries.rs, repointed species imports to the new canonical path, and promoted inline warn-once dedup into a shared WarnOnce&lt;K&gt; lib util — pure structural move, 75 windowed tests green.**

## What Happened

Three-task structural refactor with no behavior changes.

**T01 — Carve registries into render/registries.rs**: Moved 10 engine-generic presentation registry structs/resources and types (EnokiVfxRegistry, SoftParticleMaterial, EnokiEffect, EnokiLifecycle, OnEnterEffectRegistry, SkillReleaseEffectRegistry, DetonateEffectRegistry, SkillStartNodeRegistry, SpritePresentationRegistry, SpritePresentationEntry) out of the 2498-line render.rs into a new src/windowed/render/registries.rs submodule. render.rs declares `pub(in crate::windowed) mod registries` and re-exports the ten items via `pub(in crate::windowed) use registries::{...}` so all downstream call sites keep resolving unchanged during the transitional phase. matches_unit was raised to pub(in crate::windowed) since presentation_entry_for_unit (which stays in render.rs) now calls it across the module boundary. The source-contract test in renamon_extension_contract.rs was updated to split type-definition assertions (now against registries.rs) from lookup-seam assertions (still against render.rs). The unused Particle2dEffect import was dropped from render.rs to satisfy -D warnings.

**T02 — Repoint species imports and trim re-export**: Repointed agumon/mod.rs and renamon/mod.rs to import the moved engine-generic types directly from crate::windowed::render::registries (the new canonical path) rather than through render.rs's transitional re-export. Once species no longer depended on the re-export, it was downgraded from `pub(in crate::windowed) use` to a plain internal `use registries::{...}` and EnokiEffect (zero internal uses confirmed by grep) was dropped, keeping the nine names render.rs's own presentation systems actually use. windowed/mod.rs needed no change — it already contained only panels + validation + digimon::register_all wiring and never referenced the moved registry types.

**T03 — Extract WarnOnce&lt;K&gt; lib util**: Promoted the inline `Local&lt;HashSet&lt;AssetId&lt;AnimGraph&gt;&gt;&gt;` warn-once dedup pattern from animation/registry.rs into a generic, reusable `WarnOnce&lt;K: Eq + Hash&gt;` at src/warn_once.rs, exposing `should_warn(key) -&gt; bool` (mirrors HashSet::insert semantics), plus `has_warned` and `clear` for inspection/reset. Registered as `pub mod warn_once` in src/lib.rs so all windowed consumers (S08/S11/S12/S13/S14) share one surface. The animation registry was repointed to `Local&lt;WarnOnce&lt;AssetId&lt;AnimGraph&gt;&gt;&gt;`; once-per-asset-id warn behavior is identical. Two unit tests added (per-key dedup; has_warned/clear lifecycle).

## Verification

T01: RUSTFLAGS='-D warnings' cargo build --features windowed exit 0; cargo test --features windowed --test windowed_only 75/75 green.
T02: RUSTFLAGS='-D warnings' cargo build --features windowed exit 0; cargo test --features windowed --test windowed_only 75/75 green; cargo build (headless) exit 0.
T03: cargo test exit 0; cargo test --lib warn_once 2/2 passed; cargo test --features windowed --test windowed_only 75/75 green; RUSTFLAGS="-D warnings" cargo check exit 0.
Closer fresh-run: cargo test --features windowed --test windowed_only 75/75; RUSTFLAGS="-D warnings" cargo build --features windowed exit 0; cargo test exit 0. All green.

## Requirements Advanced

None.

## Requirements Validated

None.

## New Requirements Surfaced

None.

## Requirements Invalidated or Re-scoped

None.

## Operational Readiness

None.

## Deviations

None.

## Known Limitations

None.

## Follow-ups

S10 (split render into playback/spawn/effects/feedback/clock submodules) can now carve into a cleaner render.rs after the registry types have been relocated to registries.rs.

## Files Created/Modified

- `src/windowed/render/registries.rs` — New submodule — engine-generic presentation registry structs and types carved out of render.rs
- `src/windowed/render.rs` — Module declaration for registries submodule; re-export trimmed to plain internal use after T02 repointing
- `src/windowed/digimon/agumon/mod.rs` — Registry imports repointed from render::* to render::registries canonical path
- `src/windowed/digimon/renamon/mod.rs` — Registry imports repointed from render::* to render::registries canonical path
- `src/warn_once.rs` — New generic WarnOnce<K> lib util with should_warn/has_warned/clear and two unit tests
- `src/lib.rs` — pub mod warn_once registration so all consumers share one surface
- `src/animation/registry.rs` — Repointed Local<HashSet<AssetId<AnimGraph>>> to Local<WarnOnce<AssetId<AnimGraph>>>; dropped unused HashSet import
- `tests/windowed_only/renamon_extension_contract.rs` — Source-contract test split to assert type definitions against registries.rs and lookup-seam against render.rs
