# S08 Research — Enemy counterplay declaration surface

## Summary

S08 owns the R085 portion that says enemy traits and charged telegraphs must be queryable before player-facing UI work. R084 is a supporting constraint: any exposed action/resource/target/trait surface must flow through the shared query vocabulary, not UI/CLI hardcoding.

The codebase already has most of the affordance vocabulary needed for S08, but no enemy-counterplay data model yet. `src/combat/action_query.rs` already defines `ResourceKind::ChargedTelegraph`, `ResourceKind::EnemyTrait`, `LegalityReasonCode::ChargedTelegraphDeferred`, and `LegalityReasonCode::EnemyTraitDeferred`; S05/S07 also established pure query functions for deferred Tamer resources. The missing piece is a typed declaration surface on unit/enemy data and a pure query API that turns enemy unit declarations + current runtime facts into `Implemented` / `Deferred` / `Hidden` affordances.

Recommendation: implement S08 as a small data/query slice, not a combat-behavior slice. Add typed enemy counterplay declarations to unit data, extract them into `UnitQuerySnapshot`, add pure query functions in `action_query.rs`, populate canonical enemy data for Type Trap / Reactive Armor / Break Seal / Tempo Anchor / Charged Attacks, and add tests proving only Tempo Anchor and Break Seal map to implemented runtime mechanics today while Type Trap / Reactive Armor / Charged Attacks stay deferred.

## Requirements Targeted

- **R085 primary:** enemy counterplay telegraph/trait declarations must distinguish implemented mechanics from deferred/hidden so UI cannot present unimplemented warnings as executable.
- **R084 supporting:** the declarations must be exposed by the same shared query layer used by UI/CLI/engine-adjacent consumers, with stable reason codes.

## Skill / Tooling Notes

- Installed relevant skills from the prompt: `tdd` (use a red/green loop around query contract tests), `test` (run targeted Rust tests), and `verify-before-complete` (executor should run fresh verification before marking tasks complete).
- Professional skill discovery for core tech found optional Bevy/Rust skills; do not install automatically:
  - `npx skills add bfollington/terma@bevy` — 117 installs, likely Bevy-focused.
  - `npx skills add sickn33/antigravity-awesome-skills@bevy-ecs-expert` — 108 installs, likely ECS-focused.
  - `npx skills add laurigates/claude-plugins@bevy-ecs-patterns` — 25 installs, lower install count but directly relevant.
- No external library docs were needed. This is current-project Rust/serde/Bevy data/query work using existing patterns.

## Implementation Landscape

### Existing query vocabulary

`src/combat/action_query.rs` already contains the status vocabulary S08 should reuse:

- `ActionStatus`, `TargetStatus`, `ResourceStatus`, `ImplementationStatus` all support `Enabled`/`Disabled`/`Deferred`/`Hidden`-style states.
- `ResourceKind` already includes `ChargedTelegraph` and `EnemyTrait`.
- `LegalityReasonCode` already includes `ChargedTelegraphDeferred` and `EnemyTraitDeferred`.
- `query_tamer_gauge_affordance()` and `query_tamer_command_affordances()` are good patterns for queryable deferred declarations with no execution path.
- `query_energy_cap_affordance()` is the pattern for query functions that return `ResourceAffordanceDetail` with current/required when meaningful.

Suggested new types/functions in or near `action_query.rs`:

```rust
pub enum EnemyCounterplayKind {
    TypeTrap,
    ReactiveArmor,
    BreakSeal,
    TempoAnchor,
}

pub enum ChargedAttackStatus { /* probably use existing ResourceStatus instead */ }

pub struct EnemyTraitAffordance {
    pub kind: EnemyCounterplayKind,
    pub status: ImplementationStatus,
    pub reason: Option<LegalityReasonCode>,
    pub label: &'static str or String,
}

pub struct ChargedTelegraphAffordance {
    pub skill_id: Option<SkillId>,
    pub status: ResourceStatus,
    pub turns_until_fire: Option<u32>,
    pub reason: Option<LegalityReasonCode>,
}

pub fn query_enemy_trait_affordances(unit: &UnitQuerySnapshot) -> Vec<EnemyTraitAffordance>;
pub fn query_charged_telegraph_affordance(unit: &UnitQuerySnapshot) -> ResourceAffordanceDetail or richer struct;
```

Keep these pure and snapshot-based. Do not inspect Bevy `World` or UI state.

### Unit data seam

`src/data/units_ron.rs` currently owns enemy-relevant static data:

- `signature_traits: Vec<String>` is only free-text today; docs say free-text traits are not enough for UI-readiness.
- `tempo_resistant: bool` maps to an actual `TempoResistance` component in `bootstrap::spawn_unit_from_def()`.
- `toughness_category: ToughnessCategory` maps to real `Toughness::with_category(...)`; `ToughnessCategory::Shielded` is the existing Break Seal execution seam.
- `UnitDef` does **not** use `#[serde(deny_unknown_fields)]`, so adding serde-default fields is straightforward, but `round_trip_unit_def()` and `taichi_def()` must compile after any non-default fields.

