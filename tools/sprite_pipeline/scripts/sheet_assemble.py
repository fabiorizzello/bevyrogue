"""Assemble pixel frames into spritesheet PNG + JSON atlas.

Usage:
    python sheet_assemble.py --input <frames_dir> --output <sheet.png> [--cols N]
"""

import argparse
import json
from pathlib import Path
from PIL import Image


def assemble_sheet(frames, cols, output_png):
    if not frames:
        raise SystemExit("No frames to assemble")

    first = Image.open(frames[0])
    fw, fh = first.size

    n = len(frames)
    rows = (n + cols - 1) // cols

    sheet = Image.new("RGBA", (cols * fw, rows * fh), (0, 0, 0, 0))

    atlas_frames = []
    for i, frame_path in enumerate(frames):
        img = Image.open(frame_path).convert("RGBA")
        if img.size != (fw, fh):
            raise SystemExit(f"Frame size mismatch: {frame_path.name} = {img.size}, expected {(fw, fh)}")
        col = i % cols
        row = i // cols
        x, y = col * fw, row * fh
        sheet.paste(img, (x, y))
        atlas_frames.append({
            "index": i,
            "name": frame_path.stem,
            "x": x,
            "y": y,
            "w": fw,
            "h": fh,
        })

    sheet.save(output_png, "PNG")

    return {
        "sheet": output_png.name,
        "frame_width": fw,
        "frame_height": fh,
        "cols": cols,
        "rows": rows,
        "n_frames": n,
        "frames": atlas_frames,
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", required=True, help="Frames dir")
    parser.add_argument("--output", required=True, help="Output PNG sheet path")
    parser.add_argument("--cols", type=int, default=0, help="Columns (0 = single row)")
    parser.add_argument("--atlas-json", default=None, help="Output atlas JSON path")
    args = parser.parse_args()

    frames_dir = Path(args.input)
    output_png = Path(args.output)
    output_png.parent.mkdir(parents=True, exist_ok=True)

    frames = sorted(frames_dir.glob("*.png"))
    if not frames:
        raise SystemExit(f"No PNG in {frames_dir}")

    cols = args.cols if args.cols > 0 else len(frames)
    atlas = assemble_sheet(frames, cols, output_png)

    atlas_path = Path(args.atlas_json) if args.atlas_json else output_png.with_suffix(".json")
    with open(atlas_path, "w") as f:
        json.dump(atlas, f, indent=2)

    print(f"[INFO] Sheet: {output_png} ({atlas['cols']}×{atlas['rows']} grid, {atlas['n_frames']} frames)")
    print(f"[INFO] Atlas: {atlas_path}")


if __name__ == "__main__":
    main()
