#!/bin/sh

set -e

dir=$(dirname "$0")
qemu="qemu-system-x86_64 -display curses -cpu max -nic model=rtl8139 disk.img"
#qemu="qemu-system-x86_64 -display curses -cpu max -hdc disk.img -netdev user,id=u1,hostfwd=tcp::2222-:22 -device rtl8139,netdev=u1 -object filter-dump,id=f1,netdev=u1,file=/tmp/qemu.pcap"

echo "The MOROS theme at '$dir/cool-retro-term.json' have to be manually imported."

# Launch qemu inside cool-retro-term
cool-retro-term --fullscreen --profile "MOROS" --workdir "$dir/.." -e sh -c "$qemu 2>/dev/null"
