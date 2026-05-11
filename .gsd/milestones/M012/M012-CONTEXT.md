# M012: Data-driven skill legality and UI-readiness query surface

**Gathered:** 2026-04-30
**Status:** Ready for planning

## Project Description

`bevyrogue` is a headless-first Rust + Bevy 0.18 roguelite RPG combat prototype with Digimon-themed monster-taming. Combat is data-driven through RON unit/skill definitions and a Bevy ECS combat pipeline. The current M011 combat core compiles and passes headless tests, but several mechanics that a player-facing UI must display or disable are missing, only partially wired, or not represented as queryable state.

## Why This Milestone

The next major direction is graphical UI on top of the combat game. Before building that UI, the combat layer needs a truthful, data-driven affordance surface: the UI must be able to ask which actions are executable, which targets are legal, which resources are available, and which mechanics are implemented/deferred without duplicating combat rules or hardcoding skill IDs.

This milestone also resolves design-vs-code gaps that would make UI lie to the player: ally Toughness currently exists despite the design saying Toughness is enemy-only; `TargetShape` exists in RON but is ignored by resolution; Energy cap trackers exist but are not wired in the real gain path; Tamer Gauge/Commands, Child gauge boost, and enemy counterplay are design-visible but not queryable.

## User-Visible Outcome

### When this milestone is complete, the user can:

- Run the combat tests and know action legality/target validity are enforced by data-driven DSL rules rather than UI-specific hardcoding.
- Use CLI/windowed-facing affordance data where invalid actions/targets are disabled or explained before execution.
- Start the next graphical UI milestone without guessing whether a combat affordance is implemented, deferred, hidden, or illegal.

### Entry point / environment

- Entry point: `cargo test-dev`, `cargo run --bin combat_cli`, and `cargo check --features "dev windowed"`.
- Environment: local dev, headless first; windowed build must compile by the end of the milestone.
- Live dependencies involved: none beyond local RON assets and Bevy systems.

## Completion Class

- Contract complete means: the skill DSL and query API can represent current and near-future targeting/action constraints with machine-readable reason/status output.
- Integration complete means: engine validation, CLI affordance code, and windowed-facing affordance code use the same query surface rather than duplicated rules.
- Operational complete means: headless tests pass, windowed check passes, failed actions emit inspectable structured reasons, and UI cannot present unavailable mechanics as usable.

## Final Integrated Acceptance

To call this milestone complete, we must prove:

- Revive-like skills only target KO allies through the shared query and are rejected by the engine with the same reason when forced illegally.
- Damage/offensive skills only target valid live enemies unless their DSL says otherwise; future heal/cleanse/self examples are representable without skill-ID hardcoding.
- Enemy-only Toughness, `TargetShape`, Energy caps, Tamer/Command affordances, Child gauge dependency, and enemy trait/telegraph declarations are either implemented truthfully or exposed as deferred/hidden through query output.
- `cargo test-dev` passes and `cargo check --features "dev windowed"` passes before handing off to UI work.

## Architectural Decisions

### Skill legality lives in the existing DSL

**Decision:** Extend `SkillDef`/RON with first-class targeting and legality metadata, then expose a pure query API consumed by engine, CLI, UI, AI, and tests.

**Rationale:** The UI must know legality before execution, but the engine must remain authoritative. Putting target/action rules beside skill effects keeps behavior inspectable from RON and prevents UI-specific hardcoding.

**Alternatives Considered:**
- UI-only hardcoded rules — rejected because it would drift from engine behavior and fail for future mechanics.
- Separate legality registry — rejected because it creates a second DSL disconnected from skill definitions.

### M012 fixes UI-blocking semantics and declares larger mechanics

**Decision:** M012 directly fixes enemy-only Toughness, TargetShape truthfulness, and Energy cap wiring. Larger systems such as full Tamer Command execution and full enemy counterplay behavior are declared/queryable as implemented/deferred/hidden, but not fully implemented unless required by the UI-readiness contract.

**Rationale:** Some gaps make current UI false immediately and must be resolved before UI. Other gaps are large combat features; forcing all of them into M012 would derail the legality/query infrastructure. The query status surface lets UI stay truthful while future milestones implement the behavior.

**Alternatives Considered:**
- Implement every design feature before UI — rejected as too broad for this milestone.
- Ignore unimplemented design features until UI needs them — rejected because the UI would have no truthful way to hide or explain missing affordances.

## Error Handling Strategy

Illegal actions must fail before mutation with a machine-readable reason. The query layer returns reason/status codes for preflight UI/CLI affordances; the engine uses the same core rules as an authoritative safety net when external or stale `ActionIntent`s are injected. Failures continue to emit `CombatEventKind::OnActionFailed` or a successor structured reason surface. No UI-only legality rules are allowed.

Targeting/schema errors should fail loudly during RON parsing or schema validation where possible. Runtime unavailable mechanics should be represented as `Deferred` or `Hidden` status rather than silently omitted.

## Risks and Unknowns

- Legality DSL may become too rigid — future systems like Guard, Data Scan, DNA Chips, and conditional targeting need an extension path.
- Engine and UI may drift — avoided by forcing both to consume the same query core.
- Current resolution assumes single target — `TargetShape` must be implemented or truthfully gated.
- Full Tamer Commands and enemy counterplay could expand scope too far — M012 must keep execution deferred where appropriate while still declaring UI status.
- Enemy-only Toughness may require broad query/type updates — tests must prove ally break affordances disappear without regressing combat.

