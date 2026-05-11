"""Downscale + palette quantize hi-res render → pixel art frames.

Usage:
    python pixelify.py --input <render_dir> --output <output_dir> --size 64 [options]

Options:
    --colors N            Auto palette via median-cut (default 32)
    --palette FILE.gpl    Force specific palette (Aseprite/GIMP .gpl)
    --dither none|fs      Dithering mode (default none)
    --downscale-mode      box|nearest|lanczos (default box for crisp pixel art)
"""

import argparse
from pathlib import Path
from PIL import Image


def parse_gpl_palette(path):
    """Parse GIMP/Aseprite .gpl palette → list of (R,G,B) tuples."""
    colors = []
    with open(path) as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith('#') or line.startswith('GIMP') or line.startswith('Name:') or line.startswith('Columns:'):
                continue
            parts = line.split()
            if len(parts) >= 3:
                try:
                    r, g, b = int(parts[0]), int(parts[1]), int(parts[2])
                    colors.append((r, g, b))
                except ValueError:
                    continue
    return colors


def make_palette_image(colors):
    """Build PIL palette image from list of (R,G,B) tuples for quantize match."""
    pal_img = Image.new("P", (1, 1))
    flat = []
    for c in colors:
        flat.extend(c)
    # Pad palette to 256 entries (PIL P mode requires)
    while len(flat) < 768:
        flat.extend([0, 0, 0])
    pal_img.putpalette(flat[:768])
    return pal_img


def pixelify_frame(input_path, output_path, target_size, n_colors, palette_colors=None, dither_mode='none', downscale='box'):
    img = Image.open(input_path).convert("RGBA")

    # Step 1: downscale with chosen filter
    filter_map = {
        'nearest': Image.Resampling.NEAREST,
        'box': Image.Resampling.BOX,
        'lanczos': Image.Resampling.LANCZOS,
    }
    img = img.resize((target_size, target_size), filter_map[downscale])

    alpha = img.split()[3]
    rgb = img.convert("RGB")

    dither = Image.Dither.FLOYDSTEINBERG if dither_mode == 'fs' else Image.Dither.NONE

    if palette_colors:
        # Quantize against fixed palette
        pal_img = make_palette_image(palette_colors)
        quantized = rgb.quantize(palette=pal_img, dither=dither)
    else:
        # Auto median-cut palette
        quantized = rgb.quantize(colors=n_colors, dither=dither)

    rgb_rgba = quantized.convert("RGBA")
    rgb_rgba.putalpha(alpha)

    # Force fully transparent pixels back to (0,0,0,0)
    pixels = rgb_rgba.load()
    w, h = rgb_rgba.size
    for y in range(h):
        for x in range(w):
            r, g, b, a = pixels[x, y]
            if a < 128:
                pixels[x, y] = (0, 0, 0, 0)
            else:
                pixels[x, y] = (r, g, b, 255)

    rgb_rgba.save(output_path, "PNG")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", required=True, help="Input dir with hi-res frames")
    parser.add_argument("--output", required=True, help="Output dir for pixel frames")
    parser.add_argument("--size", type=int, default=64, help="Target pixel size")
    parser.add_argument("--colors", type=int, default=32, help="Palette color count (auto)")
    parser.add_argument("--palette", default=None, help="Path to .gpl palette file")
    parser.add_argument("--dither", choices=['none', 'fs'], default='none', help="Dithering")
    parser.add_argument("--downscale", choices=['nearest', 'box', 'lanczos'], default='box', help="Downscale filter")
    args = parser.parse_args()

    input_dir = Path(args.input)
    output_dir = Path(args.output)
    output_dir.mkdir(parents=True, exist_ok=True)

    palette_colors = None
    if args.palette:
        palette_colors = parse_gpl_palette(args.palette)
        print(f"[INFO] Loaded palette {args.palette}: {len(palette_colors)} colors")

    frames = sorted(input_dir.glob("*.png"))
    if not frames:
        raise SystemExit(f"No PNG frames in {input_dir}")

    for frame in frames:
        out = output_dir / frame.name
        pixelify_frame(frame, out, args.size, args.colors, palette_colors, args.dither, args.downscale)
        pal_label = f"palette={Path(args.palette).stem}" if args.palette else f"auto-{args.colors}"
        print(f"[INFO] {frame.name} → {args.size}×{args.size}, {pal_label}, dither={args.dither}, scale={args.downscale}")

    print(f"\n[INFO] Pixelified {len(frames)} frames → {output_dir}")


if __name__ == "__main__":
    main()
