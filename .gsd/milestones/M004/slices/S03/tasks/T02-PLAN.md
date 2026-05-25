---
estimated_steps: 3
estimated_files: 2
skills_used: []
---

# T02: Enrich baby_burner.detonate into a real data-driven burst + flash

Why: Today's baby_burner.detonate (vfx.ron:140-161) is a deliberate S02 placeholder — a single static size-18 flat quad reproducing the old Generic-kind detonate. The 'no hardcoded VFX paths' criterion is already satisfied (grep-guard), but the K001 visual review needs a detonate worth signing off. This must reuse the existing pure verbs (fan_out + static) and the on_expire chaining mechanism (MEM076/MEM077) — no parallel math, no novel placement verb (so no register_agumon_ext change), demonstrating the milestone's RON-only reuse path.

Do: In assets/digimon/agumon/vfx.ron, rewrite baby_burner.detonate (keep the id — render.rs:320 AGUMON_DETONATE_EFFECT_ID = "baby_burner.detonate") as a multi-particle outward burst anchored at TargetCenter using verb "agumon/baby_flame/fan_out" with FanOut params (mirroring baby_flame.impact's count/spread/ease-out scale + alpha-fade color), and chain a new bright central flash effect (e.g. "baby_burner.flash": static at TargetCenter, short ttl, alpha-fades — mirroring baby_flame.impact_flash) via on_expire: Some("baby_burner.flash"). Both effects must use only the four already-registered verbs in KNOWN_VERBS. Extend tests/animation/vfx_asset_load.rs: assert baby_burner.detonate resolves and uses fan_out, that its on_expire chains baby_burner.flash, that baby_burner.flash resolves and is static, that spawn_plan/eval_scale/eval_color give the authored deterministic values, and that validate_effects still accepts the real asset (no UnknownVerb / DanglingOnExpire / DanglingVariant). Update any vfx_asset_load assertion that counts/enumerates total effects so the two-new-effects asset still passes.

Done-when: cargo test --test animation passes (extended vfx_asset_load + unchanged variant + grep-guard tests); cargo build --features windowed compiles; cargo test --features windowed --test windowed_only passes, confirming the data-driven detonate spawn contract (spawn_effect_by_id resolving baby_burner.detonate via the Registry) still holds. Baby Burner visual quality in cargo winx is K001 human sign-off only.

## Inputs

- `assets/digimon/agumon/vfx.ron`
- `tests/animation/vfx_asset_load.rs`
- `src/windowed/render.rs`
- `tests/windowed_only/vfx_asset_impact_render.rs`
- `.gsd/milestones/M004/slices/S03/S03-RESEARCH.md`

## Expected Output

- `assets/digimon/agumon/vfx.ron`
- `tests/animation/vfx_asset_load.rs`

## Verification

cargo test --test animation 2>&1 | tail -20 && cargo build --features windowed 2>&1 | tail -5 && cargo test --features windowed --test windowed_only 2>&1 | tail -20
