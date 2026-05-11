---
phase: M012
phase_name: Data-driven skill legality and UI-readiness query surface
project: bevyrogue
generated: 2026-05-01T17:30:00Z
counts:
  decisions: 6
  lessons: 6
  patterns: 5
  surprises: 2
missing_artifacts: []
---

### Decisions

- **D053 — DSL-first legality contract.** Extended SkillDef DSL with targeting/legality metadata and exposed a pure query API (`query_action_affordance()`) shared by engine, CLI, windowed UI, and tests. Alternative of a detached hardcoded UI layer was explicitly rejected.
  Source: S03-SUMMARY.md/Key decisions; S04-SUMMARY.md/Key decisions; DECISIONS.md/D053

- **D054 — Scope boundary: fix vs. declare.** M012 fixes UI-blocking semantics directly (Toughness, TargetShape, Energy caps) and adds declarative queryable placeholders for larger future systems (Tamer Commands, enemy counterplay) rather than implementing them fully.
  Source: DECISIONS.md/D054; M012-ROADMAP.md/Boundary Map

- **D055 — Team-aware Toughness helpers over structural removal.** Enemy-only Toughness affordances gated via `is_enemy_toughness_visible` / `apply_toughness_damage_if_enemy` helpers rather than removing Toughness from allies, because Toughness also carries weakness tags used by damage affinity classification.
  Source: S02-SUMMARY.md/Key decisions; DECISIONS.md/D055

- **D056 — TargetHpRule in DSL for revive/heal-like targeting.** Added `TargetHpRule` field to `SkillTargeting` so damaged-target and KO-target legality is represented in data and queryable without skill-ID inference.
  Source: S04-SUMMARY.md/Key decisions; DECISIONS.md/D056

- **D057 — Separate SP snapshot paths for engine vs. UI/CLI.** Engine validation bypasses SP in the early guard to preserve existing SP-shortfall lifecycle behavior in `step_app()`; a separate explicit-SP path makes CLI/windowed affordances truthful before execution.
  Source: S06-SUMMARY.md/Known limitations; DECISIONS.md/D057

- **D058 — Typed EnemyCounterplayKind enum, never inferred from ToughnessCategory.** Enemy counterplay declarations (TempoAnchor, TypeTrap, ReactiveArmor, BreakSeal) live in UnitDef as typed data propagated through ECS. ReactiveArmor is never inferred from ToughnessCategory::Armored.
  Source: S08-SUMMARY.md/Key decisions; DECISIONS.md/D058

### Lessons

- **Lock vocabulary before implementation slices.** Writing `skill_legality_contract.md` and the gap matrix in S01 — before any DSL or query work — prevented vocabulary drift across 8 implementation slices. Doc-contract tests using `include_str!` enforced this at compile-time.
  Source: S01-SUMMARY.md/Patterns established

- **One legality source of truth eliminates engine/UI divergence by construction.** When S06 wired engine validation to the S04 pure query API, it removed an entire class of behavioral drift without extra test coverage — the parity is structural.
  Source: S06-SUMMARY.md/Key decisions

- **Expanding ECS actor query tuples has non-local blast radius.** Adding `EnemyCounterplayKit` to the turn system query tuple required updating every independent `Query` signature — `follow_up.rs` and `combat_cli.rs` are not covered by the turn_system alias and were caught only by compile errors.
  Source: S08-SUMMARY.md/Patterns established

- **Additive annotation sections preserve string-grep contracts under test coverage.** When adding data-alignment notes to `combat_design.md` in S09, placing them in a new additive section prevented breaking existing doc-contract tests that assert specific substrings.
  Source: S09-SUMMARY.md/Patterns established; S09-SUMMARY.md/Key decisions

- **Separate ECS snapshot paths for preflight vs. runtime validation.** The S06 design (read-only snapshot adapter + transient buffer) avoided borrow conflicts with the mutable action pipeline while keeping the query surface unchanged — a reusable pattern for future read-from-ECS-without-mutating needs.
  Source: S06-SUMMARY.md/Key decisions

- **Source-scan tests are a low-cost, high-value contract guard.** Regex assertions over consumer source files (`action_affordance_consumers.rs`) prevent skill-ID / team-hardcoded legality paths from returning without requiring exhaustive behavioral tests for every violation.
  Source: S07-SUMMARY.md/Key decisions; S08-SUMMARY.md/Patterns established

### Patterns

- **Doc-contract tests with `include_str!`.** Use `include_str!` in integration tests to assert presence of specific named substrings in contract documents. Compile-time detection, zero runtime cost, and failures identify exactly which mechanic or vocabulary item is missing.
  Source: S01-SUMMARY.md/Patterns established

- **DSL-backed legality queried from immutable snapshots, not inferred from ECS mutation.** The pure query API (`query_action_affordance`) takes a `WorldSnapshot` value and returns affordances deterministically — no side effects, cacheable, testable without a running Bevy app.
  Source: S04-SUMMARY.md/Patterns established

- **Consumer adapters are thin filters, not legality engines.** CLI and windowed adapters may display and filter affordances but must not decide legality. The query surface is the only place rules live. Source-scan tests enforce this boundary automatically.
  Source: S07-SUMMARY.md/Patterns established

- **Typed declarations over free-text traits for UI-queryable enemy behavior.** `EnemyCounterplayKind` enum instead of `signature_traits: Vec<String>` — UI can branch on enum variants, tests can enumerate them exhaustively, and no consumer needs string matching.
  Source: S08-SUMMARY.md/Key decisions

- **Stable machine-readable reason codes separate from display strings.** `LegalityReasonCode` identifiers are shared by data, engine, query, tests, and future UI — display strings are a projection layer, not the contract. This prevents UI from coupling to prose messages that may change for localization.
  Source: S01-SUMMARY.md/Key decisions; S03-SUMMARY.md/Patterns established

### Surprises

- **`cargo test-dev` profile was undefined at milestone completion.** The M012 ROADMAP listed `cargo test-dev` as the verification command, but the profile was removed or never created. Fell back to `cargo test` (default profile), which ran the full integration suite successfully. Final verification used `cargo test`.
  Source: milestone verification run / 2026-05-01

- **S04 had pre-existing `form_identity.rs` regressions unrelated to its scope.** The slice noted these as known limitations and verified only its focused test set. S05 subsequently fixed the Form Identity self-targeting and DORUgamon follow-up targeting regressions as part of live-pipeline energy cap work.
  Source: S04-SUMMARY.md/Known limitations; S05-SUMMARY.md/What Happened
