# MOROS Manual

## Boot

During boot MOROS will display its version followed by the memory layout,
memory size, processor, devices, network cards, disks, and the real time clock.

    [0.252961] MOROS v0.10.4
    [0.252961] MEM [0x00000000000000-0x00000000000FFF] FrameZero
    [0.252961] MEM [0x00000000001000-0x00000000004FFF] PageTable
    [0.252961] MEM [0x00000000005000-0x00000000016FFF] Bootloader
    [0.252961] MEM [0x00000000017000-0x00000000017FFF] BootInfo
    [0.252961] MEM [0x00000000018000-0x0000000009EFFF] Kernel
    [0.252961] MEM [0x0000000009F000-0x0000000009FFFF] Reserved
    [0.252961] MEM [0x000000000F0000-0x000000000FFFFF] Reserved
    [0.252961] MEM [0x00000000100000-0x00000000104FFF] Kernel
    [0.252961] MEM [0x00000000105000-0x00000000304FFF] KernelStack
    [0.252961] MEM [0x00000000305000-0x000000003FFFFF] Usable
    [0.252961] MEM [0x00000000400000-0x000000005EEFFF] Kernel
    [0.252961] MEM [0x000000005EF000-0x000000005FFFFF] PageTable
    [0.252961] MEM [0x00000000600000-0x00000001FDFFFF] Usable
    [0.252961] MEM [0x00000001FE0000-0x00000001FFFFFF] Reserved
    [0.252961] MEM [0x000000FEFFC000-0x000000FEFFFFFF] Reserved
    [0.252961] MEM [0x000000FFFC0000-0x000000FFFFFFFF] Reserved
    [0.252961] MEM 32720 KB
    [0.305953] CPU GenuineIntel
    [0.305953] CPU Intel(R) Core(TM) i5-8350U CPU @ 1.70GHz
    [0.306953] RNG RDRAND available
    [0.307953] PCI 0000:00:00 [8086:1237]
    [0.308953] PCI 0000:01:00 [8086:7000]
    [0.308953] PCI 0000:01:01 [8086:7010]
    [0.309953] PCI 0000:01:03 [8086:7113]
    [0.310953] PCI 0000:02:00 [1234:1111]
    [0.310953] PCI 0000:03:00 [8086:100E]
    [0.368944] NET DRV E1000
    [0.368944] NET MAC 52-54-00-12-34-56
    [0.374943] ATA 0:0 QEMU HARDDISK QM00001 (32 MB)
    [0.377942] MFS Superblock found in ATA 0:0
    [0.379942] RTC 2024-06-19 11:50:35 +0000

## Installation

