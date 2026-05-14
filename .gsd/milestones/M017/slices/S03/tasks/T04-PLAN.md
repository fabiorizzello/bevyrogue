---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T04: Chilled −20% Speed delta at AV-gain site (derived read)

Add helper `chilled_speed_delta(bag: &StatusBag, base_speed: i32) -> i32` in `src/combat/status_effect.rs` returning `-(base_speed / 5)` (rounded toward zero, i.e. integer division) when `bag.has(&Chilled)`, else 0. Negative because canon: Chilled slows. Do NOT mutate `SpeedModifier` — derived-read only (avoids stale delta after expiry mid-round). At `src/combat/turn_system/mod.rs:560-570`, extend the AV-gain query tuple to include `Option<&StatusBag>` if not already present, then compute `av_gain = (speed.0 + speed_mod.0 + chilled_speed_delta(bag, speed.0)) * AV_PER_SPEED`. Unit-test the helper in status_effect.rs#tests (3 cases: no bag entry → 0; Chilled present base=100 → −20; Chilled present base=80 → −16). Skills: bevy-ecs-expert, verify-before-complete.

## Inputs

- `src/combat/status_effect.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/speed.rs`
- `src/combat/av.rs`

## Expected Output

- `src/combat/status_effect.rs`
- `src/combat/turn_system/mod.rs`

## Verification

cargo check && cargo test combat::status_effect::tests::chilled && cargo test --test combat_coherence
