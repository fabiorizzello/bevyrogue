#!/usr/bin/env bash
# Render same animation in multiple pixel-art styles for comparison.
#
# Output structure:
#   output/<char>/styles/
#     style_01_hires/         — 512×512 source render
#     style_02_box64_auto32/  — box downscale 64px, auto 32 colors
#     style_03_box48_auto16/  — smaller 48px tighter palette
#     style_04_pico8/         — PICO-8 16-color palette
#     style_05_nes/           — NES palette
#     style_06_dithered/      — Floyd-Steinberg dither
#     style_07_box32_pico8/   — micro 32px PICO-8
#     comparison.png          — all styles in single grid

set -euo pipefail

CHAR="${1:?usage: multi_style.sh <char>}"
PIPELINE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPTS="$PIPELINE_DIR/scripts"
PALETTES="$PIPELINE_DIR/palettes"
RAW="$PIPELINE_DIR/raw_renders/$CHAR"
OUT="$PIPELINE_DIR/output/$CHAR/styles"

mkdir -p "$OUT"

CONFIG="$PIPELINE_DIR/configs/${CHAR}.json"
[ -f "$CONFIG" ] || { echo "[ERROR] $CONFIG missing"; exit 1; }

# Step 1: render hi-res once
echo "=== Render hi-res 512×512 ==="
python3 - <<PYEOF
import json
cfg = json.load(open("$CONFIG"))
cfg["render_size"] = 512
json.dump(cfg, open("/tmp/${CHAR}_hires.json", "w"), indent=2)
PYEOF
blender -b -P "$SCRIPTS/blender_render.py" -- --config /tmp/${CHAR}_hires.json 2>&1 | grep -E "(BODY|OUTLINE|Sheet|ERROR)" | tail -5

# Step 2: render same frames in N styles
ANIM="idle"  # for now hard-code idle, generalize later
HIRES_DIR="$RAW/$ANIM"

declare -a STYLES=(
    "01_box64_32         | 64  | --downscale box     | --colors 32"
    "02_box48_16         | 48  | --downscale box     | --colors 16"
    "03_box64_bevyrogue  | 64  | --downscale box     | --palette $PALETTES/bevyrogue.gpl"
    "04_box64_agumon     | 64  | --downscale box     | --palette $PALETTES/agumon.gpl"
    "05_nearest64_24     | 64  | --downscale nearest | --colors 24"
    "06_box32_agumon     | 32  | --downscale box     | --palette $PALETTES/agumon.gpl"
    "07_box64_dither     | 64  | --downscale box     | --colors 16 --dither fs"
    "08_box64_8          | 64  | --downscale box     | --colors 8"
)

for style in "${STYLES[@]}"; do
    name=$(echo "$style" | cut -d'|' -f1 | xargs)
    size=$(echo "$style" | cut -d'|' -f2 | xargs)
    flag1=$(echo "$style" | cut -d'|' -f3 | xargs)
    flag2=$(echo "$style" | cut -d'|' -f4 | xargs)

    style_dir="$OUT/style_$name/frames"
    sheet="$OUT/style_$name/sheet.png"
    mkdir -p "$style_dir"

    echo ""
    echo "=== Style: $name (size=$size) ==="
    python3 "$SCRIPTS/pixelify.py" \
        --input "$HIRES_DIR" \
        --output "$style_dir" \
        --size "$size" \
        $flag1 $flag2 2>&1 | tail -2

    python3 "$SCRIPTS/sheet_assemble.py" \
        --input "$style_dir" \
        --output "$sheet" \
        --atlas-json "$OUT/style_$name/atlas.json" 2>&1 | tail -1
done

# Build comparison sheet: all styles stacked vertically
echo ""
echo "=== Building comparison.png ==="
python3 <<PYEOF
from PIL import Image, ImageDraw, ImageFont
from pathlib import Path

OUT = Path("$OUT")
sheets = sorted(OUT.glob("style_*/sheet.png"))
print(f"Found {len(sheets)} style sheets")

# Pad each sheet to same width by taking max
imgs = [Image.open(s).convert("RGBA") for s in sheets]
max_w = max(i.width for i in imgs)
max_h = max(i.height for i in imgs)

LABEL_H = 20
total_h = (max_h + LABEL_H) * len(imgs)
canvas = Image.new("RGBA", (max_w, total_h), (40, 40, 40, 255))
draw = ImageDraw.Draw(canvas)

y = 0
for sheet, img in zip(sheets, imgs):
    name = sheet.parent.name.replace("style_", "")
    draw.text((4, y + 2), name, fill=(255, 255, 255))
    y += LABEL_H
    canvas.paste(img, (0, y), img)
    y += max_h

canvas.save(OUT / "comparison.png")
print(f"Saved: {OUT}/comparison.png")
PYEOF

echo ""
echo "=== DONE ==="
echo "Compare styles: $OUT/comparison.png"
ls "$OUT"
