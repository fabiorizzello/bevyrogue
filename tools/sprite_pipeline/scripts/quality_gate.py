#!/usr/bin/env python3
"""Quality gate for rendered sprite PNG.

Checks:
- Non-transparent pixel ratio (reject empty renders)
- Color variance (reject all-black, all-white, all-flat)
- Bounding box of opaque pixels (reject too-small subjects)

Returns dict with passed/reason/metrics.

Standalone usage:
    python quality_gate.py path/to/sprite.png
"""

import argparse
import json
import sys
from pathlib import Path
from PIL import Image


def validate(png_path: Path,
             min_alpha_ratio: float = 0.03,
             min_color_variance: float = 8.0,
             min_bbox_ratio: float = 0.10):
    """Return (passed, reason, metrics).

    metrics: dict with computed values for telemetry.
    """
    img = Image.open(png_path).convert("RGBA")
    pixels = list(img.getdata())
    n = len(pixels)
    if n == 0:
        return False, "empty_image", {}

    # Alpha ratio
    opaque = [(r, g, b) for (r, g, b, a) in pixels if a >= 128]
    alpha_ratio = len(opaque) / n
    if alpha_ratio < min_alpha_ratio:
        return False, "render_empty_or_too_small", {"alpha_ratio": alpha_ratio}

    # Color variance (rough std on RGB channels)
    rs = [c[0] for c in opaque]
    gs = [c[1] for c in opaque]
    bs = [c[2] for c in opaque]
    def std(xs):
        m = sum(xs) / len(xs)
        return (sum((x - m) ** 2 for x in xs) / len(xs)) ** 0.5
    rv, gv, bv = std(rs), std(gs), std(bs)
    color_var = (rv + gv + bv) / 3
    if color_var < min_color_variance:
        return False, "render_flat_no_detail", {
            "alpha_ratio": alpha_ratio,
            "color_variance": color_var,
        }

    # Bounding box ratio
    w, h = img.size
    xs_pos = []
    ys_pos = []
    for y in range(h):
        for x in range(w):
            idx = y * w + x
            if pixels[idx][3] >= 128:
                xs_pos.append(x)
                ys_pos.append(y)
    if not xs_pos:
        return False, "no_opaque_pixels", {}
    bbox_w = max(xs_pos) - min(xs_pos) + 1
    bbox_h = max(ys_pos) - min(ys_pos) + 1
    bbox_ratio = max(bbox_w / w, bbox_h / h)
    if bbox_ratio < min_bbox_ratio:
        return False, "subject_too_small", {
            "alpha_ratio": alpha_ratio,
            "color_variance": color_var,
            "bbox_ratio": bbox_ratio,
        }

    return True, "ok", {
        "alpha_ratio": round(alpha_ratio, 4),
        "color_variance": round(color_var, 2),
        "bbox_ratio": round(bbox_ratio, 3),
        "bbox_w": bbox_w,
        "bbox_h": bbox_h,
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("png_path")
    parser.add_argument("--json", action="store_true", help="JSON output")
    args = parser.parse_args()

    passed, reason, metrics = validate(Path(args.png_path))
    result = {"passed": passed, "reason": reason, "metrics": metrics}

    if args.json:
        print(json.dumps(result, indent=2))
    else:
        status = "✓" if passed else "✗"
        print(f"{status} {reason}  metrics={metrics}")

    sys.exit(0 if passed else 1)


if __name__ == "__main__":
    main()
