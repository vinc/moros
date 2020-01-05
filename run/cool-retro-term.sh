#!/bin/sh
dir=$(dirname "$0")
image="target/x86_64-moros/release/bootimage-moros.bin"
qemu="qemu-system-x86_64 -display curses -cpu max"

# Build image if needed
if [ ! -f "$dir/../$image" ]; then
  cd "$dir/.." && cargo xbuild --release
fi

cool-retro-term --fullscreen --profile "$dir/cool-retro-term.json" \
  --workdir "$dir/.." -e sh -c "$qemu $image"
