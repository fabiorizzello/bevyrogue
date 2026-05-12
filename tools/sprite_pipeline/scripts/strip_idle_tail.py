#!/usr/bin/env python3
"""Rimuove l'ultimo frame se chiamato `idle_tail` da
`output/<char>/latest/sprites_anime/<style>/<anim>.{png,json}`.

Atlas atomici per §2.2c (transizioni via dematerialize/rematerialize, non baked).
Idempotente: no-op se l'ultimo frame non è `idle_tail`.

Usage: python3 strip_idle_tail.py --char agumon --anim skill
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path

from PIL import Image

PIPELINE = Path(__file__).resolve().parents[1]


def strip(char: str, anim: str, style: str = "mihoyo_style_iso45") -> None:
    base = PIPELINE / "output" / char / "latest" / "sprites_anime" / style
    anim_png = base / f"{anim}.png"
    anim_json = base / f"{anim}.json"

    meta = json.loads(anim_json.read_text())
    if not meta["frames"] or meta["frames"][-1].get("name") != "idle_tail":
        print(f"{char}/{anim} ({style}): no idle_tail, skipping")
        return

    fw = meta["frame_width"]
    src = Image.open(anim_png).convert("RGBA")
    new_w = src.width - fw
    cropped = src.crop((0, 0, new_w, src.height))
    cropped.save(anim_png)

    meta["frames"].pop()
    meta["n_frames"] = len(meta["frames"])
    meta["cols"] = meta["n_frames"]
    anim_json.write_text(json.dumps(meta, indent=2))
    print(f"{char}/{anim} ({style}): stripped idle tail -> n_frames={meta['n_frames']}")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--char", required=True)
    ap.add_argument("--anim", required=True)
    ap.add_argument("--style", default="mihoyo_style_iso45")
    args = ap.parse_args()
    strip(args.char, args.anim, args.style)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
