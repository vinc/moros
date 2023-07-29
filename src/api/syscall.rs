use crate::api::fs::IO;
use crate::api::process::ExitCode;
use crate::syscall;
use crate::sys::syscall::number::*;
use crate::sys::fs::FileInfo;

use smoltcp::wire::IpAddress;
use smoltcp::wire::Ipv4Address;

pub fn exit(code: ExitCode) {
    unsafe { syscall!(EXIT, code as usize) };
}

pub fn sleep(seconds: f64) {
    unsafe { syscall!(SLEEP, seconds.to_bits()) };
}

pub fn delete(path: &str) -> Result<(), ()> {
    let path_ptr = path.as_ptr() as usize;
    let path_len = path.len();
    let res = unsafe { syscall!(DELETE, path_ptr, path_len) } as isize;
    if res >= 0 {
        Ok(())
    } else {
        Err(())
    }
}

pub fn info(path: &str) -> Option<FileInfo> {
    let path_ptr = path.as_ptr() as usize;
    let path_len = path.len();
    let mut info = FileInfo::new();
    let stat_ptr = &mut info as *mut FileInfo as usize;
    let res = unsafe { syscall!(INFO, path_ptr, path_len, stat_ptr) } as isize;
    if res >= 0 {
        Some(info)
    } else {
        None
    }
}

pub fn open(path: &str, flags: usize) -> Option<usize> {
    let ptr = path.as_ptr() as usize;
    let len = path.len();
    let res = unsafe { syscall!(OPEN, ptr, len, flags) } as isize;
    if res >= 0 {
        Some(res as usize)
    } else {
        None
    }
}

pub fn dup(old_handle: usize, new_handle: usize) -> Option<usize> {
    let res = unsafe { syscall!(DUP, old_handle, new_handle) } as isize;
    if res >= 0 {
        Some(res as usize)
    } else {
        None
    }
}

pub fn read(handle: usize, buf: &mut [u8]) -> Option<usize> {
    let ptr = buf.as_ptr() as usize;
    let len = buf.len();
    let res = unsafe { syscall!(READ, handle, ptr, len) } as isize;
    if res >= 0 {
        Some(res as usize)
    } else {
        None
    }
}

pub fn write(handle: usize, buf: &[u8]) -> Option<usize> {
    let ptr = buf.as_ptr() as usize;
    let len = buf.len();
    let res = unsafe { syscall!(WRITE, handle, ptr, len) } as isize;
    if res >= 0 {
        Some(res as usize)
    } else {
        None
    }
}

pub fn close(handle: usize) {
    unsafe { syscall!(CLOSE, handle) };
}

pub fn spawn(path: &str, args: &[&str]) -> Result<(), ExitCode> {
    let path_ptr = path.as_ptr() as usize;
    let args_ptr = args.as_ptr() as usize;
    let path_len = path.len();
    let args_len = args.len();
    let res = unsafe { syscall!(SPAWN, path_ptr, path_len, args_ptr, args_len) };
    if res == 0 {
        Ok(())
    } else {
        Err(ExitCode::from(res))
    }
}

pub fn stop(code: usize) {
    unsafe { syscall!(STOP, code) };
}

pub fn reboot() {
    stop(0xcafe);
}

pub fn halt() {
    stop(0xdead);
}

pub fn poll(list: &[(usize, IO)]) -> Option<(usize, IO)> {
    let ptr = list.as_ptr() as usize;
    let len = list.len();
    let idx = unsafe { syscall!(POLL, ptr, len) } as isize;
    if 0 <= idx && idx < len as isize {
        Some(list[idx as usize])
    } else {
        None
    }
}

pub fn connect(handle: usize, addr: IpAddress, port: u16) -> Result<(), ()> {
    let buf = addr.as_bytes();
    let ptr = buf.as_ptr() as usize;
    let len = buf.len();
    let res = unsafe { syscall!(CONNECT, handle, ptr, len, port) } as isize;
    if res >= 0 {
        Ok(())
    } else {
        Err(())
    }
}

pub fn listen(handle: usize, port: u16) -> Result<(), ()> {
    let res = unsafe { syscall!(LISTEN, handle, port) } as isize;
    if res >= 0 {
        Ok(())
    } else {
        Err(())
    }
}

pub fn accept(handle: usize) -> Result<IpAddress, ()> {
    let addr = IpAddress::v4(0, 0, 0, 0);
    let buf = addr.as_bytes();
    let ptr = buf.as_ptr() as usize;
    let len = buf.len();
    let res = unsafe { syscall!(ACCEPT, handle, ptr, len) } as isize;
    if res >= 0 {
        Ok(IpAddress::from(Ipv4Address::from_bytes(buf)))
    } else {
        Err(())
    }
}

#[test_case]
fn test_file() {
    use crate::sys::fs::{mount_mem, format_mem, dismount, OpenFlag};
    use alloc::vec;
    mount_mem();
    format_mem();

    let flags = 0;
    assert_eq!(open("/test", flags), None);

    // Write file
    let flags = OpenFlag::Create as usize;
    assert_eq!(open("/test", flags), Some(4));
    let input = "Hello, world!".as_bytes();
    assert_eq!(write(4, &input), Some(input.len()));

    // Read file
    let flags = 0;
    assert_eq!(open("/test", flags), Some(5));
    let mut output = vec![0; input.len()];
    assert_eq!(read(5, &mut output), Some(input.len()));
    assert_eq!(output, input);

    close(4);
    close(5);

    assert_eq!(open("/test", flags), Some(4));

    close(4);

    //assert!(write(1, b"Hello, World\n").is_some());

    dismount();
}
