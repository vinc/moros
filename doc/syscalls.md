# MOROS Syscalls

This list is unstable and subject to change between versions of MOROS.

## EXIT (0x1)

```rust
pub fn exit(code: usize) -> usize
```

## SPAWN (0x2)

```rust
pub fn spawn(path: &str) -> isize
```

## READ (0x3)

```rust
pub fn read(handle: usize, buf: &mut [u8]) -> isize
```

## WRITE (0x4)

```rust
pub fn write(handle: usize, buf: &mut [u8]) -> isize
```

## OPEN (0x5)

```rust
pub fn open(path: &str, flags: usize) -> isize
```

## CLOSE (0x6)

```rust
pub fn close(handle: usize)
```

## INFO (0x7)

```rust
pub fn info(path: &str, info: &mut FileInfo) -> isize
```

## DUP (0x8)

```rust
pub fn dup(old_handle: usize, new_handle: usize) -> isize
```

## SLEEP (0x9)

```rust
pub fn sleep(seconds: f64)
```

## UPTIME (0xA)

```rust
pub fn uptime() -> f64
```

## REALTIME (0xB)

```rust
pub fn realtime() -> f64
```

## DELETE (0xC)

```rust
pub fn delete(path: &str) -> isize
```

## STOP (0xD)

```rust
pub fn stop(code: usize)
```

The system will reboot with `0xcafe` and halt with `0xdead`.
