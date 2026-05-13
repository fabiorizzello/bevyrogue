# S06 Research — Observability: canon JSONL log + ValidationSnapshot

**Depth:** light (S01–S05 already migrated all source/test naming to the canon vocabulary; this slice is essentially a verification harness + a single additive snapshot field).

## Summary

S01–S05 already migrated `StatusEffectKind` (`Heated`/`Chilled`/`Paralyzed`/`Slowed`/`Blessed` + reserved `Burn`/`Shock`) across `src/` and `tests/`. The `serde::Serialize` derive on the enum emits bare canonical variant names (e.g. `"Heated"`), and all five status-related `CombatEventKind` variants (`OnStatusApplied`, `OnStatusResisted`, `OnStatusTick`, `OnStatusExpired`, plus `kit.rs` `OnStatusApplied(StatusEffectKind)` trigger payload) carry that enum, so JSONL output is **already canon-clean by construction**. What S06 still owes:

1. A **scripted scenario** (in-process integration test, or a CLI subprocess test) that applies all five active canon statuses on distinct units, captures the JSONL line stream, and asserts zero matches against the legacy vocabulary `Burn|Freeze|Shock|DeepFreeze`.
2. A new `statuses_per_unit` field on `ValidationSnapshot` (or per-unit on `ValidationUnitSnapshot`) sourced from `StatusBag`, asserted deterministic by a test fixture.

No new event variants. No new RON content. No follow-up M020 surface (source attribution stays out per D004 / context "Source attribution differita a M020").

## Implementation Landscape

### Producer surface (already canon — read-only verify)

- `src/combat/status_effect.rs:11-23` — `StatusEffectKind` derives `Serialize/Deserialize`, no `#[serde(rename = …)]`, so JSONL string forms are exactly `"Heated"`, `"Chilled"`, `"Paralyzed"`, `"Slowed"`, `"Blessed"`, `"Burn"`, `"Shock"`. Reserved variants are never instantiated by the apply pipeline (RON validator rejects ids `burn`/`shock` at load).
- `src/combat/events.rs:84-91` — `CombatEventKind::OnStatusApplied { kind }` and `OnStatusResisted { kind }` carry the canon enum.
- `src/combat/events.rs:48-56` — `OnStatusTick { kind, turns_left }` and `OnStatusExpired { kind }`.
- `src/combat/turn_system/pipeline.rs:752, 772` — emission sites for `OnStatusApplied` / `OnStatusResisted` (post-accuracy roll).
- `src/combat/turn_system/mod.rs:517-530, 552-581` — emission of `OnStatusTick` / `OnStatusExpired` plus Heated DoT (`OnDamageDealt{damage_tag:Fire}`); Slowed delay surface emits `TurnAdvance{amount_pct:-30}`.
- `src/combat/jsonl_logger.rs:7-18` — generic `serde_json::to_string(event)` over `CombatEvent`. Single file, 18 lines, env-gated `BEVYROGUE_JSONL`. **No status-aware code path** — touching it is unnecessary unless we want to add status snapshot framing alongside events (out of scope; ValidationSnapshot is the snapshot surface).
- `src/combat/log.rs:1-73` — `ActionLog` / `LogEntry`. **Does not currently carry status events** (only BasicHit/Break/Ko/Revive/ActionFailed/TurnAdvance). Out of scope to extend — JSONL is the canonical event log; `ActionLog` is the UI tail cache.

### Consumer surface (additive change)

- `src/combat/observability.rs:29-44` — `ValidationSnapshot` shape: no `statuses_per_unit` today. Two viable shapes:
  - **A. Per-unit (preferred):** add `pub statuses: Vec<ValidationStatusSnapshot>` to `ValidationUnitSnapshot` (line 120-132). Co-locates with HP/Ultimate/stun.
  - **B. Top-level map:** add `pub statuses_per_unit: Vec<(UnitId, Vec<ValidationStatusSnapshot>)>` to `ValidationSnapshot`. Matches the literal DoD wording but duplicates the unit indirection.
  - Recommend **A**; the slice DoD wording "ValidationSnapshot.statuses_per_unit" is descriptive of intent, not of the exact field name.
- `src/combat/observability.rs:220-263` — units query loop. Extend the query tuple with `Option<&StatusBag>`; map each `StatusInstance` to `ValidationStatusSnapshot { kind: StatusEffectKind, duration_remaining: u32 }`. Sort by `kind` discriminant or enum-variant name for determinism.
- `src/combat/observability.rs:310+` — `format_validation_snapshot` text builder. Append `statuses=…` token per unit (cheap, useful for snapshot diffing in tests).
- `tests/validation_snapshot.rs` — existing fixture-driven test (`snapshot_contract_covers_promised_fields_and_shape`). Extend the fixture to spawn one unit per canon kind with a known duration; assert the snapshot's per-unit `statuses` matches a hand-rolled expected vector.

### Scripted scenario test surface

Two viable patterns in-tree:

