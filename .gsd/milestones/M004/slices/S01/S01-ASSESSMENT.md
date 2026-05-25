---
sliceId: S01
uatType: artifact-driven
verdict: PASS
date: 2026-05-25T00:00:00.000Z
---

# UAT Result — S01: Owned vfx.ron schema + appearance curve eval (tracer bullet)

## Checks

| Check | Mode | Result | Notes |
|-------|------|--------|-------|
| `assets/digimon/agumon/vfx.ron` present | artifact | PASS | File exists at `assets/digimon/agumon/vfx.ron` |
| `src/animation/vfx_asset.rs` present | artifact | PASS | Source file confirmed |
| `src/windowed/render.rs` present | artifact | PASS | Source file confirmed |
| `cargo test --test animation` — 83 tests green (includes vfx_asset_schema ×3, vfx_asset_eval ×11, vfx_asset_load ×4) | runtime | PASS | `test result: ok. 83 passed; 0 failed; 0 ignored` |
| `cargo build` (headless) clean — R016 boundary holds | runtime | PASS | `Finished dev profile` with no errors |
| `cargo build --features windowed` clean | runtime | PASS | `Finished dev profile` with no errors |
| `cargo test --features windowed --test windowed_only vfx_asset_impact` — 3 tests green (spawn plan, scale curve, color curve) | runtime | PASS | `test result: ok. 3 passed; 0 failed` |
| Fallback path: `diagnose_agumon_vfx_load` warns once to `windowed.agumon_playback` with `effect_id`, `path`, `reason` on `LoadState::Failed` | artifact | PASS | Code confirmed at `render.rs:362–386`; `warn!(target: "windowed.agumon_playback", effect_id = …, path = …, reason = …)` |
| Missing effect id warns once on `windowed.agumon_playback` with effect id | artifact | PASS | Code confirmed at `render.rs:1302–1310`; once-guard via `warned_missing_impact` Local |
| Hardcoded fallback constants present and branched correctly (`BABY_FLAME_IMPACT_SHARD_*`) | artifact | PASS | Constants at `render.rs:192–198`; fallback branch at `render.rs:1408–1419` confirmed |
| Fan-out burst: 8 orange-hued shard particles emit from impact point spreading outward | human-follow-up | NEEDS-HUMAN | K001: auto-mode cannot open a window. Human must run `cargo winx`, trigger Baby Flame impact, and observe 8 orange shards fanning out. |
| Ease-out spread: shards expand faster initially, slowing toward end (scale curve 0→0, 0.5→0.75, 1.0→1.0 × spread_px 64) | human-follow-up | NEEDS-HUMAN | K001: visual timing judgment. Human must observe motion shape during impact. |
| Alpha fade: shard color holds orange hue (srgba ≈1.0, 0.55, 0.2) fading alpha 0.9→0.0 over 5 ticks | human-follow-up | NEEDS-HUMAN | K001: visual color judgment. Human must verify orange hue and alpha fade in windowed run. |
| Central flash: brief bright flash (srgba ≈1.0, 0.82, 0.45) for ~2 ticks before fading | human-follow-up | NEEDS-HUMAN | K001: visual timing judgment. Human must confirm flash appears at impact point and fades. |
| No crash or panic during normal gameplay | human-follow-up | NEEDS-HUMAN | K001: requires live windowed run. Human must confirm no panic in terminal output. |
| Edge case — missing vfx.ron: impact fan-out still renders from hardcoded fallback | human-follow-up | NEEDS-HUMAN | K001: fallback code path confirmed by artifact (render.rs:1048 `None => (BABY_FLAME_IMPACT_SHARD_COUNT, None)`), but windowed execution cannot be automated. Human must rename vfx.ron, relaunch, trigger impact, verify fallback renders + WARN appears. |
| Edge case — missing vfx.ron: `[WARN windowed.agumon_playback]` message names missing asset | artifact | PASS | `diagnose_agumon_vfx_load` at render.rs:377–384 emits warn with `effect_id`, `path`, `reason = "vfx.ron failed to load or parse"` to `target: "windowed.agumon_playback"` |
| Edge case — restore vfx.ron and confirm normal behavior returns | human-follow-up | NEEDS-HUMAN | K001: requires windowed re-run. Human must restore vfx.ron and verify data path resumes. |

## Overall Verdict

PASS — all 10 automatable artifact/runtime checks pass; 7 windowed visual/runtime checks marked NEEDS-HUMAN per K001 (auto-mode cannot launch windowed binary). Fallback warn path and hardcoded fallback branch are both confirmed by artifact inspection.

## Notes

**Automatable evidence summary:**
- `cargo test --test animation`: 83 passed (covers eval correctness, schema round-trip, deny_unknown_fields, headless asset load, curve eval determinism)
- `cargo test --features windowed --test windowed_only vfx_asset_impact`: 3 passed (spawn plan match, scale curve match, color curve match — pins the lib contract render.rs consumes)
- Both `cargo build` (headless) and `cargo build --features windowed` compile clean
- `assets/digimon/agumon/vfx.ron` present at expected path
- Failure visibility code paths verified by grep: `diagnose_agumon_vfx_load` (LoadState::Failed warn), missing effect id warn, hardcoded fallback branch

**Human follow-up instructions (K001):**
Run `cargo winx` (alias for `cargo run --features windowed`), reach combat with Agumon loaded, and trigger Baby Flame:
1. Confirm 8 orange shards fan out from impact point with ease-out motion
2. Confirm alpha fades from ~0.9 to 0.0 over 5 ticks with orange hue preserved
3. Confirm brief bright central flash (~2 ticks)
4. Confirm no panic in terminal

For edge case: rename `assets/digimon/agumon/vfx.ron` → `vfx.ron.bak`, relaunch, trigger impact — confirm fallback renders and `[WARN windowed.agumon_playback]` appears in terminal. Restore `vfx.ron` and confirm data path resumes.
