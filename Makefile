.PHONY: setup image qemu
.EXPORT_ALL_VARIABLES:

setup:
	curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain none
	rustup show
	cargo install bootimage

# Compilation options
output = video# video, serial
keyboard = qwerty# qwerty, azerty, dvorak
mode = release

# Emulation options
memory = 32
cpu = core2duo
smp = 2
nic = rtl8139# rtl8139, pcnet, e1000
snd = sb16# ac97, sb16
audio = sdl# sdl, coreaudio
signal = off# on
kvm = false
pcap = false
trace = false# e1000
monitor = false

export MOROS_VERSION = $(shell git describe --tags | sed "s/^v//")
export MOROS_KEYBOARD = $(keyboard)

user-nasm:
	basename -s .s dsk/src/bin/*.s | xargs -I {} \
    nasm dsk/src/bin/{}.s -o dsk/bin/{}.tmp
	basename -s .s dsk/src/bin/*.s | xargs -I {} \
		sh -c "printf '\x7FBIN' | cat - dsk/bin/{}.tmp > dsk/bin/{}"
	rm dsk/bin/*.tmp

user-cargo-opts = --release --no-default-features --features userspace

# Userspace programs are linked inside the user memory region, which lives
# in its own L4 page table entry.
ld-opts = -Ttext=8000800000 -Trodata=8000900000 -Tbss=8000950000
linker-opts = -C linker-flavor=ld -C link-args="$(ld-opts)"

user-rust:
	basename -s .rs src/bin/*.rs | xargs -I {} \
		touch dsk/bin/{}
	basename -s .rs src/bin/*.rs | xargs -I {} \
		cargo rustc $(user-cargo-opts) --bin {} \
			-- $(linker-opts)
	basename -s .rs src/bin/*.rs | xargs -I {} \
		cp target/x86_64-moros/release/{} dsk/bin/{}
	basename -s .rs src/bin/*.rs | xargs -I {} \
		strip dsk/bin/{}

bin = target/x86_64-moros/$(mode)/bootimage-moros.bin
img = disk.img

$(img):
	qemu-img create $(img) 32M

cargo-opts = --bin moros
ifeq ($(mode),release)
cargo-opts += --release
endif
cargo-opts += --no-default-features --features $(output)

# Rebuild MOROS if the features list changed
image: $(img)
	touch src/lib.rs
	env | grep MOROS
	cargo bootimage $(cargo-opts)
	dd conv=notrunc if=$(bin) of=$(img)

qemu-opts = -name "MOROS $$MOROS_VERSION" \
			 -m $(memory) -smp $(smp) \
			 -audiodev $(audio),id=a0 -machine pcspk-audiodev=a0 \
			 -audio driver=$(audio),model=$(snd) \
			 -netdev user,id=e0,hostfwd=tcp::8080-:80 -device $(nic),netdev=e0

ifeq ($(kvm),true)
qemu-opts += -cpu host -accel kvm
else
qemu-opts += -cpu $(cpu)
endif

ifeq ($(pcap),true)
qemu-opts += -object filter-dump,id=f1,netdev=e0,file=/tmp/qemu.pcap
endif

ifeq ($(monitor),true)
qemu-opts += -monitor telnet:127.0.0.1:7777,server,nowait
endif

ifeq ($(output),serial)
qemu-opts += -display none
qemu-opts += -chardev stdio,id=s0,signal=$(signal) -serial chardev:s0
endif

ifeq ($(mode),debug)
qemu-opts += -s -S
endif

ifeq ($(trace),e1000)
qemu-opts += -trace 'e1000*'
endif

# In debug mode, open another terminal with the following command
# and type `continue` to start the boot process:
# > gdb target/x86_64-moros/debug/moros -ex "target remote :1234"

qemu:
	qemu-system-x86_64 $(qemu-opts) -hda $(img)

test:
	cargo test --release --lib --no-default-features --features serial -- \
		-m $(memory) -cpu core2duo -display none -serial stdio \
		-device isa-debug-exit,iobase=0xF4,iosize=0x04

# Require llvm lld mtools
limine-setup:
	cd tmp
	wget https://github.com/Limine-Bootloader/Limine/releases/download/v11.3.1/limine-11.3.1.tar.gz
	cd limine-11.3.1
	./configure --enable-bios --enable-bios-cd
	make
	cd ..
	cp limine-11.3.1/bin/limine-bios-cd.bin boot/limine/
	cp limine-11.3.1/bin/limine-bios.sys boot/limine/

limine-proto = limine# limine, multiboot
limine-arch = x86_64# x86_64, i686

limine-image: RUSTFLAGS = -C link-arg=-Ttmp/boot/$(limine-proto).ld -C link-arg=-z -C link-arg=norelro
limine-image:
	cargo build $(cargo-opts),$(limine-proto) --target $(limine-arch)-moros.json
	cp target/$(limine-arch)-moros/release/moros tmp/boot/kernel.elf
	find tmp/boot
	cat tmp/boot/limine/limine.conf
	xorriso -as mkisofs \
		-b limine/limine-bios-cd.bin \
		-no-emul-boot -boot-load-size 4 -boot-info-table \
		--protective-msdos-label \
		tmp/boot -o boot.img

ifeq ($(limine-arch),i686)
qemu = qemu-system-i386
cpu = pentium3
else
qemu = qemu-system-x86_64
endif

limine-qemu:
	$(qemu) -cdrom boot.img $(qemu-opts)

website:
	cd www && sh build.sh

spell:
	cd dsk/lib/spell && sh build.sh

pkg:
	ls -1 dsk/var/pkg | grep -v index.html > dsk/var/pkg/index.html

pkg-kernel:
	cp $(bin) dsk/ini/kernel.img
	sh run/deflate.sh dsk/ini/kernel.img

clean:
	cargo clean
	rm -f www/*.html www/images/*.png
