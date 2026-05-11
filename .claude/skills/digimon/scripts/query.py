#!/usr/bin/env python3
"""Query the local Digimon catalog.

Data source: `assets/data/digimon.json` (dumped from digi-api.com).
Output: compact JSON on stdout; diagnostics on stderr.
"""
from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from functools import lru_cache
from pathlib import Path

# EN-dub → JP (DAPI-stored) level names. Note the "Ultimate"/"Mega" swap.
LEVEL_ALIASES = {
    "fresh": "Baby I",
    "baby": "Baby I",
    "baby i": "Baby I",
    "in-training": "Baby II",
    "in training": "Baby II",
    "baby ii": "Baby II",
    "rookie": "Child",
    "child": "Child",
    "champion": "Adult",
    "adult": "Adult",
    "ultimate": "Perfect",  # EN-dub "Ultimate" == JP "Perfect"
    "perfect": "Perfect",
    "mega": "Ultimate",  # EN-dub "Mega" == JP "Ultimate"
    "armor": "Armor",
    "hybrid": "Hybrid",
    "unknown": "Unknown",
}


def find_data_file() -> Path:
    """Locate `digimon.json` by walking up from this script."""
    here = Path(__file__).resolve()
    for parent in [here, *here.parents]:
        for candidate in (
            parent / "assets" / "data" / "digimon.json",
            parent / "data" / "digimon.json",
            parent / "digimon.json",
        ):
            if candidate.is_file():
                return candidate
    raise SystemExit("digimon.json not found (looked for assets/data/digimon.json upward from script)")


@lru_cache(maxsize=1)
def load_catalog(path_str: str) -> list[dict]:
    with open(path_str, encoding="utf-8") as f:
        return json.load(f)["digimon"]


def strip_images(entry: dict) -> dict:
    return {k: v for k, v in entry.items() if k != "images"}


def match_names(catalog: list[dict], query: str) -> list[dict]:
    q = query.strip().lower()
    # exact first
    exact = [d for d in catalog if d["name"].lower() == q]
    if exact:
        return exact
    # then startswith
    starts = [d for d in catalog if d["name"].lower().startswith(q)]
    if starts:
        return starts
    # then substring
    return [d for d in catalog if q in d["name"].lower()]


def emit(obj, args) -> None:
    indent = 2 if getattr(args, "pretty", False) else None
    print(json.dumps(obj, ensure_ascii=False, indent=indent))


# --- subcommands -------------------------------------------------------------


def cmd_lookup(catalog, args):
    matches = match_names(catalog, args.name)
    if not matches:
        emit({"error": "not_found", "query": args.name}, args)
        sys.exit(2)
    if len(matches) > 1:
        print(
            f"note: {len(matches)} matches for '{args.name}', returning '{matches[0]['name']}'. "
            f"Others: {[m['name'] for m in matches[1:10]]}",
            file=sys.stderr,
        )
    entry = matches[0]
    if not args.with_images:
        entry = strip_images(entry)
    emit(entry, args)


def cmd_search(catalog, args):
    q = args.query.strip().lower()
    matches = [d for d in catalog if q in d["name"].lower()]
    if args.detailed:
        emit([strip_images(m) for m in matches], args)
    else:
        emit([{"id": m["id"], "name": m["name"]} for m in matches], args)


def cmd_by_level(catalog, args):
    raw = args.level.strip().lower()
    canonical = LEVEL_ALIASES.get(raw)
    if canonical is None:
        emit(
            {
                "error": "unknown_level",
                "input": args.level,
                "known": sorted(set(LEVEL_ALIASES.values())),
            },
            args,
        )
        sys.exit(2)
    matches = [d for d in catalog if any(l["level"] == canonical for l in (d.get("levels") or []))]
    if args.detailed:
        emit([strip_images(m) for m in matches], args)
    else:
        emit(
            {
                "level": canonical,
                "alias_input": args.level,
                "count": len(matches),
                "names": [m["name"] for m in matches],
            },
            args,
        )


def _by_simple(catalog, key, subkey, value, detailed, args):
    needle = value.strip().lower()
    matches = [
        d for d in catalog if any(item[subkey].lower() == needle for item in (d.get(key) or []))
    ]
    if detailed:
        emit([strip_images(m) for m in matches], args)
    else:
        emit({"query": value, "count": len(matches), "names": [m["name"] for m in matches]}, args)


def cmd_by_attribute(catalog, args):
    _by_simple(catalog, "attributes", "attribute", args.attribute, args.detailed, args)


def cmd_by_field(catalog, args):
    _by_simple(catalog, "fields", "field", args.field, args.detailed, args)


def cmd_by_type(catalog, args):
    _by_simple(catalog, "types", "type", args.type, args.detailed, args)


