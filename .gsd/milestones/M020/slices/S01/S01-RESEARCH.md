# M020 / S01 — Research: Nuovi eventi reactive bus (UltimateUsed + UnitDied payload)

**Date:** 2026-05-14

## Summary

The slice adds two missing variants to the reactive event bus in `src/combat/events.rs::CombatEventKind`:

1. **`UltimateUsed { unit_id: UnitId }`** — fired every time a unit consumes its ultimate. The natural trigger is the `UltEffect::Reset` branch inside the pipeline's resource-hoist block (mirrors the `UltGain` event that fires on `UltEffect::GainFromBasic`).
2. **`UnitDied { status_remaining: Vec<StatusEffectKind>, heated_remaining: u32 }`** — rename of the existing payload-less `OnKO` variant. The payload requires the defender's `StatusBag` to be read at the moment of KO and snapshotted into the event.

Both changes are localized to a small set of files. The trickier of the two is `UnitDied` because `OnKO` is *currently* emitted from `apply_damage_only` in `resolution.rs` as a payload-less marker, then re-emitted via `event_writer` from `pipeline.rs` after a defender mutation. The new payload must be filled at the pipeline call site, where the `StatusBag` of the defender is in scope, not from inside `apply_damage_only` (which already takes `defender_status: Option<&StatusBag>` so an internal fill is also feasible — see Recommendation).

Risk is low: the bus is the single source of truth (CLAUDE.md convention) and the listeners that match `OnKO` are easy to grep (8 files total). No semantic change is intended — the rename is purely additive payload + name change.

## Recommendation

**Approach A (recommended): Fill the `UnitDied` payload inside `apply_damage_only`.**

- `apply_damage_only` already receives `defender_status: Option<&StatusBag>`. Compute `status_remaining` and `heated_remaining` right there, in the existing `if ko { events.push(...) }` block at `resolution.rs:559-561`.
- Pipeline emit sites (`pipeline.rs:458`, `975`, `1357`, `1690`) keep their existing pattern: receive `CombatEventKind::UnitDied { .. }` from the inner Vec and pass it through `emit_combat_event`. Only the *match arm* names change.
- For `UltimateUsed`, emit once-per-cast inside the resource-hoist block right next to the `UltGain` emission, gated by `matches!(inflight.action.ult_effect, UltEffect::Reset)`. Source/target should be `(attacker_id, attacker_id)` to mirror `UltGain`. There are **four** such hoist blocks in `pipeline.rs` (single-target ~line 561, Blast/AllEnemies ~1077, AllAllies ~1357-ish, hop loop ~1690-ish — confirm during execution); each needs the new event. A helper `emit_ultimate_used_if_reset(...)` may be worth extracting but is optional.

