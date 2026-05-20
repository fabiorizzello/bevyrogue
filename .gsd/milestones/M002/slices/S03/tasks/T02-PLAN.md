---
estimated_steps: 12
estimated_files: 2
skills_used: []
---

# T02: Wire EventReader-driven egui phase strip into UiPlugin

Why: The slice demo requires the live windowed UI path to consume the combat event stream, not a hardcoded combat-state phase. This task follows grill-me, tdd, write-docs, bevy, rust-best-practices, and verify-before-complete: keep the runtime system narrow, make its system params read-only except UI-owned state, and attach it to the existing egui pass.

Do:
1. In `src/ui/phase_strip.rs`, add an event-ingest system using Bevy message/event reader semantics (`MessageReader<CombatEvent>` or the project-compatible alias) that filters `CombatEventKind::OnCombatBeat { beat }` and updates only `PhaseStripDisplay`.
2. Add an egui render system or combined system that draws a compact top-center banner/strip using `EguiContexts` and the derived phase label. It should show nothing when there is no current beat and should avoid wall-clock/RNG-driven gameplay effects. A UI-only visibility threshold may use Bevy `Time` if needed, but it must affect only rendering.
3. Register `PhaseStripDisplay` and the phase-strip system in `UiPlugin::build()` in `src/windowed/mod.rs`, next to existing `roster_panel`, `turn_order_panel`, and `combat_panel` wiring.
4. Keep all `bevy_egui` imports feature-gated and keep the non-windowed `cargo test` path clean.

Done when: `cargo build --features windowed` compiles and the actual `UiPlugin` schedules the phase strip in `EguiPrimaryContextPass` from `CombatEvent` messages.

Threat Surface (Q3): No external input; internal combat events are trusted enum payloads. The system must not expose sensitive data.
Requirement Impact (Q4): Supports R002/R004/R005 and S03 acceptance; re-run headless full tests and windowed build.
Failure Modes (Q5): If no `CombatEvent` messages arrive, banner remains absent/stale but combat continues; if egui context is unavailable, Bevy system returns `Result` error like existing panels rather than mutating gameplay. If multiple events arrive in one frame, last beat wins for display only.
Load Profile (Q6): O(events since last frame) scan over Bevy reader cursor; expected event volume is tiny, and 10x combat events should still be negligible because the system does no world queries over units.
Negative Tests (Q7): No events leaves display inactive; non-`OnCombatBeat` events do not change current phase; multiple beats in order select the latest beat.

## Inputs

- `src/ui/phase_strip.rs`
- `src/windowed/mod.rs`
- `src/ui/combat_panel/render.rs`
- `src/combat/observability/events.rs`

## Expected Output

- `src/ui/phase_strip.rs`
- `src/windowed/mod.rs`

## Verification

cargo build --features windowed

## Observability Impact

Adds a visible phase indicator and a UI-owned current-phase resource so a future agent can inspect whether UI consumption is receiving combat beat events.
