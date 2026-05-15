---
estimated_steps: 11
estimated_files: 1
skills_used: []
---

# T05: Integration capstone: Renamon kitsune_grace passive end-to-end + JSONL round-trip + negative guards

Why: This is the slice demo gate (per ROADMAP: 'Renamon kitsune_grace verde + JSONL Blueprint round-trip + debug_assert mismatch'). It exercises T01–T04 together via the canonical reactive passive — the cleanest first proof for the whole stack.

Do:
1. Create tests/passive_kitsune_grace.rs (clone setup pattern from tests/timeline_chain_bolt_port.rs:35-57).
2. Build a minimal CompiledTimeline for kitsune_grace in the test file (no production blueprint module yet — that's S10): listener-shaped FSM `Dormant → Proc → Resolve → Dormant`. Entry beat: guard `ally non-self alive` (target from signal payload differs from self.owner AND is ally). Effect beat: emit `Intent::AdvanceTurn { target: self.owner, delta_pct: 10 }` (or whatever the canonical AV-advance intent shape is — verify in src/combat/api/intent.rs; if AdvanceTurn isn't yet implemented as an applier-handled variant, substitute a `SetBlueprintState{key:"kitsune_grace/triggered",value:1}` sentinel and assert the state map write as the trigger-fired observable).
3. In the test: spawn Renamon (Ally), Patamon (Ally), Enemy. Register `("kernel","ult_used")` in SignalTaxonomy (already done at plugin build). Insert a PassiveRunner for Renamon into PassiveListeners with triggers=[("kernel","ult_used")].
4. Test case A (positive — ally non-self): write CombatEvent::UltimateUsed{unit_id:patamon_id}; run `app.update()` until queues drain; assert (a) SignalBus drained; (b) BlueprintState shows trigger fired OR Renamon's ActionValue advanced by ~10% of MAX_AV; (c) Messages<CombatEvent> contains OnKernelTransition::Blueprint event for kitsune_grace's own re-emission (if the timeline emits one — otherwise only the kernel/ult_used signal round-trips).
5. Test case B (negative — self-ult): write CombatEvent::UltimateUsed{unit_id:renamon_id}; assert no AV change / no sentinel write (guard rejects self).
6. Test case C (negative — cross-team): write CombatEvent::UltimateUsed{unit_id:enemy_id}; assert no AV change / no sentinel write.
7. Test case D (JSONL round-trip): for the positive case's emitted OnKernelTransition::Blueprint CombatEvent, assert `serde_json::from_str::<CombatEvent>(&serde_json::to_string(&event).unwrap()).unwrap() == event`.
8. Test case E (debug_assert mismatch) — separate `#[should_panic(expected = "unregistered signal")]` test: push `Intent::BlueprintSignal{owner:"nonexistent",...}` directly through intent_applier and confirm the debug_assert! fires.

Done-when: `cargo test --test passive_kitsune_grace` green (all 5 cases). Full suite `cargo test` green (no regressions in S01–S03 fixtures, follow_up, holy_support, etc.). `cargo check --features windowed` clean. P001 guard: `rg "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/api/` returns 0 hits (P001 kernel-stays-blueprint-free).

## Inputs

- `tests/timeline_chain_bolt_port.rs`
- `tests/timeline_mode_parity.rs`
- `src/combat/api/signal.rs`
- `src/combat/api/passive_runner.rs`
- `src/combat/api/event_bridge.rs`
- `src/combat/api/applier.rs`
- `src/combat/api/intent.rs`
- `src/combat/api/timeline.rs`
- `src/combat/api/blueprint_state.rs`
- `src/combat/plugin.rs`
- `src/combat/events.rs`
- `src/combat/kernel.rs`
- `docs/future_design_draft/digimon/renamon/04_passive_kitsune_grace.md`
- `.gsd/milestones/M021/slices/S04/S04-RESEARCH.md`

## Expected Output

- `tests/passive_kitsune_grace.rs`

## Verification

cargo test --test passive_kitsune_grace && cargo test && cargo check --features windowed && rg "TwinCore|BatteryLoop|HolySupport|PredatorLoop|PrecisionMindGame|KitsuneGrace" src/combat/api/

## Observability Impact

Integration capstone exercises full JSONL pipeline: CombatEvent::UltimateUsed → bridge → Signal → PassiveRunner → Intent → applier → CombatEvent::OnKernelTransition → JSONL. Validates that every reactive primitive emits the right event for 3am-diagnosable forensics. Negative cases (self/enemy guards) ensure the passive does not silently mis-fire — observability includes the *absence* of spurious events.
