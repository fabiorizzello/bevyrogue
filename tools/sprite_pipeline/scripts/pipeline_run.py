#!/usr/bin/env python3
"""Sprite pipeline orchestrator with run isolation, manifests, quality gates.

Features:
- Run versioning: each invocation creates output/<char>/runs/<id>/ workspace
- Snapshot palettes + shaders + configs at run start (no edit conflicts)
- manifest.json with full metadata + per-variant status
- Quality gate post-render (reject empty/flat sprites)
- Resume support: --resume <run_id> skips ok variants
- Dependency pre-check: Blender version, plugin files, models
- Latest symlink: output/<char>/latest -> last successful run
- Per-variant palette enforcement via --palette config

Usage:
    pipeline_run.py --char agumon
    pipeline_run.py --char agumon-rearise --parallel 4
    pipeline_run.py --char agumon --resume 2026-04-29_103045_a3b1f2
    pipeline_run.py --char agumon --palette agumon.gpl --parallel 3

Naming convention: <digimon>[-<source>][-<variant>]
  Examples: agumon, agumon-rearise, agumon-newcentury, agumon-black,
            agumon-rearise-snow, gabumon, patamon-tk

Required files for a char <name>:
  configs/<name>.json              — render config
  palettes/<name>.gpl              — auto-detected (or palettes/<digimon>.gpl fallback)
  standards/<digimon>.md           — char-specific scoring spec
  references/<digimon>/canonical.* — char ground-truth image
"""

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from datetime import datetime
from pathlib import Path

# Windows console = cp1252 by default; force utf-8 for arrow/check glyphs in prints.
if sys.stdout.encoding and sys.stdout.encoding.lower() != "utf-8":
    try:
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
        sys.stderr.reconfigure(encoding="utf-8", errors="replace")
    except Exception:
        pass

# Import quality_gate as module
sys.path.insert(0, str(Path(__file__).parent))
from quality_gate import validate as qgate


# --- Defaults ---
# Project decision (2026-04-30): anime-only direction, iso45 only.
# See sprite_pipeline/DECISIONS.md for full rationale.
# Pass --shaders / --cameras to override for experimentation.
DEFAULT_SHADERS = [
    "mihoyo_style",  # Primary: Genshin/HSR-like cel + Fresnel rim + 3-band shading
    "anime_eevee",   # Secondary: Smooth cel + posterize
    # --- DEPRECATED but available via --shaders flag (do not remove files) ---
    # "basic_cel",         # pixel-track winner
]
DEFAULT_CAMERAS = [
    ("iso_45", "iso45"),  # Slay-the-Spire / JRPG 3/4 angle
    # --- DEPRECATED but pickable via --cameras flag ---
    # ("x", "side_right"),         # too flat for card-battler look
    # ("iso_30", "iso30"),         # too top-down: chibi T-rex loses silhouette
    # ("-x", "side_left"),         # mirror of side_right
    # ("-y", "front"),             # narrow silhouette
    # ("y", "back"),               # narrow silhouette
    # ("iso_threequarter", "iso34"),  # similar to iso45
]

# Each variant produces BOTH tracks (anime hi-res + pixel HD-2D)
# Common render size = max needed by either track. Post-process splits.
# See DECISIONS.md (2026-05-12) for the 1024→512 reduction rationale.
RENDER_SIZE = 512

TRACK_CONFIG = {
    "anime": {
        # Use raw render directly (no downscale, no pixelify, no palette).
        # Output sprites at RENDER_SIZE (smooth Cycles surfaces preserved).
        "pixelify": False,
        "palette_enforce": False,
    },
}


# --- Helpers ---

def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(8192), b""):
            h.update(chunk)
    return h.hexdigest()[:16]


def gen_run_id() -> str:
    ts = datetime.now().strftime("%Y-%m-%d_%H%M%S")
    rand = hashlib.sha256(os.urandom(8)).hexdigest()[:6]
    return f"{ts}_{rand}"


