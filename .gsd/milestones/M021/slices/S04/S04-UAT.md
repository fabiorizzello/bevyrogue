# S04: SignalBus + PassiveRunner + Ult instant + Intent::BlueprintSignal dispatcher — UAT

**Milestone:** M021
**Written:** 2026-05-15T14:34:37.114Z

## UAT: Renamon kitsune_grace reactive passive

**UAT Type:** integration / end-to-end reactive pipeline

### Preconditions
- Build is on the S04 state with SignalBus, PassiveRunner, BlueprintSignal dispatch, and UltimateUsed bridge available.
- A test app can spawn Renamon, Patamon, and a non-ally enemy.
- JSONL logging is enabled for combat events.

### Steps
1. Start an encounter with Renamon as the passive owner, Patamon as the ally ult source, and one enemy unit.
2. Inject or simulate `CombatEvent::UltimateUsed` for the ally unit.
3. Advance one app tick so the bridge and runner systems process the event.
4. Observe the SignalBus drain and the emitted Blueprint kernel transition event.
5. Confirm Renamon’s AV advances by 10% of `MAX_AV`.
6. Repeat with a self-ult event and confirm kitsune_grace does not fire.
7. Repeat with an enemy-ult event and confirm kitsune_grace does not fire.
8. Serialize the emitted `CombatKernelTransition::Blueprint` event to JSON and deserialize it back.
9. Trigger an unregistered Blueprint signal in debug mode and confirm it trips the debug assertion path.

### Expected outcomes
- Ally ult triggers the passive exactly once.
- Renamon’s AV increases by 10% of `MAX_AV`.
- Self-ult and enemy-ult do not trigger the passive.
- The Blueprint transition round-trips through JSONL serialization.
- Unregistered signal emission is rejected by the taxonomy guard in debug builds.

### Edge cases
- Multiple ult events in one tick should still respect SignalBus queue order.
- Signal cascade must stop at the MAX_HOPS circuit breaker.
- Release builds should degrade unregistered emission to warn-and-drop behavior instead of panicking.

### Not proven by this UAT
- This UAT does not prove broader passive coverage beyond kitsune_grace.
- It does not validate downstream S05 compilation of additional built-in extension functions.
- It does not prove full milestone completion for M021, only the S04 reactive kernel slice.
