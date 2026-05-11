# S01: Tentomon/Kabuterimon Battery Loop Blueprint — UAT

**Milestone:** M016
**Written:** 2026-05-09T11:58:07.640Z

# S01 UAT: Tentomon/Kabuterimon Battery Loop Blueprint

**Goal:** Verify the Battery Loop functions effectively for the Tentomon line using the new blueprint mechanism.

1. Start the combat CLI with the default preset containing Tentomon.
2. Attack an enemy using Tentomon Basic or Petit Thunder.
3. Observe the CLI dashboard output carefully.
4. Expect the `battery_loop` metrics (e.g. `static=1/3`) to increase correctly according to the action taken, indicating the blueprint translates the signals.
5. Check that after 3 attacks with `static_charge`, energy is granted to an ally properly without crashing or losing synchronization.
