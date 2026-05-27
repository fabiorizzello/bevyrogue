---
sliceId: S09
uatType: artifact-driven
verdict: PASS
date: 2026-05-27T12:00:00.000Z
---

# UAT Result — S09

## Checks

| Check | Mode | Result | Notes |
|-------|------|--------|-------|
| `RUSTFLAGS="-D warnings" cargo build --features windowed` exits 0 with zero warnings/errors | runtime | PASS | `Finished dev profile [optimized + debuginfo] target(s) in 0.18s` — exit 0 |
| `cargo test --features windowed --test windowed_only` — 75 tests pass, 0 fail | runtime | PASS | `test result: ok. 75 passed; 0 failed; 0 ignored` — exit 0 |
| `cargo test --lib warn_once` — 2 tests pass (per-key dedup; has_warned/clear lifecycle) | runtime | PASS | `warns_once_per_key` and `has_warned_reflects_state_and_clear_resets` both ok — exit 0 |
| `SpritePresentationRegistry` type definition in `src/windowed/render/registries.rs` | artifact | PASS | Defined at `registries.rs:114`; `render.rs` has only plain internal `use registries::{...}` at line 607 — no `pub use` re-export remaining |
| `render::registries` import in `agumon/mod.rs` and `renamon/mod.rs` | artifact | PASS | Both files import from `crate::windowed::render::registries` (lines 15 and 19 respectively) — canonical path confirmed |
| `WarnOnce` struct in `src/warn_once.rs`; `pub mod warn_once` in `src/lib.rs`; `Local<WarnOnce<AssetId<AnimGraph>>>` in `animation/registry.rs` | artifact | PASS | `warn_once.rs:19` defines struct; `lib.rs:7` declares `pub mod warn_once`; `registry.rs:250` uses `Local<WarnOnce<AssetId<AnimGraph>>>` |
| `cargo build` (headless, no features) exits 0 — no windowed types leaked | runtime | PASS | `Finished dev profile` — exit 0 in 3.68s |

## Overall Verdict

PASS — All 7 automatable checks green: clean windowed build (-D warnings), 75/75 windowed tests, 2/2 warn_once unit tests, registry types canonical in `render/registries.rs`, species modules import from canonical path, `WarnOnce<K>` correctly placed and declared at lib level, headless build clean.

## Notes

- No deviations from expected structure found.
- The `SpritePresentationRegistry` grep confirmed: definition in `registries.rs`, internal `use` (not `pub use`) in `render.rs`, and both species modules pointing directly to `crate::windowed::render::registries`.
- `EnokiEffect` was correctly absent from `render.rs`'s internal use list (only 9 names retained, as documented in the summary).
- Headless build required a recompile (3.68s) but exited clean — R016 invariant preserved.
