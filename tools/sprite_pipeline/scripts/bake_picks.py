#!/usr/bin/env python3
"""Bake animated WEBP+GIF picks from latest pipeline render.

Reads frames from output/<char>/latest/sprites_anime/<shader>_iso45/frames_<anim>/
and writes to output/_anime_picks_v2/<char>/<shader>/<anim>.{webp,gif}.

Project decision (2026-04-30, see DECISIONS.md): single size = 512, flat layout
(no per-anim subfolder). Both anime_eevee and mihoyo_style baked when present.

Usage:
    bake_picks.py --char agumon
    bake_picks.py --char agumon --shaders anime_eevee mihoyo_style
    bake_picks.py --all                # all chars under output/ with a latest/
    bake_picks.py --char agumon --size 512 --fps 12
"""

import argparse
import json
import os
import shutil
from concurrent.futures import ProcessPoolExecutor, as_completed
from pathlib import Path

from PIL import Image

PIPELINE_DIR = Path(__file__).resolve().parent.parent
PICKS_ROOT = PIPELINE_DIR / "output" / "_anime_picks_v2"

DEFAULT_SHADERS = ["anime_eevee", "mihoyo_style"]
DEFAULT_SIZE = 512
# Source FBX anims baked at 60fps. Per-anim playback fps = SOURCE_FPS / frame_step
# (read from config). Manual --fps overrides per-anim derivation for all anims.
SOURCE_FPS = 60
DEFAULT_FPS = None  # None = derive per-anim from config frame_step
ALPHA_THRESHOLD = 128
GIF_BG_KEY = (255, 0, 255)  # magenta keyed transparent


def load_frames(frames_dir: Path, size: int):
    pngs = sorted(frames_dir.glob("frame_*.png"))
    out = []
    for p in pngs:
        im = Image.open(p).convert("RGBA")
        if im.size != (size, size):
            im = im.resize((size, size), Image.LANCZOS)
        out.append(im)
    return out


def write_webp(frames, out_path: Path, fps: int):
    duration_ms = int(round(1000 / fps))
    frames[0].save(
        out_path,
        format="WEBP",
        save_all=True,
        append_images=frames[1:],
        duration=duration_ms,
        loop=0,
        quality=92,
        method=6,
        lossless=False,
    )


def _key_rgb(im: Image.Image) -> Image.Image:
    """RGBA → RGB with hard alpha threshold; bg = magenta key."""
    rgba = im.convert("RGBA")
    keyed = Image.new("RGB", rgba.size, GIF_BG_KEY)
    mask = rgba.split()[3].point(lambda a: 255 if a >= ALPHA_THRESHOLD else 0)
    keyed.paste(rgba.convert("RGB"), mask=mask)
    return keyed


def write_gif(frames, out_path: Path, fps: int):
    """Global-palette GIF with magenta-key transparency + disposal=2 (no flicker).

    Build single shared palette from concatenated frames so palette-index of the
    magenta key is stable across frames. Without this the transparent index drifts
    between frames and the bg flickers.
    """
    duration_ms = int(round(1000 / fps))
    keyed = [_key_rgb(f) for f in frames]
    w, h = keyed[0].size
    strip = Image.new("RGB", (w, h * len(keyed)))
    for i, f in enumerate(keyed):
        strip.paste(f, (0, i * h))
    strip_p = strip.convert("P", palette=Image.ADAPTIVE, colors=256)
    pal = strip_p.getpalette()[: 256 * 3]
    transp_idx = None
    for i in range(256):
        if tuple(pal[i * 3 : i * 3 + 3]) == GIF_BG_KEY:
            transp_idx = i
            break
    if transp_idx is None:
        # Force magenta into palette at index 0
        pal = list(GIF_BG_KEY) + pal[3:]
        transp_idx = 0
    pal_frames = []
    for f in keyed:
        pf = f.quantize(palette=strip_p, dither=Image.NONE)
        pf.putpalette(pal)
        pal_frames.append(pf)
    pal_frames[0].save(
        out_path,
        format="GIF",
        save_all=True,
        append_images=pal_frames[1:],
        duration=duration_ms,
        loop=0,
        disposal=2,
        transparency=transp_idx,
        optimize=False,
    )


def bake_anim_task(args: tuple) -> dict:
    """Worker for ProcessPoolExecutor: bake one anim of (char, shader)."""
    char, shader, anim, frames_dir, out_dir, size, fps, drop_last = args
    frames = load_frames(Path(frames_dir), size)
    if drop_last > 0 and len(frames) > drop_last + 1:
        frames = frames[:-drop_last]
    if not frames:
        return {"char": char, "shader": shader, "anim": anim, "ok": False, "reason": "no_frames"}
    out_dir = Path(out_dir)
    write_webp(frames, out_dir / f"{anim}.webp", fps)
    write_gif(frames, out_dir / f"{anim}.gif", fps)
    return {"char": char, "shader": shader, "anim": anim, "ok": True}


