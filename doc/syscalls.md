# MOROS Syscalls

This list is unstable and subject to change between versions of MOROS.

## EXIT (0x01)

```rust
fn exit(code: usize) -> usize
```

## SPAWN (0x02)

```rust
fn spawn(path: &str) -> isize
```

## READ (0x03)

```rust
fn read(handle: usize, buf: &mut [u8]) -> isize
```

## WRITE (0x04)

```rust
fn write(handle: usize, buf: &mut [u8]) -> isize
```

## OPEN (0x05)

```rust
fn open(path: &str, flags: usize) -> isize
```

## CLOSE (0x06)

```rust
fn close(handle: usize)
```

## INFO (0x07)

```rust
fn info(path: &str, info: &mut FileInfo) -> isize
```

This syscall will set the following attributes of the given structure:

```rust
pub struct FileInfo {
    kind: FileType,
    size: u32,
    time: u64,
    name: String,
}
```

## DUP (0x08)

```rust
fn dup(old_handle: usize, new_handle: usize) -> isize
```

## DELETE (0x09)

```rust
fn delete(path: &str) -> isize
```

## STOP (0x0A)

```rust
fn stop(code: usize)
```

The system will reboot with `0xCAFE` and halt with `0xDEAD`.

## SLEEP (0x0B)

```rust
fn sleep(seconds: f64)
```

## POLL (0x0C)

```rust
fn poll(list: &[(usize, IO)]) -> isize
```

## CONNECT (0x0D)

```rust
fn connect(handle, usize, addr: &str, port: u16) -> isize
```

## LISTEN (0x0E)

```rust
fn listen(handle, usize, port: u16) -> isize
```

## ACCEPT (0x0F)

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

This syscall will return a integer corresponding to the `FileType` of the given
file handle when successful:

```rust
pub enum FileType {
    Dir = 0,
    File = 1,
    Device = 2,
}
```
