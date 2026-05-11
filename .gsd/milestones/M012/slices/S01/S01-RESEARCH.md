# S01 Research — UI-readiness gap matrix and legality contract

## Summary

S01 owns the discovery/contract work for **R084** and **R085**. The current code has a functional headless combat pipeline, but action legality is split across runtime guards in `apply_effects`/`step_app`, UI/CLI target lists, and implicit skill-effect interpretation. There is **no pure shared legality/affordance query surface** yet.

Baseline evidence generated during research:

- `cargo test-dev` passed: all current lib/bin/integration/doc tests green (warnings only).
- `cargo check --features "dev windowed"` currently **fails** in `src/windowed.rs:220` because `TurnOrder` no longer has `advance()`. This is a pre-existing windowed path gap and should be fixed by or before S07, but S01 should record it in the matrix now.

Relevant installed skill rule used: `verify-before-complete` says completion claims need fresh evidence in the same message. This research includes fresh baseline verification above; do not later claim M012/S01 implementation is complete without re-running the relevant checks after code changes.

## Requirements targeted

- **R084 — data-driven/queryable skill legality and targeting before execution.** S01 should define the contract that later slices implement: action status, target status, resource readiness, and stable machine-readable reasons must come from skill data + world snapshot, not UI-specific rules.
- **R085 — UI-affecting mechanics implemented or queryably declared.** S01 should produce the gap matrix that classifies mechanics as implemented, to-fix-now, deferred, or hidden so UI cannot truthfully expose unsupported mechanics.

## Skill discovery

Core technologies are Rust and Bevy ECS. No directly relevant Rust/Bevy skill is installed in the current `<available_skills>` list. `npx skills find "Bevy Rust"` found promising optional skills; do **not** install automatically:

- `npx skills add mindrally/skills@rust` — 243 installs; broad Rust guidance.
- `npx skills add bfollington/terma@bevy` — 117 installs; Bevy-specific.
- `npx skills add sickn33/antigravity-awesome-skills@bevy-ecs-expert` — 108 installs; Bevy ECS-specific.

## Implementation landscape

### Skill DSL and effect interpretation

- `src/data/skills_ron.rs:9` defines `TargetShape { Single, Row, AllEnemies, SelfOnly }`.
- `src/data/skills_ron.rs:18` defines `Effect`, including `Damage { amount, target }`, `Revive`, `GrantEnergy`, `SelfAdvance`, status application, and other current mechanics.
- `src/data/skills_ron.rs:36` defines `SkillDef { id, name, damage_tag, sp_cost, effects }`.
- `src/combat/resolution.rs:52` maps an `ActionIntent` + `UnitSkills` + `SkillBook` to `ResolvedAction` by extracting the first matching effect type.
- Important gap: `TargetShape` is parsed but effectively collapsed away. `resolve_action` stores only one `target: UnitId` in `ResolvedAction`; `apply_effects` mutates only one defender. Row/AllEnemies/SelfOnly are not represented in `ResolvedAction`.

### Engine validation and mutation pipeline

- `src/combat/turn_system/mod.rs:34` defines `ActionIntent::{Basic, Skill, Ultimate}` — all carry exactly one target.
- `src/combat/turn_system/mod.rs:113` `resolve_action_system` reads one intent, runs `step_declaration`, emits lifecycle events, then calls `step_app`.
- `src/combat/turn_system/pipeline.rs:28` `step_declaration` only verifies that the attacker has a kit and that the skill resolves. It does not perform legality checks or emit a structured failure when attacker/skill/target lookup fails.
- `src/combat/turn_system/pipeline.rs:62` `step_app` re-finds attacker and target entities and performs several guards before mutation: attacker stunned/KO, defender KO vs revive/non-revive, then delegates to `apply_effects`.
- `src/combat/resolution.rs:170` `apply_effects` duplicates some validation: commander target, attacker KO, revive target KO state, non-revive target KO state, SP shortfall, ultimate readiness. Failures are strings in `CombatEventKind::OnActionFailed { reason: String }`.
- Natural seam: introduce a pure `combat::legality` or `combat::affordance` module with snapshot structs and reason enums, then have `step_app`/`apply_effects` consume the same result instead of duplicating ad hoc string guards.

