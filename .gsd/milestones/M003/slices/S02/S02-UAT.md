# S02: S02 — UAT

**Milestone:** M003
**Written:** 2026-05-22T12:12:44.139Z

## S02 UAT: Baby Flame and Baby Burner — rendered impact-frame release

### UAT Type
Manual (K001 visual confirmation) + Headless automated invariant

### Preconditions
- `cargo winx` builds and launches cleanly (binary includes `--features windowed`)
- Both ally Agumon and mirrored-dummy Agumon are visible on screen
- Agumon atlas PNG is loaded (no missing-texture placeholder)
- SP gauge is full (Baby Burner requires SP ≥ cost)

---

### Steps and Expected Outcomes

**1. Idle stance loops on both actors**
- Action: launch `cargo winx`, observe both sprites before any combat input
- Expected: both ally and dummy cycle the idle atlas frames at ~12 fps (not a single frozen frame, not a frantic flicker); neither sprite animates an attack

**2. Baby Flame (skill) — ally cast only**
- Action: trigger Baby Flame from the combat panel (skill input)
- Expected:
  - Only the ally sprite enters the cast → impact → recover sequence; the dummy stays on idle
  - The sequence is strictly linear (no extra windup between impact and recover)
  - Damage resolves visibly on the impact frame (health bar updates on the frame the atlas shows the impact tile, not on keypress)
  - After recover, ally returns to idle

**3. Baby Burner (ultimate) — ally cast only, SP gated**
- Action: confirm SP gauge ≥ cost, trigger Baby Burner from the combat panel (ult burst button)
- Expected:
  - Only the ally sprite enters charge → launch → recovery; dummy stays idle
  - Three distinct phases are visible in sequence; the sprite does not restart at the charge node between phases
  - Damage lands on the launch frame (impact frame within heavy_attack [23,45])
  - SP is consumed; the burst button is disabled when SP < cost (S01 gate, not regressed)
  - After recovery, ally returns to idle

**4. Baby Burner blocked when SP is insufficient**
- Action: drain SP to below ult cost; attempt to trigger Baby Burner
- Expected: burst button is disabled or press is ignored; no animation plays; no barrier is created

**5. Sprite scale is legible**
- Expected: sprites render at ~205 px (Transform scale 0.4); they are visibly smaller than the full viewport but large enough to distinguish animation frames

**6. Animation pacing is smooth**
- Expected: atlas frame advances at ~12 fps (configurable via BEVYROGUE_ANIM_FPS); idle does not complete in under 0.3 s; attack clips are not sub-0.15 s flashes

---

### Edge Cases

- **Unknown skill**: triggering a skill with no start_node mapping (unbridged) → fallback auto-release fires; no crash; bridge silently skips
- **Hot BEVYROGUE_ANIM_FPS=0**: animation clock never ticks; sprites freeze on the current frame; no panic
- **Multiple barrier events in quick succession**: only the casting sprite's barrier drives its mode; cross-unit barriers are filtered by `barrier_targets_sprite`

---

### Not Proven By This UAT

- Multi-hop bounce animation (intentionally removed; bounce is VFX/gameplay-only — MEM061)
- Hurt/flinch reaction animation on the damage target (anim_graph.ron has no hurt node; pre-existing gap)
- 4-per-team multi-slot sprite layout (SPRITE_DISPLAY_SCALE=0.4 is provisional; single-slot only)
- VFX flash particles (S03 scope)
- Non-Agumon Digimon animation (not yet authored)
