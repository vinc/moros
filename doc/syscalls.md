# MOROS Syscalls

This list is unstable and subject to change between versions of MOROS.

## EXIT (0x1)

```rust
fn exit(code: usize) -> usize
```

## SPAWN (0x2)

```rust
fn spawn(path: &str) -> isize
```

## READ (0x3)

```rust
fn read(handle: usize, buf: &mut [u8]) -> isize
```

## WRITE (0x4)

```rust
fn write(handle: usize, buf: &mut [u8]) -> isize
```

## OPEN (0x5)

```rust
fn open(path: &str, flags: usize) -> isize
```

## CLOSE (0x6)

```rust
fn close(handle: usize)
```

## INFO (0x7)

```rust
fn info(path: &str, info: &mut FileInfo) -> isize
```

## DUP (0x8)

```rust
fn dup(old_handle: usize, new_handle: usize) -> isize
```

## DELETE (0x9)

```rust
fn delete(path: &str) -> isize
```

## STOP (0xA)

```rust
fn stop(code: usize)
```

The system will reboot with `0xCAFE` and halt with `0xDEAD`.

## SLEEP (0xB)

```rust
fn sleep(seconds: f64)
```

## POLL (0xC)

```rust
fn poll(list: &[(usize, IO)]) -> isize
```

## CONNECT (0xD)

```rust
fn connect(handle, usize, addr: &str, port: u16) -> isize
```

## LISTEN (0xE)

```rust
fn listen(handle, usize, port: u16) -> isize
```

## ACCEPT (0xF)

```rust
fn accept(handle, usize, addr: &str) -> isize
```

## ALLOC (0x10)

```rust
fn alloc(size: usize, align: usize) -> *mut u8
```

## FREE (0x11)

```rust
fn free(ptr: *mut u8, size: usize, align: usize)
```

## KIND (0x12)

```rust
fn kind(handle: usize) -> isize
```
