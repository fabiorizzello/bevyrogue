---
id: T01
parent: S02
milestone: M006
key_files:
  - src/ui/cues.rs
  - src/ui/mod.rs
key_decisions:
  - flash_tint_parametric returns an sRGB (f32,f32,f32) triple (SrgbTriple), not bevy::Color, because bevy_color is render-stack-only and absent headless (R005); S03 maps it to Color::srgb(r,g,b) (MEM113)
  - ParticleBurst stores opaque effect_id: String as the enoki-isolation seam; handle resolution happens only in the S03 windowed binary
  - register() conflict is a startup fail-fast panic naming id+both defs; equal-def re-register is an idempotent no-op (D047/D044)
  - CameraShake reuses shake_offset_parametric verbatim — no separate camera math
duration: 
verification_result: passed
completed_at: 2026-05-26T11:08:17.254Z
blocker_discovered: false
---

# T01: Added headless, enoki-free CueDef/CueRegistry seam plus deterministic flash/shake parametric math in src/ui/cues.rs (ungated lib module)

**Added headless, enoki-free CueDef/CueRegistry seam plus deterministic flash/shake parametric math in src/ui/cues.rs (ungated lib module)**

## What Happened

Created src/ui/cues.rs as an ungated lib module (bevyrogue::ui::cues) and registered it in src/ui/mod.rs with NO windowed cfg gate, leaving the hit_feedback/phase_strip gates untouched and hit_feedback.rs unmodified.

CueDef (Debug/Clone/PartialEq) carries four variants: Flash { peak, ticks }, SpriteShake { amp, freq_x, freq_y, ticks }, CameraShake { amp, freq_x, freq_y, ticks }, and ParticleBurst { effect_id: String }. The String effect-id is the structural enoki-isolation seam — resolved to a bevy_enoki handle only in S03's windowed binary, never here. CueRegistry { entries: HashMap<String, CueDef> } derives Resource/Default/Debug/Clone; register() is idempotent/order-independent (re-registering an EQUAL def is a no-op) and panics naming the id and both defs on a CONFLICTING re-registration (startup fail-fast, D047/D044); get() is a plain map lookup returning None for unknown ids (the documented S03 caller-logs-and-no-ops path, never panics).

Pure math, no std::time/rand (R004): shake_offset_parametric generalizes hit_feedback::shake_offset (Vec2::ZERO guard; decay = remaining/total; amplitude = amp*decay; phase = remaining; sin/cos per-axis) and CameraShake reuses it verbatim. flash_tint_parametric generalizes hit_feedback::flash_tint (white when not flashing; per-channel lerp toward peak).

DEVIATION: the task plan specified flash_tint_parametric returns bevy::Color, but bevy_color is render-stack-only and ABSENT from the headless dependency graph (cargo tree shows 0 bevy_color headless vs 15 windowed; only bevy_math/Vec2 is present headless). Returning Color would pull bevy_color into the lib and break R005 — contradicting the slice's whole purpose. Resolved consistently with the enoki-isolation philosophy: flash_tint_parametric returns an sRGB (f32,f32,f32) triple (type alias SrgbTriple) that S03's windowed binary maps verbatim to Color::srgb(r,g,b). Legacy equivalence is preserved through that mapping (Color::srgb(lerp...) == Color::srgb(flash_tint_parametric(...))). Captured as MEM113.

Added 10 in-file #[cfg(test)] unit tests covering flash white-guard/peak/legacy-match-across-window, shake zero-guard/legacy-match-across-window/decaying-envelope, and registry get-unknown/register-get/idempotent-equal/conflict-panic.

## Verification

Ran all four task-plan verification commands plus the new lib unit tests. cargo check --no-default-features --features dev: clean. cargo test --lib cues (headless): 10/10 pass. cargo test --test dependency_gating (headless): 2/2 pass — bevy_enoki absent from headless graph, present in windowed graph, confirming the seam stays enoki-free. cargo build --features windowed: succeeds. Did not execute the windowed binary (K001). One unit test (shake envelope) initially failed on a wrong sqrt-of-sum assumption (distinct per-axis freqs mean sin^2+cos^2 != 1); fixed the bound to amplitude*sqrt(2) and it passes.

## Verification Evidence

| # | Command | Exit Code | Verdict | Duration |
|---|---------|-----------|---------|----------|
| 1 | `cargo check --no-default-features --features dev` | 0 | pass | 5040ms |
| 2 | `cargo test --no-default-features --features dev --lib cues` | 0 | pass | 1200ms |
| 3 | `cargo test --no-default-features --features dev --test dependency_gating` | 0 | pass | 290ms |
| 4 | `cargo build --features windowed` | 0 | pass | 6390ms |

## Deviations

Task plan step 6 specified flash_tint_parametric returns bevy::Color. bevy_color is render-stack-only and absent from the headless graph, so returning Color would break R005/the headless build. Changed the return type to an sRGB (f32,f32,f32) triple (type SrgbTriple), documented for S03 to map via Color::srgb. Legacy equivalence preserved through that mapping. CueDef::Flash.peak typed as SrgbTriple accordingly.

## Known Issues

none

## Files Created/Modified

- `src/ui/cues.rs`
- `src/ui/mod.rs`