def resolve_config_paths(cfg: dict, pipeline_dir: Path) -> dict:
    """Resolve relative model_path/texture_path against pipeline_dir.

    Configs ship with paths like "raw_models/agumon/chr050.fbx" so the repo
    is portable across machines. Absolute paths are preserved untouched for
    one-off overrides. Mutates and returns cfg.
    """
    for key in ("model_path", "texture_path"):
        val = cfg.get(key)
        if not val:
            continue
        p = Path(val)
        if not p.is_absolute():
            cfg[key] = str((pipeline_dir / p).resolve())
    return cfg


def check_blender() -> dict:
    try:
        result = subprocess.run(
            ["blender", "--version"], capture_output=True, text=True, timeout=10
        )
        version_line = result.stdout.strip().splitlines()[0]
        return {"available": True, "version": version_line}
    except Exception as e:
        return {"available": False, "error": str(e)}


def check_dependencies(pipeline_dir: Path, char: str, plugins_required: list = None) -> dict:
    """Run pre-flight check; returns {passed, errors, info}."""
    errors = []
    info = {}

    # Blender
    blender = check_blender()
    info["blender"] = blender
    if not blender["available"]:
        errors.append(f"blender not found: {blender.get('error', '')}")

    # Config
    config_path = pipeline_dir / "configs" / f"{char}.json"
    if not config_path.exists():
        errors.append(f"config missing: {config_path}")
    info["config"] = str(config_path)

    if config_path.exists():
        cfg = json.loads(config_path.read_text())
        resolve_config_paths(cfg, pipeline_dir)
        # Model
        model = Path(cfg.get("model_path", ""))
        if not model.exists():
            errors.append(f"model_path missing: {model}")
        else:
            info["model"] = {"path": str(model), "sha256": sha256_file(model), "size": model.stat().st_size}
        # Texture
        tex = Path(cfg.get("texture_path", ""))
        if not tex.exists():
            errors.append(f"texture_path missing: {tex}")
        else:
            info["texture"] = {"path": str(tex), "sha256": sha256_file(tex)}

    # Plugins
    plugins_dir = pipeline_dir / "plugins"
    plugin_status = {}
    for plugin in (plugins_required or []):
        p_path = plugins_dir / plugin
        plugin_status[plugin] = {"path": str(p_path), "exists": p_path.exists()}
        if not p_path.exists():
            # Plugins are optional warnings not errors
            pass
    info["plugins"] = plugin_status

    # Shaders
    shaders_dir = pipeline_dir / "scripts" / "shaders"
    shaders_found = sorted(p.stem for p in shaders_dir.glob("*.py") if p.stem != "__init__")
    info["shaders_available"] = shaders_found

    # Palettes
    palettes_dir = pipeline_dir / "palettes"
    info["palettes_available"] = sorted(p.name for p in palettes_dir.glob("*.gpl"))

    return {
        "passed": len(errors) == 0,
        "errors": errors,
        "info": info,
    }


def create_workspace(char: str, pipeline_dir: Path, run_id: str) -> Path:
    """Create isolated run dir + snapshot palettes/shaders/configs."""
    run_dir = pipeline_dir / "output" / char / "runs" / run_id
    run_dir.mkdir(parents=True, exist_ok=True)
    (run_dir / "raw_renders").mkdir(exist_ok=True)
    (run_dir / "sprites").mkdir(exist_ok=True)
    (run_dir / "logs").mkdir(exist_ok=True)

    # Snapshot palettes (clone, isolated from mid-run edits)
    snapshot_palettes = run_dir / "palettes"
    if not snapshot_palettes.exists():
        shutil.copytree(pipeline_dir / "palettes", snapshot_palettes)

    # Snapshot shaders
    snapshot_shaders = run_dir / "shaders"
    if not snapshot_shaders.exists():
        shutil.copytree(pipeline_dir / "scripts" / "shaders", snapshot_shaders)

    # Snapshot config
    config_src = pipeline_dir / "configs" / f"{char}.json"
    if config_src.exists():
        shutil.copy(config_src, run_dir / "config.json")

    return run_dir


def load_manifest(run_dir: Path) -> dict:
    p = run_dir / "manifest.json"
    if p.exists():
        return json.loads(p.read_text())
    return {}


def save_manifest(run_dir: Path, manifest: dict):
    p = run_dir / "manifest.json"
    p.write_text(json.dumps(manifest, indent=2))


