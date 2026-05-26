---
id: T02
parent: S04
milestone: M006
key_files:
  - src/windowed/render.rs
  - src/windowed/digimon/agumon/mod.rs
key_decisions:
  - Agumon-owned windowed presentation data populates generic engine registries from `src/windowed/digimon/agumon/mod.rs`, while `src/windowed/render.rs` remains species-agnostic and drives VFX lifecycle from registry data only.
duration: 
verification_result: passed
completed_at: 2026-05-26T13:23:43.807Z
blocker_discovered: false
---

# T02: Moved Agumon enoki effect ids/paths plus on-enter, release, arrival, and detonate VFX spawning into `src/windowed/digimon/agumon/mod.rs`, with `src/windowed/render.rs` consuming only generic registries.

**Moved Agumon enoki effect ids/paths plus on-enter, release, arrival, and detonate VFX spawning into `src/windowed/digimon/agumon/mod.rs`, with `src/windowed/render.rs` consuming only generic registries.**

## What Happened

This task’s code was already present on disk when I entered the auto-fix loop, but the canonical completion artifact for T02 had never been written. I verified the implementation matches the task contract: `src/windowed/render.rs` defines and init-loads the generic `EnokiVfxRegistry`, `OnEnterEffectRegistry`, `SkillReleaseEffectRegistry`, and `DetonateEffectRegistry`; `EnokiEffect` carries both `path` and `EnokiLifecycle`; projectile travel stores `on_arrival` data instead of chaining a hardcoded Agumon impact id; the on-enter/release/detonate engine paths now read registry data instead of Agumon-specific consts or closed matches. In `src/windowed/digimon/agumon/mod.rs`, the Agumon module owns the effect ids, asset paths, projectile flight timing, registry-population startup systems, and the moved registry-focused tests. I also confirmed `src/windowed/render.rs` no longer contains `AGUMON_*_EFFECT_ID`, `fn on_enter_effect_ids`, `fn load_agumon_enoki_vfx`, or `fn enoki_effect_path`. The net work in this recovery pass was to produce fresh verification evidence and write the missing T02 summary via the DB-backed completion tool.

## Verification

Fresh verification passed in this run: `RUSTFLAGS="-D warnings" cargo build --features windowed`; `cargo test --features windowed --test windowed_only`; `cargo test --features windowed --bins`; `cargo test --test dependency_gating`; and a direct source check confirming `src/windowed/render.rs` has none of the forbidden T02 tokens (`AGUMON_*_EFFECT_ID`, `fn on_enter_effect_ids`, `fn load_agumon_enoki_vfx`, `fn enoki_effect_path`).

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `RUSTFLAGS="-D warnings" cargo build --features windowed` | 0 | ✅ pass | 53434ms |
| 2 | `cargo test --features windowed --test windowed_only` | 0 | ✅ pass | 302ms |
| 3 | `cargo test --features windowed --bins` | 0 | ✅ pass | 329ms |
| 4 | `cargo test --test dependency_gating` | 0 | ✅ pass | 362ms |
| 5 | `python3 - <<'PY'
from pathlib import Path
text = Path('src/windowed/render.rs').read_text()
checks = {
    'AGUMON_.*EFFECT_ID': 'AGUMON_' in text and 'EFFECT_ID' in text,
    'fn on_enter_effect_ids': 'fn on_enter_effect_ids' in text,
    'fn load_agumon_enoki_vfx': 'fn load_agumon_enoki_vfx' in text,
    'fn enoki_effect_path': 'fn enoki_effect_path' in text,
}
violations = [k for k,v in checks.items() if v]
if violations:
    print('FAIL', violations)
    raise SystemExit(1)
print('PASS no forbidden T02 tokens remain in src/windowed/render.rs')
PY` | 0 | ✅ pass | 12ms |

## Deviations

None.

## Known Issues

None.

## Files Created/Modified

- `src/windowed/render.rs`
- `src/windowed/digimon/agumon/mod.rs`
