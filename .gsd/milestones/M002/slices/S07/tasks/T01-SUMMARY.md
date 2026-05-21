---
id: T01
parent: S07
milestone: M002
key_files:
  - src/combat/action_query/types.rs
  - src/combat/turn_system/types.rs
  - src/combat/turn_system/resolve.rs
  - src/ui/combat_panel/render.rs
  - src/ui/combat_panel/preview_cache.rs
  - src/bin/combat_cli/player.rs
  - tests/action_query/action_affordance_consumers.rs
key_decisions:
  - Renamed snapshot field ult_metadata -> gauge_meta to match T02 contract (snapshot.gauge_meta).
  - Added energy_data: Option<Energy> alongside existing scalar energy: i32 / energy_max: i32 to avoid breaking downstream readers that already use the scalars.
  - Solved the Bevy 15-tuple QueryData cap by NOT widening ResolveActorsQuery; instead exposed UltGaugeMetadata via a small sibling Query system param in resolve_action_system. Keeps all pipeline destructures (declaration/application/paths) untouched at 15 elements.
duration: 
verification_result: passed
completed_at: 2026-05-21T18:08:29.056Z
blocker_discovered: false
---

# T01: Plumbed gauge_meta and energy_data through UnitQuerySnapshot and all units_data callsites without behavior change

**Plumbed gauge_meta and energy_data through UnitQuerySnapshot and all units_data callsites without behavior change**

## What Happened

Extended UnitQuerySnapshot with two new optional fields: `gauge_meta: Option<UltGaugeMetadata>` (renamed from the prior in-progress `ult_metadata`) and `energy_data: Option<Energy>`. Both populated in `build_snapshot_from_ecs_with_sp` from the existing `Option<&UltGaugeMetadata>` and `Option<&Energy>` slots of the `units_data` tuple. The fallback initializer was switched to `..Default::default()` to be resilient to future field additions.

Resolving the 16-element ResolveActorsQuery tuple (which exceeded Bevy 0.18's 15-tuple `QueryData` cap and broke the entire build) by removing `UltGaugeMetadata` from that query and adding a small sibling read-only `Query<&UltGaugeMetadata>` system param (`gauge_meta_q`) into `resolve_action_system`. This avoids changing the 11 pipeline destructure sites (declaration, application, paths/*) that all use 15 placeholders. Inside the snapshot-builder closure, `gauge_meta` is fetched per-entity via `gauge_meta_q.get(entity).ok()`.

Fixed the readonly-iter closure: the previous `.as_deref()` calls on `ult`/`toughness` borrowed from local closure-stack Options (E0515). After `as_readonly()` the entries are already `Option<&T>`, so `.as_deref()` was dropped.

Updated all snapshot-producing callsites to carry the 13th tuple element (`Option<&UltGaugeMetadata>`):
- `src/ui/combat_panel/render.rs` — appended `gauge_meta` to the units_data push (CombatPanelUnitsQuery already exposed it).
- `src/ui/combat_panel/preview_cache.rs` — added `Option<&'static UltGaugeMetadata>` to the world query and pushed it.
- `src/bin/combat_cli/player.rs` — already 13-element; touched one peripheral closure pattern to match the 12-element CLI Query (added `UltGaugeMetadata` to that query upstream is already present).
- `tests/action_query/action_affordance_consumers.rs` — appended `None` as 13th element in both fixture-build sites (parallels T05 sweep but needed now to compile the action_query verification target).

No behavior change: gauge_meta and energy_data are exposed but not yet read by legality/resources logic — that lands in T02.

## Verification

Ran `cargo check --features windowed` (clean) and `cargo test --features windowed --test action_query --test windowed_only` (41 + 23 passed). Both per the task's stated verification command.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check --features windowed` | 0 | pass | 430ms |
| 2 | `cargo test --features windowed --test action_query --test windowed_only` | 0 | pass | 230ms |

## Deviations

Task plan listed bootstrap.rs, dashboard.rs, and combat/preview.rs as touched files — those do not actually construct units_data and required no changes. Conversely, the partially-broken in-progress state (16-tuple ResolveActorsQuery) was inherited from a prior aborted run; resolving it was strictly required to compile but went slightly beyond the plan's "plumbing-only" framing.

## Known Issues

Fallback initializer in build_snapshot_from_ecs_with_sp now uses ..Default::default() — slightly weaker than full explicit struct literal, but only triggers if actor missing from query (defensive path). T05 will sweep remaining hand-written UnitQuerySnapshot literals in test fixtures.

## Files Created/Modified

- `src/combat/action_query/types.rs`
- `src/combat/turn_system/types.rs`
- `src/combat/turn_system/resolve.rs`
- `src/ui/combat_panel/render.rs`
- `src/ui/combat_panel/preview_cache.rs`
- `src/bin/combat_cli/player.rs`
- `tests/action_query/action_affordance_consumers.rs`
