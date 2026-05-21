#!/usr/bin/env bash
# Capture a windowed smoke run for M002 / S06.
#
# Tees `cargo run --features windowed --bin bevyrogue` (stdout + stderr) into
# a timestamped log under .gsd/milestones/M002/slices/S06/uat-evidence/.
# The latest log under that directory is the canonical UAT evidence consumed
# by the slice ASSESSMENT.
#
# Per K001, auto-mode must NOT invoke this script — only the human operator
# launches the windowed binary.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EVIDENCE_DIR="${REPO_ROOT}/.gsd/milestones/M002/slices/S06/uat-evidence"
mkdir -p "${EVIDENCE_DIR}"

STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
LOG_FILE="${EVIDENCE_DIR}/windowed-smoke-${STAMP}.log"

cd "${REPO_ROOT}"

echo "[capture-windowed-smoke] logging to ${LOG_FILE}"
cargo run --features windowed --bin bevyrogue 2>&1 | tee "${LOG_FILE}"
EXIT=${PIPESTATUS[0]}
echo "[capture-windowed-smoke] cargo run exit=${EXIT} log=${LOG_FILE}"
exit "${EXIT}"
