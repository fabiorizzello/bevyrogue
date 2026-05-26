---
estimated_steps: 11
estimated_files: 2
skills_used: []
---

# T01: CueDef + CueRegistry + parametric cue math in src/ui/cues.rs (lib, no cfg gate)

Why: M006 replaces the ad-hoc Agumon-coupled flash/shake consts with a data-driven CueRegistry of cosmetic primitives (D044/MEM108). S02 delivers the generic, headless-testable seam in the lib crate; S03 consumes it for windowed dispatch. The seam MUST stay enoki-free so the headless build and the dependency_gating test stay green (R002/R005).

Do:
1. Create `src/ui/cues.rs` (NEW). Import only `std::collections::HashMap` and `bevy::prelude::*` (for `Color`, `Vec2`, `Resource`, `Reflect`-free is fine) — do NOT import any `bevy_enoki` or `bevy_render` type.
2. Define `pub enum CueDef` (derive `Debug, Clone, PartialEq`) with variants: `Flash { peak: (f32, f32, f32), ticks: u32 }`, `SpriteShake { amp: f32, freq_x: f32, freq_y: f32, ticks: u32 }`, `CameraShake { amp: f32, freq_x: f32, freq_y: f32, ticks: u32 }`, `ParticleBurst { effect_id: String }`. The `String` effect-id is the explicit enoki-isolation seam — it is resolved to a handle only in the windowed binary (S03), never here.
3. Define `#[derive(Resource, Default, Debug, Clone)] pub struct CueRegistry { entries: HashMap<String, CueDef> }`.
4. `pub fn register(&mut self, id: impl Into<String>, def: CueDef)`: insert; if the id already exists with an EQUAL def, no-op (idempotent, order-independent); if it exists with a DIFFERENT def, `panic!` with a message naming the id and both defs (startup fail-fast per D047/D044).
5. `pub fn get(&self, id: &str) -> Option<&CueDef>`: plain map get; returns `None` for unknown ids (caller logs + no-ops in S03; never panics).
6. Pure math (same file): `pub fn flash_tint_parametric(remaining: u32, total: u32, peak: (f32, f32, f32)) -> Color` — generalizes `hit_feedback::flash_tint`: returns `Color::WHITE` when `remaining==0 || total==0`, else `t = (remaining/total).clamp(0,1)` and `Color::srgb(lerp(1.0, peak.0, t), lerp(1.0, peak.1, t), lerp(1.0, peak.2, t))`. With `peak == (1.0, 0.45, 0.45)` it must equal the legacy `flash_tint`.
7. `pub fn shake_offset_parametric(remaining: u32, total: u32, amp: f32, freq_x: f32, freq_y: f32) -> Vec2` — generalizes `hit_feedback::shake_offset`: returns `Vec2::ZERO` when `remaining==0 || total==0`, else `decay = (remaining/total).clamp(0,1)`, `amplitude = amp*decay`, `phase = remaining as f32`, `Vec2::new(amplitude*(phase*freq_x).sin(), amplitude*(phase*freq_y).cos())`. With `amp==4.0, freq_x==1.7, freq_y==2.3` it must equal the legacy `shake_offset`. CameraShake reuses this exact fn (no separate camera math). NO `std::time`/`rand` — determinism (R004).
8. Add `pub mod cues;` to `src/ui/mod.rs` WITHOUT any `#[cfg(feature = "windowed")]` gate (this is the change that makes it lib/headless-accessible). Leave the `hit_feedback`/`phase_strip` gates untouched; do NOT modify or delete `hit_feedback.rs` this slice.

Done-when: `cargo check --no-default-features --features dev` is clean; `cargo test --no-default-features --features dev dependency_gating` passes (bevy_enoki absent); `cargo build --features windowed` succeeds. Module is reachable as `bevyrogue::ui::cues`.

## Inputs

- `src/ui/hit_feedback.rs`
- `src/ui/mod.rs`
- `src/animation/registry.rs`
- `.gsd/milestones/M006/slices/S02/S02-RESEARCH.md`

## Expected Output

- `src/ui/cues.rs`
- `src/ui/mod.rs`

## Verification

cargo check --no-default-features --features dev && cargo test --no-default-features --features dev dependency_gating && cargo build --features windowed

## Observability Impact

register() panic message names the conflicting id and both CueDef values (startup fail-fast). get() -> None is the documented no-op lookup path for S03.