def _load_anim_steps(char: str) -> dict:
    """Read configs/<char>.json → {anim_name: frame_step}. Falls back empty if missing."""
    cfg_path = PIPELINE_DIR / "configs" / f"{char}.json"
    if not cfg_path.exists():
        return {}
    cfg = json.loads(cfg_path.read_text())
    return {a["name"]: a.get("frame_step", 1) for a in cfg.get("animations", [])}


def collect_jobs(chars, shaders, size, fps_override, drop_last, anim_filter=None):
    """Wipe out_dirs (sequential) + flatten anim work to job tuples.

    fps_override: None = derive per-anim from frame_step. Number = force same fps
    for all anims (legacy behavior).
    """
    jobs = []
    skipped = []
    for char in chars:
        latest = PIPELINE_DIR / "output" / char / "latest"
        anim_steps = _load_anim_steps(char)
        for shader in shaders:
            variant_dir = latest / "sprites_anime" / f"{shader}_iso45"
            if not variant_dir.exists():
                skipped.append((char, shader, "no_render"))
                continue
            out_dir = PICKS_ROOT / char / shader
            if anim_filter is None and out_dir.exists():
                shutil.rmtree(out_dir)
            out_dir.mkdir(parents=True, exist_ok=True)
            for frames_dir in sorted(variant_dir.glob("frames_*")):
                anim = frames_dir.name[len("frames_"):]
                if anim_filter is not None and anim != anim_filter:
                    continue
                if fps_override is not None:
                    fps = fps_override
                else:
                    step = anim_steps.get(anim, 4)
                    fps = max(1, round(SOURCE_FPS / max(step, 1)))
                jobs.append((char, shader, anim, str(frames_dir), str(out_dir),
                             size, fps, drop_last))
    return jobs, skipped


def discover_chars() -> list:
    out_root = PIPELINE_DIR / "output"
    chars = []
    for d in sorted(out_root.iterdir()):
        if d.name.startswith("_") or not d.is_dir():
            continue
        if (d / "latest").exists():
            chars.append(d.name)
    return chars


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--char", default=None)
    ap.add_argument("--all", action="store_true", help="Bake every char with a latest/ run")
    ap.add_argument("--shaders", nargs="*", default=DEFAULT_SHADERS)
    ap.add_argument("--size", type=int, default=DEFAULT_SIZE)
    ap.add_argument("--fps", type=int, default=None,
                    help="Force fps for all anims. Default = derive per-anim from "
                         "configs/<char>.json frame_step (SOURCE_FPS=60 / step).")
    ap.add_argument("--anim", default=None,
                    help="Bake only this anim (e.g. 'idle'). When set, dir wipe is "
                         "skipped — only the named anim's webp+gif are overwritten.")
    ap.add_argument("--drop-last", type=int, default=0,
                    help="Force-drop N trailing frames before encoding (use when source "
                         "anim has hold-pose padding at the tail that breaks GIF loop). "
                         "Independent from auto-dedup of frame[0]==frame[-1].")
    ap.add_argument("--workers", type=int, default=max(1, (os.cpu_count() or 4) - 1),
                    help="Parallel anim-bake workers (ProcessPoolExecutor). Default = "
                         "cpu_count-1.")
    args = ap.parse_args()

    if args.all:
        chars = discover_chars()
    elif args.char:
        chars = [args.char]
    else:
        ap.error("--char or --all required")

    fps_label = args.fps if args.fps is not None else "per-anim"
    anim_label = args.anim if args.anim else "ALL"
    print(f"Bake picks: size={args.size} fps={fps_label} workers={args.workers} "
          f"anims={anim_label} shaders={args.shaders}")
    jobs, skipped = collect_jobs(chars, args.shaders, args.size, args.fps,
                                 args.drop_last, anim_filter=args.anim)
    for c, s, why in skipped:
        print(f"  - {c}/{s}: SKIP ({why})")
    if not jobs:
        return

    done_per_pair = {}
    if args.workers <= 1:
        for j in jobs:
            r = bake_anim_task(j)
            key = (r["char"], r["shader"])
            done_per_pair[key] = done_per_pair.get(key, 0) + (1 if r["ok"] else 0)
    else:
        with ProcessPoolExecutor(max_workers=args.workers) as ex:
            futs = [ex.submit(bake_anim_task, j) for j in jobs]
            for fut in as_completed(futs):
                r = fut.result()
                key = (r["char"], r["shader"])
                done_per_pair[key] = done_per_pair.get(key, 0) + (1 if r["ok"] else 0)
    for (c, s), n in sorted(done_per_pair.items()):
        out = PICKS_ROOT / c / s
        print(f"  + {c}/{s}: {n} anims -> {out}")


if __name__ == "__main__":
    main()
