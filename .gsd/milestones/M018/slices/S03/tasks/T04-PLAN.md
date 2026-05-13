---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T04: Two distinct Bounce fixture skills exercising different (selector, repeat, curve) tuples

Add two Implemented fixture skills to assets/data/skills.ron that exercise the kernel's generic dispatcher: (a) `chain_bolt` — Bounce{hops:3, selector:LowestHpPctAlive, repeat:NoRepeat} + Damage{base:18, curve:Constant} (canonical bounce, no curve); (b) `arc_bolt` — Bounce{hops:3, selector:NextSlotAlive, repeat:NoRepeat} + Damage{base:24, curve:Falloff{pct:25}} (slot-walking with falloff). Optionally add (c) `echo_strike` — Bounce{hops:2, selector:LowestHpPctAlive, repeat:AllowRepeat} + Damage{curve:PerHop[20,12]} to prove AllowRepeat path. Each must round-trip RON and survive SkillBook::load validation. Add focused unit tests in skills_ron.rs `#[cfg(test)] mod tests`: per-fixture deserialize + validate; assert the in-memory representation matches the expected struct (selector variant, repeat variant, curve variant). Cross-link fixture metadata (id, label) to satisfy any existing data_loaders integration test.

## Inputs

- `DSL schema from T02`
- `existing nova_burst/dark_flood fixture patterns`

## Expected Output

- `chain_bolt + arc_bolt (and optionally echo_strike) fixtures in skills.ron`
- `round-trip + validate tests per fixture`

## Verification

cargo test --lib skills_ron::tests && cargo test --test data_loaders
