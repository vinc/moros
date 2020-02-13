.PHONY: setup image qemu
.EXPORT_ALL_VARIABLES:

setup:
	curl https://sh.rustup.rs -sSf | sh
	rustup install nightly
	rustup default nightly
	rustup component add rust-src
	rustup component add llvm-tools-preview
	cargo install cargo-xbuild bootimage

output = vga
keyboard = qwerty

bin=target/x86_64-moros/release/bootimage-moros.bin
img=disk.img

# Rebuild MOROS if the features list changed
image:
	touch src/lib.rs
	cargo bootimage --no-default-features --features $(output),$(keyboard) --release
	qemu-img convert -f raw $(bin) $(img)
	qemu-img resize -f raw $(img) 32M

opts = -cpu max -nic model=rtl8139 -hda $(img)
ifeq ($(output),serial)
	opts += -display none -serial stdio
endif

qemu:
	qemu-system-x86_64 $(opts)
