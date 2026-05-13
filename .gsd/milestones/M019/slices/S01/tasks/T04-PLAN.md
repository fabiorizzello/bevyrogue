---
estimated_steps: 7
estimated_files: 1
skills_used: []
---

# T04: Integration tests: tests/dr_pipeline.rs

Create `tests/dr_pipeline.rs` (headless integration test) covering the five cases listed in S01 success criteria. Build deterministic units with the same bootstrap pattern used in `tests/status_blessed_offensive.rs`. For each case, insert a `DrBag` with specific instances on the defender directly via the world API (no new RON Effect variant — S01 is formula-side only), trigger a single damaging skill, and assert via `CombatEvent::Damage` (or `OnDamageDealt`) read from the event bus: 
- DR singolo: base=100, one DrInstance{value:0.30} → amount=70.
- DR×N sommato: two DrInstance{value:0.20} → amount=60.
- DR + resist (tag_mod=0.75): base=100, DR=0.20 → amount=60.
- DR durante Break (break_mod=2.0): base=100, DR=0.30 → amount=140.
- Clamp a 0: DR sum=1.5 → amount=0, no panic, defender hp_current unchanged, event still emitted.
Also add a sixth case asserting tick decrements: insert DrInstance{value:0.30,duration:1}, advance one turn-end, verify the instance is dropped and damage no longer mitigated. Tests must be deterministic (no wall-clock, no unseeded RNG).

## Inputs

- `tests/status_blessed_offensive.rs`
- `src/combat/buffs.rs`
- `src/combat/damage.rs`
- `src/combat/resolution.rs`
- `.gsd/milestones/M019/slices/S01/S01-RESEARCH.md`

## Expected Output

- `tests/dr_pipeline.rs`

## Verification

cargo test --test dr_pipeline && cargo test
