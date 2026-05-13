# M017: M017: Status taxonomy v0 rewrite (canon §H.1)

**Vision:** Allineare il combat kernel al canon §H.1: rimpiazzare la tassonomia attuale (`Burn`/`Freeze`/`Shock`/`DeepFreeze`) con i 5 status canon `Heated`/`Chilled`/`Paralyzed`/`Slowed`/`Blessed` (+ 2 reserved gas-era). Policy `refresh_max_dur` single-instance per (target,kind), cleanse default Debuff-only. Le 5 semantiche per-status (DoT, amp%, skip turn, delay, buff dealt) cablate nelle rispettive pipeline. JSONL log + ValidationSnapshot usano nomi canon. Nessuna nuova reactive event (è M020), nessun DR (è M019), nessun TargetShape expansion (è M018) — questo milestone è **solo** la fondazione vocabolaria + apply/refresh/tick + per-status effect base.

## Success Criteria

- `cargo check` + `cargo test` (full headless integration suite) verdi a fine milestone
- Zero referenze a `Burn`/`Freeze`/`Shock`/`DeepFreeze` nel codice o nei test (tranne `Burn`/`Shock` come variant reserved §H.1)
- 5 status canon (`Heated`/`Chilled`/`Paralyzed`/`Slowed`/`Blessed`) implementati con semantica §H.1 corretta
- Policy single-instance + refresh_max_dur verificata da test deterministici
- Cleanse rimuove solo `BuffKind::Debuff`; `Blessed` (cleanse-immune) sopravvive a cleanse
- JSONL log + ValidationSnapshot emettono nomi canon (no leak della vecchia tassonomia)
- RON loader rifiuta status id non canon a load-time con errore chiaro
- Nessuna regressione su test esistenti non-status (combat_coherence, follow_up_chains, form_identity)

## Slices

- [x] **S01: S01** `risk:high` `depends:[]`
  > After this: `cargo check` + `cargo test` full suite verdi senza referenze alla vecchia tassonomia. `grep -r 'Burn\|Freeze\|Shock\|DeepFreeze' src/ tests/` non trova match (eccetto Burn/Shock reserved).

- [x] **S02: S02** `risk:medium` `depends:[]`
  > After this: Test deterministico: apply Heated(dur=2), re-apply Heated(dur=1), check dur=2. Cleanse rimuove Debuff ma non Buff cleanse-immune.

- [x] **S03: S03** `risk:medium` `depends:[]`
  > After this: Test `status_amp_pipeline.rs`: stesso colpo fire base 100 → unit non-Heated subisce 100, unit Heated subisce 115. Stesso colpo ice base 100 su Chilled subisce 115. DoT Heated visibile in log a turn-end con 4 dmg.

- [x] **S04: S04** `risk:medium` `depends:[]`
  > After this: Test `status_paralyzed_skip.rs`: scenario con seed fisso, 100 turn iterations Paralyzed, skip count nel range deterministico atteso. Test `status_slowed_delay.rs`: applicare Slowed pusha la timeline visibilmente.

- [ ] **S05: S05** `risk:low` `depends:[]`
  > After this: Test `status_blessed_offensive.rs`: unit Blessed colpisce → dmg ×1.15. Test `status_blessed_ult_charge.rs`: unit Blessed esegue azione → +1 Ult charge oltre baseline. Test `status_blessed_cleanse_immune.rs`: cleanse non rimuove Blessed.

- [ ] **S06: Observability — canon JSONL log + ValidationSnapshot** `risk:low` `depends:[S01,S02,S03,S04,S05]`
  > After this: Scripted scenario CLI: applica Heated + Chilled + Paralyzed + Slowed + Blessed su units diversi → JSONL log analizzato via grep test, zero match su vocabolario legacy. ValidationSnapshot.statuses_per_unit deterministico in test fixture.

## Boundary Map

## Boundary Map

"## Boundary map\n\n**In scope:**\n- `src/combat/status_effect.rs` — rewrite enum + componente + apply/refresh/tick\n- `src/combat/damage.rs` — hook amp% lookup nel pipeline\n- `src/combat/speed.rs` — Chilled −20% via SpeedModifier\n- `src/combat/sp.rs` / `ultimate.rs` — Blessed +1 Ult charge hook\n- `src/combat/turn_system/pipeline.rs` — Paralyzed skip-turn check + Slowed delay-on-apply\n- `src/combat/events.rs` — payload OnStatusApplied usa nuovo StatusKind (no nuove varianti reactive — quello è M020)\n- `src/combat/observability.rs` / `log.rs` / `jsonl_logger.rs` — naming canon\n- `src/data/skills_ron.rs` — Effect::ApplyStatus payload migrato + validator\n- `assets/data/skills.ron` — id migrati\n- `tests/status_*.rs`, `tests/combat_coherence.rs`, `tests/follow_up_chains.rs`, `tests/form_identity.rs` — migrate references\n- `docs/combat_current.md` — sezione status aggiornata\n\n**Out of scope (delegato a milestone successivi):**\n- DR pipeline `BuffKind::DR` + clamp 0.5 → **M019**\n- Heal/Cleanse Effects (`Effect::EmitHeal`, `Effect::EmitCleanse`) come variant → **M019** (questo milestone usa cleanse già esistente con il nuovo filtro per kind)\n- Nuove reactive event variants (`StatusApplied` come event tipizzato, `UltimateUsed`, `UnitDied` payload extension) → **M020**\n- AdvanceTurn/DelayTurn split + cap ±50% + gauge clamp [0,200] → **M018**\n- TargetShape resolver expansion → **M018**\n- Blueprint trait/registry refactor → **M021**\n- UI / sprite render → **M023+**\n- Stack-aware status (Heated × N) → deferred §H.5 (post-M017)"
