#!/usr/bin/env python3
"""Append idle[0] frame to the tail of a per-anim sheet so the clip returns
to a neutral pose visually. Operates on output/<char>/latest/sprites_anime/
<style>/<anim>.png + <anim>.json in place.

Usage:
    python3 append_idle_tail.py --char agumon --anim heavy_attack
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path

from PIL import Image

PIPELINE = Path(__file__).resolve().parents[1]


def append(char: str, anim: str, style: str = "mihoyo_style_iso45") -> None:
    base = PIPELINE / "output" / char / "latest" / "sprites_anime" / style
    anim_png = base / f"{anim}.png"
    anim_json = base / f"{anim}.json"
    idle_png = base / "idle.png"
    idle_json = base / "idle.json"

    meta = json.loads(anim_json.read_text())
    idle_meta = json.loads(idle_json.read_text())

    fw = meta["frame_width"]
    fh = meta["frame_height"]
    assert idle_meta["frame_width"] == fw and idle_meta["frame_height"] == fh, "frame size mismatch"

    # detect tail-already-appended idempotency: last frame name marker
    if meta["frames"] and meta["frames"][-1].get("name") == "idle_tail":
        print(f"{char}/{anim}: idle tail already present, skipping")
        return

    src = Image.open(anim_png).convert("RGBA")
    idle = Image.open(idle_png).convert("RGBA")

    # crop idle frame 0
    f0 = idle_meta["frames"][0]
    idle0 = idle.crop((f0["x"], f0["y"], f0["x"] + f0["w"], f0["y"] + f0["h"]))

    # extend sheet horizontally by one frame
    new_w = src.width + fw
    out = Image.new("RGBA", (new_w, src.height), (0, 0, 0, 0))
    out.paste(src, (0, 0))
    out.paste(idle0, (src.width, 0))
    out.save(anim_png)

    new_idx = meta["n_frames"]
    meta["n_frames"] = new_idx + 1
    meta["cols"] = new_idx + 1
    meta["frames"].append({
        "index": new_idx,
        "name": "idle_tail",
        "x": new_idx * fw,
        "y": 0,
        "w": fw,
        "h": fh,
    })
    anim_json.write_text(json.dumps(meta, indent=2))
    print(f"{char}/{anim}: appended idle tail -> n_frames={meta['n_frames']}")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--char", required=True)
    ap.add_argument("--anim", required=True)
    ap.add_argument("--style", default="mihoyo_style_iso45")
    args = ap.parse_args()
    append(args.char, args.anim, args.style)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
