# Auto-Iteration via Skill

Closed-loop sistema dove un Agent subagent (Gemini CLI o Claude Code) itera autonomamente sulla pipeline shader fino al target quality.

## Invocation

In sessione interattiva:

> "iterate sprite pipeline for agumon"
> "auto-improve shaders"
> "loop sprite generation until score 80"
> "tune pipeline for agumon, max 5 iter"

Trigger skill `sprite-iterate` (definito in `.gemini/skills/sprite-iterate/SKILL.md` o `.claude/skills/sprite-iterate/SKILL.md`).

## Flow

```
User → Agent (main) → Spawn Task subagent (background)
                           ↓
                       Loop max_iter times:
                           1. Bash: sprite_pipeline/scripts/multi_shader.sh <char>
                           2. Read: output/<char>/shaders/comparison.png
                           3. Score variants 0-100 by criteria
                           4. If best >= target → STOP success
                           5. Edit: shaders/*.py or configs/*.json
                           6. Append iter_state.json
                           7. Repeat
                           ↓
                       Return final report to main session
```

## Subagent capabilities

Subagent inherita tools full:
- **Bash**: lancia render, git ops
- **Read**: comparison.png, shader source, configs
- **Edit/Write**: modifica shader/config files
- **Glob/Grep**: navigate pipeline

Scope: `--add-dir sprite_pipeline/` enforced — non tocca resto del repo.

## State

`sprite_pipeline/output/<char>/iter_state.json`:
```json
{
  "iterations": [
    {
      "iteration": 1,
      "best_variant": "basic_cel_side",
      "best_score": 65,
      "edits_applied": [...],
      "timestamp": "2026-04-29T14:30:00"
    }
  ],
  "best_score_history": [65, 72, 78, 84]
}
```

Subagent legge questo a inizio loop per resume. Append-only.

## Stop conditions

| Condition | Reason |
|-----------|--------|
| `best_score >= target_score` | Default target 80/100 |
| 3 iter consecutive no progress | Score plateau |
| max_iter reached | Hard cap (default 5) |
| Render fail 2× consecutive | Critical bug |

## Cost stima

- Render multi_shader: ~5-10 min/iter (24 variants Cycles)
- Subagent call: ~$0.50-2/iter (vision + Edit tool)
- Stima totale: ~$3-10 per 5-iter run

## Quality criteria (anime cel-shading HSR-style)

1. Crisp pixel boundaries (no AA blur)
2. Hard cel bands (2-3 levels, ColorRamp constant)
3. Black outline on silhouette
4. Vibrant Agumon orange (#F39A2E target)
5. Recognizable chibi T-rex
6. Depth via iso angle (iso_threequarter sweet spot)
7. No render failures

## Output

Quando subagent finisce, ritorna single message con:
1. **Verdict**: SUCCESS / STUCK / MAX_REACHED / FAILURE
2. **Best variant** name + score
3. **Score progression** (e.g. `[65, 72, 78, 84]`)
4. **Edits applied** count
5. **Final variant path**: `sprite_pipeline/output/<char>/shaders/<best_variant>/idle.png`
6. **Suggestions** se non SUCCESS

## Files

```
.gemini/skills/sprite-iterate/
  SKILL.md                       ← skill definition + workflow
sprite_pipeline/
  scripts/
    multi_shader.sh              ← render matrix (atomic, called by subagent)
    blender_render.py
    pixelify.py
    sheet_assemble.py
    inspect_model.py
    shaders/                     ← edited by subagent
  configs/<char>.json            ← edited by subagent
  output/<char>/iter_state.json  ← state (append-only)
```

## Vs subprocess approach (deprecato)

Versione precedente usava `auto_iterate.py` (subprocess subprocess LLM call) e `iterate_pipeline.sh` (bash loop). **Rimossi** in favore di skill — single source of truth, native Agent logic, no duplication.

Trade-off accettato:
- ✅ Single orchestration model
- ✅ Multi-turn within iter (subagent ha context completo)
- ✅ Native UX ("iterate pipeline" → done)
- ❌ No CI/cron triggering (richiede sessione interattiva)

Per CI/cron use case → riusa `multi_shader.sh` direct + scrivi review external (es. LLM SDK script).

## Manual override

Se subagent fa edit indesiderato:

```bash
git diff sprite_pipeline/        # vedi cambiamenti
git restore sprite_pipeline/...  # rollback specifico
```

State JSON in `output/` è gitignored — niente da pulire commit-side.

## Troubleshooting

- **Skill non triggera**: verifica `.gemini/skills/sprite-iterate/SKILL.md` esiste, frontmatter `name`/`description` corretto, parole chiave match user prompt.
- **Subagent crash**: log in conversation history. Common cause: render `multi_shader.sh` failure (Blender crash, FBX missing).
- **Loop infinito**: hard cap `max_iter` previene; se manca, manual interrupt.
- **State corruption**: delete `iter_state.json` per restart fresh. Backup output/ prima.
