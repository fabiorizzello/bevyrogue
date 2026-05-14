---
phase: M020
phase_name: Reactive bus uniforme + shim removal
project: bevyrogue
generated: "2026-05-14T11:00:00Z"
counts:
  decisions: 4
  lessons: 2
  patterns: 3
  surprises: 1
missing_artifacts: []
---

# M020 Learnings

### Decisions

- **UltimateUsed emitted symmetrically in all 4 hoist blocks.** Chose to emit `CombatEventKind::UltimateUsed` at every `UltEffect::Reset` site in pipeline.rs (all 4 resource hoist blocks) using the same `source`/`target = attacker_id` shape as the peer `UltGain` event. Alternative of emitting only once at the outermost fan-out entry was rejected because it would miss casts on fan-out paths (Blast, AllEnemies, AllAllies). New event variants should mirror the emission shape of their structural peer.
  Source: S01-SUMMARY.md/Key Decisions

- **ko_payload() helper centralizes StatusBag snapshot at emission site.** Rather than duplicating the StatusBag snapshot logic inline at both KO emission sites or threading the bag through callers, extracted `ko_payload(status_bag) -> (Vec<StatusEffectKind>, u32)` in `resolution.rs`. This isolates the snapshot transform and makes the empty-payload case at the stun-damage site explicit by contrast.
  Source: S01-SUMMARY.md/Key Decisions

- **Blueprint mod.rs re-exports hide the identity sub-module.** When adding canonical re-exports to blueprint mod.rs files, used `pub use identity::{...}` rather than routing call-sites to `::identity::` sub-paths. Keeps the canonical consumer surface at `blueprints::<name>::<Type>` without exposing the internal module hierarchy.
  Source: S02-SUMMARY.md/Key Decisions

- **Compiler-driven refactor: remove alias first, let compiler enumerate call-sites.** For the shim removal, deleted the three `pub use` lines from `src/combat/mod.rs` first, then compiled to get an exhaustive error list of affected files, then fixed all of them in one pass. More reliable than grep-based enumeration because Rust's path resolution is context-sensitive and grep cannot distinguish a string match from an active import.
  Source: S02-SUMMARY.md/Patterns Established

### Lessons

- **stun-damage KO path emits UnitDied with empty payload.** The `mod.rs` stun-damage KO site has no `StatusBag` in scope at the point of KO emission. `UnitDied { status_remaining: vec![], heated_remaining: 0 }` is emitted with empty fields; this is documented with a comment. Downstream listeners relying on the payload for post-KO effects will not receive status snapshot data from stun-triggered kills. Fix: if a future milestone needs the full payload on this path, either thread `StatusBag` into the stun path or emit a second event after stun resolution.
  Source: S01-SUMMARY.md/Known Limitations

- **Grep-based call-site enumeration undercounts in large test suites.** The S02 plan expected 9 affected test files; compiler-driven discovery found 11 (validation_snapshot.rs, status_observability_canon.rs, and presentation_metadata_boundary.rs were missed by the initial grep). For future shim/alias removals, always prefer the compiler error list over grep counts for Rust.
  Source: S02-SUMMARY.md/Deviations

### Patterns

- **Emit new event variants symmetrically with their structural peers.** When adding a new event at a resource-consumption site (e.g. `UltEffect::Reset`), emit it at every branch that performs that action — not just the outermost entry — using the same `source`/`target` field shape as the peer event on the same resource (UltGain ↔ UltimateUsed). Ensures all fan-out paths are covered and event consumers can rely on parity.
  Source: S01-SUMMARY.md/Patterns Established

- **ko_payload() helper pattern: centralize payload extraction at the emission site.** When a KO event needs to carry a snapshot of per-unit state (StatusBag, other components), extract a small helper function in the same module that takes the component reference and returns the payload tuple. Avoids threading the component deep into callers and makes empty-payload edge cases explicit by contrast with the normal path.
  Source: S01-SUMMARY.md/Patterns Established

- **Compiler-driven refactor is the canonical approach for alias/shim removal in Rust.** Remove the public alias first, compile, collect all errors, fix all call-sites in one pass. Do not rely on grep — Rust's path resolution is context-sensitive and grep cannot distinguish an active import from a string literal. This pattern was confirmed effective: 100% of affected files were found and fixed.
  Source: S02-SUMMARY.md/Patterns Established

### Surprises

- **S02 found 2 more affected test files than the grep-estimated 9.** The plan listed 9 test files; compiler errors surfaced 11. The extra files (validation_snapshot.rs, status_observability_canon.rs, presentation_metadata_boundary.rs) imported shim types via paths that the initial grep missed due to multi-line use declarations or intermediate re-exports. No functional impact — all were fixed atomically — but the enumeration gap validates the compiler-driven approach preference.
  Source: S02-SUMMARY.md/Deviations
