#!/usr/bin/env python3
"""Extract dominant palette from image via K-means clustering (pure Python).

Saves as Aseprite/GIMP .gpl format compatible with pixelify.py --palette.

Usage:
    python extract_palette.py --input chr050a01.png --output agumon.gpl --colors 16

Filters out fully-transparent pixels. Sorts by luminance for readability.
"""

import argparse
import random
from pathlib import Path

from PIL import Image


def kmeans(pixels: list, k: int, max_iter: int = 30, seed: int = 42) -> list:
    """K-means returning k cluster centroids (R,G,B) tuples."""
    random.seed(seed)
    n = len(pixels)
    # Init: random unique pixels as centroids
    init_idx = random.sample(range(n), k)
    centroids = [list(pixels[i]) for i in init_idx]

    for iteration in range(max_iter):
        # Assign each pixel to nearest centroid
        cluster_sums = [[0.0, 0.0, 0.0] for _ in range(k)]
        cluster_counts = [0] * k

        for px in pixels:
            best_d = float('inf')
            best_i = 0
            for i, c in enumerate(centroids):
                # Squared euclidean (avoid sqrt)
                d = (px[0] - c[0]) ** 2 + (px[1] - c[1]) ** 2 + (px[2] - c[2]) ** 2
                if d < best_d:
                    best_d = d
                    best_i = i
            cluster_sums[best_i][0] += px[0]
            cluster_sums[best_i][1] += px[1]
            cluster_sums[best_i][2] += px[2]
            cluster_counts[best_i] += 1

        new_centroids = []
        moved = 0.0
        for i in range(k):
            if cluster_counts[i] > 0:
                nc = [cluster_sums[i][j] / cluster_counts[i] for j in range(3)]
            else:
                nc = list(pixels[random.randint(0, n - 1)])
            moved += sum(abs(nc[j] - centroids[i][j]) for j in range(3))
            new_centroids.append(nc)

        centroids = new_centroids
        if moved < 1.0:
            break

    return [tuple(int(round(x)) for x in c) for c in centroids]


def luminance(rgb: tuple) -> float:
    r, g, b = rgb
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def write_gpl(colors: list, name: str, output_path: Path):
    output_path.parent.mkdir(parents=True, exist_ok=True)
    lines = ["GIMP Palette", f"Name: {name}", "#"]
    for r, g, b in colors:
        lum = luminance((r, g, b))
        if lum < 30:
            label = "shadow-deep"
        elif lum < 80:
            label = "shadow-mid"
        elif lum < 140:
            label = "mid-tone"
        elif lum < 200:
            label = "highlight"
        else:
            label = "highlight-bright"
        lines.append(f"{r:3d} {g:3d} {b:3d}\t{label}")
    output_path.write_text("\n".join(lines) + "\n")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--colors", type=int, default=16)
    parser.add_argument("--name", default=None)
    parser.add_argument("--alpha-threshold", type=int, default=200)
    parser.add_argument("--max-samples", type=int, default=10000,
                        help="Cap pixels for K-means speed")
    args = parser.parse_args()

    img = Image.open(args.input).convert("RGBA")
    w, h = img.size
    pixels_all = list(img.getdata())  # list of (r,g,b,a)

    # Filter transparent
    pixels = [(r, g, b) for (r, g, b, a) in pixels_all if a >= args.alpha_threshold]
    if not pixels:
        raise SystemExit("No opaque pixels.")

    print(f"[INFO] {len(pixels)} opaque pixels from {h}×{w}")

    # Subsample
    if len(pixels) > args.max_samples:
        random.seed(42)
        pixels = random.sample(pixels, args.max_samples)
        print(f"[INFO] Subsampled to {len(pixels)}")

    centroids = kmeans(pixels, args.colors)
    print(f"[INFO] Extracted {len(centroids)} centroids")

    centroids.sort(key=luminance)
    name = args.name or Path(args.input).stem
    write_gpl(centroids, name, Path(args.output))
    print(f"[INFO] Saved → {args.output}")
    for c in centroids:
        r, g, b = c
        print(f"  rgb({r:3d}, {g:3d}, {b:3d})  #{r:02X}{g:02X}{b:02X}")


if __name__ == "__main__":
    main()