The first time MOROS will boot in diskless mode where you can use the builtin
commands to test the system or `install` to setup the
[filesystem](filesystem.md) on a disk:

    Warning: MFS not found, run 'install' to setup the system

    /
    > install
    Welcome to MOROS v0.10.4 installation program!

    Proceed? [y/N] y

    Listing disks ...
    Path            Name (Size)
    /dev/ata/0/0    QEMU HARDDISK QM00001 (32 MB)
    /dev/mem        RAM DISK

    Formatting disk ...
    Enter path of disk to format: /dev/ata/0/0
    Disk successfully formatted
    MFS is now mounted to '/'

    Populating filesystem...
    Created '/bin'
    Created '/dev'
    Created '/ini'
    Created '/lib'
    Created '/net'
    Created '/src'
    Created '/tmp'
    Created '/usr'
    Created '/var'
    Fetched '/bin/clear'
    Fetched '/bin/halt'
    Fetched '/bin/ntp'
    Fetched '/bin/print'
    Fetched '/bin/reboot'
    Fetched '/bin/sleep'
    Created '/dev/ata'
    Created '/dev/ata/0'
    Created '/dev/ata/0/0'
    Created '/dev/ata/0/1'
    Created '/dev/ata/1'
    Created '/dev/ata/1/0'
    Created '/dev/ata/1/1'
    Created '/dev/clk'
    Created '/dev/clk/uptime'
    Created '/dev/clk/realtime'
    Created '/dev/rtc'
    Created '/dev/null'
    Created '/dev/random'
    Created '/dev/console'
    Created '/dev/net'
    Created '/dev/net/tcp'
    Created '/dev/net/udp'
    Fetched '/ini/banner.txt'
    Fetched '/ini/boot.sh'
    Fetched '/ini/lisp.lsp'
    Fetched '/ini/shell.sh'
    Fetched '/ini/version.txt'
    Created '/ini/palettes'
    Fetched '/ini/palettes/gruvbox-dark.sh'
    Fetched '/ini/palettes/gruvbox-light.sh'
    Created '/ini/fonts'
    Fetched '/ini/fonts/zap-light-8x16.psf'
    Fetched '/ini/fonts/zap-vga-8x16.psf'
    Created '/lib/lisp'
    Fetched '/lib/lisp/alias.lsp'
    Fetched '/lib/lisp/core.lsp'
    Fetched '/lib/lisp/file.lsp'
    Fetched '/tmp/alice.txt'
    Fetched '/tmp/machines.txt'
    Created '/tmp/lisp'
    Fetched '/tmp/lisp/colors.lsp'
    Fetched '/tmp/lisp/doc.lsp'
    Fetched '/tmp/lisp/factorial.lsp'
    Fetched '/tmp/lisp/fibonacci.lsp'
    Fetched '/tmp/lisp/geotime.lsp'
    Fetched '/tmp/lisp/pi.lsp'
    Fetched '/tmp/lisp/sum.lsp'
    Created '/tmp/life'
    Fetched '/tmp/life/centinal.cells'
    Fetched '/tmp/life/flower-of-eden.cells'
    Fetched '/tmp/life/garden-of-eden.cells'
    Fetched '/tmp/life/glider-gun.cells'
    Fetched '/tmp/life/pentadecathlon.cells'
    Fetched '/tmp/life/queen-bee-shuttle.cells'
    Fetched '/tmp/life/ship-in-a-bottle.cells'
    Fetched '/tmp/life/thunderbird.cells'
    Fetched '/tmp/life/wing.cells'
    Created '/tmp/beep'
    Fetched '/tmp/beep/tetris.sh'
    Fetched '/tmp/beep/starwars.sh'
    Fetched '/tmp/beep/mario.sh'
    Created '/var/log'
    Created '/var/www'
    Fetched '/var/www/index.html'
    Fetched '/var/www/moros.css'
    Fetched '/var/www/moros.png'

    Creating user...
    Username: vinc
    Password:
    Confirm:

    Installation successful!

    Quit the console or reboot to apply changes

You can then use `^D` (a key combination of `CTRL` and `D`) to quit the
diskless mode and let MOROS run the bootscript `/ini/boot.sh` to login and use
the shell.

If no disks were detected or if you prefer not to use any you can mount the
system in memory and use a virtual disk with `memory format` before `install`
or using `/dev/mem` for the disk during the setup.

## Shell

The [shell](shell.md) is the primary command line interface to use MOROS.
This is were you can type a command and its arguments to tell the system what
to do:

    ~
    > print "Hello, World!"
    Hello, World!

The system has a `help` command to help you remember the basic commands.

Most commands also have a special `--help` argument to show all their options.

## Directories

The line above the command prompt tells you where you are in the disk. The
tilde `~` means that you are in your home directory:

    ~
    > print $DIR
    /usr/vinc

You can change directory by typing it as if it was a command:

    ~
    > /tmp

    /tmp
    > print $DIR
    /tmp

From now on we'll omit the directory line in most examples.

You can list the content of a directory with `list`:

    > list /tmp
    5090 2023-04-17 06:25:54 alice.txt
      82 2023-04-17 06:25:55 beep
     324 2023-04-17 06:25:55 life
     168 2023-04-17 06:25:55 lisp
     649 2023-04-17 06:25:54 machines.txt

The command has some options to sort the results:

    > list --help
    Usage: list <options> [<dir>]

    Options:
      -b, --binary-size   Use binary size
      -a, --all           Show dot files
      -n, --name          Sort by name
      -s, --size          Sort by size
      -t, --time          Sort by time

