# S02: TargetShape resolver: Blast e AoE(All) con tie-break slot_index — UAT

**Milestone:** M018
**Written:** 2026-05-13T20:17:03.486Z

# UAT: S02 — TargetShape Blast + AoE(All) Resolver

**UAT Type:** Integration / CLI-driven headless

## Preconditions

- Rust toolchain installed (see `rust-toolchain.toml`)
- Working directory: `/home/fabio/dev/bevyrogue`
- Branch: `milestone/M018`
- `cargo build --bin combat_cli` succeeds

## Numbered Steps + Expected Outcomes

### 1. New test binaries all pass

```bash
cargo test --test target_shape_blast_spillover --test target_shape_aoe_all_order --test slot_index_tiebreak
```

**Expected:** `test result: ok` for all three binaries; 6 total tests, 0 failed.

### 2. Full test suite green (no regressions)

```bash
cargo test 2>&1 | grep -E '(FAILED|test result.*failed: [^0])'
```

**Expected:** No output — zero failures across all binaries.

### 3. M017 regression tests still green

```bash
cargo test --test status_slowed_delay --test tempo_resistance --test turn_advance_split
```

**Expected:** `test result: ok` for all three (1+14+6 tests).

### 4. aoe-blast CLI scenario determinism gate

```bash
cargo run --bin combat_cli -- --scenario aoe-blast 2>/dev/null > /tmp/aoe1.jsonl
cargo run --bin combat_cli -- --scenario aoe-blast 2>/dev/null > /tmp/aoe2.jsonl
diff /tmp/aoe1.jsonl /tmp/aoe2.jsonl && echo "DETERMINISM: PASS"
```

**Expected:** `DETERMINISM: PASS`; no diff output.

### 5. aoe-blast JSONL shows resolved target list and per-target damage

```bash
cargo run --bin combat_cli -- --scenario aoe-blast 2>/dev/null | head -10
```

**Expected output includes:**
- A line like `Resolved targets (slot_index asc): ["GobA(slot0)", "GobB(slot1)", "GobC(slot2)"]` (Blast resolving primary + adjacents)
- Per-target damage lines for each resolved unit

### 6. Windowed feature compiles cleanly

```bash
cargo check --features windowed 2>&1 | tail -3
```

**Expected:** `Finished` line, no `error` lines.

### 7. Blast fixture skill loaded by RON parser

```bash
cargo test --test target_shape_blast_spillover -- --nocapture 2>&1 | grep -i blast | head -5
```

**Expected:** Test output references Blast shape without parse errors.

## Edge Cases Covered

- KO'd adjacent slot on Blast: absorbed (skipped silently, no extra event) — verified by `target_shape_blast_spillover` edge-slot test case
- Blast at slot 0 (no left neighbor): only right adjacent + primary resolved — verified in table tests
- AllEnemies with all-alive vs one-KO'd: only alive enemies resolved, slot_index order preserved
- Resource economy: SP/ult/streak consumed once per cast even for 3-target Blast — verified by `target_shape_aoe_all_order`

## Not Proven By This UAT

- ApplyStatus effects are not fanned out per-target in multi-target loop (only damage is); status fan-out deferred to future task
- Bounce(N) chain (path-dependent) is S03 scope — not covered here
- AdjLowest / LowestHpPct / Random selectors are S04 scope — not covered here
- Windowed UI rendering of multi-target damage events (presentation layer, not combat logic)

