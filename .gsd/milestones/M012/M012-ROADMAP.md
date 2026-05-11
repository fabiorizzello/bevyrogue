# M012: Data-driven skill legality and UI-readiness query surface

**Vision:** Create a unified, data-driven action legality and UI-readiness query surface inside the existing combat DSL so player-facing UI can truthfully enable, disable, hide, or explain every combat action/resource/target before execution while the engine enforces the same rules as an authoritative safety net.

## Success Criteria

- R084 is validated: action legality and target validity are data-driven, queryable, and shared by engine/UI/CLI.
- R085 is validated: UI-affecting mechanics are either implemented truthfully or represented as queryable deferred/hidden affordances.
- Existing combat behavior remains green under `cargo test-dev`.
- Windowed path compiles after UI-affordance integration (`cargo check --features "dev windowed"`).
- Revive/Heal-like/Offensive examples demonstrate legal target filtering before execution and authoritative engine rejection after execution attempt.
- Enemy-only Toughness and TargetShape semantics no longer produce false UI claims.
- Energy gain caps are enforced in the actual pipeline.
- No UI or CLI code contains per-skill legality hardcoding.

## Slices

- [x] **S01: S01** `risk:high` `depends:[]`
  > After this: After this: a design/code gap matrix says exactly what M012 will fix, what it will declare as deferred, and what remains post-UI.

- [x] **S02: S02** `risk:high` `depends:[]`
  > After this: After this: ally Toughness no longer leaks as a break target/affordance, and Row/AllEnemies semantics cannot silently behave as single-target without the query reporting that limitation.

- [x] **S03: S03** `risk:high` `depends:[]`
  > After this: After this: canonical `skills.ron` contains targeting/legalities metadata, parses cleanly, and invalid sample skills fail loudly.

- [x] **S04: S04** `risk:high` `depends:[]`
  > After this: After this: tests can call one pure function and get action/target affordances with enabled/disabled/deferred state and reasons.

- [x] **S05: S05** `risk:medium` `depends:[]`
  > After this: After this: Energy caps are enforced in the real pipeline, and Tamer/Child command-resource dependencies are declared for UI even if full command execution is deferred.

- [x] **S06: S06** `risk:high` `depends:[]`
  > After this: After this: forcing an illegal ActionIntent directly into the Bevy message bus fails with the same reason the preflight query would have returned.

- [x] **S07: S07** `risk:medium` `depends:[]`
  > After this: After this: CLI/windowed-facing code asks the legality query for action/target affordances; revive can target KO allies in CLI/query tests without special-case UI code, and windowed compiles.

- [x] **S08: S08** `risk:medium` `depends:[]`
  > After this: After this: enemy traits and charged attacks have a queryable UI contract even if only Tempo Anchor/Break Seal execute today.

- [x] **S09: S09** `risk:medium` `depends:[]`
  > After this: After this: docs, data, and query examples agree on what shipped kits do; future mechanics can plug into legality without a new hardcoded path.

## Boundary Map

**In scope:** Extend `SkillDef`/RON with uniform targeting and legality metadata; implement pure legality query API; wire engine validation to use the query as final safety net; expose query results with machine-readable reasons for CLI/windowed UI; fix UI-blocking semantic mismatches: enemy-only Toughness, executable or explicitly-gated TargetShape, real Energy cap enforcement. Add queryable declarations for larger future UI-affecting systems: Tamer Gauge/Commands, Child gauge boost dependency, enemy traits/charged telegraphs, Heal/Cleanse/Silence/Guard affordances.

**Out of scope:** Full graphical UI polish, sprites/animation, DNA Chips implementation, full Tamer Command execution, complete enemy counterplay behavior suite, large numerical rebalance beyond legality/targeting correctness.

**Hard boundary:** No skill-ID-specific UI rules. If a rule is needed by UI, it must be represented in DSL/query output or rejected/deferred explicitly. UI must never present an ability/resource/telegraph as usable unless the query surface says it is implemented and legal.
