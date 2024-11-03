# MOROS Syscalls

This list is unstable and subject to change between versions of MOROS.

Any reference to a slice in the arguments (like `&str` or `&[u8]`) will be
converted into a pointer and a length before the syscall is made.

Any negative number returned indicates that an error has occurred. In the
higher level API, this will be typically converted to a `Result` type.

## EXIT (0x01)

```rust
fn exit(code: usize)
```

Terminate the calling process.

## SPAWN (0x02)

```rust
fn spawn(path: &str, args: &[&str]) -> isize
```

Spawn a process with the given list of arguments.

## READ (0x03)

```rust
fn read(handle: usize, buf: &mut [u8]) -> isize
```

Read from a file handle to a buffer.

Return the number of bytes read.

## WRITE (0x04)

```rust
fn write(handle: usize, buf: &[u8]) -> isize
```

Write from a buffer to a file handle.

Return the number of bytes written.

## OPEN (0x05)

```rust
fn open(path: &str, flags: usize) -> isize
```

Open a file and return a file handle.

The flags can be one or more of the following:

```rust
enum OpenFlag {
    Read     = 1,
    Write    = 2,
    Append   = 4,
    Create   = 8,
    Truncate = 16,
    Dir      = 32,
    Device   = 64,
}
```

The flags `OpenFlag::Create | OpenFlag::Dir` can be used to create a directory.

Reading a directory opened with `OpenFlag::Read | OpenFlag::Dir` will return a
list of `FileInfo`, one for each file in the directory.

## CLOSE (0x06)

```rust
fn close(handle: usize)
```

Close a file handle.

## INFO (0x07)

```rust
fn info(path: &str, info: &mut FileInfo) -> isize
```

Get informations on a file.

This syscall will set the following attributes of the given structure:

```rust
struct FileInfo {
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

Duplicate a file handle.

## DELETE (0x09)

```rust
fn delete(path: &str) -> isize
```

Delete a file.

## STOP (0x0A)

```rust
fn stop(code: usize)
```

The system will reboot with `0xCAFE` and halt with `0xDEAD`.

## SLEEP (0x0B)

```rust
fn sleep(seconds: f64)
```

The system will sleep for the given amount of seconds.

## POLL (0x0C)

```rust
fn poll(list: &[(usize, IO)]) -> isize
```

Given a list of file handles and `IO` operations:

```rust
enum IO {
    Read,
    Write,
}
```

This syscall will return the index of the first file handle in the list that is
ready for the given `IO` operation or a negative number if no operations are
available for any file handles. The syscall is not blocking and will return
immediately.

For example polling the console will show when a line is ready to be read,
or polling a socket will show when it can receive or send data.

## CONNECT (0x0D)

```rust
fn connect(handle: usize, addr: IpAddress, port: u16) -> isize
```

Connect a socket to an endpoint at the given `IpAddress` and port:

```rust
struct Ipv4Address([u8; 4]);

struct Ipv6Address([u8; 16]);

enum IpAddress {
    Ipv4(Ipv4Address),
    Ipv6(Ipv6Address),
}
```

NOTE: Only IPv4 is currently supported.

## LISTEN (0x0E)

```rust
fn listen(handle: usize, port: u16) -> isize
```

Listen for incoming connections on a socket.

## ACCEPT (0x0F)

```rust
fn accept(handle: usize, addr: IpAddress) -> isize
```

Accept incoming connection on a socket.

## ALLOC (0x10)

```rust
fn alloc(size: usize, align: usize) -> *mut u8
```

Allocate memory.

## FREE (0x11)

```rust
fn free(ptr: *mut u8, size: usize, align: usize)
```

Free memory.

## KIND (0x12)

```rust
fn kind(handle: usize) -> isize
```

This syscall will return a integer corresponding to the `FileType` of the given
file handle when successful:

```rust
enum FileType {
    Dir = 0,
    File = 1,
    Device = 2,
}
```
