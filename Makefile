.PHONY: setup image qemu
.EXPORT_ALL_VARIABLES:

setup:
	curl https://sh.rustup.rs -sSf | sh -s -- -y
	rustup install nightly
	rustup default nightly
	cargo install bootimage

output = video, # video, serial
keyboard = qwerty # qwerty, azerty, dvorak
nic = rtl8139 # rtl8139, pcnet

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
		cargo rustc --release --bin {} -- \
			-C relocation-model=static
	basename -s .rs src/bin/*.rs | xargs -I {} \
		cp target/x86_64-moros/release/{} dsk/bin/{}
	#strip dsk/bin/*

bin = target/x86_64-moros/release/bootimage-moros.bin
img = disk.img

$(img):
	qemu-img create $(img) 32M

# Rebuild MOROS if the features list changed
image: $(img)
	touch src/lib.rs
	env | grep MOROS
	cargo bootimage --no-default-features --features $(output) --release --bin moros
	dd conv=notrunc if=$(bin) of=$(img)

opts = -m 32 -nic model=$(nic) -hda $(img) -soundhw pcspk
ifeq ($(kvm),true)
	opts += -cpu host -accel kvm
else
	opts += -cpu max
endif

ifeq ($(output),serial)
	opts += -display none -chardev stdio,id=s0,signal=off -serial chardev:s0
endif

qemu:
	qemu-system-x86_64 $(opts)

test:
	cargo test --release --lib --no-default-features --features serial -- \
		-m 32 -display none -serial stdio -device isa-debug-exit,iobase=0xf4,iosize=0x04

clean:
	cargo clean
