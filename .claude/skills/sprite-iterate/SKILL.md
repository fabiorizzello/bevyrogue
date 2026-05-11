---
name: sprite-iterate
description: Use when user asks to iterate, improve, or auto-tune the 3D-to-pixel-art sprite pipeline (e.g. "iterate sprite pipeline", "auto-improve shaders", "loop sprite generation until good"). Spawns a Task subagent that runs render → review → edit loop autonomously until target score reached.
---

# Sprite Pipeline Auto-Iteration

Coordinate auto-iteration loop for 3D-to-pixel-art Blender pipeline.

Pipeline supports MULTIPLE characters, multiple sources per character, and variants. Naming convention: `<digimon>[-<source>][-<variant>]`. Examples: `agumon`, `agumon-rearise`, `agumon-newcentury`, `agumon-black`, `agumon-rearise-snow`, `gabumon`, `patamon-tk`, etc.

## When to invoke

User says one of:
- "iterate sprite pipeline"
- "auto-improve shaders for <char>"
- "loop sprite generation until good"
- "tune pipeline for <char>"
- Or invokes `/sprite-iterate` slash command if mapped.

## Workflow

### 1. Confirm parameters

Ask user (or use defaults):
- **char**: full name (must match `tools/sprite_pipeline/configs/<char>.json`). Required, no default — pipeline now multi-char.
- **max_iter**: iterations cap (default 5).
- **target_score**: minimum score for SUCCESS verdict (default 80/100).
- **mode**: `fresh` | `resume` | `validate` (default `fresh`)
  - `fresh`: ignore prior state, run iter 1 from scratch
  - `resume`: continue from last interrupted iter (crash recovery)
  - `validate`: re-score existing latest run vs current standards (no render)

Default behavior is `fresh` — criteria/refs/code may have changed since last run.

### 2. Spawn subagent for entire loop

Use Agent tool with `subagent_type: "general-purpose"` and `run_in_background: true`.

**Subagent prompt template:**

