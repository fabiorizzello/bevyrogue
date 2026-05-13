# S03: Heated + Chilled — damage amp% pipeline + DoT + speed mod

**Goal:** Wire per-status semantics for Heated and Chilled onto the existing StatusBag lifecycle: (a) damage amp% lookup (Heated→Fire +15%, Chilled→Ice +15%) integrated into calculate_damage; (b) Heated DoT (4 HP, Fire-tagged) emitted at turn-end before tick_all (including stunned units, since DoT bypasses stun-skip per canon §H.1); (c) Chilled speed delta derived at AV-gain site (−20% of base Speed) without persisting in SpeedModifier. No new event variants; reuse OnDamageDealt with damage_tag=Fire for DoT. Paralyzed/Slowed/Blessed remain no-op (S04/S05).
**Demo:** Test `status_amp_pipeline.rs`: stesso colpo fire base 100 → unit non-Heated subisce 100, unit Heated subisce 115. Stesso colpo ice base 100 su Chilled subisce 115. DoT Heated visibile in log a turn-end con 4 dmg.

## Must-Haves

- cargo check + cargo test verdi. New integration test `tests/status_amp_pipeline.rs` covers all 4 DoD scenarios: (1) Fire base=100 on non-Heated → 100; (2) Fire base=100 on Heated → 115; (3) Ice base=100 on Chilled → 115; (4) active unit with Heated takes turn → event stream contains OnDamageDealt {amount=4, damage_tag=Fire, ..}. Chilled unit AV gain shows −20% delta vs same unit without Chilled. Existing damage_tests, status_*, combat_coherence, follow_up_chains all still green.

## Proof Level

- This slice proves: integration-test

## Integration Closure

DamageBreakdown gains `status_amp_pct: i32` field (parity with tag_mod_pct/triangle_mod_pct); apply_effects gains immutable `defender_status: Option<&StatusBag>` parameter. AV-gain loop in turn_system/mod.rs reads StatusBag via existing query slot. No new CombatEventKind variants; no new components.

## Verification

- Heated DoT visible via existing OnDamageDealt event (damage_tag=Fire, amount=4); status_amp_pct exposed in DamageBreakdown for downstream log/snapshot symmetry. No JSONL schema bump required (M017/S06 will cover canon naming).

## Tasks

- [x] **T01: status_amp_pct lookup helper + unit tests** `est:S`
  Add a pure lookup `status_amp_pct(bag: &StatusBag, tag: DamageTag) -> i32` in `src/combat/status_effect.rs` returning 115 when (Heated && tag=Fire) or (Chilled && tag=Ice), else 100. Zero coupling to damage/turn pipelines. Covers the canon §H.1 amp% rule for Heated/Chilled. Add 4 unit tests in the existing `#[cfg(test)] mod tests` block: non-Heated→100, Heated+Fire→115, Heated+Ice→100 (wrong tag), Chilled+Ice→115. Skills: tdd, verify-before-complete.
  - Files: `src/combat/status_effect.rs`
  - Verify: cargo test combat::status_effect::tests::status_amp -- --nocapture && cargo check

- [x] **T02: Wire status_amp_pct into calculate_damage + apply_effects plumb** `est:M`
  Extend `DamageBreakdown` with `pub status_amp_pct: i32`. Change `calculate_damage` signature to take `defender_status: Option<&StatusBag>` and apply `× status_amp_pct/100` as a fourth multiplicative factor after tag_mod, tri_mod, break_mod; default 100 when bag is None. Update `apply_effects` (resolution.rs:185) to accept `defender_status: Option<&StatusBag>` and forward to `calculate_damage` at the call site `:281-285`. Update both call sites of `apply_effects` in `src/combat/turn_system/pipeline.rs:280` and `:576` to pass the defender's `&StatusBag` (already queried at `:67, :369`). Update all `apply_effects` callers in `src/combat/resolution_tests.rs` to pass `None` (regression-safe default). Update `DamageBreakdown` destructuring at `resolution.rs:281-285` to include `status_amp_pct` (unused by the OnDamageDealt event for now — kept in the breakdown for log/snapshot symmetry). Skills: api-design, design-an-interface, verify-before-complete.
  - Files: `src/combat/damage.rs`, `src/combat/resolution.rs`, `src/combat/turn_system/pipeline.rs`, `src/combat/resolution_tests.rs`
  - Verify: cargo check && cargo test combat::damage_tests && cargo test combat::resolution_tests

