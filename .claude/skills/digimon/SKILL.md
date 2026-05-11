---
name: digimon
description: Use when the user asks about Digimon — creature names, levels (Rookie/Child, Champion/Adult, Ultimate/Perfect, Mega/Ultimate), types, attributes (Vaccine/Virus/Data/Free), fields (Nature Spirits, Dragon's Roar, etc.), moves/skills, or evolution chains. A local 1488-entry snapshot is bundled; query via the script below — never read the raw JSON.
---

# Digimon reference

Local snapshot of **1488 Digimon** (source: digi-api.com, Wikimon-based) at `.claude/skills/digimon/data/digimon.json` (~12 MB). Query with the CLI — do **not** read the JSON directly.

## How to query

From the repo root:

```bash
python3 .claude/skills/digimon/scripts/query.py <subcommand> [args]
```

| Subcommand | Example | Returns |
|---|---|---|
| `lookup NAME` | `lookup agumon` | full record (images stripped; pass `--with-images` to keep them) |
| `search QUERY` | `search greymon` | matching `{id, name}` entries |
| `by-level LEVEL` | `by-level rookie` | names at that level |
| `by-attribute ATTR` | `by-attribute vaccine` | names by Vaccine/Virus/Data/Free/Variable |
| `by-field FIELD` | `by-field "nature spirits"` | names by field (element-like grouping) |
| `by-type TYPE` | `by-type dragon` | names by creature type |
| `evolutions NAME` | `evolutions agumon` | `{prior: [...], next: [...]}` with conditions |
| `skills NAME` | `skills agumon` | attack/skill list |
| `levels` | — | unique levels in the dataset |
| `fields` | — | unique fields |
| `attributes` | — | unique attributes |
| `stats` | — | counts per level / attribute / field |

Output is compact JSON on stdout. Add `--detailed` to list-style subcommands (`by-*`, `search`) to include full records instead of just names.

Name matching is **case-insensitive** and accepts substring matches. If multiple entries match, the CLI returns the first one and prints others to stderr — refine with `search` if needed.

## Level naming (important)

DAPI stores **Japanese** naming. English dub uses different terms. The CLI accepts both; queries map as follows:

| JP (stored) | EN-dub alias | Notes |
|---|---|---|
| Baby I | Fresh | |
| Baby II | In-Training | |
| Child | Rookie | |
| Adult | Champion | |
| Perfect | Ultimate | **ambiguous** — see below |
| Ultimate | Mega | **ambiguous** — see below |
| Armor | Armor | Digimentals |
| Hybrid | Hybrid | Spirit Digivolution |

⚠️ "Ultimate" means **different things** in JP vs EN-dub. The CLI treats `ultimate` as the EN-dub meaning (→ JP `Perfect`). To get the highest regular stage, pass `mega`.

## Fields ≈ element system

Digimon has no literal "element" stat, but `field` is the closest thing. Ten fields in the dataset:

`Nature Spirits`, `Deep Savers`, `Dragon's Roar`, `Nightmare Soldiers`, `Wind Guardians`, `Metal Empire`, `Virus Busters`, `Jungle Troopers`, `Dark Area`, `Unknown`

For game design, treat these as elemental affinities (Nature Spirits ≈ earth/plant, Deep Savers ≈ water, Dragon's Roar ≈ fire/dragon, Metal Empire ≈ steel, Nightmare Soldiers ≈ dark, Virus Busters ≈ holy, Wind Guardians ≈ wind/electric, Jungle Troopers ≈ insect/beast, Dark Area ≈ chaos, Nightmare Soldiers ≈ demon).

## Attributes ≠ elements

`Vaccine`/`Virus`/`Data`/`Free` is a **rock-paper-scissors** system: Vaccine > Virus > Data > Vaccine. Useful for combat balance, not elemental types.

## Data gaps

- 141 entries have no `levels` populated (mostly Armor / Hybrid / X-Antibody forms). They won't appear in `by-level` output.
- 1488 total entries; the index was fetched once and is frozen. Re-run `python3 scripts/dump_digimon.py` from the repo root to refresh (writes to `.claude/skills/digimon/data/digimon.json`).
