# M011 Context — Combat Architecture & Synergistic Roster (v5.3 alignment)

## Why this milestone

`combat_design.md` v5.3 (cut MVP del 2026-04-27) ha consolidato il design del combat in una forma stabile: 6 linee Digimon, 2 assi di matchup (Damage Tag + Attribute Triangle), Tempo Resistance, Toughness 3 categorie, Form Identity per Adult, Tamer Commands, DNA Chips schema. M008/M009 hanno chiuso il combat funzionale-ma-rigido; M010 ha tentato il refactoring multi-fase ma è stata interrotta lasciando un bug pipeline residuo (vedi `.gsd/M010-HANDOFF.md`) e un roster ancora in shape v4.

M011 allinea l'engine al design v5.3, sblocca il bug residuo, riscrive il roster MVP a 6 linee con archetipi distinti, e produce uno strumento di playtest CLI per validare il rebalance numerico a feel. È il milestone che trasforma combat_design.md da spec a codice eseguibile end-to-end con TTK target verificabili.

## Goals

1. **Unblock pipeline** (S01): action lifecycle multi-fase realmente funzionante, 21/21 binari integration verdi.
2. **Schema breaking changes atomici** (S02–S03): Damage Tag rename + EvoStage schema, fatti subito così il rebalance numerico lavora sulla shape finale.
3. **Playtest harness** (S04): combat_cli interactive headless come dogfood tool dalle slice intermedie in poi e vehicle UAT a fine milestone.
4. **Combat systems v5.3** (S05–S08): Resource caps + Child, Tempo Resistance, Toughness 3 cat + Break Seal, Form Identity framework con 6 Adult wired.
5. **Rebalance numerico verificabile** (S09): TTK target boss 4–7 / mini-boss 3–5 / minion 2–3 turni via 3 fixture scenario + UAT manuale 30 minuti.

## Non-goals (→ M012)

- Tamer Gauge resource + 3 Commands (Data Scan / Emergency Guard / Retreat).
- Enemy Counterplay 4 trait (Type Trap / Reactive Armor / Break Seal nemico / Tempo Anchor).
- Charged Attacks telegrafati con Danger Window.
- DNA Chips schema RON (R074 resta active schema-only fino a M012).

## Non-goals (post-MVP)

- Istanze concrete DNA Chips (lista 6–8 chip iniziali).
- Gameplay Perfect/Ultimate/SuperUltimate (schema EvoStage forward-compat in S03, ma stage gameplay non implementato).
- Linee Digimon tagliate dal MVP roster (Guilmon, V-mon, Huckmon, Impmon, Plotmon, Terriermon).

## Architectural decisions captured at planning time

- **D043** Attribute Triangle v5.3 in-line nella formula damage, no stato persistente. Supersedes D016.
- **D044** Damage Tag rename atomico (Element → DamageTag), no alias.
- **D045** Form Identity riusa l'infrastruttura follow-up: estende `FollowUpTrigger` e `Effect` con varianti round-scoped + `RoundFlags` component.
- **D046** Re-entrancy follow-up bounded dai dati (stack, cooldown, RoundFlags), non dall'engine. Supersedes D026.

D016 e D026 restano in `DECISIONS.md` con marker `Superseded by D043` / `Superseded by D046` — register append-only, why-trail conservato per ricostruzione futura.

## Key constraints

- **Headless first** (D015): tutto il combat_cli e tutti i test girano senza display interattivo. Nessuna dipendenza winit/wgpu/egui fuori da feature gate `windowed`.
- **No per-Digimon code** (D020): Form Identity, Child mechanics, Tempo Resistance, Toughness categorie — tutto data-configured in RON. Zero `if unit.id == "..."`.
- **Determinismo** (R019): test riproducibili byte-per-byte con stesso seed.
- **Combat event bus single source** (D022): UI/log/CLI leggono CombatEvent, non mutano stato.

## Risks

| Risk | Mitigation |
|---|---|
| ApplyDeferred su Bevy 0.18 non sblocca il pipeline come previsto | S01 è prima slice e prerequisito di tutto; se fallisce, blocker esplicito al planning del fix invece di proseguire |
| Rename atomico Element→DamageTag rompe assert testuali nei test | Diff meccanico; test failures sono bug di rinaming, non di logica — sweep singolo |
| D046 senza cap engine espone loop patologici non vincolati dai dati | UAT in S09 valida che le risorse finite del kit bound naturalmente la chain. Safety net per-turno solo se UAT lo richiede |

## Pre-existing artifacts to honor

- `.gsd/M010-HANDOFF.md` — diagnosi del bug pipeline da M010, base per S01.
- `docs/combat_design.md` v5.3 — spec autoritativo, da aggiornare in S02 (sez. 1+2 framing operativo D043) e S08 (sez. 9 Form Identity wired).
- `assets/data/units.ron`, `assets/data/skills.ron` — sorgenti del rebalance S09.
- `.gsd/KNOWLEDGE.md` K001 — la skill `digimon` resta canonical per dati canon (naming, attributes, evo chains).

## Demo state at milestone end

`cargo run --bin combat_cli` permette al giocatore di selezionare 4 alleati dal roster a inizio run, poi su un encounter boss (es. Devimon Adult Virus) di scegliere skill+target turno per turno, vede HP/SP/Energy/Toughness/buff post-azione in stdout, le 6 Form Identity Adult triggerano correttamente, il TTK rispetta il target (boss 4–7 turni), e il log JSONL è leggibile per debug.
