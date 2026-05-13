---
id: S03
milestone: M018
status: draft
---

# S03: TargetShape::Bounce(N) path-dependent chain con tie-break — Context (DRAFT)

## Goal

Estendere il resolver con `TargetShape::Bounce(N)`: chain path-dependent di N hop su enemy vivi, hop ≥1 selezionato via `LowestHpPctAlive` + `slot_index` asc, no-repeat policy, truncation silenziosa, damage costante per hop.

## Decisions (round 1+2)

- **Hop ≥1 selector:** `LowestHpPctAlive` + `slot_index` asc tie-break. Deterministico, no RNG plumbing in S03.
- **No repeats:** HashSet `already_hit` seedato col primary; chain trunca quando il pool si svuota.
- **Truncation:** silenziosa, SP/ult full-pay (S02 hoist pattern), no `ChainEnded` event.
- **Damage curve:** costante per hop (falloff demandato a M020+).
- **N bounds:** N ∈ [1, 8]. Bounce(0) rejected al validator. Bounce(1) ammesso (degenera a Single).
- **JSONL:** reuse `OnDamageDealt`, wrap CLI-side con `{hop_index, ...}` al print time. Zero churn schema engine.

## Scope

### In Scope
- `TargetShape::Bounce(u8)` + validator allowlist + N∈[1,8] enforcement.
- `next_bounce_hop()` pure fn (LowestHpPct + slot tie-break, exclude already_hit).
- Pipeline hop loop: rebuild snapshot per hop, `apply_damage_only`, no-repeat.
- Fixture skill `chain_bolt` (Bounce(3) + Damage) in `skills.ron`.
- `combat_cli --scenario bounce-chain` con JSONL hop-wrapped + determinism gate (2× run byte-diff).
- Integration test `tests/target_shape_bounce_chain.rs` (KO mid-chain, pool exhaustion, primary-lowest no-revisit).

### Out of Scope
- Damage falloff per hop (M020+).
- Per-skill selector override (`Bounce(N, Selector)`) — demandato a roster-identity slice post-M018.
- `BounceHop` CombatEventKind dedicato.
- Affordance UI per hop preview (windowed, fuori M018).
- Bounce su alleati / cross-team (enemies only).

## Constraints
- Determinismo hard: no wall-clock, no RNG. HP% via `(hp * 1000) / hp_max` (i32, no float).
- Pure resolver pattern (S02 D02): `next_bounce_hop` consuma `TargetableSnapshot` + `&HashSet<UnitId>`.
- `Effect::Damage{target}` deve matchare `targeting.shape` esattamente (validator skills_ron.rs:301).
- S02 Phase 1 hoist invariato: SP/ult/streak consumati una volta pre-loop.
- Headless first; zero regressioni 40+ binari test esistenti.

## Integration Points

### Consumes
- `src/data/skills_ron.rs` — extend `TargetShape` enum + validator allowlist.
- `src/combat/resolution.rs` — `target_shape_is_executable_now` allowlist; aggiungere `next_bounce_hop`.
- `src/combat/action_query.rs` — allowlist `target_status_for_unit`.
- `src/combat/turn_system/pipeline.rs:182` — multi-target arm widening + hop loop.
- `apply_damage_only` (S02) — riutilizzato per-hop.

### Produces
- `tests/target_shape_bounce_chain.rs` — integration test (4 casi: full chain, KO mid-chain, pool exhaustion, primary-lowest).
- `assets/data/skills.ron` — `chain_bolt` fixture.
- `src/bin/combat_cli.rs` — `run_bounce_chain_scenario` + dispatcher arm.

## Open Questions
- KO mid-chain timing: confermato apply damage → re-snapshot → select next hop (implicito da hoist + per-hop snapshot rebuild).
- Cross-team: Bounce solo su enemies, simmetria con Tentomon identity (non esplorato per ally-Bounce).
