# S06: Graph registry processes all matching graph events so Renamon sprite spawns — UAT

**Milestone:** M006
**Written:** 2026-05-27T08:06:52.196Z

# S06 UAT — Graph registry batch fix + Renamon sprite spawn

## UAT Type
Semi-automated: headless assertions are fully automated; windowed visual check is manual K001.

## Preconditions
- `cargo build` succeeds on the default (headless) profile.
- Both Agumon and Renamon assets are present under `assets/digimon/agumon/` and `assets/digimon/renamon/`.
- `AnimationGraphHandles` is populated at startup with handles for all configured graph RON files.

## Automated Checks (CI-safe)

### A1 — Starvation regression test
```
cargo test --test animation -- registry_starvation
```
**Expected:** `populate_graph_registries_starves_second_event_when_first_matches` passes (exit 0).
**Proves:** When a skill-path graph event and a stance-path graph event arrive in the same batch, both `SkillGraphRegistry` and `StanceGraphRegistry` are populated after one `app.update()` call.

### A2 — Full headless suite
```
cargo test
```
**Expected:** All test binaries exit 0, 0 failures.
**Proves:** The batch-fix introduced no headless regressions.

## Manual Check (K001 required)

### M1 — Renamon idle sprite present in windowed run
**Steps:**
1. `cargo run --features windowed` (or alias `cargo winx`).
2. Wait for the encounter scene to load.
3. Observe both combatant sprites.

**Expected:** Renamon's idle sprite renders on screen alongside Agumon's idle sprite. Neither sprite is invisible or replaced by a placeholder.

**Edge case:** If only one sprite appears (e.g. Renamon is invisible), check the console for the new warn-once message:
```
animation graph loaded but no registry entry could be built: graph_id=... path=...
```
This indicates a path mismatch in `StanceGraphPaths` or `SkillGraphPaths` for Renamon.

### M2 — No spurious warn-once in happy path
**Steps:** Same windowed run as M1.
**Expected:** Console contains no `animation graph loaded but no registry entry could be built` warnings for Agumon or Renamon assets.

## Not Proven By This UAT
- Renamon combat actions (skills, hurt, death) — covered by S07/S08.
- Three-or-more Digimon batch loading — the fix is general but only two-graph batches are tested.
- Hot-reload of graph assets while a match is in progress.
