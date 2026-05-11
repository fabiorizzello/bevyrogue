#!/usr/bin/env bash
# Wrapper for LibreSprite CLI from extracted AppImage.
#
# Usage:
#   libresprite_wrap.sh -b input.png --palette pal.gpl --color-mode indexed --save-as out.png
#
# Requires: AppImage extracted to plugins/squashfs-root/ via --appimage-extract.

PLUGINS="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/plugins"
LSPRITE_BIN="$PLUGINS/squashfs-root/usr/bin/libresprite"
LSPRITE_LIB="$PLUGINS/squashfs-root/usr/lib"

if [ ! -x "$LSPRITE_BIN" ]; then
    echo "[ERROR] LibreSprite not extracted. Run:" >&2
    echo "  cd $PLUGINS && ./libresprite.AppImage --appimage-extract" >&2
    exit 1
fi

LD_LIBRARY_PATH="$LSPRITE_LIB" "$LSPRITE_BIN" "$@"
