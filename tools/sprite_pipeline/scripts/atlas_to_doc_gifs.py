#!/usr/bin/env python3
"""Produce idle->anim->idle GIFs for each state-machine doc.

Output: docs/future_design_draft/digimon/<char>/gifs/<doc_stem>.gif
Mapping (per char) is hard-coded against existing design docs.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

from PIL import Image

REPO = Path("/home/fabio/dev/bevyrogue")
ATLAS_DIR = REPO / "assets" / "digimon"
DOCS_DIR = REPO / "docs" / "future_design_draft" / "digimon"
SCALE = 1  # 1 = native frame_size (atlases are pre-cropped to bbox via repack_atlas.py)
FPS = 12
FRAME_MS = int(1000 / FPS)
IDLE_HOLD_FRAMES = 6  # ~0.5s @ 12fps prima e dopo l'anim

MAPPING: dict[str, dict[str, str]] = {
    "agumon": {
        "01_basic_surudoi_tsume": "attack",
        "02_skill_baby_flame": "heavy_attack",
        "03_ult_baby_burner": "skill",
    },
}


def slice_atlas(png_path: Path, meta: dict) -> list[Image.Image]:
    img = Image.open(png_path).convert("RGBA")
    fw = meta["frame_size"]["w"]
    fh = meta["frame_size"]["h"]
    cols = meta["columns"]
    total = meta["total_frames"]
    frames: list[Image.Image] = []
    for i in range(total):
        cx = (i % cols) * fw
        cy = (i // cols) * fh
        cell = img.crop((cx, cy, cx + fw, cy + fh))
        if SCALE != 1:
            cell = cell.resize((max(1, fw // SCALE), max(1, fh // SCALE)), Image.NEAREST)
        frames.append(cell)
    return frames


def write_gif(frames: list[Image.Image], out: Path) -> None:
    out.parent.mkdir(parents=True, exist_ok=True)
    if not frames:
        return
    frames[0].save(
        out,
        save_all=True,
        append_images=frames[1:],
        duration=FRAME_MS,
        loop=0,
        disposal=2,
        optimize=True,
    )


def build_idle_anim_idle(
    all_frames: list[Image.Image],
    anims: dict,
    anim_name: str,
) -> list[Image.Image]:
    idle = anims["idle"]
    target = anims[anim_name]
    idle_frames = all_frames[idle["start_index"] : idle["end_index"] + 1]
    anim_frames = all_frames[target["start_index"] : target["end_index"] + 1]
    pre = (idle_frames * ((IDLE_HOLD_FRAMES // len(idle_frames)) + 1))[:IDLE_HOLD_FRAMES]
    post = pre
    return pre + anim_frames + post


def main() -> int:
    produced = 0
    for char, doc_to_anim in MAPPING.items():
        json_path = ATLAS_DIR / f"{char}_atlas.json"
        png_path = ATLAS_DIR / f"{char}_atlas.png"
        if not json_path.exists() or not png_path.exists():
            print(f"skip {char}: missing atlas", file=sys.stderr)
            continue
        meta_full = json.loads(json_path.read_text())
        meta = meta_full["meta"]
        anims = meta_full["animations"]
        all_frames = slice_atlas(png_path, meta)
        out_dir = DOCS_DIR / char / "gifs"
        for doc_stem, anim_name in doc_to_anim.items():
            if anim_name not in anims:
                print(f"skip {char}/{doc_stem}: anim '{anim_name}' missing", file=sys.stderr)
                continue
            seq = build_idle_anim_idle(all_frames, anims, anim_name)
            out = out_dir / f"{doc_stem}.gif"
            write_gif(seq, out)
            rel = out.relative_to(REPO)
            print(f"{rel}  frames={len(seq)}  (idle+{anim_name}+idle)")
            produced += 1
    if produced == 0:
        print("no gifs produced", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
