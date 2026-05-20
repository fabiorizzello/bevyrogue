# Verification — Recheck of INVENTORY claims (qualità, non solo numeri)

Date: 2026-05-20
Method: 6 verificatori paralleli, lettura end-to-end dei file candidati, mapping inline `mod tests` ↔ integration test, scan ulteriori siti fragili.

## Tabella riepilogo (verdict per cluster)

| Cluster | Inventory claim | Verdict | Δ vs inventory |
|---|---|---|---|
| **H1** timeline merge | ~280 delete, "95% identical" | **PARTIAL CONFIRMED** — parametrizable ma readability calo (event-order opaco nei case) | -30 LOC realistici (~250) + doc-comment per case |
| **H2** blueprint param | ~80 delete | **FULL CONFIRMED** — 100% data-driven, no perdita readability | nessun cambio |
| **H3** party validation merge | ~60 delete | **PARTIAL** — 2 di 3 overlap reali, `wrong_pick_count` NON strettamente subsumed; `party_config_deserializes_and_validates` testa **RON load path** = unico | -30 LOC (port deserialize test, mantieni `wrong_pick_count` distinto) |
| **H4** holy_support merge | ~70 delete | **DEBUNKED** — `app_with_holy_support()` ha **2 setup diversi** (minimal vs full kernel); affordance e mechanics testano **layer diversi** (observability pipeline vs state object) | **-50 LOC** (solo tautologia 84-103 ~20 LOC; mantieni file) |
| **V2 anim_registry** | 63 delete tautologia | **CONFIRMED** | nessun cambio |
| **V2 add_new_digimon_isolation** | 119 delete (3 test) | **PARTIAL** — test 1 (metadata optional) unico, 2-3 dup | -40 LOC (delete solo 2 di 3) |
| **V2 passive_canon_support** | 67 delete (2 di 3) | **CONFIRMED** | nessun cambio |
| **V2 status_cleanse_policy** | 38 delete triple-covered | **CONFIRMED** (status_blessed.rs:109-122 + cleanse_effect.rs:94-116 + properties.rs:85-111) | nessun cambio |
| **V2 source_file_loc_limit** | 65 delete meta-lint | **CONFIRMED** (sposta in CI hook) | nessun cambio |
| **V2 combat_cli_shared_surface** | ~40 delete grep | **CONFIRMED** (mantieni subprocess #[ignore], delete grep) | nessun cambio |
| **V2 holy_support_roster_contract:93-101** | 9 delete "tests Default derive" | **DEBUNKED** — testa che **RON roster** non dichiari blueprint_metadata extra per backward-compat. Invariante reale. | **-9 LOC** (KEEP) |
| **V2 passive_kitsune_grace:317-347** | 31 delete "serde derive test" | **DEBUNKED** — è **JSONL roundtrip end-to-end** su CombatEvent emesso dal vero sistema combat. Contratto M021. | **-31 LOC** (KEEP, no canonical sostitutivo necessario) |
| **V2 action_affordance_query:1086-1112** | 15 delete tautologia | **CONFIRMED** (derive PartialEq/Debug) | nessun cambio |
| **V2 deterministic_rng_contract:30-58** | 6 rewrite "seed vs se stesso" | **DEBUNKED** — testa **fork determinism**: 4 stream divergenti da stesso seed. Invariante real. | **-6 LOC rewrite** (KEEP as-is) |
| **V2 anim_graph_asset** | 41 delete (3 di 4) | **PARTIAL** — solo `agumon_sharp_claws_release_kernel_cue_parses` (~24 LOC) overlap, gli altri 2 (`malformed`, `renamon`) sono disgiunti | -17 LOC (delete solo 1 di 3, non 3) |
| **V3 setup_app** | ~180 LOC saving su 26 file | **PARTIAL** — 3 classi distinte (A minimal 12 file, B full combat 5 file, C-F parametrizzato 7 file); **richiede builder pattern**, non `fn test_app()` semplice | saving realistico ~120 LOC, +~80 LOC builder = net ~40 |
| **V3 make_unit + drain** | ~165 LOC | **CONFIRMED safe & quick** — drain pattern 17 file identico, make_unit 8 file identico struttura | nessun cambio |
| **V4 F1 validation_snapshot** | rewrite "fragile snapshot" | **DEBUNKED** — `format_validation_snapshot()` è funzione stabile; status sorted da `status_kind_ord()` (snapshot.rs:222). È **il contract surface**. | -0 (KEEP) |
| **V4 F2 battery_loop `last_transition`** | "verifica contract" | **CONTRACT confermato** — `pub last_transition: Option<BatteryLoopTransition>` in pub struct `BatteryLoopState`, alimenta validation snapshot (tentomon/mod.rs:106-109). Stesso pattern in PredatorLoopState. | -0 (KEEP, documenta) |
| **V4 F3 predator_loop[0]** | "vec index fragile" | **DEBUNKED** — test setup garantisce cardinalità=1 con `track_target(target)` singolo. Non assume ordering. | -0 (KEEP) |
| **V4 F4 predator_loop substring** | rewrite fragile | **CONFIRMED** fragile | nessun cambio |
| **V4 F5 NEW** — 3 siti fragili aggiuntivi | non in inventory | **NUOVO** — `battery_loop_kernel:138`, `dorumon_predator_runtime:164`, `holy_support_mechanics:321` tutti substring sniff su formatted output | +~30 LOC rewrite |
| **W0b ultimate.rs** | 60% delete / 40% relocate | EVIDENCE: **50% delete / 50% keep** (6 unit-level dup, 6 keep B) | -25 LOC delete |
| **W0b status_effect.rs** | 50/50 | EVIDENCE: **64% delete / 36% keep** | +35 LOC delete |
| **W0b kernel/mod.rs** | 40% delete | EVIDENCE: **0% delete / 100% keep** (kernel semantics deep, nessun overlap integration) | -74 LOC delete |
| **W0b enemy_ai.rs** | 0% delete | EVIDENCE: **100% delete** — tutti 5 test dup di `tests/enemy_ai.rs` (inventory aveva detto "no equivalent" → ERRATO) | +149 LOC delete |
| **W0b passive_runner.rs** | 50% delete | EVIDENCE: **0% delete / 100% keep** (signal filter + circuit-breaker = unique) | -70 LOC delete |
| **W0b event_bridge.rs** | 20% delete | EVIDENCE: **0% delete / 100% keep** (dual-signal semantics + batching = unique) | -27 LOC delete |
| **W0b timeline.rs** | 70% delete | EVIDENCE: **100% delete** (4 test compile-validation tutti coperti da `compiled_timeline_builtin_validation.rs`) | +37 LOC delete |
| **W0b toughness.rs** | 70% delete | EVIDENCE: **100% delete** (11 test tutti coperti da `tests/toughness_categories.rs`) | +32 LOC delete |
| **W0b follow_up/triggers.rs** | 80% delete | EVIDENCE: **0% delete / 100% keep** (`evaluate_follow_up` in isolamento, no integration equivalent) | -85 LOC delete |

## W0b totale aggiornato (basato su evidenza, non stima)

| File | Inline LOC | Delete reale | Keep (relocate) reale |
|---|---|---|---|
| ultimate.rs | 258 | 129 | 129 |
| status_effect.rs | 243 | 156 | 87 |
| kernel/mod.rs | 184 | 0 | 184 |
| enemy_ai.rs | 149 | 149 | 0 |
| passive_runner.rs | 141 | 0 | 141 |
| event_bridge.rs | 136 | 0 | 136 |
| timeline.rs | 122 | 122 | 0 |
| toughness.rs | 106 | 106 | 0 |
| follow_up/triggers.rs | 106 | 0 | 106 |
| **Totale** | **1,445** | **662** | **783** |

Prima: 720/720. Dopo: **662/783** (delete -58, relocate +63). Risultato: meno ottimismo sul delete ma anche meno illusione su quali file sono "dup ovvia".

## Bonus non in inventory

### Nuovi siti fragili (V4 F5)
Pattern `formatted.contains("<string-literal>")` su output Debug:
- `battery_loop_kernel.rs:138` → `contains("grant(5)")`
- `dorumon_predator_runtime.rs:164` → `contains("targets=[8:e2:p2]")`
- `holy_support_mechanics.rs:321` → `contains("last=build(2)")`

**Fix**: assert su campi tipizzati dello snapshot, non substring sul formatted.

### V2 doc da aggiungere (memory + DECISIONS)
`last_transition` su `BatteryLoopState`/`PredatorLoopState` è **observability contract pubblico**. Documenta perché il pattern non è fragile, così future verifiche non lo flagga di nuovo.

### W0b enemy_ai.rs è un puro winner mancato
Inventory diceva "no integration equivalent → 100% relocate". **ERRATO**: `tests/enemy_ai.rs` esiste e copre 5 test dup. **+149 LOC delete sicuri** non visti.

---

## Recap LOC corretti (Option 3, post-verification)

| Bucket | Inventory | Verified | Δ |
|---|---|---|---|
| W0a relocate (4 dirs) | 3,312 moved | 3,312 moved | — |
| W0b inline mod tests | 720 delete + 720 relocate | **662 delete + 783 relocate** | -58 / +63 |
| H1 timeline merge | 280 delete | **~250 delete** | -30 |
| H2 blueprint param | 80 delete | 80 delete | — |
| H3 party merge | 60 delete | **~30 delete** | -30 |
| H4 holy_support merge | 70 delete | **~20 delete** | **-50** |
| H5 + T1 + V2 batch | 568 delete | **~528 delete** | -40 (rescatati: roster_contract +9, kitsune_grace serde +31, anim_graph_asset +17, deterministic_rng rewrite +6, less add_new_digimon +40 = net -97 from V2 numbers; consolido a ~528 vs 568 prima) |
| V2 serde canonical | 30 net + 14 rewrite | **0** — passive_kitsune_grace:317-347 è già il canonical, non serve sostituirlo | -30 delete, -14 rewrite |
| V1 rstest/proptest | 360 delete + 79 rewrite | 360 delete + 79 rewrite | — |
| V3 common migration | 455 delete | **~250 delete + ~80 builder helper** | -125 (saving netto inferiore dopo builder pattern) |
| V4 fragility | 0 delete + 93 rewrite | 0 delete + **~95 rewrite** (incl. 3 F5 nuovi -6 deterministic_rng) | +2 rewrite |
| **TOTALE delete** | **~2,623** | **~2,260** | **-363** |
| **TOTALE relocate** | **~4,032** | **~4,095** | +63 |
| **TOTALE rewrite** | **~186** | **~174** | -12 |

## Cosa cambiare in PLAN.md

1. **W0b**: ripianifica per-file con i % evidenziati. Non più "60/40 estimate". 4 file diventano **100% delete safe** (enemy_ai, timeline, toughness, parte di status_effect/ultimate). 3 file diventano **100% relocate** (kernel/mod, passive_runner, event_bridge, follow_up/triggers).
2. **W0b enemy_ai.rs**: marker "Wave 0b winner" — 149 LOC eliminabili senza pensare, completamente subsumed.
3. **H4 ridotto**: rimuovere la wave merge holy_support; mantenere solo delete di 84-103 (tautologia round-trip ~20 LOC). Le due "overlap" sono in realtà angoli diversi e vanno tenute.
4. **H3 ridotto**: portare `party_config_deserializes_and_validates` dentro `party_selection_validation.rs` prima di delete del file. `wrong_pick_count` mantienilo separato (non subsumed).
5. **V2 keep list aggiornata**: `holy_support_roster_contract:93-101`, `passive_kitsune_grace:317-347`, `deterministic_rng_contract:30-58` NON sono tautologie.
6. **V3 builder**: PLAN.md Wave 1 deve creare `TestAppBuilder` non `fn test_app()`. Drain+make_unit restano quick-win indipendenti.
7. **V4 nuova wave o estensione**: aggiungi i 3 siti F5 al rewrite list (battery_loop:138, dorumon_predator_runtime:164, holy_support_mechanics:321).
8. **Aggiungi al backlog** (fuori scope di questa slice ma debito noto):
   - 93% test importano `bevyrogue::combat::*` — debito impl-coupling, scope futuro.
   - Documenta `last_transition` come contract pattern in DECISIONS o memory.

## Cose che restano vere

- Architettura sana: zero god module, zero cross-leaf coupling.
- W0a (dir relocate) intoccato — meccanica.
- H2 + H5 + T1 + grande parte W0b (enemy_ai, timeline, toughness) — winner sicuri.
- V3 drain + make_unit — quick win.
- V1 rstest — confirmed feasible.
