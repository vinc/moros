.PHONY: setup image qemu
.EXPORT_ALL_VARIABLES:

setup:
	curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain none
	rustup show
	cargo install bootimage

# Compilation options
memory = 32
output = video# video, serial
keyboard = qwerty# qwerty, azerty, dvorak
mode = release

# Emulation options
nic = rtl8139# rtl8139, pcnet, e1000
audio = sdl# sdl, coreaudio
signal = off# on
kvm = false
pcap = false
trace = false# e1000
monitor = false

export MOROS_VERSION = $(shell git describe --tags | sed "s/^v//")
export MOROS_MEMORY = $(memory)
export MOROS_KEYBOARD = $(keyboard)

# Build userspace binaries
user-nasm:
	basename -s .s dsk/src/bin/*.s | xargs -I {} \
    nasm dsk/src/bin/{}.s -o dsk/bin/{}.tmp
	basename -s .s dsk/src/bin/*.s | xargs -I {} \
		sh -c "printf '\x7FBIN' | cat - dsk/bin/{}.tmp > dsk/bin/{}"
	rm dsk/bin/*.tmp
user-rust:
	basename -s .rs src/bin/*.rs | xargs -I {} \
		touch dsk/bin/{}
	basename -s .rs src/bin/*.rs | xargs -I {} \
		cargo rustc --no-default-features --features userspace --release --bin {}
	basename -s .rs src/bin/*.rs | xargs -I {} \
		cp target/x86_64-moros/release/{} dsk/bin/{}
	basename -s .rs src/bin/*.rs | xargs -I {} \
		strip dsk/bin/{}

bin = target/x86_64-moros/$(mode)/bootimage-moros.bin
img = disk.img

$(img):
	qemu-img create $(img) 32M


cargo-opts = --no-default-features --features $(output) --bin moros
ifeq ($(mode),release)
	cargo-opts += --release
endif

# Rebuild MOROS if the features list changed
image: $(img)
	touch src/lib.rs
	env | grep MOROS
	cargo bootimage $(cargo-opts)
	dd conv=notrunc if=$(bin) of=$(img)

qemu-opts = -m $(memory) -drive file=$(img),format=raw \
			 -audiodev $(audio),id=a0 -machine pcspk-audiodev=a0 \
			 -netdev user,id=e0,hostfwd=tcp::8080-:80 -device $(nic),netdev=e0
ifeq ($(kvm),true)
	qemu-opts += -cpu host -accel kvm
else
	qemu-opts += -cpu core2duo
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
	qemu-system-x86_64 $(qemu-opts)

test:
	cargo test --release --lib --no-default-features --features serial -- \
		-m $(memory) -display none -serial stdio \
		-device isa-debug-exit,iobase=0xF4,iosize=0x04

website:
	cd www && sh build.sh

pkg:
	ls -1 dsk/var/pkg | grep -v index.html > dsk/var/pkg/index.html

clean:
	cargo clean
	rm -f www/*.html www/images/*.png