Recommended schema additions in `units_ron.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnemyCounterplayKind { TypeTrap, ReactiveArmor, BreakSeal, TempoAnchor }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MechanicDeclarationStatus {
    Implemented,
    Deferred { reason: LegalityReasonCode },
    Hidden { reason: LegalityReasonCode },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnemyTraitDeclaration {
    pub kind: EnemyCounterplayKind,
    pub status: MechanicDeclarationStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChargedAttackDeclaration {
    pub skill_id: SkillId,
    pub lead_turns: u32,
    pub status: MechanicDeclarationStatus,
}
```

Then on `UnitDef`:

```rust
#[serde(default)]
pub enemy_traits: Vec<EnemyTraitDeclaration>,
#[serde(default)]
pub charged_attack: Option<ChargedAttackDeclaration>,
```

Alternative: put these types in a new `src/combat/enemy_counterplay.rs` module and import from `units_ron.rs` + `action_query.rs`. That is cleaner if the planner expects growth beyond S08.

### Runtime facts already implemented

- **Tempo Anchor**: implemented as `TempoResistance` in `src/combat/resistance.rs`; `bootstrap::spawn_unit_from_def()` inserts it when `UnitDef.tempo_resistant == true`. Devimon has `tempo_resistant: true` in `assets/data/units.ron`.
- **Break Seal**: partially implemented by `ToughnessCategory::Shielded`, where `Toughness::apply_hit()` clamps current but never sets `broken`. This is the existing mechanic to declare as implemented when an enemy has shielded toughness. No canonical enemy currently uses `Shielded`; Devimon uses `Armored`.
- **Reactive Armor**: not implemented as design says. `ToughnessCategory::Armored` only halves toughness damage, while design Reactive Armor reduces HP damage from secondary actions. Do not claim this is implemented unless a new HP-damage path exists.
- **Type Trap**: no implementation found. It should be declared deferred if present in data.
- **Charged Attacks**: no telegraph/scheduled attack model found. Enemy AI simply chooses ultimate if ready, then first skill, then basic; charged telegraph should be deferred.

### Canonical data seam

`assets/data/units.ron` has three enemies:

- Devimon (`UnitId(101)`): boss, `signature_traits: ["evil", "dark"]`, `tempo_resistant: true`, `toughness_category: Armored`.
- Goblimon (`UnitId(102)`): minion, zero toughness, no counterplay traits expected.
- Ogremon (`UnitId(103)`): miniboss, standard toughness.

Suggested canonical declaration mapping for S08:

- Devimon: `TempoAnchor = Implemented`; `TypeTrap = Deferred(EnemyTraitDeferred)` if design wants the boss to advertise Type Trap; `ReactiveArmor = Deferred(EnemyTraitDeferred)` only if design says Devimon has it; `charged_attack = Some(... Deferred(ChargedTelegraphDeferred))` if boss UI should show a future telegraph contract.
- Ogremon: `charged_attack = Some(... Deferred(ChargedTelegraphDeferred))` because design says mini-boss/boss only; optionally `BreakSeal` only if `toughness_category: Shielded` is assigned.
- Goblimon: no charged attack and no trait declarations, or hidden declarations if a global query enumerates absent mechanics. Roadmap says enemy traits and charged attacks have a queryable UI contract, not that every enemy needs every hidden entry.

Important: if S08 wants to prove Break Seal is implemented today, either add a canonical `Shielded` enemy (likely Devimon) or use a focused fixture test that constructs a `UnitQuerySnapshot`/`UnitDef` with `ToughnessCategory::Shielded`. Changing Devimon from `Armored` to `Shielded` may affect boss TTK/break tests, so fixture-only proof is safer unless design explicitly wants canonical boss shielded now.

### Consumer seams

CLI/windowed already know how to format `ResourceKind::ChargedTelegraph` and `ResourceKind::EnemyTrait` labels, but they only display resource details attached to action tooltips. There is no global enemy card trait/telegraph section yet.

For S08, it is probably sufficient to add query tests and maybe minimal display helper integration:

- CLI: after printing units, optionally include enemy trait/telegraph labels from query output. Keep as display-only, not legality logic.
- Windowed: enemy card already renders HP/toughness; a small line for enemy declarations could be added, but S08 roadmap only requires queryable UI contract, not UI polish.

If changing consumers, preserve S07 rule: consumers may display/filter query output but must not decide legality locally or hardcode skill IDs.

## Natural Task Seams

