# S07: S07 — UAT

**Milestone:** M002
**Written:** 2026-05-21T21:13:57.018Z

# UAT — S07 Energy-backed ult gauge runtime migration (Agumon)

## UAT Type
Manual runtime spot-check plus headless regression confirmation.

## Preconditions
1. Project builds successfully in the current workspace.
2. Windowed feature dependencies are available.
3. Agumon roster data includes owner-keyed `ult_gauge=energy` metadata.
4. At least one legacy Digimon without ult-gauge metadata remains available as a control.

## Steps
1. Run `cargo run --features windowed --bin bevyrogue` and enter a combat session where Agumon is controllable.
2. Observe Agumon’s ultimate gauge at battle start.
3. Use Agumon basic/non-ultimate actions repeatedly to build energy.
4. Before the gauge reaches max, attempt to select or fire Ultimate.
5. Continue using energy-granting actions until the gauge reaches full.
6. As soon as the gauge reaches full, check Ultimate availability again.
7. Fire Agumon’s Ultimate once it becomes available.
8. After the cast resolves, observe the ult/energy gauge immediately.
9. Compare behavior with a legacy Digimon that has no ult-gauge metadata, if present in the same runtime or a follow-up session.
10. Cross-check the automated regression by running `cargo test --features windowed --test digimon_kits agumon_energy_gauge`.

## Expected Outcomes
1. At battle start, Agumon’s ultimate is not available unless Energy is already full.
2. Basic actions increase Agumon’s visible energy/ult progress.
3. Ultimate remains locked while `Energy.current < Energy.max`, even if legacy UltimateCharge would otherwise be primed.
4. Ultimate becomes available exactly when the gauge reaches full, not earlier.
5. Firing Ultimate drains Agumon’s gauge back to zero immediately after cast resolution.
6. Legacy Digimon without metadata continue to use the old UltimateCharge behavior and do not regress.
7. The headless regression passes and confirms fill -> lock -> ready -> drain end to end.

## Edge Cases
1. A timeline-backed basic must still grant energy; otherwise Agumon can never reach readiness.
2. Bounce/finalize paths must also drain energy on ult reset, not only the main finalize path.
3. Metadata-free Digimon must remain on legacy behavior with no accidental opt-in.

## Not Proven By This UAT
1. Exact UI art/animation polish of the gauge outside the observed readiness/drain semantics.
2. Multi-character balancing or roster-wide migration beyond Agumon plus a legacy control.
3. Long-session hot-reload behavior; that was covered by earlier slices, not this UAT.
