#!/bin/sh
SCRIPT_DIR="$(dirname "$0")"
cd "$SCRIPT_DIR/.."
curl -s "https://www.rfc-editor.org/rfc/rfc$1.txt" > "dsk/tmp/rfc/$1.txt"
sh "run/deflate.sh" "dsk/tmp/rfc/$1.txt"
find dsk/tmp/rfc/* | sort -V | sed "s/^dsk//" > "dsk/var/pkg/rfc"
