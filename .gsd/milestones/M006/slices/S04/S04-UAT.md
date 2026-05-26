# S04: Extract Agumon presentation into its own module — UAT

**Milestone:** M006
**Written:** 2026-05-26T14:02:46.867Z

# UAT Type
Structural closeout + build/test gate, with manual runtime equivalence explicitly deferred to K001 human sign-off.

# Preconditions
- Repository is at the S04 completion state in `/home/fabio/dev/bevyrogue`.
- Rust toolchain and project dependencies are installed.
- No local edits are required before running the verification commands.

# Steps
1. Run `RUSTFLAGS='-D warnings' cargo build --features windowed`.
2. Run `cargo test --features windowed --test windowed_only agumon_module_extraction -- --nocapture`.
3. Run `cargo test --features windowed --test windowed_only`.
4. Run `cargo test --test dependency_gating`.
5. For manual K001 sign-off outside auto-mode, launch the windowed build with `cargo run --features windowed` and exercise Agumon presentation flows that previously depended on inline engine wiring (idle/skill/hurt/death, hit flash/shake, Baby Flame charge/projectile/impact, Baby Burner detonate, Sharp Claws slash).

# Expected Outcomes
1. The windowed build succeeds with warnings denied, proving the registry extraction compiles cleanly.
2. The targeted `agumon_module_extraction` test reports 3 passing assertions and confirms engine files contain no `AGUMON_`, `fn on_enter_effect_ids`, `fn skill_start_node`, `fn load_agumon_enoki_vfx`, `enoki_effect_path`, or `digimon/agumon_atlas.png` tokens.
3. The full `windowed_only` harness passes, confirming the new structural seam did not regress existing windowed-only coverage.
4. `dependency_gating` passes 2/2, confirming no windowed/enoki dependency leak into the headless graph.
5. In the manual runtime check, Agumon should behave the same as before the refactor even though all Agumon-specific presentation data now lives under `src/windowed/digimon/agumon/`.

# Edge Cases
- The structural contract is intentionally token-shaped, so harmless code motion inside the Agumon module should not fail the test unless Agumon-specific data leaks back into engine files.
- Dependency gating must still pass even if future Digimon modules are added; any new leak into the headless graph should fail step 4.
- Manual runtime verification should pay special attention to projectile arrival and detonate paths because those previously depended on hardcoded effect-id wiring.

# Not Proven By This UAT
- Auto-mode does not prove pixel-for-pixel behavioral equivalence in the live windowed binary; that remains K001 manual visual sign-off.
- This UAT does not prove the S05 zero-engine-edit Renamon path yet; it only establishes the seam S05 will consume.
