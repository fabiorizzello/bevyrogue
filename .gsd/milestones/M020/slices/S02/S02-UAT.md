# S02: Rimozione shim pub use legacy (twin_core / holy_support / predator_loop) — UAT

**Milestone:** M020
**Written:** 2026-05-14T10:51:51.351Z

# S02: Rimozione shim pub use legacy — UAT

**Milestone:** M020
**Written:** 2026-05-14

## UAT Type

- UAT mode: artifact-driven
- Why this mode is sufficient: This is a purely lexical refactor with no runtime behaviour change. The success criteria are statically observable: compile-time success plus a grep that must return no matches. No new signals, sinks, or runtime paths were introduced.

## Preconditions

- Rust toolchain present (see rust-toolchain.toml)
- Working directory: /home/fabio/dev/bevyrogue
- No uncommitted source changes outside this slice

## Smoke Test

Run `cargo check` and confirm exit 0; then run `rg "combat::twin_core|combat::holy_support|combat::predator_loop" src tests` and confirm no output (exit 1).

## Test Cases

### 1. Headless compile check

1. Run `cargo check`
2. **Expected:** Exits 0 with no new errors. Pre-existing warnings (unused_mut, dead_code, unused_imports on re-export entries) are acceptable.

### 2. Windowed compile check

1. Run `cargo check --features windowed`
2. **Expected:** Exits 0 with no new errors.

### 3. Full integration test suite

1. Run `cargo test`
2. **Expected:** All test suites pass, 0 failures. Baseline count ≥67 tests (actual: 500+).

### 4. Zero legacy shim references

1. Run `rg -n "combat::twin_core|combat::holy_support|combat::predator_loop" src tests`
2. **Expected:** No output, exit code 1 (no matches). Any match is a failure.

## Edge Cases

### Windowed-feature-gated code still compiles

1. Run `cargo check --features windowed`
2. **Expected:** Exit 0. This confirms ui/ code that may import blueprint types via the crate path also works correctly.

### identity:: sub-path not exposed externally

1. Run `rg -n "blueprints::agumon::identity|blueprints::patamon::identity|blueprints::dorumon::identity" src tests`
2. **Expected:** No matches in call-sites (only inside the blueprint mod.rs re-exports themselves, which is internal).

## Failure Signals

- Any `cargo check` compile error referencing twin_core, holy_support, or predator_loop indicates an unpatched call-site.
- Any `cargo test` failure in blueprint-seam or twin_core_* / holy_support_* / predator_* test suites indicates a bad path substitution.
- `rg` returning matches indicates shim aliases were not fully removed or call-sites still reference the old path.

## Not Proven By This UAT

- Runtime behaviour of the blueprint state machines (unchanged by this refactor; covered by pre-existing integration tests).
- Performance or memory impact (none expected; purely lexical change).
- Any M021 integration that may consume these canonical paths (deferred to M021 scope).

## Notes for Tester

The unused-imports warnings on the blueprint mod.rs files (TAG_* constants, some transition functions) are intentional: the re-exports are prophylactic to make the full canonical surface available even if not all symbols are currently consumed by call-sites. These warnings do not indicate a problem.
