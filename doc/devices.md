# MOROS Devices

Creating the devices in the file system:

```sh
write /dev/
write /dev/ata/
write /dev/ata/0/
write /dev/ata/0/0 -d ata-0-0
write /dev/ata/0/1 -d ata-0-1
write /dev/ata/1/
write /dev/ata/1/0 -d ata-1-0
write /dev/ata/1/1 -d ata-1-1
write /dev/clk/
write /dev/clk/boot -d clk-boot
write /dev/clk/epoch -d clk-epoch
write /dev/clk/rtc -d clk-rtc
write /dev/console -d console
write /dev/net/
write /dev/net/tcp -d tcp
write /dev/net/udp -d udp
write /dev/null -d null
write /dev/random -d random
write /dev/speaker -d speaker
write /dev/vga/
write /dev/vga/buffer -d vga-buffer
write /dev/vga/font -d vga-font
write /dev/vga/mode -d vga-mode
write /dev/vga/palette -d vga-palette
```

## Clock Devices

Reading the number of seconds since boot:

```
> read /dev/clk/boot
89.570360
```

Reading the number of seconds since Unix Epoch:

```
> read /dev/clk/epoch
1730398385.175973
```

Reading the real time clock (RTC):

```
> read /dev/clk/rtc
2024-10-31 18:20:02
```

Changing the system time:

```
> print "2025-01-01 00:00:00" => /dev/clk/rtc
[580.327629] RTC 2025-01-01 00:00:00 +0000
```

## Console Device

Reading `/dev/console` with a 4 bytes buffer will return a character from the
keyboard or the serial interface. Reading with a larger buffer will return a
complete line.

## Network Devices

Opening `/dev/net/tcp` or `/dev/net/udp` with the `OPEN` syscall and the device
flag will return a file handle for a TCP or UDP socket supporting the standard
`READ` and `WRITE` syscalls after establishing a connection using the
`CONNECT`, or `LISTEN` and `ACCEPT` syscalls.

The size of those files give the maximum size of the buffer that can be used
when reading or writing to a socket:

```
> list /dev/net
1446 2024-09-28 09:57:55 tcp
1458 2024-09-28 09:57:55 udp
```

Reading a socket with a 1 byte buffer will return the status of the socket:

```
+-----+--------------+
| Bit | Status       |
+-----+--------------+
|  0  | Is Listening |
|  1  | Is Active    |
|  2  | Is Open      |
|  3  | Can Send     |
|  4  | May Send     |
|  5  | Can Recv     |
|  6  | May Recv     |
|  7  | Reserved     |
+-----+--------------+
```

## Speaker Device

Playing a 440 Hz sound on the PC speaker:

```
> print 440 => /dev/speaker
```

Stopping the sound:

```
> print 0 => /dev/speaker
```

## Null Device

Writing to `/dev/null` will discard any data sent to it:

```
> print hello
hello

> print hello => /dev/null
```

It can be used to suppress errors:

```
> copy none.txt some.txt
Error: Could not read file 'none.txt'

> copy none.txt some.txt [2]=> /dev/null
```

## Random Device

Reading from `/dev/random` will return bytes from a cryptographically secure
random number generator that uses the [HC-128][1] algorithm seeded from the
[RDRAND][2] instruction when available.

[1]: https://en.wikipedia.org/wiki/HC-256
[2]: https://en.wikipedia.org/wiki/RDRAND

## VGA Devices

### VGA Font Device

Changing the VGA font:

```
> copy /ini/fonts/zap-light-8x16.psf /dev/vga/font
```

### VGA Mode Device

Changing the VGA mode:

```
> print 320x200 => /dev/vga/mode
```

The accepted modes are:

- `80x25` for the primary text mode with 16 colors
- `320x200` for the primary graphics mode with 256 colors
- `640x480` for the secondary graphics mode with 16 colors

It is possible to read the current mode from this device file.

### VGA Palette Device

Changing the VGA palette is done by writting a 768 bytes buffer to
`/dev/vga/palette` containing the RGB values of 256 colors.

It is possible to read the current palette from this device file.

### VGA Buffer Device

Changing the VGA framebuffer is done by writting a 64 KB bytes buffer to
`/dev/vga/buffer` containing the index of the color of each pixel on the
screen while in `320x200` mode.
