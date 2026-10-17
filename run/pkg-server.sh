#!/bin/sh
SCRIPT_DIR="$(dirname "$0")"
cd "$SCRIPT_DIR/.."
python3 -m http.server -d dsk 8181
