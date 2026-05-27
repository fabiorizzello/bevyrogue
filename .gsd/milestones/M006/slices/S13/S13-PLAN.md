# S13: Port one ranged and one aura/AoE Digimon end-to-end through the windowed seam

**Goal:** Port two more Digimon end-to-end through the windowed presentation seam — one ranged attacker and one aura/AoE type — as a real scale proof that the extension-first architecture works without engine control-flow edits. This validates S06-S12's claims under load with genuinely different effect shapes.
**Demo:** Two more Digimon render and act with no engine control-flow edits — real scale proof

## Must-Haves

- Two additional Digimon render (sprite present), have legal skills, and cast with their own VFX in windowed, added purely by new data + a per-species register() call. git diff confirms zero edits to render core control flow. Headless tests cover their catalog discovery and skill legality; manual K001 confirms visual presentation.

## Proof Level

- This slice proves: headless tests (discovery + skill legality) + manual windowed sign-off (K001)

## Verification

- The two new species exercise the warn-once spawn-miss / cue-miss diagnostics from S06/S08/S12; confirm no diagnostic fires on their happy paths.

## Tasks

- [ ] **T01: Author data + windowed module for a ranged Digimon** `est:L`
  Pick a ranged attacker from the existing roster data (e.g. one with a projectile skill), author its anim_graph/clip/stance/vfx assets and a src/windowed/digimon/<name>/mod.rs register() that populates only its own entries. No render core edits.
  - Files: `src/windowed/digimon/mod.rs`, `assets/digimon`
  - Verify: cargo test (headless green); manual cargo winx shows the ranged Digimon render and cast

- [ ] **T02: Author data + windowed module for an aura/AoE Digimon** `est:L`
  Add a second Digimon whose effect shape is an aura/AoE (different from projectile and melee) the same way, exercising keyed effect registration with a distinct effect topology.
  - Files: `src/windowed/digimon/mod.rs`, `assets/digimon`
  - Verify: cargo test (headless green); manual cargo winx shows the aura/AoE Digimon render and cast

- [ ] **T03: Scale-proof tests and zero-edit assertion** `est:M`
  Add headless tests for the two new species' catalog discovery and skill legality, and extend the source-contract test to assert the render core control flow was not edited to accommodate them. Per the S15 anti-churn rule, the contract-test additions must assert ONLY the durable invariant — the engine render core contains no per-species id — as an absence-guard, NOT exact file shape, so S15 does not have to undo them.
  - Files: `tests/assets_data/catalog_discovery.rs`, `tests/windowed_only/renamon_extension_contract.rs`
  - Verify: cargo test (headless green); cargo test --features windowed --test windowed_only (green)

## Files Likely Touched

- src/windowed/digimon/mod.rs
- assets/digimon
- tests/assets_data/catalog_discovery.rs
- tests/windowed_only/renamon_extension_contract.rs
