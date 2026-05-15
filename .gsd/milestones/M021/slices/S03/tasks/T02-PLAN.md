---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T02: Mode-parity integration test: Execute ≡ DryRun ≡ Preview on a branched timeline

WHY: I2/D024 — the Intent stream must be mode-independent on a BRANCHED timeline. DO (per D006/D007): create tests/timeline_mode_parity.rs modeled on tests/timeline_chain_bolt_port.rs (same setup_app/spawn_unit scaffolding, ExtRegistries, IntentQueue, CastIdGen). Build ONE branched CompiledTimeline: an entry Impact/Cast beat with a hook that enqueues an Intent::DealDamage, then TWO outgoing edges from it — edge A gated by a predicate that reads target HP from `ctx.world` (e.g. target hp_current below a threshold) routing to a 'finisher' beat with a distinct hook (different amount), edge B unconditional (fallback) routing to a 'normal' beat. Branching MUST be via the edge-gate predicate reading world (NOT SelectorCtx — D007); selector stays hard-coded/None like the chain_bolt port. Spawn world so the predicate actually takes the branch (deterministic, no RNG). Run a fresh BeatRunner over a freshly-spawned identical world for each of SkillCtxMode::Execute, ::DryRun, ::Preview (run_to_completion, do NOT drain pending through intent_applier — D006). Add a test-local `fn normalize(p: &VecDeque<Intent>) -> Vec<String> { p.iter().map(|i| format!("{:?}", i)).collect() }`. Assert normalize(execute) == normalize(dryrun) && == normalize(preview), and assert the stream is non-empty and reflects the branch taken (e.g. contains the finisher amount). DONE-WHEN: cargo test --test timeline_mode_parity passes; the timeline genuinely branches (a second test case or a documented assertion that flipping the spawned HP routes the other edge, proving the predicate is live, not dead).

## Inputs

- `src/combat/api/runner.rs`
- `src/combat/api/skill_ctx.rs`
- `src/combat/api/timeline.rs`
- `src/combat/api/intent.rs`
- `src/combat/api/applier.rs`
- `tests/timeline_chain_bolt_port.rs`

## Expected Output

- `tests/timeline_mode_parity.rs`

## Verification

cargo test --test timeline_mode_parity 2>&1 | tail -5 (all cases pass, including the branch-routing assertion)
