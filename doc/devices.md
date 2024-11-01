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

## Clocks

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

## Network

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

## Speaker

Playing a 440 Hz sound on the PC speaker:

```
> print 440 => /dev/speaker
```

Stopping the sound:

```
> print 0 => /dev/speaker
```

## VGA

### Font

Changing the VGA font:

```
> copy /ini/fonts/zap-light-8x16.psf /dev/vga/font
```

### Mode

Changing the VGA mode:

```
> print 320x200 => /dev/vga/mode
```

### Palette

### Buffer
