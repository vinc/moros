#!/bin/sh
set -e

# Usage: sh run/grub-floppy.sh [output]
output=${1:-moros-i686-floppy.img}
kernel=target/i686-moros/release/moros
grub=/usr/lib/grub/i386-pc
tmp=tmp/floppy

if [ ! -f "$kernel" ]; then
  echo "Missing $kernel" >&2
  exit 1
fi

rm -rf $tmp
mkdir -p $tmp

grub-mkimage -O i386-pc -d $grub -p "(fd0)/boot/grub" \
  -o $tmp/core.img biosdisk fat multiboot2 gzio normal configfile

strip -o $tmp/kernel.elf "$kernel"
gzip -9 $tmp/kernel.elf

cat > $tmp/grub.cfg <<EOF
set timeout=3
menuentry "MOROS" {
    multiboot2 /boot/kernel.elf.gz
    boot
}
EOF

size=$(wc -c < $tmp/core.img)
reserved=$(( 1 + (size + 511) / 512 ))

rm -f "$output"
mkfs.fat -C -F 12 -R $reserved -n MOROS "$output" 1440

dd if=$grub/boot.img of="$output" bs=1 count=3 conv=notrunc
dd if=$grub/boot.img of="$output" bs=1 skip=62 seek=62 count=450 conv=notrunc
dd if=$tmp/core.img of="$output" bs=512 seek=1 conv=notrunc

mmd -i "$output" ::/boot ::/boot/grub
mcopy -i "$output" $tmp/grub.cfg ::/boot/grub/
mcopy -i "$output" $tmp/kernel.elf.gz ::/boot/
mdir -i "$output" ::/boot
