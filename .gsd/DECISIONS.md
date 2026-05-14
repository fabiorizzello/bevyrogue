# Decisions Register

<!-- Append-only. Never edit or remove existing rows.
     To reverse a decision, add a new row that supersedes it.
     Read this file at the start of any planning or research phase. -->

| # | When | Scope | Decision | Choice | Rationale | Revisable? | Made By |
|---|------|-------|----------|--------|-----------|------------|---------|
| D001 | Slice M019/S04 planning — closes follow-up #3 from M018 (PerHop length guard) | kernel-combat | DamageCurve::PerHop runtime behaviour when curve length is less than hops_planned | Truncate the bounce loop to v.len() and emit a CombatEventKind::OnActionFailed diagnostic event before the loop. Kernel never panics. Load-time validator in skills_ron.rs remains the primary defence; the runtime guard exists for dynamically constructed ResolvedActions (future blueprint emitters, tests) that bypass it. | Silent clamp masks bugs in blueprint emitters M020+ will introduce. Truncation preserves total resolution (no half-applied action, no panic, action bounces what it can) and matches the existing pool-exhaustion truncate pattern at pipeline.rs:844. Reusing OnActionFailed keeps event taxonomy stable and aligns with how other kernel anomalies are reported (SP shortfall, attacker stunned/KO). Diagnostic event makes the bug observable in JSONL traces — combat events are the single source of truth. | Yes; if a dedicated event variant proves more useful for tooling, the diagnostic can switch to OnDamageCurveTruncated without changing the truncate-with-event semantics. | agent |
