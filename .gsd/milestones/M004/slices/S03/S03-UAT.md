# S03: Variant-selection seam + Baby Burner detonate enrichment — UAT

**Milestone:** M004
**Written:** 2026-05-25T14:49:51.313Z

# S03 UAT

## Headless CI (automated)

- [ ] `cargo test --test animation` passes (≥110 tests, 0 failures) — covers variant-selection determinism, detonate/flash load + curve assertions, validate_effects with DanglingVariant
- [ ] `cargo build --features windowed` compiles clean (no windowed leak from VfxContext or variant map)
- [ ] `cargo test --features windowed --test windowed_only` passes (≥32 tests) — windowed detonate spawn contract holds

## Visual K001 (manual — cannot be CI-asserted)

- [ ] Run `cargo run --features windowed` (alias `cargo winx`)
- [ ] Trigger Baby Burner detonate in-game
- [ ] Confirm: multi-shard outward burst at target, followed by a bright central flash pop
- [ ] Sign off: visual quality acceptable for milestone review
