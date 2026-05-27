# S05: Second digimon (Renamon) with zero engine edits — UAT

**Milestone:** M006
**Written:** 2026-05-26T18:53:24.822Z

## UAT: Renamon as a zero-engine-edit extension

### Automated (verified in auto mode)
- [x] cargo test passes (headless suite green).
- [x] cargo test --features windowed --test windowed_only passes (67 tests incl. renamon_extension_contract).
- [x] cargo test --test dependency_gating passes (enoki present in windowed, absent from headless).
- [x] RUSTFLAGS=-D warnings cargo build --features windowed builds clean.
- [x] Source contracts: engine/core files (render.rs, mod.rs) carry no Renamon identifiers; Renamon lives only under src/windowed/digimon/renamon/ plus assets.

### Manual (K001 — requires user)
- [ ] Run cargo winx. Renamon appears as a combatant.
- [ ] Renamon plays idle / skill / hurt / death presentation correctly.
- [ ] Cue-driven flash/shake fires on the diamond_storm skill (ReleaseKernel cue).
- [ ] git diff shows changes limited to the renamon module tree plus assets plus its registration call — zero edits to engine/core files.
