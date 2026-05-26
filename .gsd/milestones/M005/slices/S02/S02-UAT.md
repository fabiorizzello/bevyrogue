# S02: Death reaction and field exit — UAT

**Milestone:** M005
**Written:** 2026-05-26T08:46:38.011Z

## UAT: S02 — Death reaction and field exit

### UAT Type
**Hybrid**: Automated proof covers the build/test boundary (all cargo commands green, dep-gating confirmed). Manual visual sign-off (K001) required for the windowed binary.

### Preconditions
- `cargo build --features windowed` exits 0 ✓ (confirmed by automated verification)
- `cargo test --features windowed` exits 0 ✓ (confirmed by automated verification)
- `cargo winx` (or `cargo run --features windowed`) launches the windowed binary with two Agumon sprites in a test encounter

### Automated Verification (Already Confirmed)

| Check | Verdict |
|---|---|
| `cargo test --lib` (headless, 21 tests) | PASS |
| `cargo test --features windowed --lib` (29 tests) | PASS |
| `cargo test --features windowed --bins` (22+2 tests) | PASS |
| Full integration test suite headless | PASS |
| Full integration test suite windowed (33 windowed-only) | PASS |
| `cargo build --features windowed` | PASS |
| `cargo build` (headless) | PASS |
| dep-leak grep (`bevy::render\|wgpu\|winit\|egui\|bevy_render`) | PASS (no matches) |

### Manual Test Steps (K001 — requires cargo winx)

1. Launch `cargo winx` (or `cargo run --features windowed`)
2. Let the encounter proceed until one unit's HP reaches 0
3. Observe the sprite of the KO'd unit

### Expected Outcomes

1. **Death interrupts in-flight skill** — if the KO'd unit was mid-cast, the skill animation stops immediately and the death frames (14–22 per the authored stance graph) begin playing without re-entering the skill node
2. **Death frames play in full** — the sprite plays the complete death animation before any fade begins (post-KO overshoot observability preserved per M002)
3. **Fade starts after death node exits** — once the death animation completes, the sprite's alpha starts decreasing over ~8 animation ticks (~0.67 s at 12 fps)
4. **Entity despawns at alpha 0** — the sprite entity is removed from the field when the fade completes; no ghost entities remain
5. **Non-KO'd sprites unaffected** — sprites that are NOT KO'd continue their normal idle/hurt/skill animations without regression

### Edge Cases

- **Death-precedence**: if both `UnitDied` and `OnHitTaken` arrive for the same target in the same frame, death wins (enforced by `.after(drive_hurt_reactions)` system ordering + `DeathExiting` reconciliation guard)
- **No double FadeOut seed**: guarded by `fade_out.is_none()` in the `advance.exited` death branch — safe to call from multiple frames
- **No divide-by-zero in fade**: `fade_alpha` guards `total_ticks.max(1)`; verified by unit test `fade_alpha_lerps_full_to_zero`
- **Mid-fade despawn by another path**: the `advance_death_fade` query simply stops yielding the entity — no panic

### Not Proven By This UAT
- Visual quality of death frame artwork (asset-authored; not S02 scope)
- Simultaneous multi-unit death in a single frame (not exercised by the single two-sprite encounter)
- K001 visual sign-off — must be confirmed by a human running `cargo winx`