def cmd_evolutions(catalog, args):
    matches = match_names(catalog, args.name)
    if not matches:
        emit({"error": "not_found", "query": args.name}, args)
        sys.exit(2)
    m = matches[0]
    emit(
        {
            "name": m["name"],
            "id": m["id"],
            "prior": [
                {"name": e["digimon"], "id": e["id"], "condition": e.get("condition", "")}
                for e in m.get("priorEvolutions") or []
            ],
            "next": [
                {"name": e["digimon"], "id": e["id"], "condition": e.get("condition", "")}
                for e in m.get("nextEvolutions") or []
            ],
        },
        args,
    )


def cmd_skills(catalog, args):
    matches = match_names(catalog, args.name)
    if not matches:
        emit({"error": "not_found", "query": args.name}, args)
        sys.exit(2)
    m = matches[0]
    emit({"name": m["name"], "skills": m.get("skills") or []}, args)


def cmd_levels(catalog, args):
    vals = Counter()
    for d in catalog:
        for l in d.get("levels") or []:
            vals[l["level"]] += 1
    emit([{"level": k, "count": v} for k, v in vals.most_common()], args)


def cmd_fields(catalog, args):
    vals = Counter()
    for d in catalog:
        for f in d.get("fields") or []:
            vals[f["field"]] += 1
    emit([{"field": k, "count": v} for k, v in vals.most_common()], args)


def cmd_attributes(catalog, args):
    vals = Counter()
    for d in catalog:
        for a in d.get("attributes") or []:
            vals[a["attribute"]] += 1
    emit([{"attribute": k, "count": v} for k, v in vals.most_common()], args)


def cmd_stats(catalog, args):
    lv, at, fd = Counter(), Counter(), Counter()
    no_level = 0
    for d in catalog:
        if not d.get("levels"):
            no_level += 1
        for l in d.get("levels") or []:
            lv[l["level"]] += 1
        for a in d.get("attributes") or []:
            at[a["attribute"]] += 1
        for f in d.get("fields") or []:
            fd[f["field"]] += 1
    emit(
        {
            "total": len(catalog),
            "without_level": no_level,
            "by_level": dict(lv.most_common()),
            "by_attribute": dict(at.most_common()),
            "by_field": dict(fd.most_common()),
        },
        args,
    )


# --- CLI ---------------------------------------------------------------------


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description="Query the local Digimon catalog.")
    p.add_argument("--data", type=Path, help="Path to digimon.json (auto-detected by default).")
    p.add_argument("--pretty", action="store_true", help="Indent JSON output.")
    sub = p.add_subparsers(dest="cmd", required=True)

    s = sub.add_parser("lookup", help="Get full record for a Digimon.")
    s.add_argument("name")
    s.add_argument("--with-images", action="store_true")
    s.set_defaults(func=cmd_lookup)

    s = sub.add_parser("search", help="Search Digimon by name substring.")
    s.add_argument("query")
    s.add_argument("--detailed", action="store_true")
    s.set_defaults(func=cmd_search)

    s = sub.add_parser("by-level", help="List Digimon at a level (JP or EN-dub names).")
    s.add_argument("level")
    s.add_argument("--detailed", action="store_true")
    s.set_defaults(func=cmd_by_level)

    s = sub.add_parser("by-attribute", help="Filter by Vaccine/Virus/Data/Free/Variable.")
    s.add_argument("attribute")
    s.add_argument("--detailed", action="store_true")
    s.set_defaults(func=cmd_by_attribute)

    s = sub.add_parser("by-field", help="Filter by field (element-like grouping).")
    s.add_argument("field")
    s.add_argument("--detailed", action="store_true")
    s.set_defaults(func=cmd_by_field)

    s = sub.add_parser("by-type", help="Filter by creature type (Reptile, Dragon, ...).")
    s.add_argument("type")
    s.add_argument("--detailed", action="store_true")
    s.set_defaults(func=cmd_by_type)

    s = sub.add_parser("evolutions", help="Prior and next evolutions for a Digimon.")
    s.add_argument("name")
    s.set_defaults(func=cmd_evolutions)

    s = sub.add_parser("skills", help="Attack / skill list for a Digimon.")
    s.add_argument("name")
    s.set_defaults(func=cmd_skills)

    sub.add_parser("levels", help="Unique levels in the dataset.").set_defaults(func=cmd_levels)
    sub.add_parser("fields", help="Unique fields in the dataset.").set_defaults(func=cmd_fields)
    sub.add_parser("attributes", help="Unique attributes in the dataset.").set_defaults(func=cmd_attributes)
    sub.add_parser("stats", help="Aggregate counts by level/attribute/field.").set_defaults(func=cmd_stats)

    return p


def main() -> int:
    args = build_parser().parse_args()
    data_path = args.data or find_data_file()
    catalog = load_catalog(str(data_path))
    args.func(catalog, args)
    return 0


if __name__ == "__main__":
    sys.exit(main())
