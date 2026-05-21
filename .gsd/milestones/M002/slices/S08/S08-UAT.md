# S08: Remediate graph purity and failure visibility — UAT

**Milestone:** M002
**Written:** 2026-05-21T22:15:22.785Z

# UAT Type
Executable regression and observability verification.

# Preconditions
1. Repository is at the S08 implementation state.
2. Rust toolchain and project test dependencies are installed.
3. Windowed feature dependencies build successfully in the local environment.

# Steps
1. Run `cargo test --test animation anim_graph_input_purity`.
2. Observe that the test binary passes the typed-input contract cases.
3. Run `cargo test --test timeline r013_failure_visibility`.
4. Observe that the cue-timeout and dead-target failure-visibility cases pass.
5. Run `cargo test --test animation anim_registry_failure_visibility`.
6. Observe that the missing-skill fallback, boot load-state visibility, and hot-reload-next-spawn cases pass.
7. Run `cargo test --features windowed --test animation --test timeline --test windowed_only`.
8. Observe that the combined animation, timeline, and windowed-only regression suites pass with no new failures.

# Expected Outcomes
1. The animation purity harness proves only closed typed roles are accepted and no stringly/custom world-read path is available.
2. The timeline failure-visibility harness proves a never-released cue times out after the bounded frame budget, leaves structured diagnostic state, and force-resumes rather than deadlocking.
3. The animation registry harness proves runtime missing skill-id lookup falls back deterministically with structured diagnostics, boot-time missing canonical assets remain inspectable, and hot reload updates only newly spawned/resolved players.
4. The dead-target-mid-loop case proves the same cast continues through presentation with observable post-KO overshoot rather than branching on target liveness.
5. The windowed regression sweep proves S08 does not regress prior S01-S07 animation, timeline, or windowed behavior.

# Edge Cases
1. Verify the cue-timeout path counts only frames after suspension is latched, so timeout observability is not off by one.
2. Verify the runtime fallback path remains deterministic even when the requested skill graph is missing.
3. Verify an in-flight player keeps its pre-reload graph snapshot while a newly resolved player sees the updated graph.
4. Verify the dead-target observability signal is visible both in combat events and ActionLog output.

# Not Proven By This UAT
1. Real-time human-observed on-screen feel of the timeout or reload behaviors in a manual windowed session.
2. Long-duration operational soak behavior beyond the automated regression suite.
3. S09 closeout artifacts such as the producer/consumer boundary map and operational evidence packaging.
