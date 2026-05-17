# S11: UI and AI consumers via SkillCtx Preview

**Goal:** Expose a shared `SkillCtxMode::Preview` intent-stream query and wire it into both windowed combat UI damage preview and enemy AI action scoring without mutating world state.
**Demo:** UI preview damage via stream; AI score ottimale via stream.

## Must-Haves

- ## Must-Haves
- Add a single shared `query_skill_preview` seam that resolves the same compiled timeline path used by timeline-backed actions, runs `BeatRunner` in `SkillCtxMode::Preview`, and returns the pending `Intent` stream without applying it.
- Windowed combat UI shows previewed damage derived from preview-stream `Intent::DealDamage` output for the currently pending action/target path, without making `combat_panel` itself an exclusive system.
- Enemy AI ranks candidate actions through the same preview-stream contract instead of the current toughness-ratio-only heuristic, while preserving deterministic fallback behavior when no previewable candidate exists.
- Preview runs remain non-mutating: calling the preview seam must not change HP, toughness, SP, ult charge, or other combat state.
- Headless and windowed entrypoints both keep the preview consumers wired so runtime behavior matches tests.
- ## Threat Surface
- **Abuse**: malformed or missing timeline data must degrade to no preview / fallback AI choice rather than mutating combat state or panicking mid-turn.
- **Data exposure**: none; this slice reads local combat state and exposes numeric preview values only.
- **Input trust**: preview consumers rely on compiled timeline IDs, skill-book assets, and current ECS combat state; all are local but can be incomplete during boot or tests.
- ## Requirement Impact
- **Requirements touched**: none preloaded.
- **Re-verify**: preview/query parity, no-apply contract, enemy-turn decision routing, windowed panel wiring, headless/windowed registration.
- **Decisions revisited**: D004.
- ## Proof Level
- This slice proves: integration
- Real runtime required: yes
- Human/UAT required: no
- ## Verification
- `cargo test --test skill_preview`
- `cargo test --features windowed --test windowed_preview_cache`
- `cargo test --test enemy_ai`
- `cargo test --test enemy_ai_preview`
- `cargo check`
- `cargo check --features windowed`
- ## Observability / Diagnostics
- Runtime signals: preview consumers stay explainable via preview intent streams and cacheable per-action/per-target damage summaries rather than bespoke UI-side prediction logic.
- Inspection surfaces: focused integration tests, `PreviewCache` resource state, and enemy-AI preview score assertions.
- Failure visibility: missing timeline / disabled preview paths should surface as empty previews or deterministic fallback choices in tests, not silent world mutation.
- Redaction constraints: none.
- ## Integration Closure
- Upstream surfaces consumed: `BeatRunner`, `SkillCtxMode`, compiled timeline lookup in the action pipeline, `combat_panel`, `enemy_ai`, and turn-system registration entrypoints.
- New wiring introduced in this slice: a shared preview helper plus narrow world-backed bridge systems/resources for UI cache refresh and enemy decision scoring.
- What remains before the milestone is truly usable end-to-end: S12 still needs roster and validation-snapshot registration to become blueprint-keyed.

## Proof Level

- This slice proves: integration

## Integration Closure

S11 closes the consumer side of M021 by making both UI preview and enemy AI depend on the same preview intent-stream contract instead of local heuristics. After this slice, the remaining milestone work is S12's roster-entry and validation-snapshot registry keying cleanup.

## Verification

- Add a shared preview seam whose outputs can be asserted directly in tests, plus runtime cache/state surfaces for windowed preview display and AI decision scoring so future agents can diagnose whether a bad preview came from timeline lookup, preview execution, or consumer wiring.

## Tasks

- [x] **T01: Add shared `query_skill_preview` seam and non-mutating preview-stream coverage** `est:2h`
  Skills used: bevy, rust-best-practices, rust-testing, verify-before-complete.
  - Files: `src/combat/preview.rs`, `src/combat/mod.rs`, `src/combat/turn_system/pipeline.rs`, `tests/skill_preview.rs`
  - Verify: cargo test --test skill_preview

- [x] **T02: Refresh windowed damage preview through a cached preview-stream bridge** `est:2.5h`
  Skills used: bevy, frontend-design, bevy, rust-best-practices, verify-before-complete.
  - Files: `src/combat/preview.rs`, `src/ui/combat_panel.rs`, `src/windowed.rs`, `tests/windowed_preview_cache.rs`
  - Verify: cargo test --features windowed --test windowed_preview_cache

- [x] **T03: Route enemy action choice through preview-stream scoring and deterministic turn-system wiring** `est:3h`
  Skills used: bevy, rust-best-practices, rust-testing, verify-before-complete.
  - Files: `src/combat/enemy_ai.rs`, `src/combat/preview.rs`, `src/combat/turn_system/mod.rs`, `src/headless.rs`, `src/windowed.rs`, `src/bin/combat_cli.rs`, `tests/enemy_ai.rs`, `tests/enemy_ai_preview.rs`
  - Verify: cargo test --test enemy_ai_preview

## Files Likely Touched

- src/combat/preview.rs
- src/combat/mod.rs
- src/combat/turn_system/pipeline.rs
- tests/skill_preview.rs
- src/ui/combat_panel.rs
- src/windowed.rs
- tests/windowed_preview_cache.rs
- src/combat/enemy_ai.rs
- src/combat/turn_system/mod.rs
- src/headless.rs
- src/bin/combat_cli.rs
- tests/enemy_ai.rs
- tests/enemy_ai_preview.rs
