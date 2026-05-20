# Decisions Register

<!-- Append-only. Never edit or remove existing rows.
     To reverse a decision, add a new row that supersedes it.
     Read this file at the start of any planning or research phase. -->

| # | When | Scope | Decision | Choice | Rationale | Revisable? | Made By |
|---|------|-------|----------|--------|-----------|------------|---------|
| D001 | M021/S07 | combat | `BlockReactionTriggered` trigger scope. | Shared event for any 1-shot mitigation (Tentomon + passives). | One observability seam for all pre-damage mitigation. | Yes | agent |
| D002 | M021 | arch | TwinCore transition routing. | `Blueprint { owner:"twin_core", name, payload }`. | Kernel generic; twin_core module decodes its own state. | Yes | agent |
| D003 | M021/S10 | combat | Digimon-named seams in shared code. | Remove named variants; use owner-gated snapshots/blueprint. | S10 exit: src/combat must be digimon-free. Generic envelopes. | Yes | agent |
| D004 | M021/S11 | arch | World-backed preview integration. | Narrow bridge systems (UI/AI preview cache). | Both need `&mut World`; bridges minimize churn, share contract. | Yes | agent |
| D005 | M021/S12 | arch | Digimon-named roster/validation fields. | Owner-keyed `UnitDef` payload + `ValidationExt` registry. | Decouples schema; registry contributors satisfy C2/C3. | Yes | agent |
| D006 | M021 | blueprint | `BlueprintSignal` payload type. | Opaque payload with owner-scoped downcast. | Closed enum recreates coupling; opaque keeps kernel generic. | Yes | collaborative |
| D007 | M021 | blueprint | `BlueprintRegistry` lifecycle. | Startup-frozen; no hot-reload in M021. | No dynamic need; mutable registration adds invalidation cost. | Yes | collaborative |
| D008 | M021 | observ. | `BlueprintSignal` routing. | Route through kernel transition stream first. | Bypassing drops state from JSONL; hop keeps mutations observable. | Yes | collaborative |
| D009 | M021 | skill | Chain hook shape (D023). | Rust fns by ID, not declarative enum. | Kills bloat; kernel stays closed; unified with passives. | No | collaborative |
| D010 | M021 | skill | Preview support for hooks (D024). | `SkillCtx::Mode { Live, DryRun }`. | Mode switch keeps `preview ≡ execute`; contains side-effects. | Yes | collaborative |
| D011 | M021 | skill | Execution model (D025). | Timeline FSM (beats/edges) + `BeatRunner`. | Monolith forces kernel enum expansion; FSM keeps semantics data. | No | collaborative |
| D012 | M021 | timing | Clock model (D026). | 2-clock: Headless resolution vs Windowed presentation. | UI timing must not alter stream; determinism + presentation. | Yes | collaborative |
| D013 | M021 | skill | Skill mutation (D027). | Runtime edge gating over `TimelinePatchOp`. | Patching hides gating; predicates keep one visible timeline. | Yes | collaborative |
| D014 | M021 | signals | Signal bus (D028). | Closed taxonomy, no free-form strings. | Open strings invite typos; closed bus allows boot validation. | Yes | collaborative |
| D015 | M021 | kernel | Beat edge dispatch (D029). | Declaration order + first-passing + mandatory fallback. | Precedence explicit; fallback prevents stuck timelines. | No | collaborative |
| D016 | M021 | combat | Target selector shape (D030). | Primitives in core; custom selectors by ID. | Custom bounce/adjacency shouldn't expand kernel enums. | No | collaborative |
| D017 | M021 | kernel | Extensibility framework (D031). | Unified `Registry<E>` under `ExtRegistries`. | Avoids drifting impls; fn pointers suffice over trait objects. | Yes | collaborative |
| D018 | M021 | blueprint | Blueprint structure (D032). | Module with `register(reg)`, no trait instances. | Wiring static/simple/allocation-free; matches fn-by-id. | No | collaborative |
| D019 | M021 | kernel | Loop semantics (D022). | Native Loop beats + `hop_index` on `BeatEvent`. | Immutable skilltree; Loop beats capture repeats natively. | Yes | collaborative |
| D020 | M021 | blueprint | Blueprint state mutation (D034). | `Intent::SetBlueprintState` is canonical path. | Direct mutation bypasses transition stream and observability. | Yes | collaborative |
| D021 | M021 | blueprint | registry redesign (D035). | Supersede trait-objects with fn-by-id + `ExtRegistries`. | Fn-by-id covers duties with less indirection. | No | collaborative |
| D022 | M016 | arch | Kernel-Blueprint decoupling. | `src/combat/kernel.rs` never imports blueprints. | Registration at composition layer (`BlueprintsPlugin`). | Yes | agent |
| D023 | M021 | arch | Custom mechanic decoupling. | `KernelEffect` bus + `CombatEvent`s; NO `ExtRegistries`. | Reacting/emitting generic data keeps kernel unaware of Digimon. | Yes | agent |
| D024 |  | sprites | Sprite atlas auto-crop. | 512px uniform frames; no per-character cropping. | Simpler atlas load; consistent scale across animations. | No | human |
| D025 | M002 S02 planning | runtime/two-clock | S02 timeline cue barrier ownership | Represent manual presentation timing with a generic combat runtime resource (`TimelineClock` plus suspended timeline/cue state) and let windowed systems opt into `Clock::Windowed` and release the cue; keep headless/default execution on `Clock::HeadlessAuto`. | The combat kernel remains feature-agnostic and deterministic, while windowed rendering controls only when a queued barrier is released. This avoids windowed dependencies in `src/combat` and keeps the I3 headless-vs-windowed intent stream contract testable without wgpu. | Yes | agent |
| D026 | M002 S04 planning | combat/reactive-vfx | S04 Baby Burner detonate architecture | Implement Baby Burner detonate as a Rust-only post-application reaction registered under the Agumon blueprint, using an owner-neutral reaction seam and generic blueprint transition/intent outputs rather than AnimGraph KernelEvent branching, RON timeline migration, or beat-by-beat timeline commits. | The current timeline runner buffers effects until finalization and AnimGraph KernelEvent predicates are not runtime-executable, so a Rust-side post-application seam proves the reactive contract headlessly while preserving deterministic two-clock behavior and keeping Digimon-specific logic out of shared combat code. | Yes | agent |