### UI/CLI consumers that currently make their own assumptions

- `src/bin/combat_cli.rs:295` builds target entries as **all non-KO non-self units across both teams**. This hides KO allies, so revive cannot be chosen legally in CLI, and it exposes allies/enemies without skill-specific filtering.
- `src/ui/combat_panel.rs:231` enables Basic purely from active ally state.
- `src/ui/combat_panel.rs:362` enables target buttons only for non-KO enemies. This hardwires enemy-only targeting and prevents ally/KO targeting such as revive/heal; it also cannot explain disabled targets.
- `src/ui/combat_panel.rs` query type requires `&Toughness` for all displayed units, so removing ally `Toughness` structurally will require optional toughness in the windowed display query.
- `src/windowed.rs:220` calls removed `TurnOrder::advance()` and currently breaks `cargo check --features "dev windowed"`.

### Snapshot/observability precedent

- `src/combat/observability.rs:21` defines `ValidationSnapshot`; it already proves a precedent for pure-ish snapshot structs extracted from `World`.
- Current `ValidationSnapshot` requires `Toughness` and `UltimateCharge` on every unit (`ValidationSnapshotError::MissingToughness` etc.), which conflicts with the R085 preferred direction of enemy-only Toughness. Later slices must update this snapshot and tests if allies stop spawning `Toughness`.

## UI-readiness gap matrix

