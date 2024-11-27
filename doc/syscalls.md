# MOROS Syscalls

This list is unstable and subject to change between versions of MOROS.

Each syscall is documented with its high-level Rust API wrapper and details
of the raw interface when needed.

Any reference to a slice in the arguments (like `&str` or `&[u8]`) will need to
be converted into a pointer and a length for the raw syscall.

Any negative number returned by a raw syscall indicates that an error has
occurred. In the high-level API, this will be typically converted to an
`Option` or a `Result` type.

At the lowest level a syscall follows the System V ABI convention with its
number set in the `RAX` register, and its arguments in the `RDI`, `RSI`, `RDX`,
and `R8` registers. The `RAX` register is reused for the return value.

Hello world example in assembly using the `WRITE` and `EXIT` syscalls:

```nasm
[bits 64]

section .data
msg: db "Hello, World!", 10
len: equ $-msg

global _start
section .text
_start:
  mov rax, 4                ; syscall number for WRITE
  mov rdi, 1                ; standard output
  mov rsi, msg              ; addr of string
  mov rdx, len              ; size of string
  int 0x80

  mov rax, 1                ; syscall number for EXIT
  mov rdi, 0                ; no error
  int 0x80
```

## EXIT (0x01)

```rust
fn exit(code: ExitCode)
```

Terminate the calling process.

The code can be one of the following:

```rust
pub enum ExitCode {
    Success        =   0,
    Failure        =   1,
    UsageError     =  64,
    DataError      =  65,
    OpenError      = 128,
    ReadError      = 129,
    ExecError      = 130,
    PageFaultError = 200,
    ShellExit      = 255,
}
```

The `ExitCode` is converted to a `usize` for the raw syscall.

## SPAWN (0x02)

```rust
fn spawn(path: &str, args: &[&str]) -> ExitCode
```

Spawn a process with the given list of arguments.

This syscall will block until the child process is terminated. It will return
the `ExitCode` passed by the child process to the `EXIT` syscall.

## READ (0x03)

```rust
fn read(handle: usize, buf: &mut [u8]) -> Option<usize>
```

Read from a file handle to a buffer.

Return the number of bytes read on success.

## WRITE (0x04)

```rust
fn write(handle: usize, buf: &[u8]) -> Option<usize>
```

Write from a buffer to a file handle.

Return the number of bytes written on success.

## OPEN (0x05)

```rust
fn open(path: &str, flags: u8) -> Option<usize>
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
fn info(path: &str) -> Option<FileInfo>
```

Get information on a file.

A `FileInfo` will be returned when successful:

```rust
struct FileInfo {
    kind: FileType,
    size: u32,
    time: u64,
    name: String,
}
```

The raw syscall takes the pointer and the length of a mutable reference to a
`FileInfo` that will be overwritten on success and returns a `isize` to
indicate the result of the operation.

## DUP (0x08)

```rust
fn dup(old_handle: usize, new_handle: usize) -> Result<(), ()>
```

Duplicate a file handle.

## DELETE (0x09)

```rust
fn delete(path: &str) -> Result<(), ()>
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
fn poll(list: &[(usize, IO)]) -> Option<(usize, IO)>
```

Given a list of file handles and `IO` operations:

```rust
enum IO {
    Read,
    Write,
}
```

The index of the first file handle in the list that is ready for the given `IO`
operation is returned by the raw syscall on success or a negative number if no
operations are available for any file handles. The syscall is not blocking and
will return immediately.

For example polling the console will show when a line is ready to be read,
or polling a socket will show when it can receive or send data.

## CONNECT (0x0D)

```rust
fn connect(handle: usize, addr: IpAddress, port: u16) -> Result<(), ()>
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
fn listen(handle: usize, port: u16) -> Result<(), ()>
```

Listen for incoming connections to a socket.

## ACCEPT (0x0F)

```rust
fn accept(handle: usize) -> Result<IpAddress, ()>
```

Accept an incoming connection to a socket.

The raw syscall takes the pointer and the length of a mutable reference to an
`IpAddress` that will be overwritten on success and returns a `isize`
indicating the result of the operation.

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
fn kind(handle: usize) -> Option<FileType>
```

Return the file type of a file handle.

A `FileType` will be returned when successful:

```rust
enum FileType {
    Dir = 0,
    File = 1,
    Device = 2,
}
```

The raw syscall returns a `isize` that will be converted a `FileType` if the
number is positive.
