#!/bin/sh
set -e

kernel="$1"
shift

dir=$(mktemp -d)
trap 'rm -rf "$dir"' EXIT
mkdir -p "$dir/limine"

cp "$kernel" "$dir/kernel.elf"
cp "$LIMINE_DIR/bin/limine-bios-cd.bin" "$dir/limine/"
cp "$LIMINE_DIR/bin/limine-bios.sys" "$dir/limine/"

cat > "$dir/limine/limine.conf" <<CONF
timeout: 0
graphics: no
serial: no
/moros
    protocol: multiboot2
    textmode: yes
    kernel_path: boot():/kernel.elf
CONF

xorriso -as mkisofs \
    -b limine/limine-bios-cd.bin \
    -no-emul-boot -boot-load-size 4 -boot-info-table \
    -partition_offset 16 \
    --protective-msdos-label \
    "$dir" -o "$dir/moros.iso" >/dev/null 2>&1

"$LIMINE_DIR/bin/limine" bios-install "$dir/moros.iso" >/dev/null 2>&1

set +e
qemu-system-i386 -cdrom "$dir/moros.iso" "$@"
code=$?
set -e

[ "$code" -eq 33 ] && exit 0
exit "$code"
