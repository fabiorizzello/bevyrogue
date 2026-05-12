---
spike: SP5
name: Sprite pipeline determinism check
status: done
created: 2026-05-12
completed: 2026-05-12
parallel_with: any
effort: 30min
inputs:
  - tools/sprite_pipeline/ (configs, scripts, raw_models)
  - latest pipeline commit: a4fea2b
outputs:
  - empirical hash check (agumon_atlas.json/.png — two fresh runs vs committed)
  - static analysis of non-determinism sources
  - go/no-go for SP4
---

# SP5 — Sprite pipeline determinism check

## Goal

Confirm `tools/sprite_pipeline/` produces byte-identical `_atlas.json` (and adjacent `_atlas.png`) across repeated runs from the same intermediate inputs. If non-deterministic, SP4 schema validation is shaky.

## Method actually used

Original brief assumed a full 6-Digimon Blender re-run sweep. That is out of scope for a 30-min spike (Blender Cycles render takes ≫5min per Digimon, requires GPU verification). Substituted approach:

1. **Static analysis** of all pipeline Python scripts in `tools/sprite_pipeline/scripts/` for known non-determinism sources (timestamps, dict ordering, RNG without seed, parallel ordering, filesystem traversal order, floating-point ordering).
2. **Empirical sub-step verification**: re-ran the *final* JSON-emission stage — `bevy_atlas_generator.py --char agumon` — twice into fresh temp dirs against the existing intermediate PNGs in `output/agumon/latest/sprites_anime/mihoyo_style_iso45/`. Compared sha256 of `_atlas.json` and `_atlas.png` across two fresh runs and against the committed artifact.

The empirical step covers the artifact SP4/S04 actually consume (`_atlas.json` + `_atlas.png`). It does NOT cover Blender stochastic render determinism (the intermediate frame PNGs), which is acknowledged below as residual risk.

## Empirical result (Agumon)

| Artifact | Run A sha256 | Run B sha256 | Committed |
|---|---|---|---|
| `agumon_atlas.json` | `33bd4e27ecffa776…` | `33bd4e27ecffa776…` | `33bd4e27ecffa776…` |
| `agumon_atlas.png`  | `6ea480e9886dc95a…` | `6ea480e9886dc95a…` | `6ea480e9886dc95a…` |

Three-way byte-identical match. The packing + JSON emission stage is fully deterministic given identical intermediate inputs.

## Findings table — non-determinism sources

Classification: **present** = will cause drift, **mitigated** = code defends against it, **absent** = source not used, **unknown** = not verifiable without full empirical run.