| Area | Current code/data | UI truth risk | Contract recommendation | Likely slice |
|---|---|---|---|---|
| Offensive single-target damage | Most skills are `Damage(... target: Single)` in `assets/data/skills.ron`; engine applies to one target. | UI can pick allies/invalid targets in CLI and only enemies in windowed; engine does not check side. | Query target legality should classify live enemies legal for offensive effects and live allies illegal with reason e.g. `WrongSide`. | S03/S04/S06/S07 |
| Revive | `Effect::Revive` exists and engine rejects revive on non-KO target. `patamon_revive` has `sp_cost: 6`, above current `SpPool::max=5` in default resource, while memory MEM045 says tests lowered cost pattern where needed. | CLI/windowed hide KO allies, so revive-like skills are not player-usable through affordance UI. `first_aid` is revive too, not heal. | DSL needs target-policy metadata (e.g. allies + KO required) or derivation from `Revive`; query returns KO allies legal, live units disabled with reason `TargetNotKo`, enemies disabled by side. Decide what to do with `patamon_revive` cost > max. | S03/S04/S06/S07 |
| Heal-like examples | No `Heal` effect exists; docs describe Patamon/Angemon healing/cleanse, but shipped data mostly uses damage/revive. | UI cannot represent damaged-ally-only legality except as future fixture. | Either add a minimal `Heal` effect/fixture or declare `Heal` as deferred in query contract. S01 should decide matrix status; later tests can use fixture skills if not canonical. | S03/S04/S09 |
| Cleanse/Silence/Guard | No first-class `Cleanse`, `Silence`, or `Guard` effect in `Effect`; docs describe them. Status system has Burn/Freeze/Shock only. | UI could falsely render doc-described mechanics as shipped. | Mark as `Deferred`/`Hidden` affordance with stable reason code until implemented. Avoid skill-ID exceptions. | S08/S09 |
| TargetShape Row/AllEnemies | `TargetShape` enum and RON contain several `Row` skills: `heat_viper`, `greymon_ult`, `mega_blaster_aoe`, `kabuterimon_ult`, `kyubimon_ult`, `angemon_ult`. Engine still resolves only one target. | UI would advertise AoE/row skills while only one target is affected. | For S02/S03 either implement true multi-target execution or mark non-`Single` shapes as `Deferred(UnimplementedTargetShape)` in query. Roadmap prefers making Row/AllEnemies unable to silently behave as single-target. | S02/S03/S04/S06 |
| SelfOnly | `TargetShape::SelfOnly` exists but no canonical RON usage found. `SelfAdvance` effect targets source in engine. | Future self skills may be shown with wrong target picker. | Contract should include self-target policy even if canonical data does not use it; tests can add fixture. | S03/S04 |
| Mixed effect target semantics | `angemon_ult` currently combines `Damage(Row)`, `ToughnessHit`, and `Revive(20)` in one skill. `apply_effects` treats any revive skill as revive-only and skips damage path. | Data claims a heavy AoE + revive; engine executes only revive semantics and requires KO target. | Matrix should flag `angemon_ult` as data/code mismatch. Later slices should either split effects, declare unsupported mixed targeting, or add explicit per-effect targeting semantics. | S01 decision → S03/S09 |
| Ally Toughness | `assets/data/units.ron` gives allies nonzero `toughness_max` and weaknesses; `spawn_unit_from_def` always inserts `Toughness` (`src/combat/bootstrap.rs:140`). Design says Toughness/Break exists enemy-only. | UI/windowed/CLI can show ally toughness or enemy AI can consider ally toughness ratios. | Preferred direction from context: do not spawn `Toughness` for allies, or make ally toughness optional/hidden. Query must expose toughness affordance only for enemies with real bars. Update all `Query<&Toughness>` to optional and tests accordingly. | S02 |
| Enemy Toughness with zero max | Goblimon has `toughness_max: 0`; still spawns `Toughness` component. | UI may display `0/0` bar; enemy AI divides by max but currently maps max via snapshot default max `toughness.map(|t| t.max).unwrap_or(1)` in one path and raw target info elsewhere. | Query should classify zero/maxless toughness as hidden/not breakable. S02 should define whether zero-toughness enemies have no component or hidden affordance. | S02/S04 |
| Energy caps | `RoundEnergyTracker` exists (`src/combat/energy.rs:43`) and is spawned on all units (`src/combat/bootstrap.rs:169`). Real pipeline grants energy via direct `energy.gain(inflight.action.energy_grant)` (`src/combat/turn_system/pipeline.rs:298`), bypassing tracker. | UI resource bars/caps can lie; R073 is marked validated historically but current pipeline does not enforce cap for GrantEnergy/Form Identity. | S05 should route every real energy grant through `RoundEnergyTracker::try_gain(source, amount)`, emit actual delta, and reset tracker at round/turn boundary. Need source classification (`SecondaryAction` vs `External`) in query/engine. | S05 |
| SP/ultimate readiness | SP spend and ultimate readiness are enforced late in `apply_effects`; UI separately enables Ultimate only if ready, Skill menu ignores SP. | UI cannot explain SP shortfall or child discount; engine failure reasons are strings. | Query should report action-level disabled reasons: `SpShortfall`, `UltimateNotReady`, `AttackerKo`, `AttackerStunned`. Include effective SP cost after Child discount. | S04/S06/S07 |
| Attacker state | `step_app` rejects stunned/KO attacker with string reasons; `advance_turn_system` skips stunned units in some flows. | UI may still show action options from stale state. | Query action affordances should include actor KO/stunned/commander state and current phase/active unit. | S04/S06/S07 |
| Commander target | `apply_effects` rejects target commander with string `Target is a Commander`; CLI target list excludes commanders only for enemy AI, not player prompt. | Player may target Taichi/commander in CLI if non-KO; windowed only enemies so less visible. | Query target legality should include `TargetIsCommander`. Engine must use same reason. | S04/S06/S07 |
| Tamer Gauge/Commands | Docs define Data Scan/Emergency Guard/Retreat, but no `TamerGauge` component or command intents found. Taichi is a commander unit with skills (`rally`, `first_aid`, `taunt`). | UI might show commands from docs that do not execute. | Query should expose commands as `Deferred` or `Hidden` with names/costs if desired, not executable until a later milestone. | S05/S08/S09 |
| Child Tamer Gauge boost | Docs say Child basic vs revealed intent boosts Tamer Gauge. No intent-reveal/gauge surface found. | UI could show a missing resource dependency. | Declare deferred/hidden with reason `TamerGaugeDeferred` unless M012 promotes minimal gauge implementation. | S05/S08/S09 |
| Enemy counterplay/telegraphs | Docs define Type Trap, Reactive Armor, Break Seal, Tempo Anchor, Charged Attacks. Implemented pieces: `TempoResistance`, `Break Seal` via `RoundFlags.break_sealed`, `ToughnessCategory::{Armored,Shielded}`. No charged attack telegraph or trait declaration surface in `UnitDef` beyond `signature_traits` strings. | UI cannot tell players which enemy traits/telegraphs are real. | Add queryable enemy trait/telegraph declarations: implemented (`TempoAnchor`, `BreakSeal`/`Shielded`, `Armored`), deferred (`TypeTrap`, `ReactiveArmor`, `ChargedAttack`) or hidden. Avoid free-text `signature_traits` as the only source. | S08/S09 |
| Failure reasons | `CombatEventKind::OnActionFailed { reason: String }` uses strings: `Attacker is stunned`, `Target is KO`, `SP shortfall`, etc. | UI/tests cannot reliably match structured reason unless string-stable. | Introduce reason enum + display mapping, or a structured successor event while preserving compatibility. S06 acceptance needs engine rejection reason matches preflight query. | S04/S06 |
| Windowed active unit | `combat_panel` uses `order.future_preview.first()` for active ally, but AV system uses `TurnOrder.active_unit`. | Windowed may not enable actions even after compile fix. | S07 should use query/active unit surface, probably `active_unit`, and derive action buttons from affordance query. | S07 |