## Existing Codebase / Prior Art

- `src/data/skills_ron.rs` — current `SkillDef`, `Effect`, and `TargetShape` schema; target shape exists but is not fully executed.
- `assets/data/skills.ron` — canonical skill catalog to migrate with targeting/legalities metadata.
- `src/combat/resolution.rs` — current hardcoded action validation and effect application; must shift to shared legality core.
- `src/combat/turn_system/pipeline.rs` — Bevy action pipeline and event emission; must enforce shared legality and wire Energy caps.
- `src/combat/toughness.rs` / `src/combat/bootstrap.rs` — current Toughness component and spawn path; currently Toughness is spawned for all units.
- `src/combat/energy.rs` — Energy and `RoundEnergyTracker`; tracker exists but real pipeline wiring must be verified/fixed.
- `src/bin/combat_cli.rs` — current CLI target/action picker; currently assumes live targets and must consume shared query output.
- `src/ui/combat_panel.rs` / `src/windowed.rs` — current windowed UI path; must compile and stop relying on stale turn-order assumptions.
- `docs/combat_design.md` — design source for UI-affecting mechanics and current doc/data drift.

## Relevant Requirements

- R084 — Primary requirement: skill legality and target validity are data-driven and queryable before execution.
- R085 — UI-affecting combat mechanics are implemented or queryably declared before player-facing UI.
- R070/R071 — Existing event pipeline and reaction ordering must remain intact.
- R073 — Resource caps, especially Energy caps, must be enforced in real pipeline behavior.
- R082 — CLI harness remains the practical manual affordance/UAT surface.
- R074 — DNA Chips remains deferred, but future chip legality should reuse this milestone's query surface.

## Scope

### In Scope

- Skill targeting/legalities metadata in DSL/RON.
- Pure snapshot-based legality/affordance query API.
- Engine validation using the shared legality core.
- CLI/windowed affordance integration sufficient to compile and truthfully expose target/action states.
- Enemy-only Toughness correction or explicit inert/hide contract.
- TargetShape execution or query-level gating to prevent false UI claims.
- Real Energy cap enforcement where Energy is gained.
- Queryable implemented/deferred/hidden status for Tamer Commands/Gauge, Child gauge boost, enemy traits, and charged telegraphs.
- Doc/data alignment for known kit mismatches.

### Out of Scope / Non-Goals

- Full graphical UI polish.
- Sprites, animation, and presentation layer design.
- Full Tamer Command execution unless a minimal stub is required by the query contract.
- Full enemy counterplay behavior suite for Type Trap, Reactive Armor, and Charged Attacks.
- DNA Chips implementation.
- Large numerical rebalance beyond changes required by legality/targeting correctness.

## Technical Constraints

- Headless-first: all core legality/query code must compile and test without `windowed`.
- UI and CLI must not hardcode skill IDs for legality.
- The query API must be usable without Bevy `Commands` or mutable world access.
- Existing combat event lifecycle must remain stable unless explicitly migrated with tests.
- RON migrations must fail loudly for invalid skill declarations.
- No hidden UI affordance may imply a mechanic is implemented when query status says deferred/hidden.

## Integration Points

- Skill DSL/RON — source of target/legality metadata.
- Combat snapshot/query layer — pure API for actions, targets, resources, status, and implementation state.
- Engine pipeline — authoritative enforcement and structured failure events.
- CLI harness — immediate consumer proving target/action affordance correctness.
- Windowed panel — compile-time and minimal affordance consumer before full UI milestone.
- Design docs/GSD requirements — source of truth for deferred vs implemented mechanics.

## Testing Requirements

Run targeted tests per slice plus `cargo test-dev` for code changes. S07 and later must also run `cargo check --features "dev windowed"`.

Required test coverage includes:

- Revive legal only on KO allies.
- Heal-like fixture legal only on damaged allies and disabled on full HP.
- Offensive damage legal only on valid live enemies.
- SP, ultimate readiness, attacker KO/stunned, commander target, target state/side mismatch, and unimplemented shape/effect produce stable reason codes.
- Engine rejection reason matches preflight query reason for illegal injected `ActionIntent`s.
- Energy caps enforced through real action pipeline.
- UI/CLI affordance adapters derive legal targets/actions from query output.
- Enemy counterplay declarations expose implemented/deferred/hidden status accurately.

## Acceptance Criteria

- R084 and R085 are validated.
- `cargo test-dev` passes.
- `cargo check --features "dev windowed"` passes by end of M012.
- No UI/CLI per-skill legality hardcoding remains.
- UI-readiness query can truthfully answer: legal, illegal(reason), deferred(reason), or hidden for every action/resource/target/trait it exposes.
- Known doc/data mismatches are resolved or explicitly documented as MVP simplifications/deferred behavior.

## Open Questions

- Should `Row` be implemented as true row targeting now, aliased to `AllEnemies`, or marked `Deferred(UnimplementedShape)` until battlefield positioning exists?
- Should Heal/Cleanse/Silence be implemented as executable effects now or only represented as future legality fixtures?
- Should Tamer Gauge get a minimal resource implementation in M012 or remain a declarative deferred affordance until the Tamer Commands milestone?
- Should ally Toughness be removed structurally (`Option<Toughness>`) or kept internally but hidden/inert by contract? Preferred direction: remove from allies / enemy-only option.
