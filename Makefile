.PHONY: setup image qemu
.EXPORT_ALL_VARIABLES:

setup:
	curl https://sh.rustup.rs -sSf | sh
	rustup install nightly
	rustup default nightly
	rustup component add rust-src
	rustup component add llvm-tools-preview
	cargo install bootimage

output = video
keyboard = qwerty
nic = rtl8139

export MOROS_KEYBOARD = $(keyboard)

bin = target/x86_64-moros/release/bootimage-moros.bin
img = disk.img

$(img):
	qemu-img create $(img) 32M

# Rebuild MOROS if the features list changed
image: $(img)
	touch src/lib.rs
	env | grep MOROS
	cargo bootimage --no-default-features --features $(output),$(nic) --release
	dd conv=notrunc if=$(bin) of=$(img)

opts = -m 32 -cpu max -nic model=$(nic) -hda $(img) -soundhw pcspk
ifeq ($(output),serial)
	opts += -display none -serial stdio
endif

qemu:
	qemu-system-x86_64 $(opts)

test:
	cargo test --lib --no-default-features --features serial,$(nic) -- \
		-m 32 -display none -serial stdio -device isa-debug-exit,iobase=0xf4,iosize=0x04

clean:
	cargo clean