def update_latest_symlink(char: str, pipeline_dir: Path, run_id: str):
    latest = pipeline_dir / "output" / char / "latest"
    target = Path("runs") / run_id
    abs_target = pipeline_dir / "output" / char / target
    # Cleanup existing (handle Windows junctions which look like dirs but aren't real)
    if latest.is_symlink():
        latest.unlink()
    elif latest.exists():
        if os.name == "nt" and latest.is_dir():
            # Try junction removal first; falls back to rmtree if it's a real dir
            try:
                os.rmdir(latest)  # works for junctions
            except OSError:
                shutil.rmtree(latest)
        elif latest.is_dir():
            shutil.rmtree(latest)
        else:
            latest.unlink()
    try:
        latest.symlink_to(target)
    except (OSError, NotImplementedError):
        # Windows fallback: use directory junction (no admin needed)
        if os.name == "nt":
            subprocess.run(
                ["cmd", "/c", "mklink", "/J", str(latest), str(abs_target)],
                capture_output=True, check=False,
            )
        else:
            raise


def render_variant(variant_id: str,
                   shader: str,
                   cam_axis: str,
                   cam_label: str,
                   base_config: dict,
                   run_dir: Path,
                   pipeline_dir: Path,
                   palette_path: Path = None,
                   cycles_device: str = "AUTO") -> dict:
    """Render single variant ONCE + produce BOTH tracks (anime + pixel)."""
    started = time.time()
    log_path = run_dir / "logs" / f"render_{variant_id}.log"
    raw_dir = run_dir / "raw_renders" / variant_id

    # Build per-variant config — single render at RENDER_SIZE
    cfg = dict(base_config)
    cfg["shader_variant"] = shader
    cfg["cam_axis"] = cam_axis
    cfg["output_root"] = str(raw_dir)
    cfg["shader_opts"] = dict(base_config.get("shader_opts", {}))
    cfg["render_size"] = RENDER_SIZE
    cfg["cycles_device"] = cycles_device

    # Ensure raw_dir exists before Blender writes to it
    raw_dir.mkdir(parents=True, exist_ok=True)

    # Plugin templates (auto-detect)
    plugins_dir = pipeline_dir / "plugins"
    if shader == "astropulse":
        bp_path = plugins_dir / "BlenderToPixels.blend"
        if bp_path.exists():
            cfg["shader_opts"]["astropulse_template"] = str(bp_path)
    if shader == "lospec_toolkit":
        ls_path = plugins_dir / "lospec-blender-toolkit" / "Lospec_Blender_Toolkit.blend"
        if ls_path.exists():
            cfg["shader_opts"]["lospec_template"] = str(ls_path)

    cfg.pop("posterize_levels", None)

    cfg_path = run_dir / "logs" / f"config_{variant_id}.json"
    cfg_path.write_text(json.dumps(cfg, indent=2))

    # Run Blender
    cmd = [
        "blender", "-b", "-P",
        str(pipeline_dir / "scripts" / "blender_render.py"),
        "--", "--config", str(cfg_path),
    ]
    with log_path.open("w") as logf:
        proc = subprocess.run(cmd, stdout=logf, stderr=subprocess.STDOUT)
    if proc.returncode != 0:
        return {
            "id": variant_id,
            "shader": shader,
            "camera": cam_label,
            "status": "render_fail",
            "duration_sec": round(time.time() - started, 2),
            "log": str(log_path.relative_to(run_dir)),
        }

    # Produce BOTH tracks from single render
    for track_name, track_cfg in TRACK_CONFIG.items():
        sprite_dir = run_dir / f"sprites_{track_name}" / variant_id
        sprite_dir.mkdir(parents=True, exist_ok=True)

        for anim_dir in raw_dir.iterdir():
            if not anim_dir.is_dir():
                continue
            anim_name = anim_dir.name
            frames_out = sprite_dir / f"frames_{anim_name}"

            if track_cfg["pixelify"]:
                # PIXEL TRACK: downscale NEAREST + palette enforce
                pixelify_cmd = [
                    sys.executable,
                    str(pipeline_dir / "scripts" / "pixelify.py"),
                    "--input", str(anim_dir),
                    "--output", str(frames_out),
                    "--size", str(track_cfg["pixel_size"]),
                    "--downscale", track_cfg["downscale"],
                ]
                if track_cfg["palette_enforce"] and palette_path:
                    pixelify_cmd += ["--palette", str(palette_path)]
                else:
                    pixelify_cmd += ["--colors", str(track_cfg.get("n_colors", 32))]
                subprocess.run(pixelify_cmd, capture_output=True)
            else:
                # ANIME TRACK: copy raw frames as-is (no pixelify, smooth preserved)
                frames_out.mkdir(parents=True, exist_ok=True)
                for frame_png in sorted(anim_dir.glob("*.png")):
                    shutil.copy(frame_png, frames_out / frame_png.name)

            # Sheet assemble per track
            subprocess.run(
                [
                    sys.executable,
                    str(pipeline_dir / "scripts" / "sheet_assemble.py"),
                    "--input", str(frames_out),
                    "--output", str(sprite_dir / f"{anim_name}.png"),
                    "--atlas-json", str(sprite_dir / f"{anim_name}.json"),
                ],
                capture_output=True,
            )

    # Quality gate validates BOTH track outputs
    track_results = {}
    overall_passed = True
    for track_name in TRACK_CONFIG.keys():
        main_sprite = run_dir / f"sprites_{track_name}" / variant_id / "idle.png"
        if main_sprite.exists():
            passed, reason, metrics = qgate(main_sprite)
            track_results[track_name] = {"passed": passed, "reason": reason, "metrics": metrics}
            if not passed:
                overall_passed = False
        else:
            track_results[track_name] = {"passed": False, "reason": "no_output", "metrics": {}}
            overall_passed = False

    return {
        "id": variant_id,
        "shader": shader,
        "camera": cam_label,
        "cam_axis": cam_axis,
        "render_size": RENDER_SIZE,
        "palette": str(palette_path.name) if palette_path else "auto-32",
        "status": "ok" if overall_passed else "quality_fail",
        "duration_sec": round(time.time() - started, 2),
        "outputs": {
            track_name: f"sprites_{track_name}/{variant_id}/idle.png"
            for track_name in TRACK_CONFIG.keys()
        },
        "quality_gate_per_track": track_results,
        "log": str(log_path.relative_to(run_dir)),
    }