## Natural seams for implementation planning

1. **Contract/data model seam** — `src/data/skills_ron.rs` is where targeting/legalities metadata belongs. Plan should decide whether to derive some rules from `Effect` or add explicit metadata fields. The milestone decision D053 says legality lives in existing DSL, not a separate UI registry.
2. **Pure query seam** — add a headless module such as `src/combat/legality.rs` or `src/combat/affordance.rs`. It should accept plain snapshots and `SkillBook`, not `World`/`Commands`, then return action/target/resource affordances.
3. **World adapter seam** — a thin adapter can extract snapshots from Bevy queries/world. Existing `observability.rs` is precedent but should not be reused directly because it currently requires Toughness on all units.
4. **Engine safety seam** — `step_declaration`/`step_app` are the right integration points. The engine should ask the query before mutation, emit the same structured reason, then call `apply_effects` only for legal intents.
5. **UI/CLI consumer seam** — `combat_cli.rs` and `ui/combat_panel.rs` should become adapters over query output. Do this after core query + engine validation to avoid creating a second rule layer.
6. **Mechanic declaration seam** — larger future mechanics (Tamer commands, enemy traits, charged telegraphs) can be separate query/declaration structs rather than executable `ActionIntent`s for now.

## Recommended contract shape

A practical M012 contract should separate four concepts:

```rust
ActionStatus = Enabled | Disabled { reason } | Deferred { reason } | Hidden { reason }
TargetStatus = Legal | Illegal { reason } | Deferred { reason } | Hidden { reason }
ResourceStatus = Ready | Insufficient { reason, current, required } | Deferred | Hidden
ImplementationStatus = Implemented | Deferred { reason } | Hidden { reason }
```

Recommended stable reason enum families:

- Actor/action: `NotActiveUnit`, `WrongPhase`, `AttackerKo`, `AttackerStunned`, `MissingSkill`, `SpShortfall`, `UltimateNotReady`, `UnimplementedEffect`, `UnimplementedTargetShape`.
- Target: `TargetNotFound`, `TargetIsSelf`, `TargetIsCommander`, `WrongSide`, `TargetKo`, `TargetNotKo`, `TargetFullHp`, `TargetNotDamaged`, `NoValidTargets`.
- Mechanics/resources: `ToughnessEnemyOnly`, `TamerGaugeDeferred`, `TamerCommandDeferred`, `ChargedTelegraphDeferred`, `EnemyTraitDeferred`, `EnergyCapReached`.

