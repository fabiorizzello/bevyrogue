---
id: T03
parent: S06
milestone: M004
key_files:
  - docs/uat/M004-vfx-signoff.md
  - scripts/capture-windowed-m004-vfx.sh
  - tests/windowed_only/vfx_asset_impact_render.rs
key_decisions:
  - Preserved the K001 honesty boundary: automated verification stops at framework/script/test proof and makes no claim that the manual `cargo winx` visual review has been completed.
  - Aligned the windowed-only Baby Flame proof with the current authored two-stage chain (`baby_flame.impact` central burst -> `baby_flame.impact_flash` fan-out) rather than treating `baby_flame.impact` itself as the shard fan-out.
duration: 
verification_result: passed
completed_at: 2026-05-25T20:39:30.929Z
blocker_discovered: false
---

# T03: Verified the S06 manual UAT framework, syntax-checked the K001 capture helper, updated a stale windowed-only Baby Flame proof to the current authored impact chain, and re-proved the full S05 regression set green without launching the windowed binary.

**Verified the S06 manual UAT framework, syntax-checked the K001 capture helper, updated a stale windowed-only Baby Flame proof to the current authored impact chain, and re-proved the full S05 regression set green without launching the windowed binary.**

## What Happened

First verified the two S06 deliverables already existed and were well-formed: `docs/uat/M004-vfx-signoff.md` still states the framework-complete / human-capture-pending K001 boundary with per-skill PASS/FAIL/WAIVED fields, and `scripts/capture-windowed-m004-vfx.sh` remains executable and explicitly forbids auto-mode invocation while teeing `cargo winx` output into the milestone-local `.gsd/.../uat-evidence/` directory. Then ran the requested non-windowed verification suite. The initial cargo run exposed one real regression in the proof surface: `tests/windowed_only/vfx_asset_impact_render.rs` still assumed `baby_flame.impact` was the shard fan-out, but the current authored asset now uses a two-stage chain where `baby_flame.impact` is the central static burst and its `on_expire` chains `baby_flame.impact_flash` for the radiating follow-through. I updated that single windowed-only proof file to match the current authored contract (central burst spawn plan, impact_flash fan-out scale/color assertions, and explicit projectile -> impact -> impact_flash chain assertions) without changing runtime behavior. After that, reran the full T03 verification suite and all required checks passed. Throughout, K001 was preserved: no `cargo winx` execution, no claim that the visual acceptance bar was met, and the human signoff/waiver remains pending by design.

## Verification

Confirmed the runbook is present and still advertises the human-only K001 boundary; confirmed the capture helper is executable, passes `bash -n`, and contains the `auto-mode must NOT invoke` banner; reran the full S05 headless/windowed-contract regression set with all commands exiting 0: `cargo test --test animation vfx_asset_load`, `cargo test --test animation vfx_asset_eval`, `cargo test --test animation render_no_vfx_kind_guard`, `cargo check --features windowed`, `cargo test --features windowed --test windowed_only vfx_asset_impact_render`, and `cargo test --features windowed --test windowed_only vfx_rendering_acceptance`. No windowed binary was launched, so the manual visual verdict remains pending/waived only when a human fills the runbook.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `test -s docs/uat/M004-vfx-signoff.md` | 0 | ✅ pass | 0ms |
| 2 | `test -x scripts/capture-windowed-m004-vfx.sh` | 0 | ✅ pass | 0ms |
| 3 | `bash -n scripts/capture-windowed-m004-vfx.sh` | 0 | ✅ pass | 1ms |
| 4 | `grep -q 'auto-mode must NOT invoke' scripts/capture-windowed-m004-vfx.sh` | 0 | ✅ pass | 1ms |
| 5 | `cargo test --test animation vfx_asset_load` | 0 | ✅ pass | 186ms |
| 6 | `cargo test --test animation vfx_asset_eval` | 0 | ✅ pass | 163ms |
| 7 | `cargo test --test animation render_no_vfx_kind_guard` | 0 | ✅ pass | 153ms |
| 8 | `cargo check --features windowed` | 0 | ✅ pass | 187ms |
| 9 | `cargo test --features windowed --test windowed_only vfx_asset_impact_render` | 0 | ✅ pass | 255ms |
| 10 | `cargo test --features windowed --test windowed_only vfx_rendering_acceptance` | 0 | ✅ pass | 247ms |

## Deviations

During verification, updated `tests/windowed_only/vfx_asset_impact_render.rs` because its assertions had drifted behind the current authored Baby Flame impact chain. This was a proof-surface correction only; no runtime/source behavior outside the test contract changed.

## Known Issues

Human visual signoff in `docs/uat/M004-vfx-signoff.md` remains pending by design under K001; auto-mode did not and must not claim PASS for the windowed visual bar.

## Files Created/Modified

- `docs/uat/M004-vfx-signoff.md`
- `scripts/capture-windowed-m004-vfx.sh`
- `tests/windowed_only/vfx_asset_impact_render.rs`
