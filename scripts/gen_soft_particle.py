#!/usr/bin/env python3
"""Generate the radial-gradient soft-particle texture (deterministic, stdlib-only).

The single highest-leverage VFX fix: enoki's default `ColorParticle2dMaterial`
draws flat-colored squares (frag = `in.color * color`), so no curve/HDR tuning
makes a procedural effect read as fire/water — scattered squares look like
confetti. `SpriteParticle2dMaterial::from_texture(soft.png)` multiplies the
particle color by this texture's alpha (sprite frag = `out * textureSample`),
turning each quad into a soft round blob; overlapping blobs + HDR bloom = a
glowing body. See `.agents/skills/bevy-enoki-vfx/references/soft-particle-and-layering.md`.

Output: assets/vfx/soft_particle.png — white RGB, radial alpha falloff (1.0 at
center → 0.0 at edge), 64x64 RGBA. No external deps (no PIL): writes the PNG by
hand via zlib so the asset is reproducible on any machine. Deterministic (R004).
"""

import struct
import zlib
from pathlib import Path

SIZE = 64
# Smoothstep falloff exponent: higher = tighter hot core, softer skirt.
FALLOFF = 1.6


def alpha_at(x: int, y: int) -> int:
    """Radial alpha 255 at center → 0 at/after the inscribed radius."""
    cx = cy = (SIZE - 1) / 2.0
    # Normalize so the edge midpoint sits at r = 1.0 (inscribed circle).
    r = (((x - cx) ** 2 + (y - cy) ** 2) ** 0.5) / (SIZE / 2.0)
    t = max(0.0, 1.0 - r)
    # Smoothstep for a soft, round, non-linear skirt, then a falloff power.
    s = t * t * (3.0 - 2.0 * t)
    return int(round((s ** FALLOFF) * 255.0))


def build_rgba() -> bytes:
    rows = bytearray()
    for y in range(SIZE):
        rows.append(0)  # PNG filter type 0 (None) per scanline
        for x in range(SIZE):
            a = alpha_at(x, y)
            rows += bytes((255, 255, 255, a))  # white RGB, radial alpha
    return bytes(rows)


def chunk(tag: bytes, data: bytes) -> bytes:
    return (
        struct.pack(">I", len(data))
        + tag
        + data
        + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
    )


def main() -> None:
    ihdr = struct.pack(">IIBBBBB", SIZE, SIZE, 8, 6, 0, 0, 0)  # 8-bit RGBA
    idat = zlib.compress(build_rgba(), 9)
    png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", ihdr) + chunk(b"IDAT", idat) + chunk(b"IEND", b"")
    out = Path(__file__).resolve().parent.parent / "assets" / "vfx" / "soft_particle.png"
    out.write_bytes(png)
    print(f"wrote {out} ({len(png)} bytes, {SIZE}x{SIZE} RGBA radial soft particle)")


if __name__ == "__main__":
    main()