| # | Source | Class | Citation | Notes |
|---|---|---|---|---|
| 1 | Timestamp in atlas JSON | absent | `bevy_atlas_generator.py:74-84` | `master_meta` contains only `character`, `version`, `frame_size`, `columns`, `rows`, `total_frames` — no clock/date fields. Inspected `output/bevy_atlases/agumon_atlas.json`: clean. |
| 2 | Timestamp in *run manifest* | present (out of scope) | `pipeline_run.py:103, 521, 616` | `gen_run_id()` uses `datetime.now()` + `os.urandom(8)`. Affects `manifest.json` under `output/<char>/runs/<id>/`. **Does NOT propagate to `_atlas.json`**. SP4/S04 consume only the bevy atlas. No risk. |
| 3 | Dict insertion order | mitigated | `bevy_atlas_generator.py:20, 87` | `anim_pngs = sorted(list(char_output.glob("*.png")))` then iteration in sorted order builds `animations` dict; Python 3.7+ preserves insertion order; `json.dump` writes in that order. |
| 4 | Filesystem traversal order | mitigated | `bevy_atlas_generator.py:20`; `sheet_assemble.py:68`; `pixelify.py:110`; `bake_picks.py:40,159`; `pipeline_run.py:182,187,354,424` | Every `glob()` call in the pipeline is wrapped in `sorted(...)`. Grep verified: no unsorted `glob` / `iterdir` / `os.walk` results feed into output ordering. |
| 5 | RNG without seed | mitigated | `extract_palette.py:21,111` | `kmeans()` defaults `seed=42`; `random.seed(42)` before `random.sample`. Only stochastic Python step; explicitly seeded. |
| 6 | Cycles render seed | unknown / low-risk | `blender_render.py:441-461` | `scn.cycles.samples = cfg.get("cycles_samples", 1)` with denoising off, BOX pixel filter, filter_width=0.01. `cycles.seed` not explicitly set → defaults to 0 (stable across runs). At samples=1 with seed=0 + denoise off, Cycles is essentially deterministic on a given GPU/driver pair. Cross-machine determinism is NOT guaranteed (GPU vendor / driver float ordering), but same-machine should be byte-stable. Not verified empirically this spike. |
| 7 | Parallel job ordering | mitigated for output | `pipeline_run.py:591-603` | `ThreadPoolExecutor` returns via `as_completed` (nondeterministic). However, `manifest["variants"]` ordering does not propagate to `_atlas.json`; the bevy generator re-derives ordering from sorted PNG glob. Manifest order is irrelevant to S04 consumers. |
| 8 | `json.dump` without `sort_keys` | mitigated by insertion order | `sheet_assemble.py:77`; `bevy_atlas_generator.py:115`; `repack_atlas.py:102` | No `sort_keys=True` anywhere. Relies on insertion-order stability, which is guaranteed by Python 3.7+ AND by sorted globs upstream. Empirical run confirms stability. Defensive recommendation: add `sort_keys=True` as belt-and-braces — but not required for SP4. |
| 9 | Float ordering | absent | n/a | No float aggregation / dict-of-floats serialization in atlas JSON. All fields are ints. |
| 10 | PNG encoding determinism | mitigated (empirical) | `bevy_atlas_generator.py:113`; `repack_atlas.py:95` | PIL `Image.save(out, "PNG")` is deterministic given identical pixel buffer and identical PIL version. Empirical SHA match confirms for current environment. Cross-PIL-version drift possible but out of scope. |

## Verdict

**PASS — high confidence** for the SP4/S04 consumption path (`_atlas.json` + `_atlas.png`).

- The JSON-emission stage was empirically verified byte-identical across two fresh runs (and matches the committed artifact).
- All identifiable Python-level non-determinism sources are either absent from the bevy atlas output or explicitly mitigated (sorted globs, fixed RNG seed, insertion-order dict serialization).
- The one timestamp source in the codebase (`pipeline_run.py` run manifest) does not propagate into `_atlas.json`.

**Residual risk (acknowledged, not blocking)**:

- Blender Cycles render stage was NOT empirically re-run. Same-machine determinism is highly likely (samples=1, denoise off, default seed=0, BOX filter). Cross-machine reproducibility (different GPU/driver) is an open question but is an asset-regeneration concern, not an SP4 schema-validation concern. Once frames exist, downstream is deterministic.
- No `sort_keys=True` on `json.dump` — current stability depends on sorted-glob discipline upstream. If anyone introduces an unsorted iteration upstream, atlas JSON ordering could drift silently.

## If fail or partial

N/A (verdict is pass). Defensive nit (not required): add `sort_keys=True` to the three `json.dump` calls in `bevy_atlas_generator.py:115`, `sheet_assemble.py:77`, `repack_atlas.py:102`. Cheap belt-and-braces, but not a blocker.

## Recommendation for SP4 and S04

**SP4 can proceed assuming determinism.** The bevy `_atlas.json` schema validation work has a stable artifact to validate against. S04 consumption of `_atlas.json` is safe.

Two notes for milestone planner:

1. **Asset regeneration is a separate concern.** If/when the other 5 Digimon need regen, that work depends on Blender render determinism, which this spike did not empirically validate. Recommend a 1-Digimon regen sanity check (e.g. re-run gabumon end-to-end and sha-diff the atlas) as part of the asset regeneration slice, NOT as a blocker for SP4.
2. **Consider a follow-up micro-task** (~10min) to add `sort_keys=True` defensively. File a low-priority task; do not block S04.

## Out of scope (confirmed)

- Fixing non-determinism (none required for SP4 path).
- Modifying pipeline scripts.
- Regenerating sprite assets.
- Cross-machine / cross-GPU reproducibility audit.
