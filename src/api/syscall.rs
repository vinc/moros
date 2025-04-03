use crate::api::fs::IO;
use crate::api::process::ExitCode;
use crate::sys::fs::{FileInfo, FileType};
use crate::sys::syscall::number::*;
use crate::syscall;

use core::convert::TryInto;
use core::convert::TryFrom;
use core::sync::atomic::{fence, Ordering};
use alloc::string::{String, ToString};
use smoltcp::wire::{IpAddress, Ipv4Address};

const SYS_READ_DIR_ENTRY: usize = 45; // Verifique o número correto da syscall no kernel

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

pub fn kind(handle: usize) -> Option<FileType> {
    let res = unsafe { syscall!(KIND, handle) } as isize;
    if res >= 0 {
        FileType::try_from(res as usize).ok()
    } else {
        None
    }
}

pub fn open(path: &str, flags: u8) -> Option<usize> {
    let ptr = path.as_ptr() as usize;
    let len = path.len();
    let res = unsafe { syscall!(OPEN, ptr, len, flags) } as isize;
    if res >= 0 {
        Some(res as usize)
    } else {
        None
    }
}

pub fn dup(old_handle: usize, new_handle: usize) -> Result<(), ()> {
    let res = unsafe { syscall!(DUP, old_handle, new_handle) } as isize;
    if res >= 0 {
        Ok(())
    } else {
        Err(())
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

pub fn read_dir_entry(handle: usize) -> Option<String> {
    let mut buf = [0u8; 256]; // Buffer para armazenar o nome do arquivo/diretório
    let res = unsafe { syscall!(SYS_READ_DIR_ENTRY, handle, buf.as_mut_ptr() as usize, buf.len()) } as isize;

    if res > 0 {
        // Converte os bytes lidos para uma String
        Some(String::from_utf8_lossy(&buf[..res as usize]).to_string())
    } else {
        None
    }
}

pub fn spawn(path: &str, args: &[&str]) -> ExitCode {
    let path_ptr = path.as_ptr() as usize;
    let args_ptr = args.as_ptr() as usize;
    let path_len = path.len();
    let args_len = args.len();
    let res = unsafe {
        syscall!(SPAWN, path_ptr, path_len, args_ptr, args_len)
    };

    // Without the fence `res` would always be `0` instead of the code passed
    // to the `exit` syscall by the child process.
    fence(Ordering::SeqCst);

    ExitCode::from(res)
}

pub fn stop(code: usize) {
    unsafe { syscall!(STOP, code) };
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
    let IpAddress::Ipv4(addr) = addr; // Only IPv4 is supported
    let tmp = addr.octets();
    let buf = tmp.as_slice();
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
    let addr = Ipv4Address::new(0, 0, 0, 0);
    let tmp = addr.octets();
    let buf = tmp.as_slice();
    let ptr = buf.as_ptr() as usize;
    let len = buf.len();
    let res = unsafe { syscall!(ACCEPT, handle, ptr, len) } as isize;
    if res >= 0 {
        if let Ok(buf) = buf.try_into() {
            return Ok(IpAddress::from(Ipv4Address::from_octets(buf)));
        }
    }
    Err(())
}

pub fn alloc(size: usize, align: usize) -> *mut u8 {
    unsafe { syscall!(ALLOC, size, align) as *mut u8 }
}

pub fn free(ptr: *mut u8, size: usize, align: usize) {
    unsafe {
        syscall!(FREE, ptr, size, align);
    }
}

pub unsafe fn ioctl(fd: i32, request: usize, arg: usize) -> i32 {
    #[cfg(target_arch = "x86_64")]
    {
        let result: i32;
        core::arch::asm!(
            "syscall",
            in("rax") 16,      // Número da syscall ioctl (16 no Linux x86_64)
            in("rdi") fd,      // Descritor de arquivo (0 = stdin)
            in("rsi") request, // Código da requisição (TIOCGWINSZ)
            in("rdx") arg,     // Ponteiro para struct winsize
            lateout("rax") result,
            options(nostack, preserves_flags)
        );
        result
    }
    
    #[cfg(target_arch = "aarch64")]
    {
        let result: i64;
        core::arch::asm!(
            "mov x8, 29",      // Número da syscall ioctl (29 no Linux ARM64)
            "svc 0",
            in("x0") fd as i64,
            in("x1") request as i64,
            in("x2") arg as i64,
            lateout("x0") result,
            options(nostack, preserves_flags)
        );
        result as i32
    }
    
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        -1 // Implementação genérica para erro
    }
}

#[test_case]
fn test_file() {
    use crate::sys::fs::{dismount, format_mem, mount_mem, OpenFlag};
    use alloc::string::ToString;
    use alloc::vec;

    mount_mem();
    format_mem();

    let flags = 0;
    assert_eq!(open("/test", flags), None);

    // Write file
    let flags = OpenFlag::Create as u8;
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
    assert_eq!(info("/test").map(|info| info.kind()), kind(4));
    assert_eq!(info("/test").map(|info| info.name()), Some("test".to_string()));
    assert_eq!(info("/test").map(|info| info.size()), Some(input.len() as u32));

    close(4);

    //assert!(write(1, b"Hello, World\n").is_some());

    dismount();
}
