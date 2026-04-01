# =============================================================================
# MOROS MAKEFILE USAGE GUIDE
# =============================================================================
#
# 1. BASIC COMMANDS:
#    make setup             - Install Rust tools and bootimage crate.
#    make image             - Compile the kernel and create 'disk.img'.
#    make qemu              - Launch the OS in QEMU with default settings.
#    make clean             - Remove all build artifacts and the disk image.
#
# 2. HARDWARE OVERRIDES (Pass as arguments to 'make qemu'):
#    cpu=...                - Set CPU model (e.g., Skylake-Client, athlon64, pentium4).
#    memory=...             - Set RAM in MB (default is 512).
#    smp=...                - Set number of CPU cores (default is 4).
#    kvm=true               - Enable hardware acceleration (Linux hosts only).
#
# 3. OUTPUT & DEBUGGING:
#    output=serial          - Redirect OS output to the terminal (headless mode).
#    mode=debug             - Build without optimizations and wait for GDB.
#
# 4. USERSPACE DEVELOPMENT:
#    make user-nasm         - Assemble .s files from 'dsk/src/bin/' to '/bin/'.
#    make user-rust         - Compile .rs files from 'src/bin/' to '/bin/'.
#
# EXAMPLE: make qemu cpu=Skylake-Client memory=1024 smp=4 kvm=true
# =============================================================================

.PHONY: setup image qemu clean user-nasm user-rust
.EXPORT_ALL_VARIABLES:

# Default Hardware Settings
cpu     ?= core2duo
memory  ?= 512
smp     ?= 4
kvm     ?= false
output  ?= video
mode    ?= release
nic     ?= rtl8139

# Internal Paths
bin = target/x86_64-moros/$(mode)/bootimage-moros.bin
img = disk.img

# QEMU CPU Configuration
ifeq ($(kvm),true)
	QEMU_CPU = host -accel kvm
else
	QEMU_CPU = $(cpu)
endif

# MOROS Environment
export MOROS_VERSION = $(shell git describe --tags 2>/dev/null || echo "0.0.0")
export MOROS_KEYBOARD = qwerty

setup:
	curl https://rustup.rs -sSf | sh -s -- -y --default-toolchain none
	rustup show
	cargo install bootimage

user-nasm:
	@mkdir -p dsk/bin
	basename -s .s dsk/src/bin/*.s | xargs -I {} \
		nasm dsk/src/bin/{}.s -o dsk/bin/{}.tmp
	basename -s .s dsk/src/bin/*.s | xargs -I {} \
		sh -c "printf '\x7FBIN' | cat - dsk/bin/{}.tmp > dsk/bin/{}"
	rm -f dsk/bin/*.tmp

user-rust:
	@mkdir -p dsk/bin
	basename -s .rs src/bin/*.rs | xargs -I {} \
		cargo rustc --no-default-features --features userspace --release --bin {} \
		-- -C linker-flavor=ld -C link-args="-Ttext=0x800000"
	basename -s .rs src/bin/*.rs | xargs -I {} \
		cp target/x86_64-moros/release/{} dsk/bin/{}

image: $(img)
	touch src/lib.rs
	cargo bootimage --no-default-features --features $(output) --bin moros $(if $(filter release,$(mode)),--release,)
	dd conv=notrunc if=$(bin) of=$(img)

$(img):
	qemu-img create $(img) 32M

qemu:
	qemu-system-x86_64 \
		-name "MOROS v$(MOROS_VERSION)" \
		-cpu $(QEMU_CPU) \
		-m $(memory) \
		-smp $(smp) \
		-drive file=$(img),format=raw \
		-netdev user,id=e0,hostfwd=tcp::8080-:80 -device $(nic),netdev=e0 \
		$(if $(filter serial,$(output)),-display none -serial stdio,) \
		$(if $(filter debug,$(mode)),-s -S,)

clean:
	cargo clean
	rm -f $(img)
	rm -rf dsk/bin/*