Keep display strings separate from reason codes. The engine can still emit `OnActionFailed { reason: String }` short-term, but S06 should prove the string is derived from the same reason enum returned by preflight.

## What to build/prove first

1. **Matrix artifact first**: S01 should persist this matrix (or a refined version) as the slice deliverable before implementation slices proceed.
2. **S02 before DSL expansion**: enemy-only Toughness impacts query shapes everywhere. If allies lose `Toughness`, many Bevy queries must become optional. Fixing this early prevents designing a query around false ally bars.
3. **TargetShape truthfulness before metadata migration**: decide whether `Row`/`AllEnemies` become executable or deferred. Do not add UI affordance output that says AoE works while `ResolvedAction` still has one target.
4. **Reason enum before engine integration**: engine/query parity is easier if stable reason codes exist before wiring `OnActionFailed`.
5. **Fixture tests before RON migration**: write small pure tests with fixture skills for damage, revive, heal-like, self, Row/AllEnemies before migrating all canonical RON.

## Verification strategy for later slices

Baseline commands:

- Headless: `cargo test-dev` (currently passes).
- Windowed compile: `cargo check --features "dev windowed"` (currently fails at `src/windowed.rs:220`; planner should treat as known debt).

Suggested targeted tests by area:

- `tests/legality_query.rs` for pure query contract: offensive targets, revive KO allies, heal damaged allies, full HP disabled, SP shortfall, ultimate readiness, attacker KO/stunned, commander target, unimplemented shape/effect.
- `tests/engine_legality_rejection.rs` for forced illegal `ActionIntent` through Bevy messages: preflight reason equals emitted failure reason and no mutation occurs.
- `tests/toughness_enemy_only.rs` for no ally toughness affordance and no ally break UI surface.
- `tests/energy_caps_pipeline.rs` for real `GrantEnergy`/Form Identity cap enforcement through `resolve_action_system`/follow-up pipeline.
- CLI/windowed integration can be compile-time plus small adapter tests if the query output is pure; avoid requiring interactive `inquire` in tests.

## Forward intelligence / gotchas

- Memory MEM043 confirms `RoundEnergyTracker` is intended as a per-unit component; use it instead of a resource.
- Memory MEM061 says removed dead Effect variants should not be resurrected for MVP unless necessary; DORUgamon/Angemon use separate skills as the canonical workaround.
- Memory MEM045 warns SP max/cost mismatches can make revive structurally impossible. Canonical `patamon_revive` still costs 6 while default SP max is 5; decide if this is intentional deferred/balance debt or should be adjusted.
- Removing ally `Toughness` will break several existing queries/tests that assume every unit has it: `spawn_unit_from_def`, `ValidationSnapshot`, `combat_panel`, many integration test fixtures. Prefer optional query fields and helper builders.
- `ActionIntent` has exactly one target. True AoE will require either expanding intent/resolution to multiple resolved targets or treating AoE shape as deferred until battlefield positioning exists.
- Do not put per-skill rules in CLI/windowed. The hard boundary says UI must only consume DSL/query output.

## Sources read

- `src/data/skills_ron.rs`
- `assets/data/skills.ron`
- `assets/data/units.ron`
- `src/combat/resolution.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/bootstrap.rs`
- `src/combat/toughness.rs`
- `src/combat/energy.rs`
- `src/combat/follow_up.rs`
- `src/combat/kit.rs`
- `src/combat/observability.rs`
- `src/bin/combat_cli.rs`
- `src/ui/combat_panel.rs`
- `src/windowed.rs`
- `src/combat/turn_order.rs`
- `docs/combat_design.md`
- `tests/resource_caps.rs`
