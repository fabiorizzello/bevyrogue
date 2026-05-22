# S09: Remediate validation evidence and operational closeout — UAT

**Milestone:** M002
**Written:** 2026-05-22T08:32:24.809Z

# S09 UAT: M002 Operational Closeout

## Preconditions
- Rust toolchain installed, repo at commit including all S09 changes
- No display required for headless checks (steps 1–7); display required only for step 8 (manual, deferred)

## Test Steps

### Headless (automated)

1. **Frame-time unit proof**
   ```
   cargo test --lib frame_time
   ```
   Expected: 10 passed, 0 failed. Covers empty accumulator (no panic, zeroed stats), single sample, p95 with unsorted series, pass/fail regression verdicts, 2ms absolute tolerance edge case, baseline-toggle string mapping, and `validation_frametime:` format prefix.

2. **Frame-time soak wiring (integration)**
   ```
   cargo test --features windowed --test windowed_only frame_time
   ```
   Expected: 2 passed, 0 failed. Asserts baseline-toggle truthy/falsey/garbage string mapping and that a known delta series through the soak's `FrameTimeAccumulator` yields the expected count/mean/p95/max/min and structured `validation_frametime:` line for both `full` and `baseline` modes.

3. **Skill-graph mapping extensibility proof**
   ```
   cargo test --test animation skill_graph_mapping_extensibility
   ```
   Expected: 3 passed, 0 failed. Proves 1:1 non-default skill-graph resolution, InstantFallback for unregistered ids with diagnostic, and non-empty stance-graph `entry` for `return_to_idle` boundary.

4. **VFX handle seam proof**
   ```
   cargo test --test animation vfx_handle_seam
   ```
   Expected: 4 passed, 0 failed. Proves lossless RON round-trip, closed-enum rejection for unknown VfxLocus and VfxMotion variants, and absence of numeric gameplay payload.

5. **Regression guard**
   ```
   cargo test --features windowed --test animation --test timeline --test windowed_only
   ```
   Expected: all tests pass, exit 0. No regressions in animation, timeline, or windowed_only suites.

6. **clip_atlas_parity (R003)**
   ```
   cargo test --test animation clip_atlas_parity
   ```
   Expected: 2 passed (agumon + renamon geometry parity).

7. **Boundary map machine-check**
   ```
   bash -c 'set -e; M=.gsd/milestones/M002/slices/S09/M002-BOUNDARY-MAP.md; C=.gsd/milestones/M002/slices/S09/S09-CLOSEOUT.md; test -f "$M"; test -f "$C"; for t in tests/timeline/boundary_contract.rs tests/windowed_only/phase_strip_readonly.rs tests/preview_ai/presentation_metadata_boundary.rs tests/animation/skill_graph_mapping_extensibility.rs tests/animation/vfx_handle_seam.rs; do test -f "$t" || exit 1; grep -qF "$t" "$M" || exit 1; done; for kw in kernel timeline anim-graph cue CombatEvent SkillGraphRegistry ParticleId; do grep -qiF "$kw" "$M" || exit 1; done; echo BOUNDARY_MAP_AND_CLOSEOUT_OK'
   ```
   Expected: prints `BOUNDARY_MAP_AND_CLOSEOUT_OK`, exit 0.

### Manual (display-required, deferred)

8. **Live windowed soak frame-time comparison (per K001)**
   ```
   # Full run (with anim-graph + render):
   cargo run --features windowed 2>&1 | grep validation_frametime
   # Kernel-only baseline:
   BEVYROGUE_VALIDATION_BASELINE=1 cargo run --features windowed 2>&1 | grep validation_frametime
   ```
   Expected: both emit `validation_frametime: count=N mean_ms=X p95_ms=Y max_ms=Z min_ms=W mode=full|baseline`. D027 pass bar: mean regression ≤15%, p95 regression ≤20%, absolute mean delta ≤2ms. Fill pending results table in `frame-time-comparison.md`.

## Edge Cases Covered

- Empty frame-time accumulator → zeroed stats, no panic (T01 unit)
- Unknown VfxLocus/VfxMotion variant → deserialization error, not panic (T04)
- Unregistered skill-id → InstantFallback + diagnostic, not panic (T03)
- Repeated hits same frame in hurt countdown → no underflow (windowed_target_hurt suite)

## UAT Type

Mixed: unit (pure frame-time math), integration (headless animation/timeline/windowed_only contracts), documentation review (boundary map machine-check), and deferred manual windowed soak (display-dependent).

## Not Proven By This UAT

- Actual measured live frame-time numbers (windowed soak display-dependent; D027 threshold math proven headlessly)
- End-to-end VFX rendering correctness (consumer is validate-only in M002)
- Data-driven SkillGraphRegistry wiring (hardcoded constants remain; M003+ lift, boundary enforced by T03)
- Hot-reload correctness under the new soak path

