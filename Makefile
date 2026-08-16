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
arch = x86_64# x86_64, i686
cpu = core2duo
smp = 2
memory = 32
nic = rtl8139# rtl8139, pcnet, e1000
snd = sb16# ac97, sb16
audio = sdl# sdl, coreaudio
signal = off# on
kvm = false
pcap = false
trace = false# e1000
monitor = false
bootloader = rust# rust, limine, grub
bootloader-proto = limine# limine, multiboot

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

bin = moros-$(arch).img
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
	cp target/x86_64-moros/$(mode)/bootimage-moros.bin $(bin)
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

ifeq ($(bootloader),rust)
qemu-opts += -drive file=$(img),format=raw
else
qemu-opts += -drive file=$(bin),format=raw
endif

ifeq ($(arch),i686)
qemu = qemu-system-i386
cpu = pentium3
else
qemu = qemu-system-x86_64
endif

# In debug mode, open another terminal with the following command
# and type `continue` to start the boot process:
# > gdb target/x86_64-moros/debug/moros -ex "target remote :1234"

qemu:
	$(qemu) $(qemu-opts)

test:
	cargo test --release --lib --no-default-features --features serial -- \
		-m $(memory) -cpu core2duo -display none -serial stdio \
		-device isa-debug-exit,iobase=0xF4,iosize=0x04

limine-version = 11.3.1
limine-url = https://github.com/Limine-Bootloader/Limine/releases/download/v$(limine-version)/limine-$(limine-version).tar.gz
limine-dir = tmp/limine-$(limine-version)

# Require llvm lld xorriso
limine-setup:
	mkdir -p tmp
	wget -O $(limine-dir).tar.gz $(limine-url)
	tar xf $(limine-dir).tar.gz -C tmp
	cd $(limine-dir) && ./configure --enable-bios --enable-bios-cd && make
	cp $(limine-dir)/bin/limine-bios-cd.bin run/boot/limine/
	cp $(limine-dir)/bin/limine-bios.sys run/boot/limine/

limine-image: RUSTFLAGS = -C link-arg=-Trun/boot/$(bootloader-proto).ld -C link-arg=-z -C link-arg=norelro
limine-image:
	cargo build $(cargo-opts),$(bootloader-proto) --target $(arch)-moros.json
	cp target/$(arch)-moros/$(mode)/moros run/boot/kernel.elf
	sed -i.old "s/default_entry:.*/default_entry: $(bootloader-proto)/" run/boot/limine/limine.conf
	rm run/boot/limine/limine.conf.old
	xorriso -as mkisofs \
		-b limine/limine-bios-cd.bin \
		-no-emul-boot -boot-load-size 4 -boot-info-table \
		-partition_offset 16 \
		--protective-msdos-label \
		run/boot -o $(bin)
	$(limine-dir)/bin/limine bios-install $(bin)

grub-image: RUSTFLAGS = -C link-arg=-Trun/boot/multiboot.ld -C link-arg=-z -C link-arg=norelro
grub-image:
	cargo build $(cargo-opts),multiboot --target i686-moros.json
	cp target/i686-moros/$(mode)/moros run/boot/kernel.elf
	grub-mkrescue -d /usr/lib/grub/i386-pc -o $(bin) /boot=run/boot

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
	rm -f moro-*.img
