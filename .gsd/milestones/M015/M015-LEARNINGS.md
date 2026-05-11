---
phase: M015
phase_name: M013 Closure and Combat Architecture Coherence
project: bevyrogue
generated: 2026-05-08T00:00:00Z
counts:
  decisions: 7
  lessons: 6
  patterns: 4
  surprises: 2
missing_artifacts:
  - M013 closure artifacts remain historically incomplete and are superseded by M015 proof
---

# M015 Learnings

### Decisions
- Normalize clear combat drift, but split rewrite-scale work into a follow-up milestone rather than widening M015 into a broad rewrite.
  Source: DECISIONS.md/D010
- Keep per-Digimon Rust blueprint modules as the identity layer; shared mechanic modules remain primitives, not the primary ownership abstraction.
  Source: DECISIONS.md/D011
- Keep RON declarative for numbers, metadata, presentation triggers, and custom-signal declarations, not gameplay authority.
  Source: DECISIONS.md/D012
- Treat `CombatKernelTransition` as canonical observable/mutation output after blueprint resolution, not the source of unique Digimon behavior.
  Source: DECISIONS.md/D013
- Treat presentation triggers, beat metadata, and animation notifies as non-authoritative cues; gameplay authority stays in combat/blueprint/kernel logic.
  Source: DECISIONS.md/D014
- Supersede missing or contradictory M013 closure evidence through M015 artifacts instead of rewriting history in place.
  Source: DECISIONS.md/D017
- Rewrite stale tests to current source-of-truth contracts; do not restore removed APIs just to get green tests.
  Source: DECISIONS.md/D018

### Lessons
- Classify stale or obsolete tests before touching code; otherwise a repair pass can quietly reanimate dead contracts.
  Source: S01-SUMMARY.md/What Happened
- The real CLI proof must run the shared combat surfaces end-to-end; a smoke wrapper is not enough.
  Source: S05-SUMMARY.md/What Happened
- Seed blueprint seams should emit generic kernel transitions from typed per-Digimon logic, not branch ladders inside shared kernel code.
  Source: S03-SUMMARY.md/What Happened
- Presentation metadata can be rich, but legality and outcomes must remain in combat/blueprint/kernel logic.
  Source: S04-SUMMARY.md/What Happened
- Closure documentation should preserve follow-up caveats even when the milestone is green, so future readers do not overread the proof.
  Source: M015-VALIDATION.md/Cross-Slice Integration
- A deterministic verifier plus a failure ledger gives later slices a stable truth baseline without redoing research.
  Source: S01-SUMMARY.md/Verification

### Patterns
- Use a failure ledger plus deterministic verifier to classify blockers before repair, so later slices can reuse the truth set without rediscovering it.
  Source: S01-SUMMARY.md/What Happened
- Use an authority map plus drift ledger to pin down single-source-of-truth boundaries across data, runtime, kernel, presentation, snapshots, CLI, and tests.
  Source: S02-SUMMARY.md/What Happened
- Seed per-Digimon seams as typed custom signals flowing through a Rust blueprint module into generic kernel transitions, keeping RON declarative.
  Source: S03-SUMMARY.md/What Happened
- Prove shared combat architecture with the real CLI against action query, events, beats, kernel observability, and validation snapshots.
  Source: S05-SUMMARY.md/What Happened

### Surprises
- The validation pass found the S01→S02 handoff explicitness gap even though the slice chain itself was functionally coherent.
  Source: M015-VALIDATION.md/Cross-Slice Integration
- The closure packaging also had no downstream consumer summary available, so the final truth package had to rely on the artifact chain rather than a direct consumer proof.
  Source: M015-VALIDATION.md/Cross-Slice Integration
