#!/usr/bin/env bash
set -euo pipefail

cargo test
cargo check
cargo check --features windowed

if rg -n "apply_effects\\(|\\beffects:" src tests assets/data/skills.ron; then
  echo "legacy apply_effects/effects: field references still present" >&2
  exit 1
fi