`status_remaining` should snapshot **live kinds only** (those currently in the bag at the moment damage is applied — i.e. before this turn's tick). `heated_remaining` is the duration of the `Heated` instance specifically, or `0` if absent. This is the most useful payload for downstream listeners (post-death revenge effects, dot-extinguish bookkeeping) and is trivial to compute from `StatusBag::iter` + `StatusBag::get_dur(&StatusEffectKind::Heated).unwrap_or(0)`.

**Why this approach over a pipeline-side fill:** keeping the payload assembly next to the KO decision keeps `apply_damage_only` self-contained and avoids touching every emit site at pipeline level beyond renaming the variant. The `StatusBag` parameter is already wired in.

## Implementation Landscape

### Key Files

- `src/combat/events.rs` — define `UltimateUsed { unit_id }` and rename `OnKO → UnitDied { status_remaining, heated_remaining }` in `CombatEventKind` (line 41 for `OnKO`; new variant goes near the existing `UltGain` block around lines 72-76).
- `src/combat/resolution.rs` — at line 559-561 inside `apply_damage_only`, change `events.push(CombatEventKind::OnKO)` to push `UnitDied { status_remaining, heated_remaining }` computed from the `defender_status: Option<&StatusBag>` parameter already in scope (line 464). Same pattern at the second emit site near line 780 (second `apply_*_only` family). Update the docstring at line 455 to mention `UnitDied` instead of `OnKO`. Update self-tests at lines 1298 and 1338 to match the new variant name.
- `src/combat/turn_system/pipeline.rs` — four resource-hoist blocks consume `UltEffect::Reset`:
  - ~line 561 (single-target block) — emit `UltimateUsed { unit_id: attacker_id }` next to the existing `UltGain` emit at line 628-638, gated by `matches!(inflight.action.ult_effect, UltEffect::Reset)` (Reset and GainFromBasic are mutually exclusive, so a separate `if` block is cleaner than a refactor of the existing one).
  - ~line 1077 (Blast/AllEnemies fan-out hoist) — same emission pattern next to line 1144.
  - ~line 1357 — match arm currently matches `OnKO`, rename to `UnitDied { .. }` (with field pattern `{ .. }` to discard payload at that arm if not used).
  - ~line 1690 — same `OnKO → UnitDied` rename in the per-target consume loop.
  - Match arms at lines 458 and 975 (where the KO inserts the `Ko` component and pushes `LogEntry::Ko`) also rename `OnKO → UnitDied { .. }`. None of them currently use the payload — destructure with `{ .. }` to preserve behavior.
- `src/combat/turn_system/mod.rs` — line 488 emits `CombatEventKind::OnKO` directly (likely a stun/death-on-poison path). Read the surrounding code; it needs a `StatusBag` lookup before emission. If the defender entity is reachable, fetch its bag via the actors query; otherwise emit with `status_remaining: vec![]` and `heated_remaining: 0` and add a code comment explaining the constraint.
- `src/combat/follow_up.rs` — no current `OnKO` reference (it listens to `OnEnemyKill` for kill triggers), so no rename needed; but verify with a fresh `rg OnKO src/combat/follow_up.rs` during execution.
- Tests touching `OnKO` (rename only; payload deconstruction with `{ .. }` for the cheapest update):
  - `tests/combat_coherence.rs:451`
  - `tests/pipeline_dispatch.rs:253, 268`
  - `tests/follow_up_triggers.rs:193`
  - `tests/toughness_enemy_only.rs:208`
  - `tests/event_stream.rs:251, 268`

  The JSON shape in serialized-event tests (`combat_coherence.rs`, `follow_up_triggers.rs`) currently emits `{"kind":"OnKO"}`. After the change, serde will emit `{"kind":"UnitDied","status_remaining":[...],"heated_remaining":N}`. Update the expected strings — for tests without status setup, the expected JSON becomes `{"kind":"UnitDied","status_remaining":[],"heated_remaining":0}`.

- New test files for this slice (`tests/` flat layout, functional names per CLAUDE.md):
  - `tests/ultimate_event.rs` — drive an `ActionIntent::Ultimate` (see existing patterns in `tests/ultimate_meter.rs`, `tests/follow_up_triggers.rs`) with `attacker.ult.current == max` so legality passes; assert exactly one `CombatEventKind::UltimateUsed { unit_id }` is emitted with the attacker's id, and that no `UltimateUsed` fires on a Basic or non-Reset Skill action.
  - `tests/unit_died_payload.rs` — set up a defender with `Heated` (duration 2) and `Slowed` (duration 1) in their `StatusBag`, fatally damage them, assert the resulting `UnitDied` event carries `status_remaining` containing both kinds and `heated_remaining == 2`.

### Build Order

1. **Variant rename + new variant in `events.rs` first.** This breaks the build deterministically and surfaces every consumer the compiler needs help with. Do this in isolation, then let `cargo check` drive the change list.
2. **Update `resolution.rs` emit sites** (two KO push sites; payload assembly is mechanical) — `cargo check` should re-green excluding pipeline & tests.
3. **Update `pipeline.rs` match arms** (six sites: four match arms renaming, two-plus emit sites for `UltimateUsed`). After this, headless `cargo check` is green.
4. **Update `turn_system/mod.rs:488` OnKO emit site** with empty/best-effort payload (or pull the bag from the actors query if the entity is in scope).
5. **Rename in existing tests** (six files, payload destructure `{ .. }` or update expected JSON).
6. **Add the two new tests** (`ultimate_event.rs`, `unit_died_payload.rs`).
7. **Verify**: `cargo test` and `cargo check --features windowed`.

The `UltimateUsed` work is fully independent of the `UnitDied` rename — they could be split into two tasks for cleaner commits.

### Verification Approach

- `cargo check` after step 1 should fail with N compile errors, each pointing at a `OnKO` consumer — useful as a worklist.
- `cargo test` after step 6 expects ≥ 72 tests green (per milestone success criteria) plus the two new ones (≥ 74 total).
- `cargo check --features windowed` must produce no new warnings (success criteria).
- For the `UltimateUsed` test: confirm `unit_id` equals the attacker's `UnitId` (not the target's), and that the event is emitted **once** per ultimate cast (not duplicated across fan-out per-target loops). The single emission lives in the once-per-cast hoist block — duplicate emission would be a regression to catch.
- For the `UnitDied` test: assert both fields of the payload; the `status_remaining` order should follow `StatusBag` insertion order (it's the iterator order of the `Vec<StatusInstance>` backing the bag).

## Constraints

- **Headless first** (CLAUDE.md): no winit/wgpu/egui imports may sneak into combat code paths.
- **Single source of truth**: emit events; do not duplicate state writes in match arms. The existing pattern (mutate world in the match arm, emit downstream via `event_writer`) must be preserved.
- **Determinism**: tests must not depend on wall-clock or unseeded RNG. The new tests follow the `apply_effects` direct-call style or Bevy-world style already in use (see `tests/ultimate_meter.rs` for the Bevy-world ultimate driving pattern; `tests/status_blessed_offensive.rs` for the direct-call style).
- **JSON serde compatibility**: `CombatEventKind` derives `serde::Serialize`. The rename will change the serialized `kind` discriminator from `"OnKO"` to `"UnitDied"`. Search for any external consumers of the JSONL stream (`src/combat/jsonl_logger.rs`) and any docs/examples that snapshot wire format.

## Common Pitfalls

- **Duplicate `UltimateUsed` emission.** The hoist block has multiple variants (single-target, Blast/AllEnemies, AllAllies, hop-loop) — each owns its own resource consumption and runs **once per cast**. The pitfall is emitting `UltimateUsed` per-target inside the fan-out loop instead. Anchor the emit next to the existing `UltGain` site in each block — those are already gated correctly.
- **`UnitDied` from inside the actors mutable loop.** `pipeline.rs:458, 975` is inside the per-target consume loop where the defender entity is being mutated (insert `Ko`). The match arm pattern there must use `{ .. }` to ignore the payload; the payload is filled upstream in `apply_damage_only`. Do not try to fetch the bag again at the pipeline site — risk of stale or borrow-checker issues.
- **`StatusBag::iter` clones.** Returns `&StatusInstance`; map to `inst.kind.clone()` for the Vec — `StatusEffectKind` is small enum, clone is cheap.
- **MEM001 reminder.** If any test or new system adds components to the resolution query, the local `ResolveActorsQuery` in `follow_up.rs` must be updated in lockstep. The current slice does not add components, but if a payload-emission helper needs new query access, this applies.
- **`turn_system/mod.rs:488` empty payload.** If the entity's `StatusBag` cannot be fetched at that emit site (likely a stunned/effect-driven death path), emit with empty defaults and write a one-line comment explaining why. Better than panicking or routing a new query just for this corner.

## Open Risks

- The fourth pipeline hoist block (hop loop ~line 1856-1861) emits `UltGain` but may not gate ult_effect the same way. Re-verify during execution whether `UltimateUsed` belongs there — a `PerHop` ultimate is unusual but legal in principle.
- `mod.rs:488`'s OnKO emit context (toughness break? status tick death?) is the one place we lose payload fidelity. Worth a quick code-read at planning time to decide whether to widen the surrounding query or accept empty defaults.

## Relevant Memories

- **MEM005 (pattern)** — single-target effect handlers follow KO-guard → compute → mutate → emit. The `apply_damage_only` UnitDied work fits the **emit** stage: don't reorder it.
- **MEM001 (gotcha)** — `follow_up.rs` has its own actors query; if we modify the resolution query (we don't here, but if a future task adds a `StatusBag` access there), the follow-up query needs matching updates.

## Sources

- Inline source review of `src/combat/events.rs`, `src/combat/resolution.rs`, `src/combat/turn_system/pipeline.rs`, `src/combat/turn_system/mod.rs`, `src/combat/status_effect.rs`. No external library docs required — pure local refactor.
