---
phase: M002
phase_name: First on-screen combat (Agumon-only)
project: bevyrogue
generated: 2026-05-22T09:00:00Z
counts:
  decisions: 6
  lessons: 3
  patterns: 8
  surprises: 3
missing_artifacts: []
---

# M002 Learnings

### Decisions

- **D-L01: Closed typed AnimGraphRole/AnimGraphInput as R009 contract, preserving legacy default-input wrappers as shims.**
  AnimGraph evaluation is exposed through a closed `AnimGraphRole` enum and a read-only `AnimGraphInput` set; the legacy player entrypoints are thin default-input wrappers rather than opening world-read or mutable-context paths. Stringly-typed and unknown roles are rejected at the seam boundary.
  Source: S08-SUMMARY.md/Key Decisions

- **D-L02: Cue timeout recovery force-resumes through the same released-runner path as normal cue completion.**
  The cue barrier has a bounded 180-frame timeout that force-resumes through the standard release path rather than a special error branch, so headless authority is never corrupted by timeout recovery. Structured post-timeout diagnostic state (cast, skill, timeline, beat, cue, hop, animation) is retained after resume.
  Source: S08-SUMMARY.md/Key Decisions

- **D-L03: Hot reload for presentation assets uses next-spawn snapshot binding, not in-flight mutation.**
  Animation players bind cloned resolved-graph snapshots at spawn/resolve time; registry updates from hot reload affect only future spawns, not in-flight players. This prevents mid-cast state corruption at the cost of requiring a new spawn to see the reloaded asset.
  Source: S08-SUMMARY.md/Key Decisions

- **D-L04: Energy-backed ult gauge keeps metadata-free Digimon on the legacy UltimateCharge path.**
  Only Digimon with `ult_gauge=energy` metadata opt into the Energy-backed ult readiness and drain path; all others continue using `UltimateCharge`. This avoids a big-bang roster migration and keeps the seam testable one Digimon at a time.
  Source: S07-SUMMARY.md/Key Decisions

- **D-L05: Bevy 15-tuple QueryData limit worked around by stitching additional components via a sibling read-only query.**
  `UltGaugeMetadata` was added to `UnitQuerySnapshot` through a separate sibling query rather than widening `ResolveActorsQuery`, avoiding the Bevy compile error at 16+ components. This is the canonical pattern for adding optional snapshot data without hitting the tuple limit.
  Source: S07-SUMMARY.md/Key Decisions

- **D-L06: One shared snapshot helper for ult readiness and resource reporting prevents availability/display value drift.**
  Both the legality check (is Ultimate castable?) and the resource display (current/max values shown in HUD) are computed by the same helper reading the same snapshot fields. This eliminates the class of bugs where the UI shows a ready bar but the action is blocked (or vice versa).
  Source: S07-SUMMARY.md/Key Decisions

---

### Lessons

- **L-L01: Bevy's 15-tuple QueryData compile limit is a hard constraint; sibling queries are the escape hatch.**
  Adding any component beyond the 15th directly to a wide Bevy query causes a compile error. The fix is a sibling read-only query that's joined in Rust code rather than in the ECS query itself. Plan for this when adding optional per-entity snapshot data.
  Source: S07-SUMMARY.md/What Happened

- **L-L02: CombatEvent read-only constraint must be enforced by a structural test, not just by convention.**
  The D008 "UI observes, never mutates" rule is only safe if a structural test proves the UI code path cannot mutate `CombatState`. Convention alone is not auditable across session boundaries; a test that fails on any mutation import in the UI module is the minimum enforcement.
  Source: S03-SUMMARY.md/What Happened

- **L-L03: Failure-visibility paths must leave inspectable structured state, not just transient console output.**
  After cue timeout force-resume and after missing-graph fallback, the runtime retains queryable structured diagnostics (cast context, skill id, fallback source) so automated tests can assert on what happened. Silent swallows and log-only errors are not verifiable in headless CI.
  Source: S08-SUMMARY.md/What Happened

---

### Patterns

- **P-L01: Closed typed input lenses as the preferred AnimGraph evaluation seam.**
  `AnimGraphRole` (closed enum) + `AnimGraphInput` (read-only set) form a pure-function seam: graph evaluation is deterministic given these inputs without any world-global read or mutable context. Legacy player entrypoints become thin default-input wrappers. Apply this pattern to all future graph types.
  Source: S08-SUMMARY.md/Patterns Established

