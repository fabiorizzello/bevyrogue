# S05 Research — Blessed: buff dmg ×1.15 + Ult charge +1 + cleanse-immune

## Summary
Blessed is already a first-class kind (S02). `BuffKind::Buff` classification is wired and
`StatusBag::cleanse_debuffs()` already excludes it. The S05 work is purely two attacker-side
hooks in the action-resolution path: a dmg-dealt multiplier on outgoing damage, and a +1 Ult
charge increment per action while the actor has Blessed. The cleanse-immune DoD test is
essentially a free byproduct of S02 (an analogous test exists at `tests/status_cleanse_policy.rs`).

## Implementation Landscape

- `src/combat/status_effect.rs` — `StatusEffectKind::Blessed` variant exists; `classify_buff_kind`
  maps it to `BuffKind::Buff`; `StatusBag::cleanse_debuffs` already preserves it. **No change needed**
  for cleanse-immune; just exercise it from a new test file with the slice-required name.

- `src/combat/damage.rs` — `calculate_damage(attacker, attack, defender, weaknesses)` is the
  single dmg-dealt site. Final formula: `round(base * tag_mod * tri_mod * break_mod)`. **Seam:**
  add an attacker-buff multiplier `blessed_mod = 1.15 if attacker has Blessed else 1.0` as a new
  factor in the product, OR (cleaner) accept a `&StatusBag` for the attacker (or a precomputed
  `attacker_dmg_mult: f32`) and fold it in. Signature change cascades to one call site
  (`resolution.rs:285`) and the existing `damage_tests.rs` (additive — extend, don't break).
  Recommended path: pass `attacker_dmg_mult: f32` to keep `damage.rs` agnostic of status taxonomy.

- `src/combat/resolution.rs` — `apply_effects(...)` is the single dispatch into damage. Currently
  it does NOT receive the attacker's `StatusBag`. **Seam:** add a parameter
  `attacker_statuses: Option<&StatusBag>` (or a precomputed bool/multiplier) to:
    1. Compute the 1.15 mult for the `calculate_damage` call.
    2. Trigger a +1 `attacker_ult.try_add(1)` per action when Blessed is present.
  The Ult-charge bump is action-scoped: place it **once at the end of apply_effects**, gated on
  `outcome.succeeded`, so Basic/Skill/Ult all benefit (canon §H.1 says "per action taken"). Note
  the existing `UltEffect::Reset` branch zeroes `attacker_ult.current` — the +1 must happen
  BEFORE the Ult cast resets the meter, or be skipped when `ult_effect == Reset` (design call;
  recommend: skip on Reset — Blessed charges *future* Ult, not the one currently firing).

- `src/combat/turn_system/pipeline.rs:280, 576` — only two call sites of `apply_effects`. Both
  have access to the attacker entity and could fetch `Option<&StatusBag>` via the existing query
  (StatusBag is already part of the attacker tuple in `mod.rs:67`). Trivial wire-up.

- `src/combat/ultimate.rs` — no change. The +1-from-Blessed lives in `apply_effects` (same
  pattern as `UltEffect::GainFromBasic` which already mutates `attacker_ult` directly there). The
  event-bus accumulator (`ult_accumulation_system`) is for trigger-typed party-event charging and
  is orthogonal to per-actor Blessed.

- `src/combat/sp.rs` — no change. Blessed affects Ult charge, not SP.

## Recommendation — build order

1. **T01: Cleanse-immune test (cheapest first proof).** Add `tests/status_blessed_cleanse_immune.rs`
   using `StatusBag::apply(Blessed, …)` + `cleanse_debuffs()` — proves S02 wiring already satisfies
   this DoD line. Zero src changes. Locks the canon line as a regression guard at the slice level.
2. **T02: Dmg-dealt ×1.15 hook.** Thread `attacker_dmg_mult: f32` (default 1.0) through
   `apply_effects` → `calculate_damage`. Compute it at the call sites as
   `if attacker_bag.has(Blessed) { 1.15 } else { 1.0 }`. Add `tests/status_blessed_offensive.rs`:
   spawn attacker with Blessed in StatusBag, run a Basic, assert damage dealt equals
   `round(base * tag * tri * 1.15)` vs control without Blessed.
3. **T03: Ult-charge +1 per action hook.** In `apply_effects`, after the `match resolved.ult_effect`
   block, if attacker had Blessed AND `ult_effect != Reset` AND `outcome.succeeded`, call
   `attacker_ult.try_add(1)`. Wire `Option<&StatusBag>` parameter at both pipeline call sites.
   Add `tests/status_blessed_ult_charge.rs`: baseline run (no Blessed) → record ult delta;
   Blessed run → assert delta is baseline+1.

## Verification

```bash
cargo check
cargo test --test status_blessed_cleanse_immune
cargo test --test status_blessed_offensive
cargo test --test status_blessed_ult_charge
cargo test                          # full suite — guard against regressions in damage/ult tests
```

## Risks (low)

- **Signature churn**: `apply_effects` already has 11 params; adding a 12th (StatusBag or
  multiplier) touches ~20 call sites in `resolution_tests.rs`. Mitigation: pass `Option<&StatusBag>`
  with `None` default at test sites — they all currently pass no buff context, so a sweeping
  `None` insertion is mechanical.
- **Double-charge on basics**: `UltEffect::GainFromBasic` already adds `charge_per_event`. The
  Blessed +1 is a separate additive bump, applied after that branch — verify with the
  ult_charge test that Basic-with-Blessed yields `charge_per_event + 1`, not double.
- **Ult-on-Ult interaction**: design call — does Blessed grant +1 to the meter that's about to
  be reset by the Ult action itself? Recommend skipping the +1 when `ult_effect == Reset` to
  avoid wasted charge and surprising "self-feeding" semantics. Confirm with canon §H.1 wording
  before T03; if ambiguous, file a decision capture.
- **Cap clamp**: `UltimateCharge::try_add` already clamps at `cap`, so +1 at cap is a no-op; the
  test should pick a charge state below cap to observe the +1.