1. **Schema + data declarations**
   - Add typed enemy counterplay/charged attack declarations and serde defaults.
   - Update `UnitDef` round-trip test, `taichi_def()`, and canonical `assets/data/units.ron` declarations.
   - Keep old free-text `signature_traits` intact; typed declarations are the UI contract.

2. **Snapshot/query API**
   - Extend `UnitQuerySnapshot` to carry enemy trait declarations / charged telegraph declarations / toughness category or implementation facts needed for query.
   - Update `build_snapshot_from_ecs_with_sp()` tuple inputs only if runtime ECS data is needed. Prefer carrying declarations from `UnitDef` only in tests/query construction if avoiding a wide bootstrap/query tuple change is possible; however, UI/CLI snapshots currently derive from ECS components, so a component may be needed to preserve data after spawn.
   - Consider a lightweight `EnemyCounterplayKit` Bevy component inserted in `spawn_unit_from_def()` so declarations survive from RON into runtime snapshots.

3. **Runtime extraction**
   - If using `EnemyCounterplayKit`, update bootstrap spawn and CLI/windowed ECS query aliases to include it.
   - Populate `UnitQuerySnapshot` from the component.
   - Map `tempo_resistant` and `ToughnessCategory::Shielded` to implemented declarations either at data load time or in query projection.

4. **Tests**
   - Add focused tests in `tests/action_affordance_query.rs` or a new `tests/enemy_counterplay_affordance.rs`.
   - Add data/schema tests in `src/data/units_ron.rs` for canonical declarations.
   - Extend docs tests only if docs are updated in S08; S09 is the doc alignment slice, so keep S08 docs minimal.

## What to Build or Prove First

Build the pure query contract first. The riskiest part is not Bevy mechanics; it is avoiding another free-text trait list that UI cannot rely on. A good first red test:

```rust
#[test]
fn enemy_counterplay_declarations_distinguish_implemented_and_deferred() {
    // Devimon-like unit with TempoAnchor implemented, TypeTrap/ReactiveArmor deferred,
    // charged attack deferred.
    // Assert query returns ImplementationStatus::Implemented for TempoAnchor,
    // Deferred(EnemyTraitDeferred) for TypeTrap/ReactiveArmor,
    // Deferred(ChargedTelegraphDeferred) for charged telegraph.
}
```

Second proof: Break Seal fixture:

```rust
#[test]
fn shielded_toughness_declares_break_seal_as_implemented() {
    // Unit with Team::Enemy + ToughnessCategory::Shielded or explicit BreakSeal declaration.
    // Assert BreakSeal is queryable as implemented.
}
```

Third proof: canonical RON parses and enemy data contains the expected declarations for boss/miniboss/minion without relying on `signature_traits` strings.

## Risks / Constraints

- **Do not equate `Armored` with Reactive Armor.** Existing `Armored` halves toughness damage; design Reactive Armor halves HP damage from secondary actions. Claiming it as implemented would violate R085.
- **Break Seal canonical change could affect TTK.** `Shielded` never breaks; changing Devimon/Ogremon canonical toughness from `Armored`/`Standard` to `Shielded` may break scenario tests expecting `OnBreak`. Prefer fixture proof or declare Break Seal implemented-by-engine but not assigned to canonical content yet.
- **Data must survive into runtime snapshots.** `UnitDef` data is lost after spawn except for components. If UI/CLI need runtime enemy cards to query declarations, add a component rather than rereading RON in consumers.
- **No skill-ID-specific paths.** Charged attack declarations may reference a `SkillId`, but consumers must display the declaration returned by query, not match specific enemy ult names.
- **Serde defaults matter.** Adding non-default fields to `UnitDef` will force updates across tests and `taichi_def()`. Use `#[serde(default)]` for backward-compatible parsing and update round-trip fixture explicitly.

## Verification Plan

Targeted verification for S08 executors:

```bash
cargo test-dev --test action_affordance_query
cargo test-dev --test action_affordance_consumers
cargo test-dev --test scenario_boss_ttk --test scenario_miniboss_ttk
cargo test-dev
cargo check --features "dev windowed"
```

If a new test file is added, run it directly first, e.g.:

```bash
cargo test-dev --test enemy_counterplay_affordance
```

Also run `cargo test-dev --test roster_catalog` or unit tests under `src/data/units_ron.rs` if canonical `units.ron` declarations change.

## Recommendation

Plan S08 as 3 executor tasks:

1. Add typed enemy counterplay/telegraph declaration schema + canonical data + parse/validation tests.
2. Add runtime component/snapshot extraction + pure query functions returning implemented/deferred/hidden status.
3. Add consumer-facing regression tests and, only if low-risk, minimal CLI/windowed display of returned trait/telegraph labels.

Do not implement Type Trap, Reactive Armor, or Charged Attack behavior in S08. The slice promise is a declaration surface where only Tempo Anchor/Break Seal execute today; over-implementing would expand scope and risk existing combat regressions.