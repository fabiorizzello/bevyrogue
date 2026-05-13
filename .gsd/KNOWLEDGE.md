# Knowledge

Project-specific rules, patterns, and lessons learned. Append-only. Read at the start of every unit.

---

## K001 — Fonte canonica dati Digimon: skill `digimon` (locale)

**Data:** 2026-04-21

Il progetto ha tema Digimon. Per qualsiasi dato su nomi, livelli (Rookie/Champion/Ultimate/Mega), attributi (Vaccine/Virus/Data/Free), fields, skill reali, evoluzioni: **usare la skill locale `digimon`**, NON conoscenza dal training.

- **Skill location (canonical):** `.claude/skills/digimon/SKILL.md`
- **Skill location (pi loader):** `~/.agents/skills/digimon` → symlink alla stessa directory; permette a pi/gsd di esporre la skill nella lista `<available_skills>` (Claude Code la trovava già da `.claude/skills/`, pi no).
- **Snapshot dati:** `.claude/skills/digimon/data/digimon.json` (1488 entry, ~12 MB, sorgente digi-api.com / Wikimon)
- **Query CLI:** `python3 .claude/skills/digimon/scripts/query.py <subcommand>` (dalla repo root)
- **Regola:** mai leggere `.claude/skills/digimon/data/digimon.json` direttamente — passare sempre dal CLI (output JSON compatto, evita di saturare il context).

**Subcomandi utili:** `lookup NAME`, `search QUERY`, `by-level rookie`, `by-attribute vaccine`, `by-field "nature spirits"`, `evolutions NAME`, `skills NAME`, `stats`.

**Gotcha naming:** il dataset usa naming JP. `ultimate` nel CLI = JP "Perfect" (penultima evo). Per Mega/ultimo stadio usare `mega`. Rookie = JP "Child".

**Gotcha naming — nomi EN → canon JP nel dataset** (lista non esaustiva, aggiornare quando emergono):
- `Veemon` → **`V-mon`**
- `Salamon` → **`Plotmon`**

Se `lookup NAME_EN` non trova nulla, ritentare con la forma JP prima di dichiarare il digimon assente.

**Quando consultarla:**
- Definizione/revisione del roster di partenza o del pool catturabili.
- Attribuzione di attributi (Vaccine/Virus/Data/Free) ai digimon — il design del combat si regge su questo asse.
- Derivazione di skill reali dai kit franchise (punto di partenza per il design delle skill del prototipo).
- Catene evolutive (non nel prototipo v1, ma probabile in evoluzioni future del design).

**Quando NON consultarla:**
- Design di meccaniche (skill point, turn order, ultimate) — sono originali del nostro progetto, i dati Digimon non rispondono.
- Bilanciamento numerico (HP/Atk/Def delle skill) — i dati del franchise non mappano 1:1 sul nostro modello.
