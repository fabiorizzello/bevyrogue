#!/usr/bin/env python3
"""Repack a character atlas by cropping every frame to the union alpha bbox.

Reads `assets/digimon/<char>_atlas.{png,json}`, computes the bounding box of
non-transparent pixels across ALL frames (so character motion stays in-frame),
applies a uniform margin, then re-emits a tightly packed atlas with the same
animation indexing.

Run from repo root:
    python3 tools/sprite_pipeline/scripts/repack_atlas.py agumon gabumon ...
    python3 tools/sprite_pipeline/scripts/repack_atlas.py --all
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from PIL import Image

REPO = Path("/home/fabio/dev/bevyrogue")
ATLAS_DIR = REPO / "assets" / "digimon"
MARGIN = 8
ALPHA_THRESHOLD = 8  # ignore near-transparent residue


def slice_atlas(img: Image.Image, meta: dict) -> list[Image.Image]:
    fw = meta["frame_size"]["w"]
    fh = meta["frame_size"]["h"]
    cols = meta["columns"]
    total = meta["total_frames"]
    frames = []
    for i in range(total):
        cx = (i % cols) * fw
        cy = (i // cols) * fh
        frames.append(img.crop((cx, cy, cx + fw, cy + fh)))
    return frames


def union_bbox(frames: list[Image.Image]) -> tuple[int, int, int, int]:
    min_x = min_y = 10**9
    max_x = max_y = -1
    for f in frames:
        alpha = f.split()[-1]
        if ALPHA_THRESHOLD > 0:
            alpha = alpha.point(lambda v: 255 if v >= ALPHA_THRESHOLD else 0)
        bb = alpha.getbbox()
        if bb is None:
            continue
        x0, y0, x1, y1 = bb
        if x0 < min_x:
            min_x = x0
        if y0 < min_y:
            min_y = y0
        if x1 > max_x:
            max_x = x1
        if y1 > max_y:
            max_y = y1
    if max_x < 0:
        raise SystemExit("union bbox empty — atlas appears fully transparent")
    return min_x, min_y, max_x, max_y


def repack(char: str) -> None:
    json_path = ATLAS_DIR / f"{char}_atlas.json"
    png_path = ATLAS_DIR / f"{char}_atlas.png"
    meta_full = json.loads(json_path.read_text())
    meta = meta_full["meta"]
    src = Image.open(png_path).convert("RGBA")
    src_w, src_h = src.size
    frames = slice_atlas(src, meta)
    fw_old = meta["frame_size"]["w"]
    fh_old = meta["frame_size"]["h"]

    x0, y0, x1, y1 = union_bbox(frames)
    # apply margin, clip to original frame bounds
    x0 = max(0, x0 - MARGIN)
    y0 = max(0, y0 - MARGIN)
    x1 = min(fw_old, x1 + MARGIN)
    y1 = min(fh_old, y1 + MARGIN)
    new_w = x1 - x0
    new_h = y1 - y0

    cols = meta["columns"]
    rows = meta["rows"]
    total = meta["total_frames"]
    out = Image.new("RGBA", (new_w * cols, new_h * rows), (0, 0, 0, 0))
    for i, frame in enumerate(frames):
        cropped = frame.crop((x0, y0, x1, y1))
        cx = (i % cols) * new_w
        cy = (i // cols) * new_h
        out.paste(cropped, (cx, cy))

    out.save(png_path, optimize=True)
    meta["frame_size"] = {"w": new_w, "h": new_h}
    meta["repack"] = {
        "bbox": {"x0": x0, "y0": y0, "x1": x1, "y1": y1},
        "margin": MARGIN,
        "source_frame_size": {"w": fw_old, "h": fh_old},
    }
    json_path.write_text(json.dumps(meta_full, indent=2) + "\n")
    print(
        f"{char}: {fw_old}x{fh_old} -> {new_w}x{new_h}"
        f"  atlas {src_w}x{src_h} -> {new_w*cols}x{new_h*rows}"
        f"  frames={total}"
    )


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("chars", nargs="*")
    ap.add_argument("--all", action="store_true")
    args = ap.parse_args()
    if args.all:
        chars = sorted(p.stem.replace("_atlas", "") for p in ATLAS_DIR.glob("*_atlas.json"))
    else:
        chars = args.chars
    if not chars:
        print("provide character names or --all", file=sys.stderr)
        return 2
    for c in chars:
        repack(c)
    return 0


if __name__ == "__main__":
    sys.exit(main())
