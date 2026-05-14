---
estimated_steps: 1
estimated_files: 15
skills_used: []
---

# T03: CastId propagation in CombatEvent + pipeline::step_app + emit sites

Aggiungere cast_id: CastId come campo di CombatEvent e propagarlo da pipeline::step_app a tutti i call-site emit (~50). Emit pre-cast usano CastId::ROOT. CastIdGen Resource monotonic. Aggiorna tutti i call-site CombatEvent {...} via rg. Aggiorna test pattern-match con .. rest. Test tests/cast_id_propagation.rs: (a) eventi durante cast condividono cast_id; (b) cast-scoped ≠ ROOT; (c) eventi pre-cast = ROOT.

## Inputs

- `src/combat/events.rs`
- `src/combat/api/intent.rs`
- `src/combat/turn_system/pipeline.rs`

## Expected Output

- `src/combat/events.rs`
- `src/combat/api/intent.rs`
- `src/combat/turn_system/pipeline.rs`
- `src/combat/turn_system/mod.rs`
- `src/combat/follow_up.rs`
- `src/combat/damage.rs`
- `src/combat/resolution.rs`
- `src/combat/status_effect.rs`
- `src/combat/stun.rs`
- `src/combat/toughness.rs`
- `src/combat/ultimate.rs`
- `src/combat/sp.rs`
- `src/combat/kernel.rs`
- `src/combat/jsonl_logger.rs`
- `tests/cast_id_propagation.rs`

## Verification

cargo check (headless + windowed) puliti. cargo test full suite (~74) verde. rg 'CombatEvent \{' src/ | rg -v 'cast_id' → 0. cargo test --test cast_id_propagation verde (3 assertion). JSONL output contiene cast_id su ogni evento.

## Observability Impact

CombatEvent JSONL nuovo campo cast_id:u32 (additivo non-breaking). Pre-cast usano 1 (ROOT) — distinguibili da cast reali (>1).
