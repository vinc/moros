#!/bin/sh

set -e

dir=$(dirname "$0")

# Build image if needed
cd "$dir/.." && cargo bootimage --release

# Clean up lock files that Bochs creates
rm -f "$dir/../target/x86_64-moros/release/bootimage-moros.bin.lock"
rm -f "$dir/../disk.img.lock"

# Run Bochs (type "continue" if debuger is active)
cd "$dir" && bochs -qf "bochs.rc"