```
You are running a closed-loop iteration on a Blender 3D-to-pixel-art pipeline for character: <CHAR>.

## Environment
- Working dir: /home/fabio/dev/bevyrogue
- Pipeline dir: tools/sprite_pipeline/
- Pipeline runner: tools/sprite_pipeline/scripts/pipeline_run.py
- Latest output: tools/sprite_pipeline/output/<CHAR>/latest/  (symlink to most recent run)
- Sprites dirs: output/<CHAR>/latest/sprites_anime/<variant>_<camera>/idle.png
              + output/<CHAR>/latest/sprites_pixel/<variant>_<camera>/idle.png
- Comparison images: output/<CHAR>/latest/comparison_anime.png + comparison_pixel.png
- Shader variants: tools/sprite_pipeline/scripts/shaders/*.py
- Config: tools/sprite_pipeline/configs/<CHAR>.json
- State: tools/sprite_pipeline/output/<CHAR>/iter_state.json
- Standards (REQUIRED reads):
  - tools/sprite_pipeline/standards/global.md  (cross-char rules, per-shader specs, scoring)
  - tools/sprite_pipeline/standards/<CHAR_BASE>.md  (char-specific specs — color, signature features, fix recipes)
    where CHAR_BASE = char name with -<source>/-<variant> suffixes stripped
    (e.g. agumon-rearise → agumon; agumon-rearise-snow → agumon)
- References:
  - tools/sprite_pipeline/references/global/  (octopath2dhighdef.jpeg, celshadingzolo.jpg — style targets)
  - tools/sprite_pipeline/references/<CHAR_BASE>/  (canonical.* — char ground truth)

## Goal
Achieve best_score >= <TARGET_SCORE>/100 for at least one shader variant per track. Targets defined by standards/<CHAR_BASE>.md (color, signature features) + standards/global.md (style, shader specs, scoring rubric).

## Loop (max <MAX_ITER> iterations)

For each iteration:
1. **Read standards FIRST** (mandatory): standards/global.md + standards/<CHAR_BASE>.md. These define ALL scoring rules. Do NOT use generic anime/pixel art criteria.
2. **Read references**: references/<CHAR_BASE>/canonical.* (signature ground truth) + references/global/* (style targets).
3. **Render**: Run `python3 tools/sprite_pipeline/scripts/pipeline_run.py --char <CHAR> --parallel 4 --skip-deps-check` via Bash. Pipeline auto-detects palette from `palettes/<CHAR>.gpl` or falls back to `palettes/<CHAR_BASE>.gpl`. Wait completion.
4. **Review**: Inspect SINGLE sprite renders (NOT comparison.png — too compressed/small). Read `output/<CHAR>/latest/sprites_anime/<variant>_<camera>/idle.png` and same under sprites_pixel/. Score each variant 0-100 against standards.
5. **Decide**:
   - If best_score per track >= <TARGET_SCORE> → STOP, report success.
   - Otherwise → identify weakest variants and apply fix recipes from standards.
6. **Edit**: Use Edit/Write tools to modify shader files / config / palette. Apply concrete fixes.
7. **Track**: Append iteration record to `output/<CHAR>/iter_state.json`: iteration number, best_variant, best_score, edits_applied list, timestamp.
8. **Continue**.

## Stop conditions

Evaluated AFTER each iteration's render+review (NEVER at iter 0 against pre-existing state).

- best_score >= <TARGET_SCORE> in current iter → SUCCESS
- 3 consecutive iterations with no score improvement → STUCK
- Iter count >= <MAX_ITER> → MAX_REACHED
- Render fails twice in a row → FAILURE (revert recent edits)

CRITICAL: every invocation MUST execute at least 1 actual render+review cycle (unless mode=validate). DO NOT bail early because previous run met target — standards/code/refs may have changed.

## Quality criteria — STRICT scoring rubric

Apply rubric from `standards/global.md` per-variant specs + `standards/<CHAR_BASE>.md` char specs. Score 0-100, AVERAGE for variant total. Apply STRICTLY — do NOT round up. If criterion fails (visibly broken), score that criterion ≤30.

Generic criteria (weights):

1. **Color saturation** (25%) — body hue must match canonical hue range from char standards file. Out-of-range = ≤30. Pale/desat dominant = ≤40. Pure-white silhouette >15% body = automatic fail this criterion.
2. **Cel-shading bands** (15%) — 2-3 HARD-stepped levels (CONSTANT interp). Smooth gradient = ≤30.
3. **Outline definition** (15%) — clean 1-2px black on full silhouette.
4. **Crispness** (10%) — pixel-perfect, no AA blur for pixel track; no jaggies for anime.
5. **Char recognizability** (15%) — silhouette + signature features per char standards.
6. **Depth perception** (10%) — iso views show 2-3 faces.
7. **Render integrity** (5%) — not all-black/all-white/empty.
8. **Production-readiness** (5%) — minimal cleanup needed.

**HARD SIGNATURE CHECK** (mandatory, drives criterion 5 cap):
Each character has signature features defined in `standards/<CHAR_BASE>.md` under "Signature features (HARD CHECK)". Examples:
- Agumon: claws (hands+feet, white/cream), green eye, snout, tooth
- Gabumon: horn, fur pattern, pelt
- Patamon: ears (oversized), wings
- (defined per char in standards file)

If signature features absent / muddy / occluded → cap variant total at 70 regardless other criteria. Test: zoom 200% on output, can you count individual signature features as listed in char standards? If no → fail.

## Score thresholds

- **80+** = SUCCESS (production-ready)
- **70-79** = close, ONE more iter
- **<70** = NEEDS_ITERATION

## Anti-soft-scoring rules

- DO NOT score 80+ unless variant **passes color saturation criterion** (score ≥60).
- DO NOT score 75+ if outline missing or body desaturated.
- DO NOT round up. Between 70 and 75, choose 70.
- BE skeptical of "OK"-looking variants — compare against char canonical ref.
- White-dominated silhouette CANNOT score above 60 regardless other criteria.

## Validation against ground truth

### Step 1: Read standards at iter start

Mandatory reads, in order:
1. `tools/sprite_pipeline/standards/global.md` — cross-char rules, per-shader specs, camera multipliers, hard fails
2. `tools/sprite_pipeline/standards/<CHAR_BASE>.md` — char identity, signature features, color anchors, fix recipes

These are SOURCE OF TRUTH for scoring. Apply per-variant max caps from global.md table.

### Step 2: Read references

- `references/<CHAR_BASE>/canonical.*` — char-specific ground truth
- `references/global/octopath2dhighdef.jpeg` — pixel HD-2D target
- `references/global/celshadingzolo.jpg` — anime cel-shading target

### Step 3: Per-variant comparison

Inspect SINGLE sprite renders (sprites_anime/sprites_pixel subdirs), NOT comparison.png — comparison too small/compressed for fine detail (claws, signature features).

For each variant:
1. **Color match**: visually compare body hue against `references/<CHAR_BASE>/canonical.*`. If pale/wrong-hue vs reference → criterion ≤40.
2. **Outline match**: hard 1-2px black on silhouette.
3. **Cel-shading match**: shadow band hardness vs target style (Zolo for anime, Octopath for pixel).
4. **Crispness**: pixel grid integrity (pixel track) / no jaggies (anime track).
5. **Recognizability**: silhouette + signature features per char standards.

### Hard checks before SUCCESS verdict

- Body color hue in declared char range — confirmed visual compare to canonical.
- Silhouette < 15% white/pale interior.
- Outline visible without zoom.
- Palette appropriate: if `palettes/<CHAR>.gpl` muted, hand-tune to char anchors from standards/<CHAR_BASE>.md.

If ground truth unmet, do NOT score success even if other criteria pass. Override via palette hand-tuning + re-render preferred.

## Fix recipes by issue

Generic recipes here. Char-specific recipes in `standards/<CHAR_BASE>.md`.

### Pale / desaturated body
- Verify palette enforced (auto-detected, or `--palette <name>.gpl`).
- If palette muted → hand-tune `palettes/<CHAR>.gpl` to char anchors from standards.
- Reduce rim-light additive (`add_rim.Fac` ↓).
- Reduce lit-tint white in ColorRamp.
- Bump posterize 4→3 for harder zones.

### White rim light dominant
- mihoyo_style: lower `Fresnel.IOR` 1.5→1.2 + reduce rim brightness.
- threelevel: `add_rim.Fac` 0.4→0.15.

### All-black render
- Check UV map name (use `uv.uv_map = ""` for active layer).
- For matcap textures (sphere-mapped, NOT UV) → use `shader_variant: matcap`.
- Shader needs Cycles samples bump (1→16+) AND sun light, OR convert to emission-based.

### Empty/white render
- Camera misconfig or material alpha=0. Verify ortho_scale, alpha output.

### Desaturated despite palette
- ColorRamp white-only stops desaturate. Replace lit-tint (1,1,1)→(1.1, 0.95, 0.85) warm bias.

### Missing outline
- Single-mesh source (no separate outline mesh): enable `auto_solidify` + `freestyle_outline` in config.
- Multi-mesh source with outline mesh: hide it in config + enable `freestyle_outline` (avoids covering thin features like claws).

### Soft cel bands
- ColorRamp interpolation MUST be `CONSTANT`.

### Iso pose collapsed
- Try lower pitches in `iso_presets` (blender_render.py).

### Compositor not engaging (astropulse / lospec_toolkit)
- Output identical to flat_emission = compositor template not loaded.
- Verify `scene.compositing_node_group` assigned + `use_nodes = True`.

## Output

When loop completes, return single message containing:
1. **Final verdict**: SUCCESS / STUCK / MAX_REACHED / FAILURE
2. **Best variant per track** (anime + pixel) name and score
3. **Score progression** across iterations
4. **Total edits applied** count
5. **Final variant paths**: `output/<CHAR>/latest/sprites_anime/<best_anime>/idle.png` + same for pixel
6. **Suggestions** if not SUCCESS

## Constraints

- Do NOT modify files outside `tools/sprite_pipeline/`.
- Do NOT delete `iter_state.json` — append only.
- Do NOT regress previously-good variants.
- Prefer minimal change (single param tweak) over rewrite.
- ALWAYS verify edit applied (Read after Edit).
- Do NOT modify `standards/*.md` or `references/` — those are scoring inputs, not iteration outputs.
```

### 3. Replace placeholders

Substitute in template:
- `<CHAR>` → user-provided full char name (e.g. `agumon-rearise`)
- `<CHAR_BASE>` → first segment before any `-` (e.g. `agumon-rearise` → `agumon`, `gabumon` → `gabumon`)
- `<MAX_ITER>` → user-provided iter cap
- `<TARGET_SCORE>` → user-provided threshold

### 4. Background execution

Spawn agent with `run_in_background: true` so user can do other work. Long renders (~15 min/iter × 5 iter = up to 75 min) don't block.

### 5. Report back

When subagent completes, summarize:
- Final verdict
- Score trajectory
- Best variant + path
- Total cost (~$1-2/iter rough)

## Vs subprocess version (`auto_iterate.py`)

| Aspect | Subprocess | Subagent (this skill) |
|--------|-----------|------------------------|
| Multi-turn within iter | No | Yes |
| Restart cross-session | Yes (state file) | No |
| Run from CI/cron | Yes | No |
| Setup complexity | Low | Lower |

Use subagent for interactive sessions. Use subprocess for CI / scheduled / batch processing many Digimon.

## Example invocation

User: *"iterate sprite pipeline for agumon-rearise, max 4 iter, target 80"*

Steps:
1. Confirm parameters.
2. Spawn Task subagent with prompt template (placeholders substituted: `<CHAR>=agumon-rearise`, `<CHAR_BASE>=agumon`).
3. Set `run_in_background: true`.
4. Tell user: "Subagent started. Will notify on completion (~30-60 min for 4 iter)."

When agent completes:
5. Read agent result.
6. Summarize for user with verdict + best variants per track + paths.

## Edge cases

- **Existing iter_state.json + mode=fresh** (default): RENAME to `iter_state.json.bak.<timestamp>` and start iter 1 fresh.
- **Existing iter_state.json + mode=resume**: read state, continue from last iter.
- **Existing iter_state.json + mode=validate**: read state, re-score existing latest run vs CURRENT standards (no render).
- **Missing standards/<CHAR_BASE>.md**: abort, instruct user to create it (use `standards/agumon.md` as template).
- **Missing references/<CHAR_BASE>/canonical.***: warn, fall back to global refs only — scoring less precise.
- **No models in raw_models/**: instruct user to drop FBX first, abort cleanly.
- **Stop conditions DO NOT pre-check existing state**: stop conditions evaluated AFTER each iter's render+review. Forces actual work each invocation unless mode=validate.
