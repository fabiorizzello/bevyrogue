# S07: Modifier pipeline + Migrate 6 passive canon — UAT

**Milestone:** M021
**Written:** 2026-05-16T11:58:36.480Z

# UAT — S07: Modifier pipeline + Migrate 6 passive canon

**UAT Type:** Automated integration (no human UAT required per slice plan)

## Preconditions
- Clean checkout of `milestone/M021` branch
- `cargo test` compiles and links without errors
- `cargo check --features windowed` produces no new warnings

## Test Cases

### TC-01: Composite passive event routing
**Command:** `cargo test --test passive_event_filters`

**Steps:**
1. Run the test suite.
2. Confirm all sub-tests pass: composite `all`/`any` filter matching, same-frame cascade, and the 256-hop loop breaker.

**Expected:** All tests pass, exit 0.

---

### TC-02: Pre-damage Block Reaction pipeline
**Command:** `cargo test --test block_reaction_pipeline`

**Steps:**
1. Run the test suite.
2. Confirm the baseline no-op path (no modifier) produces unmodified damage.
3. Confirm the armed path halves incoming damage before DR.
4. Confirm event ordering: `IncomingDamage` → `OnDamageDealt` → `BlockReactionTriggered`.
5. Confirm replay with the same seed produces identical results.

**Expected:** All tests pass, exit 0.

---

### TC-03: Agumon, Gabumon, Patamon, Renamon passive canon
**Command:** `cargo test --test passive_canon_support`

**Steps:**
1. Run the test suite.
2. Confirm Twin Flame, Holy Support, Predator Loop, and Kitsune Grace each install at plugin boot.
3. Confirm blueprint reactions fire and guard-state writes are correct.

**Expected:** All tests pass, exit 0.

---

### TC-04: Dorumon + Tentomon reactive passives — deterministic Block Reaction
**Command:** `cargo test --test passive_reactive_canon`

**Steps:**
1. Run the test suite.
2. Confirm Dorumon's enemy-kill listener fires and routes through PassiveRunner.
3. Confirm Tentomon's block arm triggers `BlockReactionTriggered` on success.
4. Confirm the no-proc guard path is respected (no event when not armed).
5. Confirm the seed search finds a deterministic armed-path result.

**Expected:** All tests pass, exit 0.

---

### TC-05: Full regression suite
**Command:** `cargo test`

**Steps:**
1. Run the entire test suite.
2. Confirm no previously passing tests regressed.

**Expected:** All tests pass, exit 0.

---

### TC-06: Windowed feature gate
**Command:** `cargo check --features windowed`

**Steps:**
1. Run the windowed check.
2. Confirm no new warnings or errors introduced by S07 changes.

**Expected:** Exit 0, no new warnings.

---

## Edge Cases Covered
- One-pass outer-cycle stop prevents passive timelines from spinning on stale state reads.
- `BlockReactionTriggered` is emitted exactly once per damage application when a passive modifier is consumed (not repeatedly per modifier layer).
- Passive listeners installed at boot are not duplicated across test runs (canonical UnitId guards).

## Not Proven By This UAT
- End-to-end roster-driven Digimon instantiation (S08–S10).
- UI preview damage stream (S11).
- Final kernel digimon-free grep verification (S10).
- Human-observable play session with windowed UI.
