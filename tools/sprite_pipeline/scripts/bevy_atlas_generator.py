#!/usr/bin/env python3
"""
Generates a master Bevy-compatible spritesheet for a character.
Combines all animations (idle, attack, etc.) into a single atlas.
"""
import json
import argparse
from pathlib import Path
from PIL import Image

def generate_bevy_atlas(char_name, output_dir):
    pipeline_dir = Path(__file__).parent.parent
    char_output = pipeline_dir / "output" / char_name / "latest" / "sprites_anime" / "mihoyo_style_iso45"
    
    if not char_output.exists():
        print(f"[ERROR] Output for {char_name} not found at {char_output}")
        return

    # Find all animation PNGs
    anim_pngs = sorted(list(char_output.glob("*.png")))
    # Exclude comparison images if any
    anim_pngs = [p for p in anim_pngs if p.stem not in ("comparison", "sheet")]
    
    if not anim_pngs:
        print(f"[ERROR] No animation PNGs found for {char_name}")
        return

    print(f"[INFO] Found {len(anim_pngs)} animations for {char_name}")

    # Load all animations and their frame metadata
    animations = {}
    total_frames = 0
    max_frame_w = 0
    max_frame_h = 0
    
    for anim_png in anim_pngs:
        anim_name = anim_png.stem
        json_path = anim_png.with_suffix(".json")
        if not json_path.exists():
            continue
            
        with open(json_path) as f:
            meta = json.load(f)
            
        img = Image.open(anim_png)
        frames = meta.get("frames", [])
        animations[anim_name] = {
            "img": img,
            "frames": frames,
            "count": len(frames)
        }
        total_frames += len(frames)
        
        # All frames are expected to be same size in our pipeline (e.g. 1024x1024)
        if frames:
            f0 = frames[0]
            max_frame_w = max(max_frame_w, f0["w"])
            max_frame_h = max(max_frame_h, f0["h"])

    if not animations:
        print(f"[ERROR] No valid animations with metadata found for {char_name}")
        return

    # Layout strategy: Grid
    # We'll try to keep the atlas somewhat square
    import math
    cols = int(total_frames**0.5) + 1
    rows = math.ceil(total_frames / cols)
    
    atlas_w = cols * max_frame_w
    atlas_h = rows * max_frame_h
    
    master_sheet = Image.new("RGBA", (atlas_w, atlas_h), (0, 0, 0, 0))
    master_meta = {
        "meta": {
            "character": char_name,
            "version": "bevy-v1",
            "frame_size": {"w": max_frame_w, "h": max_frame_h},
            "columns": cols,
            "rows": rows,
            "total_frames": total_frames
        },
        "animations": {}
    }

    curr_idx = 0
    for anim_name, data in animations.items():
        anim_start_idx = curr_idx
        img = data["img"]
        frames = data["frames"]
        
        for f_rect in frames:
            # Extract frame from the animation sheet
            frame_img = img.crop((f_rect["x"], f_rect["y"], f_rect["x"] + f_rect["w"], f_rect["y"] + f_rect["h"]))
            
            # Place in master sheet
            row = curr_idx // cols
            col = curr_idx % cols
            master_sheet.paste(frame_img, (col * max_frame_w, row * max_frame_h))
            curr_idx += 1
            
        master_meta["animations"][anim_name] = {
            "start_index": anim_start_idx,
            "end_index": curr_idx - 1,
            "count": data["count"]
        }

    # Save
    out_path = Path(output_dir) / f"{char_name}_atlas.png"
    json_path = out_path.with_suffix(".json")
    
    out_path.parent.mkdir(parents=True, exist_ok=True)
    master_sheet.save(out_path)
    with open(json_path, "w") as f:
        json.dump(master_meta, f, indent=2)
        
    print(f"[SUCCESS] {char_name} master atlas: {out_path} ({cols}x{rows} grid, {total_frames} frames)")

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--char", help="Character name")
    parser.add_argument("--all", action="store_true", help="All characters")
    parser.add_argument("--output", default="sprite_pipeline/output/bevy_atlases", help="Output dir")
    args = parser.parse_args()

    if args.all:
        pipeline_dir = Path(__file__).parent.parent
        chars = [d.name for d in (pipeline_dir / "output").iterdir() if d.is_dir() and not d.name.startswith("_") and (d / "latest").exists()]
        for char in chars:
            generate_bevy_atlas(char, args.output)
    elif args.char:
        generate_bevy_atlas(args.char, args.output)
    else:
        print("Specify --char or --all")

if __name__ == "__main__":
    main()
