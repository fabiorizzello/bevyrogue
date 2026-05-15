---
estimated_steps: 4
estimated_files: 6
skills_used: []
---

# T01: Make CompiledTimeline asset-safe and built-in-hook-capable

Skills used: bevy, rust-best-practices, rust-testing, verify-before-complete.

Why: the current timeline model is test-only because beat ids, hook ids, and presentation ids assume `&'static str`, and hooks have no place to read per-beat effect parameters. That blocks any non-leaky compiler path from `skills.ron`.

Do: convert the timeline data model to owned string storage where asset-backed compilation needs it, add the minimal typed beat payload needed by generic built-in hooks, introduce `src/combat/api/builtins.rs`, register the kernel built-ins during combat plugin setup, and keep registry lookup semantics keyed by `&str`. Update validation/tests so built-in hook, selector, and predicate refs still fail with precise axis/site reporting.

Done when: a hand-built timeline using the new owned-id/payload shape validates against registered built-ins, the combat plugin installs those built-ins automatically, and the new model is ready for asset compilation without string leaking.

## Inputs

- `src/combat/api/timeline.rs`
- `src/combat/api/registry.rs`
- `src/combat/api/mod.rs`
- `src/combat/plugin.rs`
- `tests/timeline_validate_typo.rs`
- `tests/timeline_chain_bolt_port.rs`

## Expected Output

- `src/combat/api/timeline.rs`
- `src/combat/api/registry.rs`
- `src/combat/api/builtins.rs`
- `src/combat/api/mod.rs`
- `src/combat/plugin.rs`
- `tests/compiled_timeline_builtin_validation.rs`

## Verification

cargo test --test compiled_timeline_builtin_validation

## Observability Impact

Improves failure diagnostics at the validation boundary: missing built-ins and malformed beat payloads become localized to skill/beat sites instead of appearing as generic runtime panics.
