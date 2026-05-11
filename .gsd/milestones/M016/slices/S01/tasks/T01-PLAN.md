---
estimated_steps: 1
estimated_files: 2
skills_used: []
---

# T01: Define Tentomon custom signals in RON schema and data

Define `TentomonCustomSignal` enum in `src/data/skills_ron.rs` with variants matching the battery loop capabilities (e.g. `BuildStaticCharge`, `BuildCircuitCharge`, `SpendCircuitCharge`). Expand `SkillCustomSignal` to include `Tentomon(TentomonCustomSignal)`. Then update `assets/data/skills.ron` to inject these signals into Tentomon and Kabuterimon's skills (e.g. `tentomon_basic`, `petit_thunder`, `mega_blaster`).

## Inputs

- `src/data/skills_ron.rs`
- `assets/data/skills.ron`

## Expected Output

- `src/data/skills_ron.rs`
- `assets/data/skills.ron`

## Verification

cargo check && cargo test --no-run