- **In-process app (recommended):** mirrors `tests/status_amp_pipeline.rs`, `tests/status_paralyzed_skip.rs`, `tests/status_slowed_delay.rs`. Build a minimal `App`, register `add_message::<ActionValueUpdated>()`, spawn 5 units (one per status), apply each via `StatusBag::apply` (skipping the accuracy roll for determinism), drive `advance_turn_system`/`resolve_action_system` enough to emit `OnStatusApplied`+`OnStatusTick`+`OnStatusExpired`+DoT+TurnAdvance. Collect the `CombatEvent` stream from a `MessageReader`, serialize each to JSON via `serde_json::to_string`, assert no string match for `/(?i)\bfreeze\b|\bdeepfreeze\b/` and that the only `Burn|Shock` substrings (if any) are from `damage_tag:Fire` (false positive avoidance: use word-boundary anchored regex against `kind:"…"` payload, **not** raw substring on `Fire`/etc).
- **CLI subprocess:** mirrors `tests/combat_cli_shared_surface.rs`. The default `BEVYROGUE_CLI_PROOF=1` run does **not** apply any status today (verified: `BEVYROGUE_JSONL=1 BEVYROGUE_CLI_PROOF=1 cargo run --bin combat_cli` yields zero `OnStatus*` lines). A subprocess test would require either a new CLI flag/preset that scripts all-5-status apply, or a dedicated encounter. Heavier than in-process; skip unless explicitly requested.

### Grep regex for "no legacy leak"

Anchor on `kind:"…"` payload to avoid false positives from `damage_tag:"Fire"` (which is *not* a legacy status name but contains "Fire"):

```text
# Forbidden in JSONL output:
"kind":"Freeze"     "kind":"DeepFreeze"
# Reserved-but-not-applied (must also be absent at runtime):
"kind":"Burn"       "kind":"Shock"
```

The reserved variants are syntactically valid serde output but the RON validator (S01/T02) rejects them at load and the apply pipeline never constructs them — so a runtime grep for `"kind":"Burn"` / `"kind":"Shock"` in the scenario JSONL is the cleanest assertion. Use exact-quoted byte matches, not regex word boundaries.

## Natural Seams (for planner decomposition)

Three independent units of work, ordered by risk-reduction value:

1. **Seam A — `ValidationSnapshot` status field (low risk, additive):**
   - Add `ValidationStatusSnapshot { kind, duration_remaining }` type.
   - Extend `ValidationUnitSnapshot` with `statuses: Vec<ValidationStatusSnapshot>`.
   - Extend the units query tuple at `observability.rs:220` with `Option<&StatusBag>` and populate.
   - Update `format_validation_snapshot` to print `statuses=[…]` per unit.
   - Update the existing `tests/validation_snapshot.rs` fixture: spawn one unit with `StatusBag` pre-loaded with all 5 canon kinds, assert sorted output deterministic.
   - **Files:** `src/combat/observability.rs`, `tests/validation_snapshot.rs`.

2. **Seam B — Scripted scenario integration test (medium risk, new file):**
   - New file `tests/status_observability_canon.rs`.
   - Spawn 5 units, apply one canon status each via `StatusBag::apply` (deterministic, bypasses RNG).
   - Drive enough ticks to produce `OnStatusApplied` + `OnStatusTick` + (Heated) `OnDamageDealt{damage_tag:Fire,amount:4}` + (Slowed first apply) `TurnAdvance{amount_pct:-30}` + (Paralyzed) `OnActionFailed{reason:"paralyzed"}`.
   - Drain `CombatEvent` stream into a `Vec<String>` via `serde_json::to_string`.
   - Assert: every canon kind name appears at least once; substring `"kind":"Freeze"`, `"kind":"DeepFreeze"`, `"kind":"Burn"`, `"kind":"Shock"` count == 0.
   - Capture a `ValidationSnapshot` at scenario end; assert per-unit `statuses` matches expected (deterministic).
   - **Files:** `tests/status_observability_canon.rs` (new), no `src/` changes if Seam A landed first.

3. **Seam C — Optional grep guard on the live CLI stream:** if planner wants belt-and-braces, add a second test that runs the `combat_cli` binary headless with `BEVYROGUE_CLI_PROOF=1` and asserts the stdout contains zero `"kind":"Freeze"` / `"kind":"DeepFreeze"` / `"kind":"Burn"` / `"kind":"Shock"` substrings. This is **regression coverage**, not DoD coverage — the default encounter doesn't apply statuses, so it only protects against future encounter additions accidentally re-introducing legacy names.

Seams A and B are independent; either could land first. C depends on neither but adds little (default encounter has no status apply).

## First Proof

**Land Seam A first.** It's the smallest patch (≈30 lines) and unblocks deterministic snapshot assertion in Seam B. Verification command: `cargo test --test validation_snapshot` plus full `cargo test`.

After A, write Seam B with the snapshot assertion using the now-populated field. Verification: `cargo test --test status_observability_canon` plus full `cargo test`.

## Verification

Slice DoD per roadmap:

