# ANGLE 3 — Visual / windowed-proxy tests

**Confidence: high.** Read the files; verified what each assertion actually proves.

The windowed VFX cluster splits cleanly into "genuine data contract" and "proxy
for a look only a human can judge."

## KEEP — genuine parse / data contracts

These `ron::from_str::<T>()` real git-tracked assets and assert structure. They
catch a concretely-bad outcome: a malformed `.ron` that would crash or silently
no-op at runtime. Nothing visual about them.

- `enoki_skill_effects_parse.rs` (5 fns) — parses the 5 Agumon particle `.ron`
  files. A broken particle asset fails to load → no effect. Real contract.
- `vfx_asset_impact_render.rs` (7 fns) — parses `agumon/vfx.ron` into `VfxAsset`.
- `enoki_impact_effect_parses.rs` (1 fn) — same family.
- The asset-shape half of `renamon_extension_contract` (stance/clip/anim_graph
  `.ron` token contracts) — pins the data shape a new Digimon must satisfy.

## CUT — visual proxy via source text

- `vfx_windowed_contracts.rs` (1 fn, 5 assertions): asserts `render.rs` source
  text *contains* `"Bloom"`, `"Hdr"`, `"Tonemapping::"`, `"DebandDither::Enabled"`,
  `"Color::linear_rgba"`. This proves the source *mentions* bloom — it cannot
  prove bloom renders, is wired to the camera, or looks right. The actual intent
  (HDR bloom + overbright color → a specific on-screen look) is **K001-manual by
  definition**; the headless signoff for it is the UAT doc
  (`docs/uat/M004-vfx-signoff.md`, already WAIVED). This is the textbook
  "purely-visual test faked as a source-text presence check." It buys nothing the
  human signoff doesn't already own, and it churns whenever the camera setup is
  refactored.

## Boundary note for the cut

`vfx_windowed_contracts.rs` does **not** protect dep-gating (R005/R016 — "no
windowed deps leak into headless"). That invariant is enforced by the build
itself (`#![cfg(feature = "windowed")]` + the headless default build) and by
`tests/dependency_gating.rs`, not by this file. Deleting it removes a weak visual
proxy, not a dep-gating guard. Overbright/HDR *headless proxy* coverage that is
worth keeping lives in the data-parse tests above (the colors are in the parsed
`.ron`), not in the source-text mention of `"Bloom"`.

## Verdict

Cut `vfx_windowed_contracts.rs`. Keep every parse/data contract — they are not
visual tests, they are asset-integrity tests.
