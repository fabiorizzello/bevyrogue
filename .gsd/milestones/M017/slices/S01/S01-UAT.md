# S01: Enum rewrite + RON migration + tests cascade — UAT

**Milestone:** M017
**Written:** 2026-05-12T16:51:44.121Z

# S01 UAT: Status Taxonomy Vocabulary Rewrite

## UAT Type
Headless integration — no windowed/UI required.

## Preconditions
- Clean checkout of `milestone/M017` branch
- Rust toolchain matches `rust-toolchain.toml`
- No uncommitted changes to `assets/data/*.ron`

## Steps

1. **Grep guard — src/ and tests/ clean**
   ```bash
   grep -rEn '\b(Burn|Freeze|Shock|DeepFreeze)\b' src/ tests/
   ```
   **Expected:** Only matches in `src/combat/status_effect.rs` (enum declarations for reserved Burn/Shock + inline unit tests), `src/data/skills_ron.rs` (validator guard `StatusEffectKind::Burn | StatusEffectKind::Shock`), and `src/combat/turn_system/mod.rs` (reserved no-op match arms). Zero matches in `tests/`.

2. **Grep guard — canon variants present**
   ```bash
   grep -rE '\b(Heated|Chilled|Paralyzed|Slowed|Blessed)\b' src/ tests/ assets/
   ```
   **Expected:** Multiple matches across src/combat/status_effect.rs, src/data/skills_ron.rs, assets/data/skills.ron, assets/data/units.ron, and test files.

3. **cargo check headless**
   ```bash
   cargo check
   ```
   **Expected:** Exit 0, no errors.

4. **Full integration test suite**
   ```bash
   cargo test
   ```
   **Expected:** All targets green, 0 failed, 0 ignored.

5. **RON load validation — canon ids accepted**
   ```bash
   cargo run --bin combat_cli
   ```
   **Expected:** Exit 0, no "invalid status id" loader errors, combat events flow normally.

6. **RON validator rejects legacy ids** (manual)
   Edit `assets/data/skills.ron`, change one status id to `"burn_v0"`, run `cargo run --bin combat_cli`.
   **Expected:** Load-time error naming the invalid id and listing the 5 valid canon ids. Restore the file after verification.

## Edge Cases

- `Burn` and `Shock` in the enum are declared but not applicable at load-time: if a skill RON uses `"burn"` or `"shock"` as an id, the validator must reject them with a clear error (not silently apply a no-op).
- Skill names containing legacy words (e.g. "Freeze Fang") are cosmetic and must not be rejected by the status id validator.
- `assets/data/units.ron` comments mentioning "Burn" or "Shock" as role descriptions are not status ids and must not be affected.

## Expected Outcomes

- `cargo test` green with 0 failures and 0 ignored across all test targets
- `grep` guard confirms zero taxonomy leakage in src/ and tests/
- `combat_cli` smoke run produces combat events without panics or loader errors

## Not Proven By This UAT

- Per-status semantics: Heated DoT, Chilled −20% speed, Paralyzed skip-turn, Slowed delay-on-apply, Blessed buff dealt + Ult charge (→ S03/S04/S05)
- Policy refresh_max_dur and Debuff-only cleanse behavior (→ S02)
- JSONL log and ValidationSnapshot emitting canon status names (→ S06)
- Stack-aware Heated × N (→ post-M017 §H.5)
