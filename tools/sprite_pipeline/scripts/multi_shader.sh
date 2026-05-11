#!/usr/bin/env bash
# Render same animation across multiple shader variants + camera presets.
#
# Output: tools/sprite_pipeline/output/<char>/shaders/<variant_camname>/...
# Comparison: output/<char>/shaders/comparison.png
#
# Usage:
#   ./multi_shader.sh <char>
#   PARALLEL_JOBS=3 ./multi_shader.sh <char>   # 3 concurrent Blender renders
#
# Edit SHADER_VARIANTS and CAM_VARIANTS arrays to control matrix.

set -euo pipefail

CHAR="${1:?usage: multi_shader.sh <char>}"
PARALLEL_JOBS="${PARALLEL_JOBS:-1}"
PIPELINE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPTS="$PIPELINE_DIR/scripts"
PLUGINS="$PIPELINE_DIR/plugins"
CONFIG_BASE="$PIPELINE_DIR/configs/${CHAR}.json"
OUT="$PIPELINE_DIR/output/$CHAR/shaders"

[ -f "$CONFIG_BASE" ] || { echo "[ERROR] $CONFIG_BASE missing"; exit 1; }
mkdir -p "$OUT"

# Shader variants (file in scripts/shaders/<name>.py)
SHADER_VARIANTS=(
    "basic_cel"
    "toon_bsdf"
    "anime_eevee"
    "threelevel"
    "flat_emission"
    "astropulse"
    "mihoyo_style"
    "dithered"
    "freestyle_outline"
    "painterly"
    "lospec_toolkit"
)

# Camera presets (cam_axis values)
CAM_VARIANTS=(
    "x:side_right"
    "-x:side_left"
    "-y:front"
    "y:back"
    "iso_threequarter:iso34"
    "iso_45:iso45"
    "iso_30:iso30"
)

ASTROPULSE_TEMPLATE="$PLUGINS/BlenderToPixels.blend"

# Worker function: render + pixelify single variant.
# Args: $1=shader, $2=cam_axis, $3=cam_label
render_variant() {
    local shader="$1"
    local cam_axis="$2"
    local cam_label="$3"
    local variant_id="${shader}_${cam_label}"
    local TMP_CFG="/tmp/${CHAR}_${variant_id}.json"
    local LOG="/tmp/render_${variant_id}.log"

    # Build config
    python3 - <<PYEOF
import json
cfg = json.load(open("$CONFIG_BASE"))
cfg["shader_variant"] = "$shader"
cfg["cam_axis"] = "$cam_axis"
cfg["output_root"] = "$PIPELINE_DIR/raw_renders/$CHAR/shaders/$variant_id"
cfg["shader_opts"] = cfg.get("shader_opts", {})
if "$shader" == "astropulse":
    cfg["shader_opts"]["astropulse_template"] = "$ASTROPULSE_TEMPLATE"
cfg.pop("posterize_levels", None)
json.dump(cfg, open("$TMP_CFG", "w"), indent=2)
PYEOF

    # Render
    if ! blender -b -P "$SCRIPTS/blender_render.py" -- --config "$TMP_CFG" > "$LOG" 2>&1; then
        echo "  ✗ FAIL [$variant_id] — log: $LOG"
        tail -3 "$LOG" | sed 's/^/      /'
        return 1
    fi

    # Pixelify + sheet assemble
    local SRC="$PIPELINE_DIR/raw_renders/$CHAR/shaders/$variant_id"
    local DST="$OUT/$variant_id"
    mkdir -p "$DST"
    for anim_dir in "$SRC"/*/; do
        [ -d "$anim_dir" ] || continue
        local anim_name
        anim_name=$(basename "$anim_dir")
        python3 "$SCRIPTS/pixelify.py" \
            --input "$anim_dir" \
            --output "$DST/frames_$anim_name" \
            --size 64 --colors 32 --downscale box > /dev/null
        python3 "$SCRIPTS/sheet_assemble.py" \
            --input "$DST/frames_$anim_name" \
            --output "$DST/${anim_name}.png" \
            --atlas-json "$DST/${anim_name}.json" > /dev/null
    done
    echo "  ✓ DONE [$variant_id]"
    return 0
}

export -f render_variant
export CHAR PIPELINE_DIR SCRIPTS PLUGINS CONFIG_BASE OUT ASTROPULSE_TEMPLATE

# Build (shader, cam_axis, cam_label) job tuples
JOB_FILE=$(mktemp)
trap 'rm -f "$JOB_FILE"' EXIT
for shader in "${SHADER_VARIANTS[@]}"; do
    for cam_pair in "${CAM_VARIANTS[@]}"; do
        cam_axis="${cam_pair%%:*}"
        cam_label="${cam_pair##*:}"
        echo "$shader|$cam_axis|$cam_label" >> "$JOB_FILE"
    done
done

TOTAL=$(wc -l < "$JOB_FILE")
echo "==========================================="
echo "Variants: $TOTAL  |  Parallel jobs: $PARALLEL_JOBS"
echo "==========================================="

START=$(date +%s)

if [ "$PARALLEL_JOBS" -le 1 ]; then
    # Sequential
    while IFS='|' read -r shader cam_axis cam_label; do
        echo "[$shader/$cam_label]"
        render_variant "$shader" "$cam_axis" "$cam_label" || true
    done < "$JOB_FILE"
else
    # Parallel via xargs -P
    cat "$JOB_FILE" | xargs -I {} -P "$PARALLEL_JOBS" bash -c '
        IFS="|" read -r shader cam_axis cam_label <<< "{}"
        echo "[$shader/$cam_label] starting..."
        render_variant "$shader" "$cam_axis" "$cam_label"
    '
fi

ELAPSED=$(($(date +%s) - START))
echo ""
echo "Total render time: ${ELAPSED}s"

# Build comparison sheet
echo ""
echo "==========================================="
echo "Building comparison.png"
echo "==========================================="
python3 - <<PYEOF
from PIL import Image, ImageDraw
from pathlib import Path

OUT = Path("$OUT")
sheets = sorted(OUT.glob("*/idle.png"))
print(f"Found {len(sheets)} shader sheets")
if not sheets:
    raise SystemExit("No sheets found")

imgs = [(s.parent.name, Image.open(s).convert("RGBA")) for s in sheets]
max_w = max(i.width for _, i in imgs)
max_h = max(i.height for _, i in imgs)

LABEL_H = 24
total_h = (max_h + LABEL_H) * len(imgs)
canvas = Image.new("RGBA", (max_w + 200, total_h), (40, 40, 40, 255))
draw = ImageDraw.Draw(canvas)

y = 0
for name, img in imgs:
    draw.text((4, y + 6), name, fill=(255, 255, 255))
    y += LABEL_H
    canvas.paste(img, (200, y), img)
    y += max_h

canvas.save(OUT / "comparison.png")
print(f"Saved: {OUT}/comparison.png")
PYEOF

echo ""
echo "DONE"
echo "Compare: $OUT/comparison.png"
ls "$OUT"
