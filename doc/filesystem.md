# MOROS Filesystem

## Hard drive

A hard drive is separated in block of 512 bytes, grouped into three areas. The
first is reserved for future uses, the second is used as a bitmap mapping the
allocated blocks in the third area. The data stored on the hard drive use the
blocks of the third area.

During the first boot of the OS, the root dir will be allocated, using the
first block of the data area.

A location on the tree of dirs and files is named a path:

  - The root dir is represented by a slash: `/`
  - A dir inside the root will have its name appended to the slash: `/usr`
  - Subsequent dirs will append a slash and their names: `/usr/admin`

### Creation with QEMU

    $ qemu-img create disk.img 128M
    Formatting 'disk.img', fmt=raw size=134217728

### Setup in diskless console

During boot MOROS will detect the hard drives present on the ATA buses, then
the filesystems on those hard drives. If no filesystem is found, MOROS will
open a console in diskless mode to allow the user to create one with the `mkfs`
command:

    > mkfs /dev/ata/0/0

## Data

### BlockBitmap

Bitmap of allocated blocks in the data area.

### Block

A block is small area of 512 bytes on a hard drive, and it is also part of
linked list representing a file or a directory.

The first 4 bytes of a block is the address of the next block on the list and
the rest of block is the data stored in the block.

### DirEntry

A directory entry represent a file or a directory contained inside a directory.
Each entry use a variable number of bytes that must fit inside the data of one
block. Those bytes represent the kind of entry (file or dir), the address of
the first block, the filesize (max 4GB), and the filename (max 255 chars) of
the entry.

Structure:

  - 0..1: kind
  - 1..5: addr
  - 5..10: size
  - 10..11: name len
  - 11..n: name buf

### Dir

A directory contains the address of the first block where its directory entries
are stored.

### File

A file contains the address of its first block along with its filesize and
filename, and a reference to its parent directory.
