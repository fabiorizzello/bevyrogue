# S03: Generalize windowed sprite + wire cue dispatch — UAT

**Milestone:** M006
**Written:** 2026-05-26T11:35:57.621Z

# S03 UAT — Generalize windowed sprite + wire cue dispatch

## UAT Type
K001 Manual — windowed binary must be run by a human; auto-mode cannot launch it (MEM030).

## Preconditions
- Clean `cargo build --features windowed` (exit 0, zero warnings) — confirmed by automated verification.
- Automated gates green: windowed_only 59/59, dependency_gating 2/2.
- Run `cargo run --features windowed` in a terminal; let the battle scene load.

## Test Steps

### 1. Stance/skill playback unchanged
1. Observe Agumon in idle stance — sprite should cycle the idle animation normally.
2. Trigger a skill (e.g., Baby Flame) — sprite should transition through windup → strike → recovery and return to idle.
3. Trigger hurt — sprite should briefly show the hurt reaction and snap back.
4. **Expected:** All stance/skill/hurt/death playback works identically to pre-S03. No freeze, no invisible sprite, no wrong pose.

### 2. Hit flash still fires on impact
1. Land a hit on the opponent.
2. **Expected:** The struck sprite briefly flashes a reddish tint (~8 frames) and returns to normal. Tint peak matches the pre-S03 feel (peak ≈ (1.0, 0.45, 0.45)).

### 3. Sprite shake still fires on impact
1. Land a hit.
2. **Expected:** The struck sprite oscillates with a visible lateral shake for ~8 frames, then snaps back to rest position. No residual offset after shake window expires.

### 4. Camera shake fires on impact (new behaviour)
1. Land a hit.
2. **Expected:** The entire camera view jolts briefly (~8 frames) with a visible shake, then snaps cleanly back to the rest position. No drift or progressive offset accumulation across multiple hits.

### 5. No drift after multiple rapid hits
1. Land 3–4 hits in quick succession (within the same 8-frame windows).
2. **Expected:** After each burst, camera and sprites return cleanly to their rest positions. No accumulated offset.

### 6. RUST_LOG trace confirmation (optional, for trace evidence)
1. Run `RUST_LOG=windowed.agumon_playback=trace cargo run --features windowed`.
2. Land a hit.
3. **Expected:** Terminal shows lines containing "flash+shake armed" and "camera-shake armed" on target `windowed.agumon_playback`.

## Edge Cases
- Rapid multi-hit (same window): idempotent arm resets the window — no double-decay crash.
- Death sequence during flash: flash is suppressed under DeathExiting/FadeOut — sprite should fade cleanly without a tint artefact.

## Not Proven By This UAT
- S04 extraction of AGUMON_* consts into src/windowed/digimon/agumon/ (deferred to S04).
- Second Digimon (Renamon) registration (deferred to S05).
- Particle VFX quality (K001 already covered in S01 UAT).
- Headless combat logic correctness (covered by the headless test suite, not visual UAT).
