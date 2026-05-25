---
estimated_steps: 1
estimated_files: 1
skills_used: []
---

# T02: Add the K001-bannered M004 windowed capture helper script

Why: when the human (or a later session) does run the windowed build, the run's stdout/stderr — including the windowed.agumon_playback VFX load diagnostics (MEM071) — should be teed into a timestamped, discoverable evidence log, exactly as M002/S06 did. The script must also carry a loud K001 banner so auto-mode never invokes it. Do: Create scripts/capture-windowed-m004-vfx.sh mirroring scripts/capture-windowed-smoke.sh, with these M004 differences: (1) header comment scopes it to M004/S06 and keeps the explicit 'Per K001, auto-mode must NOT invoke this script — only the human operator launches the windowed binary' banner; (2) EVIDENCE_DIR points at .gsd/milestones/M004/slices/S06/uat-evidence (mkdir -p it); (3) prefer the human alias — run `cargo winx` (i.e. `cargo run --features 'dev windowed'`) rather than the raw `--features windowed --bin bevyrogue`, so it matches the runbook's launch command; keep the `2>&1 | tee "$LOG_FILE"` + PIPESTATUS exit propagation. set -uo pipefail, REPO_ROOT derived from BASH_SOURCE. chmod +x the file. Done-when: scripts/capture-windowed-m004-vfx.sh is executable, passes `bash -n`, contains the K001 banner, references the M004/S06 uat-evidence dir, and invokes cargo winx. Do NOT execute the script (K001).

## Inputs

- `scripts/capture-windowed-smoke.sh`
- `.cargo/config.toml`

## Expected Output

- `scripts/capture-windowed-m004-vfx.sh`

## Verification

test -x scripts/capture-windowed-m004-vfx.sh && bash -n scripts/capture-windowed-m004-vfx.sh && grep -qi 'K001' scripts/capture-windowed-m004-vfx.sh && grep -q 'auto-mode must NOT invoke' scripts/capture-windowed-m004-vfx.sh && grep -q 'M004/slices/S06/uat-evidence' scripts/capture-windowed-m004-vfx.sh && grep -q 'winx' scripts/capture-windowed-m004-vfx.sh
