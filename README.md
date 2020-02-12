# MOROS: Obscure Rust Operating System

```
.100  110.  .1100.  111110.  .1001.  .01000.
00'1001`11 .11  01. 00  `00 .10  00. 10'  11
01  00  10 10    00 001101' 01    00 `100.
10  01  10 01    11 01`00   01    11   `100.
00  01  01 `00  11' 10 `11. `00  11' 01   00
11  10  10  `1001'  00   01  `0110'  `01101'
```

MOROS is a toy operating system written in Rust for the x86 architecture.

This project started from the [seventh post][1] of the second edition of
[Writing an OS in Rust][2] by Philipp Oppermann and by reading the
[OSDev wiki][3] along with many open source kernels.

## Features

- [x] External bootloader (using [bootloader](https://github.com/rust-osdev/bootloader))
- [x] x86 CPU support (using [x86_64](https://crates.io/crates/x86_64))
- [x] Hardware interrupts (using [pic8259_simple](https://crates.io/crates/pic8259_simple))
- [x] PS/2 Keyboard (using [pc-keyboard](https://crates.io/crates/pc-keyboard))
- [x] VGA Text mode output
- [x] Serial output (using [uart_16550](https://crates.io/crates/uart_16550))
- [x] Paging
- [x] Heap allocation (using [linked_list_allocator](https://crates.io/crates/linked_list_allocator))
- [x] RTC clock
- [x] PCI enumeration
- [x] ATA PIO mode
- [x] Random number generator
- [x] RTL8139 network card
- [x] DHCP/IP/TCP/UDP/DNS/HTTP protocols (using [smoltcp](https://crates.io/crates/smoltcp))
- [x] Basic filesystem
- [x] Basic shell
- [x] Basic text editor
- [x] Basic file and network commands
- [x] A LOT OF SHORTCUTS TO GET EVERYTHING WORKING
- [x] HERE BE DRAGONS
- [ ] Processes
- [ ] Multitasking
- [ ] A real userspace

## Setup

Install tools:

    curl https://sh.rustup.rs -sSf | sh
    rustup install nightly
    rustup default nightly
    rustup component add rust-src
    rustup component add llvm-tools-preview
    cargo install cargo-xbuild bootimage

## Usage

Run QEMU with VGA Text Mode (and default qwerty keyboard):

    cargo xrun --release -- \
      -cpu max \
      -nic model=rtl8139 \
      -hdc disk.img

Run QEMU with a serial console (instead of vga screen):

    cargo xrun --release --no-default-features --features serial,dvorak -- \
      -display none \
      -serial stdio \
      -cpu max \
      -nic model=rtl8139 \
      -hdc disk.img

Run QEMU with bootloader, kernel, and data on the same disk:

    qemu-system-x86_64 \
      -cpu max \
      -nic model=rtl8139 \
      -hda disk.img

Run Bochs instead of QEMU:

    sh run/bochs.sh

Run `cool-retro-term` for a retro console look:

    sh run/cool-retro-term.sh

Run on a native x86 computer:

    sudo dd if=target/x86_64-moros/release/bootimage-moros.bin of=/dev/sdb && sync
    sudo reboot

### Disk image

Create secondary disk for the data:

    qemu-img create disk.img 32M

Or combine bootloader, kernel (with dvorak keyboard), and data on the same disk:

    cargo bootimage --no-default-features --features vga,dvorak --release
    qemu-img convert -f raw target/x86_64-moros/release/bootimage-moros.bin disk.img
    qemu-img resize -f raw disk.img 32M

Then later inside the diskless console of MOROS you will have to create the
filesystem with `mkfs /dev/ata/bus/1/dsk/0` or `mkfs /dev/ata/bus/0/dsk/0`
respectively and reboot MOROS.

**Be careful not to overwrite the disk of your OS when using `dd` inside your OS
or `mkfs` inside MOROS.**

## LICENSE

MOROS is released under MIT.

[1]: https://github.com/phil-opp/blog_os/tree/post-07
[2]: https://os.phil-opp.com
[3]: https://wiki.osdev.org
