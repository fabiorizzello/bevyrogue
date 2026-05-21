#!/usr/bin/env bash
# repomix-review.sh — generate repomix XML pack for R015 architectural review.
# Excludes build/cache/asset directories so the pack focuses on reviewable source.
set -euo pipefail

OUT=".gsd/milestones/M002/slices/S06/repomix-pack.xml"
mkdir -p "$(dirname "$OUT")"

npx --yes repomix@1.14.0 \
  --style xml \
  --output "$OUT" \
  --ignore 'target/**,.gsd/**,.planning/**,.audits/**,assets/**,*.lock'