You can write a directory in the disk with `write`:

    > write test/

    > list
    5090 2023-04-17 06:25:54 alice.txt
      82 2023-04-17 06:25:55 beep
     324 2023-04-17 06:25:55 life
     168 2023-04-17 06:25:55 lisp
     649 2023-04-17 06:25:54 machines.txt
       0 2023-04-17 07:06:18 test

The slash `/` at the end of `test/` is there to tell the `write` command to
create a directory instead of a file.

## Files

You can create a file by redirecting the output of a command with an arrow `=>`
to the file:

    > print "Hello, World!" => hello.txt

The command `read` will read the content of the file:

    > read hello.txt
    Hello, World!

You can edit a file with the `edit` command that will run the text editor.

Use `^W` (a key combination of `CTRL` and `W`) inside the editor to write the
content to the file and `^Q` to quit the editor and go back to the shell.

The `help` command has a subcommand `help edit` to list the editor commands:

    > help edit
    MOROS text editor is a very simple editor inspired by Pico.

    Commands:
      ^Q    Quit editor
      ^W    Write to file
      ^X    Write to file and quit
      ^T    Go to top of file
      ^B    Go to bottom of file
      ^A    Go to beginning of line
      ^E    Go to end of line
      ^D    Cut line
      ^Y    Copy line
      ^P    Paste line

## Time

You can print the date with `date`:

    > date
    2001-01-01 00:00:00 +0000

You can update the real time clock by writing the correct time to its device
file:

    > print "2023-03-21 10:00:00" => /dev/rtc

    > date
    2023-03-21 10:00:00 +0000

You can also set the `TZ` environment variable to use your preferred timezone:

    > calc "2 * 60 * 60"
    7200

    > env TZ 7200

    > date
    2023-03-21 12:00:00 +0200

Add `env TZ 7200` to `/ini/boot.sh` before `shell` to save the timezone:

    > read /ini/boot.sh
    vga set font /ini/fonts/zap-light-8x16.psf
    shell /ini/palettes/gruvbox-dark.sh
    read /ini/banner.txt
    user login
    env TZ 7200
    shell

There's a device file to get the number of seconds elapsed since Unix Epoch:

    > read /dev/clk/realtime
    1682105344.624905

And another one since boot:

    > read /dev/clk/uptime
    1169.384929

## Aliases

You can add custom commands to the shell with the `alias` command.

For example you can define an `uptime` command that will read the device file
described above:

    > alias uptime "read /dev/clk/uptime"

    > uptime
    1406.304852

You can add that command to `/ini/shell.sh` to save it.

Some shortcuts have been defined in that file for the most frequent commands,
for example you can use `e` instead of `edit` to edit a file.

    > read /ini/shell.sh
    # Command shortcuts
    alias c    copy
    alias d    delete
    alias e    edit
    alias f    find
    alias h    help
    alias l    list
    alias m    move
    alias p    print
    alias q    quit
    alias r    read
    alias w    write

    alias sh   shell
    alias dsk  disk
    alias mem  memory
    alias kbd  keyboard

## Network

You can setup the [network](network.md) manually with `net` or automatically
with `dhcp`:

    > dhcp
    [8.801660] NET IP 10.0.2.15/24
    [8.804659] NET GW 10.0.2.2
    [8.808659] NET DNS 10.0.2.3

A few tools are available like the generalist `socket` command that be used to
send and receive TCP packets:

    > socket 10.0.2.2:1234
    Hello, World!

Or the more specialized `http` command to request a document from a web server:

    > http moros.cc /test.html
    <!doctype html>
    <html>
      <head>
        <meta charset="utf-8">
        <title>MOROS</title>
        <link rel="stylesheet" type="text/css" href="/moros.css">
      </head>
      <body>
        <h1>MOROS</h1>
      </body>
    </html>

There is also a `ntp` script to synchronize the clock over the network:

    > ntp
    2023-03-21 10:00:00

    > ntp => /dev/rtc
    [12.111156] RTC 2023-03-21 10:00:00 +0000
