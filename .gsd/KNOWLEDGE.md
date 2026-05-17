# Project Knowledge

Append-only register of project-specific rules, patterns, and lessons learned.
Agents read this before every unit. Add entries when you discover something worth remembering.
## Rules

| # | Scope | Rule | Why | Added |
|---|-------|------|-----|-------|
| R001 | Passive runtime bootstrap | Canonical passive listeners for each Digimon are bootstrapped from `CombatPlugin` after core resources are initialized, using fixed canonical `UnitId` owners and shared `kernel/ult_used` passive triggers with per-blueprint guard keys. | This avoids per-test scaffolding and keeps passive wiring declarative at plugin boot. | 2026-05-17 |

## Patterns

| # | Pattern | Where | Notes |
|---|---------|-------|-------|
| P001 | Keep combat dispatch metadata on `InFlightAction`, not `ResolvedAction`. | `src/combat/turn_system/` | `ResolvedAction` stays semantic; routing details for the pipeline live on the in-flight execution wrapper. |
| P002 | Reuse the same compiled timeline resolution/interner path for preview and execute. | `src/combat/preview.rs`, timeline-backed consumers | Preview should run `BeatRunner` in preview mode and return the pending queue without touching `intent_applier`, so preview/execution drift is caught early. |
| P003 | Use owner-gated generic Blueprint envelopes while preserving typed owner-side observability seams. | Blueprint runtime modules + observability surfaces | Raw transport can stay generic without breaking downstream assertions if the owner module preserves the typed resolved-state contract. |
| P004 | Apply damage modifiers in canonical order: Intrinsic → Status → Buff → Passive. | Damage modifier ledger / incoming-damage pipeline | Fixed fold order keeps layered mitigation deterministic and replayable regardless of insertion order. |

## Lessons Learned

| # | What Happened | Root Cause | Fix | Scope |
|---|--------------|------------|-----|-------|
| L001 | Passive listeners that enqueue state changes for later predicates can loop forever or read stale state. | Queued intents were not flushed between `BeatRunner` steps, so later predicates observed old state. | Flush queued intents between passive-runner steps; for outer passive timelines, stop when the cursor cycles back to the entry beat while keeping explicit `BeatKind::Loop` bodies on the normal 256-hop breaker. | Passive timelines / follow-up runtime |
| L002 | Pre-damage events like `IncomingDamage` did not affect the current hit even when reactions subscribed to them. | `IncomingDamage` is observational only; modifiers were being armed too late. | Arm target-scoped modifiers in state before the damage intent is processed, then let the damage applier drain the ledger and emit the post-mitigation trigger once. | Incoming-damage / mitigation pipeline |
