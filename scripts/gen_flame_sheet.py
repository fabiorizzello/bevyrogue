#!/usr/bin/env python3
"""Generate the Baby Flame "defined flame" flipbook sprite-sheet (deterministic, stdlib-only).

A radial soft blob (see gen_soft_particle.py) can carry a glow *body*, but it has no
silhouette — it can never read as a "defined flame" (the stage the user asked for).
That requires an L3 flipbook: a sprite-sheet whose frames the
`SpriteParticle2dMaterial::new(tex, hframes, vframes)` frag advances over each
particle's lifetime (`floor(lifetime_frac * total_frames)`, particle_sprite_frag.wgsl),
so one emitter renders a flickering flame tongue rather than a static dot.

Output: assets/vfx/flame_sheet.png — a 4x4 grid (16 frames) of 64x64 RGBA cells
(256x256 total). Each frame is a flame-tongue silhouette: rounded wide base, pointed
licking tip, the centerline and lobe wobble advancing per frame so the cycle reads as
a live flicker. RGB carries internal value contrast (white-hot core -> cooler edges);
the particle `color_curve` (HDR) tints + blooms on top. Alpha is the flame silhouette.

Frame ORDER matters: the sprite frag reads rows BOTTOM-UP — frame 0 is grid cell
(hframe 0, vframe 0), and vframe 0 samples the BOTTOM quarter of the image
(v_offset = (max_vframe - vframe - 1) * frame_height). So animation frame `f` is placed
at image grid row counted from the BOTTOM. This script handles that mapping so the
flipbook plays in order.

No external deps (no PIL/numpy): writes the PNG by hand via zlib. Deterministic (R004).
See `.agents/skills/bevy-enoki-vfx/references/target-catalog.md` (L3 flipbook recipe).
"""

import math
import struct
import zlib
from pathlib import Path

HFRAMES = 4
VFRAMES = 4
FRAMES = HFRAMES * VFRAMES
CELL = 64                       # px per frame
SHEET_W = HFRAMES * CELL
SHEET_H = VFRAMES * CELL
TAU = 2.0 * math.pi


def smoothstep(edge0: float, edge1: float, x: float) -> float:
    if edge0 == edge1:
        return 0.0 if x < edge0 else 1.0
    t = max(0.0, min(1.0, (x - edge0) / (edge1 - edge0)))
    return t * t * (3.0 - 2.0 * t)


def flame_pixel(u: float, v: float, phase: float) -> tuple[float, float, float, float]:
    """One flame-tongue pixel.

    u: horizontal, ~[-1,1] across the cell. v: vertical, 0 at bottom, 1 at top.
    phase: 0..1 around the flicker cycle. Returns (r, g, b, a) in 0..1.
    """
    if v <= 0.0 or v >= 1.0:
        return (0.0, 0.0, 0.0, 0.0)

    ph = phase * TAU

    # Centerline sway: the tongue licks side to side, more near the tip (scaled by v).
    cx = 0.20 * math.sin(v * math.pi * 1.4 + ph) * v

    # Tongue profile: rounded wide base, tapering to a pointed tip at v=1.
    # (1 - v) gives the point; the sin lobe rounds/bellies the lower body.
    base = 0.60
    profile = base * (1.0 - v) * (0.55 + 0.45 * math.sin(v * math.pi))
    # Licking lobes: width ripples along the height and advances with the phase.
    profile *= 1.0 + 0.16 * math.sin(v * 7.0 + ph * 2.0)
    # Pinch the very bottom so the base reads as a flame root, not a flat bar.
    profile *= smoothstep(0.0, 0.12, v)
    if profile <= 1e-4:
        return (0.0, 0.0, 0.0, 0.0)

    dist = abs(u - cx) / profile  # 0 at centerline, 1 at the silhouette edge

    # Fairly crisp cel-ish edge (soft over a narrow band, not a fuzzy blob).
    alpha = smoothstep(1.0, 0.72, dist)
    if alpha <= 0.0:
        return (0.0, 0.0, 0.0, 0.0)

    # Internal value contrast: white-hot at the lower core, cooler toward the
    # edges and tip. The particle color_curve (HDR) does the actual hue + bloom,
    # and the frag multiplies texture RGB by it — so the texture is a clean
    # WARM-WHITE (r>=g>=b at a fixed ratio), only modulated by luminance. A
    # neutral warm white keeps the curve's hue intact (an olive/green texture
    # would skew the tint); the flame's heart is bright, its edges dimmer.
    lum = 1.0 - 0.45 * v - 0.35 * dist
    lum = max(0.35, min(1.0, lum))
    r = lum
    g = lum * 0.86
    b = lum * 0.62
    return (r, g, b, alpha)


def render_frame(f: int) -> list[tuple[int, int, int, int]]:
    """Render flame frame `f` as CELL*CELL RGBA tuples (row-major, top row first)."""
    phase = f / FRAMES
    px = []
    for py in range(CELL):
        for pxx in range(CELL):
            u = (pxx - (CELL - 1) / 2.0) / (CELL / 2.0)
            v = 1.0 - py / (CELL - 1)  # bottom of cell = high v
            r, g, b, a = flame_pixel(u, v, phase)
            px.append((
                int(round(r * 255)),
                int(round(g * 255)),
                int(round(b * 255)),
                int(round(a * 255)),
            ))
    return px


def build_rgba() -> bytes:
    # Render every frame, then place it at the grid cell the frag expects.
    frames = [render_frame(f) for f in range(FRAMES)]

    # sheet[image_y][image_x] -> (r,g,b,a)
    sheet = [[(0, 0, 0, 0)] * SHEET_W for _ in range(SHEET_H)]
    for f in range(FRAMES):
        col = f % HFRAMES                 # hframe
        vframe = f // HFRAMES             # 0 = first frames; vframe 0 = BOTTOM row
        grid_row_from_top = (VFRAMES - 1) - vframe
        ox = col * CELL
        oy = grid_row_from_top * CELL
        cell = frames[f]
        for cy in range(CELL):
            for cx in range(CELL):
                sheet[oy + cy][ox + cx] = cell[cy * CELL + cx]

    rows = bytearray()
    for y in range(SHEET_H):
        rows.append(0)  # PNG filter type 0 (None)
        for x in range(SHEET_W):
            rows += bytes(sheet[y][x])
    return bytes(rows)


def chunk(tag: bytes, data: bytes) -> bytes:
    return (
        struct.pack(">I", len(data))
        + tag
        + data
        + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
    )


def main() -> None:
    ihdr = struct.pack(">IIBBBBB", SHEET_W, SHEET_H, 8, 6, 0, 0, 0)  # 8-bit RGBA
    idat = zlib.compress(build_rgba(), 9)
    png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", ihdr) + chunk(b"IDAT", idat) + chunk(b"IEND", b"")
    out = Path(__file__).resolve().parent.parent / "assets" / "vfx" / "flame_sheet.png"
    out.write_bytes(png)
    print(
        f"wrote {out} ({len(png)} bytes, {SHEET_W}x{SHEET_H} RGBA, "
        f"{HFRAMES}x{VFRAMES} flame flipbook)"
    )


if __name__ == "__main__":
    main()