- **P-L02: Failure-visibility paths should leave inspectable structured state after recovery.**
  Cue barrier timeout retains `CueBarrierStatus` fields (cast, skill, timeline, beat, cue, hop, animation); registry fallback retains `AnimationGraphLookupDiagnostics`; boot failures surface `AnimationGraphLoadState`. Tests assert on these fields. Do not rely on console output as the only observable.
  Source: S08-SUMMARY.md/Patterns Established

- **P-L03: Presentation hot reload should apply at next spawn via snapshot binding, not mid-flight.**
  When a presentation asset (animation graph, VFX config) is hot-reloaded, the new version should take effect only for players/entities spawned after the reload, not for those already in flight. Bind a cloned snapshot at resolution time and update only the registry state for future resolutions.
  Source: S08-SUMMARY.md/Patterns Established

- **P-L04: Energy-backed combat resources exposed as optional snapshot data + compatibility scalars, consumed by a shared helper.**
  `UnitQuerySnapshot` carries optional `UltGaugeMetadata` plus `Energy` as supplementary fields alongside the legacy `UltimateCharge` scalar. A shared helper (not inline legality logic) reads the right fields based on metadata presence. This pattern is reusable for future alternate gauges (MP, heat, momentum).
  Source: S07-SUMMARY.md/Patterns Established

- **P-L05: Runtime finalize seams that own per-cast resource effects must each honor the resource drain for opted-in actors.**
  Every finalize seam in the turn pipeline that applies `UltEffect::Reset` must check for energy-backed actors and drain `Energy.current` in addition to `UltimateCharge.current`. A single central post-pass that assumes one gauge is insufficient once multiple gauge types coexist.
  Source: S07-SUMMARY.md/Patterns Established

- **P-L06: Boundary map rows should cite actual on-disk test function names, making each contract machine-checkable.**
  `M002-BOUNDARY-MAP.md` rows identify the enforcing test by function name (extracted by grep), not just file path. A one-liner verification script checks all cited names resolve on disk and all five row keywords appear in the map. Apply this for all future boundary maps.
  Source: S09-SUMMARY.md/What Happened

- **P-L07: VFX seam uses opaque `ParticleId(String)` + closed `VfxLocus`/`VfxMotion` enums; no gameplay payload leaks through the serialization seam.**
  `SpawnParticle` round-trips through RON without any numeric gameplay value (damage, hit count, loop budget) appearing in the serialized form. Unknown locus/motion variants fail to deserialize (closed-enum guarantee). This is the extensibility contract that lets a future RON VFX pipeline replace Rust-configured entities without touching the anim graph or kernel.
  Source: S09-SUMMARY.md/What Happened

- **P-L08: CombatEvent-driven phase strip (§9) with a structural test proving no mutation is the pattern for all game UI.**
  The §9 phase strip derives all display state from `EventReader<CombatEvent>` only; a structural test proves the UI code path has no write access to `CombatState`. This is the operationalized form of D008 and should be applied to every new UI panel reading kernel state.
  Source: S03-SUMMARY.md/What Happened

---

### Surprises

- **SUR-01: S07 T05 regression surfaced a stale roster contract, not a generic snapshot shape issue.**
  The initial T05 hypothesis was that the remaining regression was a snapshot-fixture shape mismatch. The actual root cause was that `holy_support_roster_contract.rs` still expected Agumon's `ult_gauge_metadata` to be empty (pre-migration state). Fixing required updating the contract to accept `ult_gauge=energy` for Agumon while explicitly keeping Gabumon as the metadata-free legacy control.
  Source: S07-SUMMARY.md/Deviations

- **SUR-02: Reviewer subagent was unavailable during S08 closeout due to environment usage limits.**
  The S08 closeout attempted a fresh-context reviewer subagent dispatch but the agent was blocked by environment usage limits. The slice relied on direct gsd_exec verification evidence and task summaries instead. This is a process risk for high-stakes slices that benefit from independent review.
  Source: S08-SUMMARY.md/Known Limitations

- **SUR-03: K001 — auto-mode cannot launch the windowed binary; live soak frame-time data must be collected manually.**
  D027 threshold math is proven headlessly (10 unit tests + 2 windowed_only tests), but the full-vs-baseline frame-time comparison requires a live `cargo run --features windowed` session with a display. Auto-mode cannot satisfy this requirement; `frame-time-comparison.md` documents the pending results table and the manual capture commands.
  Source: S09-SUMMARY.md/Known Limitations