def build_comparison(run_dir: Path):
    """Build separate comparison sheets per track (anime + pixel)."""
    from PIL import Image, ImageDraw, ImageFont

    # Load TrueType font for readable labels (Linux + Windows + macOS).
    font_candidates = [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans-Bold.ttf",
        "C:/Windows/Fonts/arialbd.ttf",
        "C:/Windows/Fonts/segoeuib.ttf",
        "C:/Windows/Fonts/arial.ttf",
        "C:/Windows/Fonts/segoeui.ttf",
        "/Library/Fonts/Arial Bold.ttf",
        "/System/Library/Fonts/SFNS.ttf",
    ]
    font_path = next((p for p in font_candidates if Path(p).exists()), None)

    for track_name in ("anime", "pixel"):
        sprites_dir = run_dir / f"sprites_{track_name}"
        if not sprites_dir.exists():
            continue
        sheets = sorted(sprites_dir.glob("*/idle.png"))
        if not sheets:
            continue

        imgs = [(s.parent.name, Image.open(s).convert("RGBA")) for s in sheets]
        max_w = max(i.width for _, i in imgs)
        max_h = max(i.height for _, i in imgs)

        # Font size proportional to max_h (~10-15% of row height)
        font_size = max(24, max_h // 6)
        if font_path:
            font = ImageFont.truetype(font_path, font_size)
        else:
            font = ImageFont.load_default()

        # Label panel width based on font size
        label_width = max(280, font_size * 14)
        # Each row = label area on left + sprite on right (centered vertically)
        ROW_H = max_h
        total_h = ROW_H * len(imgs)

        canvas = Image.new("RGBA", (max_w + label_width, total_h), (40, 40, 40, 255))
        draw = ImageDraw.Draw(canvas)

        y = 0
        for name, img in imgs:
            label = f"[{track_name}] {name}"
            # Vertically center label in row
            text_y = y + (ROW_H - font_size) // 2
            draw.text((12, text_y), label, fill=(255, 255, 255), font=font)
            canvas.paste(img, (label_width, y), img)
            y += ROW_H

        canvas.save(run_dir / f"comparison_{track_name}.png")


# --- Main ---

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--char", required=True)
    parser.add_argument("--pipeline-dir", default=str(Path(__file__).parent.parent))
    parser.add_argument("--parallel", type=int, default=3)
    parser.add_argument("--palette", default=None,
                        help="Palette name in palettes/ (e.g. 'agumon.gpl'). Default: auto-32 quantize")
    parser.add_argument("--resume", default=None,
                        help="Run ID to resume; skip variants with status='ok'")
    parser.add_argument("--shaders", nargs="*", default=None,
                        help="Shader variants to run (default: all in DEFAULT_SHADERS)")
    parser.add_argument("--cameras", nargs="*", default=None,
                        help="Camera labels (default: all)")
    parser.add_argument("--skip-deps-check", action="store_true")
    parser.add_argument("--cycles-device", choices=["AUTO", "GPU", "CPU"], default="AUTO",
                        help="Force Cycles backend. AUTO=samples-threshold heuristic, "
                             "GPU=force GPU (probe OPTIX/CUDA/HIP/ONEAPI/METAL), CPU=force CPU.")
    args = parser.parse_args()

    pipeline = Path(args.pipeline_dir).resolve()

    print("=" * 60)
    print(f"  PIPELINE RUN  |  char: {args.char}")
    print("=" * 60)

    # Pre-flight dependency check
    if not args.skip_deps_check:
        print("\n→ Pre-flight dependency check...")
        plugins_required = ["BlenderToPixels.blend",
                            "lospec-blender-toolkit/Lospec_Blender_Toolkit.blend"]
        deps = check_dependencies(pipeline, args.char, plugins_required)
        if not deps["passed"]:
            print("✗ Pre-flight FAILED:")
            for e in deps["errors"]:
                print(f"  - {e}")
            sys.exit(2)
        print(f"  ✓ Blender: {deps['info']['blender']['version']}")
        print(f"  ✓ Model: {deps['info']['model']['path']} ({deps['info']['model']['sha256']})")
        print(f"  ✓ Shaders: {len(deps['info']['shaders_available'])} available")
        print(f"  ✓ Palettes: {len(deps['info']['palettes_available'])} available")
        for plugin, st in deps["info"]["plugins"].items():
            mark = "✓" if st["exists"] else "○"
            print(f"  {mark} Plugin: {plugin}")

    # Resume or new run
    if args.resume:
        run_id = args.resume
        run_dir = pipeline / "output" / args.char / "runs" / run_id
        if not run_dir.exists():
            print(f"✗ Resume run_id not found: {run_id}")
            sys.exit(2)
        manifest = load_manifest(run_dir)
        print(f"\n→ Resuming run: {run_id}")
    else:
        run_id = gen_run_id()
        run_dir = create_workspace(args.char, pipeline, run_id)
        manifest = {
            "run_id": run_id,
            "char": args.char,
            "started_at": datetime.now().isoformat(),
            "blender_version": deps["info"]["blender"]["version"] if not args.skip_deps_check else "unknown",
            "git_commit": subprocess.check_output(
                ["git", "-C", str(pipeline.parent.parent), "rev-parse", "--short", "HEAD"],
                text=True
            ).strip() if (pipeline.parent.parent / ".git").exists() else "unknown",
            "model": deps["info"].get("model", {}) if not args.skip_deps_check else {},
            "texture": deps["info"].get("texture", {}) if not args.skip_deps_check else {},
            "palette_used": args.palette or "auto-detect",
            "variants": [],
        }
        print(f"\n→ New run: {run_id}")
        print(f"  workspace: {run_dir}")

    # Load base config + resolve relative paths against the live pipeline dir.
    # (Snapshot config keeps the original relative form for reproducibility.)
    base_config = json.loads((run_dir / "config.json").read_text())
    resolve_config_paths(base_config, pipeline)

    # Build matrix
    shaders = args.shaders or DEFAULT_SHADERS
    cameras_filter = set(args.cameras) if args.cameras else None
    cameras = [(ax, lab) for (ax, lab) in DEFAULT_CAMERAS
               if cameras_filter is None or lab in cameras_filter]

    palette_path = None
    palette_name = args.palette
    if not palette_name:
        # Auto-detect: try palettes/<char>.gpl, then strip -<source>/-<variant> suffix
        candidates = [f"{args.char}.gpl"]
        base_name = args.char.split("-", 1)[0]
        if base_name != args.char:
            candidates.append(f"{base_name}.gpl")
        for cand in candidates:
            cand_path = run_dir / "palettes" / cand
            if cand_path.exists():
                palette_name = cand
                print(f"  Auto-detected palette: {cand}")
                break
    if palette_name:
        palette_path = run_dir / "palettes" / palette_name
        if not palette_path.exists():
            print(f"✗ Palette not found: {palette_path}")
            sys.exit(2)

    # Build job list (skip already-ok variants if resuming)
    done_ids = {v["id"] for v in manifest.get("variants", []) if v.get("status") == "ok"}
    jobs = []
    for shader in shaders:
        for cam_axis, cam_label in cameras:
            variant_id = f"{shader}_{cam_label}"
            if variant_id in done_ids:
                continue
            jobs.append((variant_id, shader, cam_axis, cam_label))

    print(f"  Variants to render: {len(jobs)} (skipped {len(done_ids)} done)")
    print(f"  Parallel jobs: {args.parallel}")
    print(f"  Palette: {palette_name or 'auto-32'}")
    print(f"  Cycles device: {args.cycles_device}\n")
    manifest["cycles_device"] = args.cycles_device

    started = time.time()
    completed = []

    def worker(job):
        vid, shader, cam_axis, cam_label = job
        return render_variant(vid, shader, cam_axis, cam_label, base_config,
                              run_dir, pipeline, palette_path,
                              cycles_device=args.cycles_device)

    if args.parallel > 1:
        with ThreadPoolExecutor(max_workers=args.parallel) as ex:
            futures = {ex.submit(worker, j): j for j in jobs}
            for fut in as_completed(futures):
                result = fut.result()
                completed.append(result)
                mark = "✓" if result["status"] == "ok" else "✗"
                print(f"  {mark} [{result['id']}] {result['status']} "
                      f"({result['duration_sec']}s)")
                # Update manifest incrementally
                manifest["variants"] = [v for v in manifest.get("variants", [])
                                        if v["id"] != result["id"]] + [result]
                save_manifest(run_dir, manifest)
    else:
        for j in jobs:
            result = worker(j)
            completed.append(result)
            mark = "✓" if result["status"] == "ok" else "✗"
            print(f"  {mark} [{result['id']}] {result['status']} ({result['duration_sec']}s)")
            manifest["variants"] = [v for v in manifest.get("variants", [])
                                    if v["id"] != result["id"]] + [result]
            save_manifest(run_dir, manifest)

    elapsed = time.time() - started
    manifest["duration_sec"] = round(elapsed, 2)
    manifest["finished_at"] = datetime.now().isoformat()

    # Stats
    n_ok = sum(1 for v in manifest["variants"] if v["status"] == "ok")
    n_fail = sum(1 for v in manifest["variants"] if v["status"] != "ok")
    manifest["summary"] = {"total": len(manifest["variants"]), "ok": n_ok, "fail": n_fail}
    save_manifest(run_dir, manifest)

    # Comparison sheet
    build_comparison(run_dir)

    # Latest symlink (only if at least one ok)
    if n_ok > 0:
        update_latest_symlink(args.char, pipeline, run_id)

    print(f"\n{'=' * 60}")
    print(f"  DONE  |  {n_ok}/{len(manifest['variants'])} ok  |  {elapsed:.1f}s")
    print(f"  Run: {run_dir}")
    print(f"  Manifest: {run_dir / 'manifest.json'}")
    print(f"  Comparison: {run_dir / 'comparison.png'}")
    print(f"  Latest symlink: {pipeline / 'output' / args.char / 'latest'}")


if __name__ == "__main__":
    main()
