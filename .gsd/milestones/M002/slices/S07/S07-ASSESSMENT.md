---
sliceId: S07
uatType: artifact-driven
verdict: PASS
date: 2026-05-21T21:35:39Z
---

# UAT Result — S07

## Checks

| Check | Mode | Result | Notes |
|-------|------|--------|-------|
| Preconditions 1-2: project builds successfully and windowed feature dependencies are available | artifact | PASS | Ran `cargo test --features windowed --test digimon_kits agumon_energy_gauge` via `gsd_exec` run `ca7d73e6-8b58-4404-9bc7-9bd0662a2227`; test binary compiled and executed successfully under `windowed`, proving the current workspace builds with the required feature set. |
| Preconditions 3-4: Agumon has owner-keyed `ult_gauge=energy` metadata and a legacy metadata-free control still exists | artifact | PASS | Ran `cargo test --features windowed --test digimon_kits holy_support_roster_contract` plus `rg` evidence via `gsd_exec` run `623f37c3-8f8f-4781-beba-b1f41f855a92`; observed `blueprint_metadata_stays_optional_for_legacy_units_while_agumon_opts_in ... ok` and source assertions showing `Agumon -> ult_gauge -> Some("energy")` while `Gabumon.blueprint_metadata.0.is_empty()`. |
| Runtime steps 1-9 / Expected outcomes 1-6: live combat spot-check in the windowed app (battle start, visible gauge fill, lock before full, ready exactly at full, drain after ult, legacy-control comparison) | human-follow-up | NEEDS-HUMAN | This UAT lane is artifact-driven, so the agent did not claim a visual/manual PASS for the interactive combat session. Objective automated evidence below proves the same readiness/drain semantics in the real combat runtime, but a human must still perform the on-screen/windowed observation sequence if visual confirmation is required. |
| Automated cross-check step 10 / Expected outcome 7: headless regression confirms fill -> lock -> ready -> drain end to end | artifact | PASS | Ran `cargo test --features windowed --test digimon_kits agumon_energy_gauge` via `gsd_exec` run `ca7d73e6-8b58-4404-9bc7-9bd0662a2227`; observed `test agumon_energy_gauge::agumon_energy_gauge_fills_locks_and_drains_end_to_end ... ok` and `test result: ok. 1 passed; 0 failed`. This directly proves Agumon basics fill the real energy gauge, Ultimate stays locked before full energy, becomes ready at full energy, and drains on cast resolution. |
| Edge case 1: a timeline-backed basic still grants energy so Agumon can reach readiness | artifact | PASS | Covered by the passing end-to-end regression in `ca7d73e6-8b58-4404-9bc7-9bd0662a2227`; the test only reaches the asserted ready state by applying the real basic-action energy grant path. |
| Edge case 2: finalize/bounce reset paths drain energy on ult reset | artifact | PASS | Covered by the passing end-to-end regression in `ca7d73e6-8b58-4404-9bc7-9bd0662a2227`, which asserts post-ultimate drain semantics through the runtime pipeline rather than a stubbed helper. |
| Edge case 3: metadata-free Digimon remain on the legacy behavior without accidental opt-in | artifact | PASS | Covered by `gsd_exec` run `623f37c3-8f8f-4781-beba-b1f41f855a92`; the contract test passed and the captured source snippet shows Gabumon intentionally remains metadata-free while Agumon alone opts into `ult_gauge=energy`. |

## Overall Verdict

PASS — all automatable artifact-driven checks passed, and the only remaining work is optional human visual confirmation of the same combat behavior in the live windowed session.

## Notes

- Primary evidence: `ca7d73e6-8b58-4404-9bc7-9bd0662a2227` (`agumon_energy_gauge_fills_locks_and_drains_end_to_end`) and `623f37c3-8f8f-4781-beba-b1f41f855a92` (legacy roster metadata contract).
- Manual follow-up, if desired: run `cargo run --features windowed --bin bevyrogue`, enter an Agumon combat, and visually confirm the gauge starts unready, fills with basics, unlocks exactly at full energy, then drains to zero after Ultimate; compare with Gabumon as the metadata-free legacy control.
- `Not Proven By This UAT` remains unchanged from the UAT file: exact UI art/animation polish, roster-wide migration beyond Agumon plus one legacy control, and long-session hot-reload behavior were not claimed here.
