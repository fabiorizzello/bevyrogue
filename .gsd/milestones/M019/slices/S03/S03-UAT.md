# S03: Effect::Cleanse { count: Option<u8> } primitive — UAT

**Milestone:** M019
**Written:** 2026-05-14T09:26:27.473Z

# UAT — S03: Effect::Cleanse Primitive

**UAT Type:** Integration (direct `apply_effects` calls, no Bevy world)
**Environment:** `cargo test` headless, deterministic seed

---

## Preconditions

- `cargo check --tests` exits 0 (no compile errors)
- Full test suite green before this UAT (no pre-existing failures)
- `tests/cleanse_effect.rs` present with 8 test cases

---

## Test Cases

### 1. count=Some(2) removes the 2 longest-duration debuffs

**Steps:**
1. Build a `StatusBag` with 3 debuffs: durations 5, 3, 1.
2. Call `apply_cleanse_only` with `count=Some(2)` on an alive unit.

**Expected:**
- `OnCleansed { kinds }` emitted with 2 entries (duration-5 and duration-3 removed).
- Debuff with duration 1 remains in the bag.
- No panic.

---

### 2. Tiebreak: lower insertion index removed first when durations equal

**Steps:**
1. Build a `StatusBag` with 3 debuffs: durations 3, 3, 1.
2. Call `apply_cleanse_only` with `count=Some(2)`.

**Expected:**
- The two duration-3 debuffs removed (lower insertion indices first among ties).
- Duration-1 debuff remains.
- `OnCleansed` emitted with 2 entries.

---

### 3. count=None removes all non-immune debuffs, Blessed survives

**Steps:**
1. Build a `StatusBag` with 2 debuffs and 1 Blessed (buff).
2. Call `apply_cleanse_only` with `count=None`.

**Expected:**
- Both debuffs removed; Blessed entry remains in bag.
- `OnCleansed { kinds }` contains exactly the 2 debuff kinds.

---

### 4. count=Some(0) is a no-op, empty OnCleansed emitted

**Steps:**
1. Build a `StatusBag` with 2 debuffs.
2. Call `apply_cleanse_only` with `count=Some(0)`.

**Expected:**
- Bag unchanged (2 debuffs still present).
- `OnCleansed { kinds: [] }` emitted (telemetry parity with OnHealed amount=0).

---

### 5. Blessed-only bag: no-op, empty OnCleansed emitted

**Steps:**
1. Build a `StatusBag` containing only Blessed.
2. Call `apply_cleanse_only` with `count=Some(2)`.

**Expected:**
- Blessed remains; `OnCleansed { kinds: [] }` emitted.

---

### 6. count exceeds debuff count: all debuffs removed, no panic

**Steps:**
1. Build a `StatusBag` with 2 debuffs.
2. Call `apply_cleanse_only` with `count=Some(10)`.

**Expected:**
- Both debuffs removed.
- `OnCleansed` with 2 entries emitted.
- No panic or out-of-bounds error.

---

### 7. KO target: silent no-op, no event emitted

**Steps:**
1. Build a unit with `hp=0` (KO).
2. Build a `StatusBag` with 2 debuffs.
3. Call `apply_cleanse_only`.

**Expected:**
- No `OnCleansed` event emitted.
- Bag unchanged.
- `sp_ok=true` returned.

---

### 8. Empty bag: empty OnCleansed emitted

**Steps:**
1. Build a unit with empty `StatusBag`.
2. Call `apply_cleanse_only` with `count=Some(2)`.

**Expected:**
- `OnCleansed { kinds: [] }` emitted (alive unit, no debuffs to remove).

---

## Edge Cases Covered

- Buff immunity (Blessed) — never removed, no kernel hardcoding
- count=None (remove all) vs count=Some(0) (no-op)
- count > actual debuffs (no panic)
- KO policy (silent no-op, no event)
- Empty bag (telemetry event still emitted)

---

## Not Proven By This UAT

- Bevy world / system-level event propagation to JSONL logger (covered by observability wiring in existing combat tests)
- Selective cleanse by `StatusEffectKind` (deferred to M021)
- Mixed Heal+Cleanse on a single skill at runtime (validator blocks at DSL level; M021 defers runtime mix)
- Cleanse over AllEnemies/Bounce target shapes (rejected by T01 validator)