- [ ] **T03: Heated DoT emission at turn-end (bypasses stun)** `est:M`
  In `src/combat/turn_system/mod.rs` around lines 465-513, emit Heated DoT BEFORE `tick_all` and BEFORE the stun-skip early-return. Canon §H.1: Heated ticks 4 HP Fire damage on the affected unit at its own turn-end, and bypasses Paralyzed/Stunned skip (DoT does not require the unit to act). Concretely: restructure the per-turn block so the StatusBag is inspected first — for each `StatusEffectKind::Heated` instance, mutate `unit.hp_current -= 4` (clamped, skip if already KO), then push `CombatEventKind::OnDamageDealt { amount: 4, kind: DamageKind::Neutral, damage_tag: DamageTag::Fire, tag_mod_pct: 100, triangle_mod_pct: 100 }` via `emit_combat_event`. Emit `OnKO` if hp_current ≤ 0 post-tick. Run this DoT pass UNCONDITIONALLY (above stun continue) so a Heated+Stunned unit still burns. Then proceed with the existing stun continue and the pre-existing OnStatusTick/tick_all flow. Audit `src/combat/follow_up.rs` to confirm OnDamageDealt listeners do not require a preceding OnSkillCast (research risk note). Skills: bevy-ecs-expert, verify-before-complete.
  - Files: `src/combat/turn_system/mod.rs`, `src/combat/follow_up.rs`
  - Verify: cargo check && cargo test --test combat_coherence && cargo test --test follow_up_chains

- [ ] **T04: Chilled −20% Speed delta at AV-gain site (derived read)** `est:S`
  Add helper `chilled_speed_delta(bag: &StatusBag, base_speed: i32) -> i32` in `src/combat/status_effect.rs` returning `-(base_speed / 5)` (rounded toward zero, i.e. integer division) when `bag.has(&Chilled)`, else 0. Negative because canon: Chilled slows. Do NOT mutate `SpeedModifier` — derived-read only (avoids stale delta after expiry mid-round). At `src/combat/turn_system/mod.rs:560-570`, extend the AV-gain query tuple to include `Option<&StatusBag>` if not already present, then compute `av_gain = (speed.0 + speed_mod.0 + chilled_speed_delta(bag, speed.0)) * AV_PER_SPEED`. Unit-test the helper in status_effect.rs#tests (3 cases: no bag entry → 0; Chilled present base=100 → −20; Chilled present base=80 → −16). Skills: bevy-ecs-expert, verify-before-complete.
  - Files: `src/combat/status_effect.rs`, `src/combat/turn_system/mod.rs`
  - Verify: cargo check && cargo test combat::status_effect::tests::chilled && cargo test --test combat_coherence

- [ ] **T05: Integration test tests/status_amp_pipeline.rs (slice DoD)** `est:M`
  New integration test file `tests/status_amp_pipeline.rs` covering all S03 DoD scenarios in a single deterministic headless harness. Build minimal apps (no UI, no RNG) and assert: (A) Fire base=100, defender non-Heated, neutral attrs, no weakness → final damage = 100; (B) same with Heated applied → 115; (C) Ice base=100, defender Chilled, neutral attrs → 115; (D) active unit with Heated takes its turn → event stream contains an OnDamageDealt {amount:4, damage_tag: Fire, ..} attributed to that unit. Optional 5th case: Chilled unit AV-gain delta vs control. Use `combat::bootstrap` or direct spawn pattern as in existing `tests/status_*.rs`. Headless first; no `windowed` features. Skills: tdd, verify-before-complete.
  - Files: `tests/status_amp_pipeline.rs`
  - Verify: cargo test --test status_amp_pipeline && cargo test

## Files Likely Touched

- src/combat/status_effect.rs
- src/combat/damage.rs
- src/combat/resolution.rs
- src/combat/turn_system/pipeline.rs
- src/combat/resolution_tests.rs
- src/combat/turn_system/mod.rs
- src/combat/follow_up.rs
- tests/status_amp_pipeline.rs