> Scripted scenario CLI: applica Heated + Chilled + Paralyzed + Slowed + Blessed su units diversi → JSONL log analizzato via grep test, zero match su vocabolario legacy. ValidationSnapshot.statuses_per_unit deterministico in test fixture.

Translated to concrete commands:

- `cargo check` — clean.
- `cargo test --test validation_snapshot` — extended fixture asserts per-unit `statuses` ordering deterministic.
- `cargo test --test status_observability_canon` — new test: 5 statuses driven, JSONL stream emitted, substring guards green.
- `cargo test` (full suite) — zero regressions on S01–S05 tests (`status_amp_pipeline`, `status_paralyzed_skip`, `status_slowed_delay`, `status_blessed_offensive`, `status_blessed_ult_charge`, `status_blessed_cleanse_immune`, `status_refresh_max_dur`, `status_multi_kind_coexist`, `status_cleanse_policy`, `combat_coherence`, `follow_up_chains`, `form_identity`).
- Grep guard (informational): `rg '\bFreeze\b|\bDeepFreeze\b' src/ tests/ | wc -l == 0`; reserved `Burn|Shock` matches only in `status_effect.rs` declaration site and `skills_ron.rs` validator allow-list code per S01-S04 baseline.

## Constraints & Watch-outs

- **Headless-first (D008):** no `windowed` feature, no winit/wgpu. All tests already follow this pattern.
- **Determinism (CLAUDE.md):** use `StatusBag::apply` directly, bypassing the accuracy roll, OR pre-seed `CombatRng::from_seed(0)` as `tests/status_slowed_delay.rs` does. Do **not** call wall-clock or unseeded RNG.
- **`add_message::<ActionValueUpdated>()` registration gotcha** (from S03 forward intel): any test app that runs `advance_turn_system` must register this MessageWriter or assertion harness panics. Pattern in `tests/status_paralyzed_skip.rs` / `tests/status_slowed_delay.rs`.
- **`get_cursor_current()` not `get_cursor()`** (from S04 forward intel): when reading `CombatEvent` post-loop, initialize the MessageCursor with `events.get_cursor_current()`; `get_cursor()` starts at position 0 and double-counts due to the 2-frame message buffer.
- **Substring matching on `Fire` is a false-positive trap.** The `damage_tag` field also serializes to `"Fire"`. Always anchor the legacy-vocab guard on the `kind` discriminant: `"kind":"Freeze"`, `"kind":"DeepFreeze"`, `"kind":"Burn"`, `"kind":"Shock"` — these are unambiguous in the `CombatEventKind::OnStatus*` payloads.
- **Reserved Burn/Shock are valid serde output if instantiated.** Pipeline never instantiates them (RON validator gates at load). The runtime grep guard on JSONL still treats them as "must not appear" — that's the canon read of §H.1 in v0.
- **`source` attribution stays out** (D004, M020 deferred): do not extend `OnStatusApplied` payload with a `source_blueprint_id` field. S06 is purely renaming/snapshotting, not bus shape changes.
- **`ActionLog`/`LogEntry` is not the JSONL surface.** It's a 5-entry UI tail. Do not extend it with status entries — JSONL via `CombatEvent` is the canonical event log.
- **`format_validation_snapshot` is consumed by `combat_cli` (line 23 of `src/bin/combat_cli.rs`).** Adding a `statuses=…` token is safe (additive); confirm no test in `tests/combat_cli_shared_surface.rs` does an exact-string assertion on the format output (a quick grep shows only `assert_contains`/`assert_not_contains`, both substring-tolerant).

## Skills Discovered

None. The work is in-tree Rust/Bevy with established local patterns from S03/S04. The bundled `bevy-ecs-expert` skill could be invoked by the planner if query-tuple changes hit borrow-checker surprises, but the precedent at `observability.rs:220` is unambiguous.

## Sources

- `src/combat/status_effect.rs:11-23` — canon enum + serde derive.
- `src/combat/events.rs:48-91` — status event variants carrying `StatusEffectKind`.
- `src/combat/jsonl_logger.rs:7-18` — JSONL writer (no status-specific code).
- `src/combat/observability.rs:29-44, 120-132, 158-308, 310+` — `ValidationSnapshot` shape, capture path, formatter.
- `src/combat/turn_system/pipeline.rs:752, 772` — emission of `OnStatusApplied`/`OnStatusResisted`.
- `src/combat/turn_system/mod.rs:517-581` — tick + expire emission and Heated DoT branch.
- `tests/validation_snapshot.rs:1-80` — existing fixture pattern.
- `tests/combat_cli_shared_surface.rs:1-60` — CLI subprocess test pattern (reference; not preferred for S06).
- `tests/status_paralyzed_skip.rs`, `tests/status_slowed_delay.rs`, `tests/status_amp_pipeline.rs` — in-process app harness patterns recommended for Seam B.
- `.gsd/milestones/M017/M017-CONTEXT.md` — D004, D008, D009; S06 acceptance text.
- S04 summary (forward intel: `get_cursor_current()` MessageCursor pattern, `add_message::<ActionValueUpdated>()` registration).
