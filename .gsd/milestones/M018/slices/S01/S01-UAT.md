# S01: Time-manipulation split: AdvanceTurn / DelayTurn con cap ±50% e clamp [0,200] — UAT

**Milestone:** M018
**Written:** 2026-05-13T15:59:51.559Z

# UAT: S01 — Time-manipulation split AdvanceTurn / DelayTurn

**UAT Type:** Headless integration + CLI scenario

**Preconditions:**
- Rust toolchain active (see rust-toolchain.toml)
- `cargo build` compiles clean (no errors)
- Working directory: `/home/fabio/dev/bevyrogue`

---

## Checks

### 1. No legacy TurnAdvance in codebase

**Command:**
```bash
rg -n 'Effect::TurnAdvance\|CombatEventKind::TurnAdvance' src/ assets/ tests/
```

**Expected outcome:** Command returns exit 0 with zero matching lines. Any match is a regression.

---

### 2. Boundary test suite

**Command:**
```bash
cargo test --test turn_advance_split
```

**Expected outcome:** `test result: ok. 6 passed; 0 failed` — all boundary cases green.

**Cases verified:**
- (a) DelayTurn(80) capped → 50, AV change −5000 from MAX_AV
- (b) AdvanceTurn(80) capped → 50, AV change +5000
- (c) Double AdvanceTurn(50) from AV=10_000 → ceiling 20_000
- (d) Third AdvanceTurn(50) — no movement past ceiling
- (e) DelayTurn(50) from AV=2000 → floor 0
- (f) TempoResistance(0.25) reduces raw delay by 75% on delay path

---

### 3. M017 Slowed regression

**Command:**
```bash
cargo test --test status_slowed_delay && cargo test --test tempo_resistance
```

**Expected outcome:** Both test suites pass. AV outcome for Slowed (5000→2000) unchanged. Event variant is now `DelayTurn { amount_pct: 30 }` (not `TurnAdvance`).

---

### 4. CLI scenario with cap and floor evidence

**Command:**
```bash
cargo run --bin combat_cli -- --scenario advance-delay-cap
```

**Expected outcome:** Exit 0. Output includes 4 JSONL lines, one per step:
- Step 1: `AdvanceTurn`, amount_pct_requested=50, amount_pct_capped=50, av_pre=0, av_delta=5000, av_post=5000
- Step 2: `AdvanceTurn`, amount_pct_requested=50, amount_pct_capped=50, av_pre=5000, av_delta=5000, av_post=10000
- Step 3: `DelayTurn`, amount_pct_requested=80, **amount_pct_capped=50** ← cap visible, av_pre=5000, av_delta=−5000, av_post=0
- Step 4: `DelayTurn`, amount_pct_requested=50, amount_pct_capped=50, **av_delta=0** ← floor clamp visible

---

### 5. Full suite regression gate

**Command:**
```bash
cargo test 2>&1 | grep -E 'FAILED|test result'
```

**Expected outcome:** Zero `FAILED` lines. All `test result:` lines show `ok`. Minimum 500 tests passing total.

---

### 6. Windowed feature gate

**Command:**
```bash
cargo check --features windowed
```

**Expected outcome:** Clean compile — warnings only, zero errors.

---

## Edge Cases

- **TempoResistance on AdvanceTurn:** not applied (advance path is uncurved) — verified by boundary test (f) on the delay path only.
- **AV at ceiling before AdvanceTurn:** third advance in case (d) — no overflow, no panic.
- **AV at 0 before DelayTurn:** case (e) and CLI step 4 — clamp to 0, no negative AV.

---

## Not Proven By This UAT

- Multi-unit AV interaction under concurrent AdvanceTurn/DelayTurn in the same turn (S02+ scope)
- TargetShape-driven advance/delay chaining (S03/S04 scope)
- Windowed UI rendering of AdvanceTurn/DelayTurn events (visual only, feature-gated)

