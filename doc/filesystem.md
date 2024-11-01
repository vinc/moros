# MOROS Filesystem


## Hard drive

A hard drive is separated in blocks of 512 bytes, grouped into 4 areas:

    +------------+
    | Boot       | (8192 blocks)
    +------------+
    | Superblock | (2 blocks)
    +------------+
    | Bitmap     | (n blocks)
    +------------+
    | Data       | (n * 512 * 8 blocks)
    +------------+

The first area contains the bootloader and the kernel, the second is a
superblock with a magic string to identify the file system, the third is a
bitmap mapping the allocated data blocks of the last area.

A location on the tree of dirs and files is named a path:

  - The root dir is represented by a slash: `/`
  - A dir inside the root will have its name appended to the slash: `/usr`
  - Subsequent dirs will append a slash and their names: `/usr/admin`


### Creation with QEMU

    $ qemu-img create disk.img 128M
    Formatting 'disk.img', fmt=raw size=134217728


### Setup in diskless console

During boot MOROS will detect any hard drives present on the ATA buses, then
look for a filesystem on those hard drives. If no filesystem is found, MOROS
will open a console in diskless mode to allow the user to create one with
the `disk format` command:

    > disk format /dev/ata/0/0

This command will format the first disk on the first ATA bus by writing a magic
string in a superblock, mounting the filesystem, and allocating the root
directory.

The next step during setup is to create the directory structure:

    > write /bin/           # Binaries
    > write /dev/           # Devices
    > write /ini/           # Initialisation files
    > write /lib/           # Libraries
    > write /net/           # Network
    > write /src/           # Sources
    > write /tmp/           # Temporary files
    > write /usr/           # User directories
    > write /var/           # Variable files

See the [devices](devices.md) documentation to create the device files in the
`/dev` directory.

Then the following should be added to the boot script with the
command `edit /ini/boot.sh` to allow MOROS to finish booting:

    user login
    shell

Finally a user can be created with the following command:

    > user create

All of this can be made more easily by running the `install` command instead.
This installer  will also add additional files contained in the `dsk`
repository of the source code, like a nice login banner :)


## Data Structures


### BlockBitmap

Bitmap of allocated blocks in the data area.


### Block

A block is small area of 512 bytes on a hard drive, and it is also part of
linked list representing a file or a directory.

The first 4 bytes of a block is the address of the next block on the list and
the rest of block is the data stored in the block.

Structure:

     0
     0 1 2 3 4 5 6      n
    +-+-+-+-+-+-+-+ // +-+
    | addr  | data       |
    +-+-+-+-+-+-+-+ // +-+

    n = 512


### Superblock

     0                   1                   2
     0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2    n
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+ // +-+
    | signature     |v|b| count | alloc | reserved       |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+ // +-+

    signature = "MOROS FS"
    v = version number of the FS
    b = size of a block in 2 ^ (9 + b) bytes
    count = number of blocks
    alloc = number of allocated blocks


### File

The first block of a file contains the address of the next block where its
contents is stored and the beginning of its contents in the rest of the block.

If all contents can fit into one block the address of the next block will be
empty.

Structure:

     0
     0 1 2 3 4 5 6 7 8      n
    +-+-+-+-+-+-+-+-+-+ // +-+
    | addr  | contents       |
    +-+-+-+-+-+-+-+-+-+ // +-+

    n = 512


### Dir

The first block of a directory contains the address of the next block where its
directory entries are stored and the first entries in the rest of the block.

If all entries can fit into one block the address of the next block will be
empty.

Structure:

     0                   1
     0 1 2 3 4 5 6 7 8 9 0                            n
    +-+-+-+-+-+-+-+-+-+-+-+ // +-+-+-+-+-+-+-+-+ // +-+
    | addr  | dir entry 1        | dir entry 2        |
    +-+-+-+-+-+-+-+-+-+-+-+ // +-+-+-+-+-+-+-+-+ // +-+

    n = 512


### DirEntry

A directory entry represents a file or a directory contained inside a
directory. Each entry use a variable number of bytes that must fit inside the
data of one block. Those bytes represent the kind of entry (file or dir), the
address of the first block, the filesize (max 4 GB), the last modified time in
seconds since Unix Epoch, the length of the filename, and the filename (max
255 chars) of the entry.

Structure:

     0                   1                   2
     0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4      m
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+ // +-+
    |k| addr  | size  | time          |n| name buffer        |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+ // +-+

    k = kind of entry
    n = length of name buffer
    m = 17 + n


### FileInfo

The `INFO` syscall on a file or directory and the `READ` syscall on a directory
return a subset of a directory entry for userspace programs.

Structure:

     0                   1                   2
     0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0      m
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+ // +-+
    |k| size  | time          |n| name buffer        |
    +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+ // +-+

    k = kind of entry
    n = length of name buffer
    m = 13 + n
