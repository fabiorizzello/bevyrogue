#!/usr/bin/env bash
# Pipeline orchestrator: 3D model → pixel art spritesheet
#
# Usage:
#   ./pipeline.sh <char_name> [pixel_size] [n_colors]
#
# Example:
#   ./pipeline.sh agumon 64 32

set -euo pipefail

CHAR="${1:?usage: pipeline.sh <char> [size] [colors]}"
PIXEL_SIZE="${2:-64}"
N_COLORS="${3:-32}"

PIPELINE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPTS="$PIPELINE_DIR/scripts"
CONFIGS="$PIPELINE_DIR/configs"
RAW_RENDERS="$PIPELINE_DIR/raw_renders/$CHAR"
OUTPUT="$PIPELINE_DIR/output/$CHAR"

CONFIG="$CONFIGS/$CHAR.json"

if [ ! -f "$CONFIG" ]; then
    echo "[ERROR] Config not found: $CONFIG"
    exit 1
fi

echo "=== [1/3] Blender render → $RAW_RENDERS ==="
blender -b -P "$SCRIPTS/blender_render.py" -- --config "$CONFIG"

echo ""
echo "=== [2/3] Pixelify all animations ==="
mkdir -p "$OUTPUT/frames"
for anim_dir in "$RAW_RENDERS"/*/; do
    anim_name=$(basename "$anim_dir")
    out="$OUTPUT/frames/$anim_name"
    python3 "$SCRIPTS/pixelify.py" \
        --input "$anim_dir" \
        --output "$out" \
        --size "$PIXEL_SIZE" \
        --colors "$N_COLORS"
done

echo ""
echo "=== [3/3] Sheet assembly ==="
mkdir -p "$OUTPUT/sheets"
for anim_dir in "$OUTPUT/frames"/*/; do
    anim_name=$(basename "$anim_dir")
    python3 "$SCRIPTS/sheet_assemble.py" \
        --input "$anim_dir" \
        --output "$OUTPUT/sheets/${anim_name}.png" \
        --atlas-json "$OUTPUT/sheets/${anim_name}.json"
done

echo ""
echo "=== DONE: $OUTPUT/sheets/ ==="
ls -la "$OUTPUT/sheets/"
