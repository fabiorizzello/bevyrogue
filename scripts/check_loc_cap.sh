#!/usr/bin/env bash
# Architectural guard: no src/**/*.rs file may exceed MAX_LOC physical lines.
# Replaces the former `tests/source_file_loc_limit.rs` integration test so the
# style gate stays decoupled from the test suite (R002/R003 hygiene).
#
# Usage:   ./scripts/check_loc_cap.sh
# Exit 0:  no offenders
# Exit 1:  one or more files over the cap; report printed to stdout

set -euo pipefail

MAX_LOC=500
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="$ROOT/src"

if [[ ! -d "$SRC" ]]; then
    echo "error: $SRC not found" >&2
    exit 2
fi

offenders=()
while IFS= read -r -d '' file; do
    loc=$(wc -l < "$file")
    if (( loc > MAX_LOC )); then
        offenders+=("$loc|$file")
    fi
done < <(find "$SRC" -type f -name '*.rs' -print0)

if (( ${#offenders[@]} == 0 )); then
    exit 0
fi

# Largest first; ties broken by path.
mapfile -t sorted < <(printf '%s\n' "${offenders[@]}" | sort -t'|' -k1,1nr -k2,2)

echo "${#sorted[@]} source file(s) exceed the ${MAX_LOC} LOC cap (split them into scoped submodules):"
for entry in "${sorted[@]}"; do
    loc="${entry%%|*}"
    path="${entry#*|}"
    rel="${path#"$ROOT"/}"
    printf "  %5d LOC  %s  (+%d over)\n" "$loc" "$rel" "$((loc - MAX_LOC))"
done

exit 1
