# Plan — W0c Tier-B inline `mod tests` relocate

Draft. Status: pending (Phase 1).
Parent: `260520-1-reduce-loc-tests` SUMMARY backlog #1.

## Wave breakdown

One atomic commit per source file (Tier-A precedent: `aa22a1f`, `a3b53aa`, `ff6e069`, `ba3c9f1`). Wave naming continues the Tier-A sequence.

| Wave | Source | Target | Gate | Commit format |
|---|---|---|---|---|
| W0c-5 | `src/combat/runtime/registry.rs` (45 LOC) | `tests/registry_internals.rs` | — | `refactor(tests): W0c-5 — relocate registry inline tests` |
| W0c-6 | `src/combat/state.rs` (43 LOC) | `tests/combat_state_internals.rs` | — | `refactor(tests): W0c-6 — relocate combat state inline tests` |
| W0c-7 | `src/windowed/render.rs` (43 LOC) | **SKIPPED** — binary-private | `#[cfg(feature = "windowed")]` | n/a (see "Skipped waves") |
| W0c-8 | `src/combat/mechanics/sp.rs` (40 LOC) | `tests/sp_mechanics_internals.rs` | — | `refactor(tests): W0c-8 — relocate sp mechanics inline tests` |
| W0c-9 | `src/combat/runtime/event_filter.rs` (38 LOC) | `tests/event_filter_internals.rs` | — | `refactor(tests): W0c-9 — relocate event filter inline tests` |
| W0c-10 | `src/combat/mechanics/modifiers.rs` (34 LOC) | `tests/modifiers_internals.rs` | — | `refactor(tests): W0c-10 — relocate modifiers inline tests` |
| W0c-11 | `src/windowed/mod.rs` (31 LOC) | **SKIPPED** — binary-private | `#[cfg(feature = "windowed")]` | n/a (see "Skipped waves") |

## Skipped waves

W0c-7 and W0c-11 — `src/windowed/` is part of the **binary crate** `bevyrogue` (`src/main.rs:6 mod windowed;`), not the library crate. Their inline `mod tests` blocks reach `pub(super)`/private symbols of that binary. Integration tests in `tests/` only see the library's public API; they cannot import from `windowed::`. Relocating these would require either:
  - promoting `windowed` to a `pub mod` of the library (architectural change, out of scope for a R003 LOC refactor), or
  - moving the tested helpers themselves into the library (also architectural).

Both inline blocks are <50 LOC each (well below the R003 100 LOC hard cap). They stay in place; surface again only when an architectural slice splits the binary/library boundary.

## Per-wave procedure

For each wave:

1. **Read** the inline `mod tests` block end-to-end.
2. **Classify** each test (decision matrix from INVENTORY § "Reference pattern"):
   - pure relocate (everything `pub`/`pub(crate)`)
   - promote-and-relocate (one or two private symbols cleanly liftable to `pub(crate)`)
   - rewrite-as-integration (private invariant — do NOT widen visibility)
   - delete (if integration coverage already cites it — must produce `file:line` proof in commit body)
3. **Move** the block via the appropriate operation (file delete + `tests/<name>_internals.rs` create; or rewrite).
4. **Add feature gate** when source was windowed-gated.
5. **Verify** before commit:
   - `cargo test --test <name>_internals` for the new file
   - `cargo check --tests` (or `--features windowed`) on changed files
   - For windowed waves: `cargo build --features windowed` and `cargo test --features windowed --test <name>_internals`
6. **Commit** atomically. One file change per commit (source delete + new test file).

## Stop conditions

Halt the wave and downgrade to "skip with note" if any of:

- A test asserts on a value path that has no public projection AND widening visibility would expose internal state across `pub(crate)` boundaries the kernel doesn't currently expose.
- The test is structurally a duplicate of an existing integration assertion — record `file:line` of the duplicate in the commit body and pure-delete.
- The source file's inline block contains a helper macro or non-test private fn the rest of the file relies on — relocate only the `mod tests` body, leave shared helpers in place.

## Final verify (Phase 3)

After all 7 waves committed:

```
cargo test --tests                    # all binaries green
cargo test --features windowed --tests
cargo check --tests
cargo check --features windowed --tests
scripts/check_loc_cap.sh              # still 0 offenders
```

Then re-scan `src/` for inline `#[cfg(test)] mod tests` blocks ≥30 LOC. Expected result: only Tier-C residue (~159 LOC across 6 files) and intentional outliers (e.g. `status_slowed_delay`, annotated W7c).

## Out of scope

See INVENTORY § "Out of scope". Tier-C and architectural slices are NOT part of this workflow.

## Risk

Low. All targets are <50 LOC inline blocks, the pattern is proven by Tier-A (4 commits, all green). The main novel surface is the 2 windowed-gated files — must remember `#[cfg(feature = "windowed")]` on the relocated test file or `cargo test --tests` (no features) will pick it up and fail to compile windowed-only imports.
