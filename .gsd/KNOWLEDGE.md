# Knowledge

Project-specific rules, patterns, and lessons learned. Append-only. Read at the start of every unit.

---

## K001 — Dati Digimon: skill locale `digimon` (2026-04-21)

Per nomi, livelli, attributi, fields, skill, evoluzioni: usare la skill `digimon`, non il training.

- Skill: `.claude/skills/digimon/SKILL.md` (symlink `~/.agents/skills/digimon` per pi/gsd)
- Dati: `.claude/skills/digimon/data/digimon.json` (1488 entry) — **mai leggere direttamente**, usare il CLI
- CLI: `python3 .claude/skills/digimon/scripts/query.py <sub>` — `lookup|search|by-level|by-attribute|by-field|evolutions|skills|stats`

**Naming JP nel dataset:** `rookie`=Child, `ultimate`=Perfect (penultima), `mega`=ultima. Nomi EN→JP noti: Veemon→`V-mon`, Salamon→`Plotmon`. Se `lookup` EN fallisce, ritentare in JP prima di dichiarare assente.

**Usare per:** roster/pool catturabili, attribuzione Vaccine/Virus/Data/Free, derivazione skill franchise, catene evolutive.
**Non usare per:** design meccaniche originali (SP, turn order, ult), bilanciamento numerico (HP/Atk/Def).

## P001 — Kernel generico, specifiche fuori dal kernel (global)

Il kernel combat (`src/combat/`) deve esporre **solo primitive di gameplay generiche**. Nomi, identità, e quirk franchise-specifici Digimon non vivono qui.

- **Dentro al kernel:** turn order, SP/Ult, damage/toughness/stun, status effects, target shapes, event bus, **meccanismo follow-up** (coda FIFO, dispatch, ordine di risoluzione), **`SkillCtx` API** (query read-only + enqueue `Intent` write-deferred), **esecuzione `Intent`** (formula damage, mitigation, break, status tick — single source of truth).
- **Fuori dal kernel:** roster Digimon, signal/identity per-creatura, hook narrativi, mapping a franchise (vedi `src/combat/blueprints/<digimon>/`, `assets/data/*.ron`), **logica delle skill** (`trait Skill::resolve(&mut SkillCtx, &Params)` in Rust, vive nei blueprint), **condizioni che triggerano follow-up ed effetti specifici eseguiti**. RON tiene solo numeri/tag (dmg, hops, sp_cost, scaling, target_shape base), non logica.
- **Regola di mutazione:** le skill non mutano stato direttamente. Producono `Intent` via `ctx.enqueue(...)`; il kernel li risolve nel pipeline. Garantisce determinismo, ordine, single source of truth per le formule.
- **Test:** se aggiungere una feature al kernel richiede di nominare un Digimon specifico, è un segnale che la feature appartiene a un blueprint o ai dati RON, non al kernel.
- **Motivo:** mantenere il kernel riusabile per altri roster/temi e tenibile come motore di regole puro.
- **Stato (2026-05-13):** v0 (M017→M020) usa enum `Effect` + `TargetShape` data-driven; **M021 introduce `trait Skill` + `SkillCtx`** (vedi D-M021-SKILL-CTX-INTENT) e migra le skill esistenti — il vincolo qui sopra è il *target post-M021*, raggiunto incrementalmente.
