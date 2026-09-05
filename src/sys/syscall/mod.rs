mod handler;
pub mod number;
pub mod service;

pub use handler::handler;

use crate::api::process::ExitCode;
use crate::sys;
use crate::sys::fs::{FileInfo, SeekFrom};

use core::arch::asm;
use core::convert::TryInto;
use smoltcp::wire::IpAddress;
use smoltcp::wire::Ipv4Address;

fn utf8_from_raw_parts(ptr: *mut u8, len: usize) -> &'static str {
    unsafe {
        let slice = core::slice::from_raw_parts(ptr, len);
        core::str::from_utf8_unchecked(slice)
    }
}

pub fn dispatcher(
    n: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize
) -> usize {
    if n < 1 || n > number::count() {
        return -1 as isize as usize;
    }

    sys::process::increment_syscall_count(n);

    match n {
        number::EXIT => service::exit(ExitCode::from(arg1)) as usize,
        number::SLEEP => {
            service::sleep(f64::from_bits(arg1 as u64));
            0
        }
        number::DELETE => {
            let ptr = sys::process::ptr_from_addr(arg1);
            let len = arg2;
            let path = utf8_from_raw_parts(ptr, len);
            service::delete(path) as usize
        }
        number::INFO => {
            let ptr = sys::process::ptr_from_addr(arg1);
            let len = arg2;
            let path = utf8_from_raw_parts(ptr, len);
            let info = unsafe { &mut *(arg3 as *mut FileInfo) };
            service::info(path, info) as usize
        }
        number::KIND => {
            let handle = arg1;
            service::kind(handle) as usize
        }
        number::OPEN => {
            let ptr = sys::process::ptr_from_addr(arg1);
            let len = arg2;
            let path = utf8_from_raw_parts(ptr, len);
            let flags = arg3 as u8;
            service::open(path, flags) as usize
        }
        number::READ => {
            let handle = arg1;
            let ptr = sys::process::ptr_from_addr(arg2);
            let len = arg3;
            let buf = unsafe {
                core::slice::from_raw_parts_mut(ptr, len)
            };
            service::read(handle, buf) as usize
        }
        number::WRITE => {
            let handle = arg1;
            let ptr = sys::process::ptr_from_addr(arg2);
            let len = arg3;
            let buf = unsafe {
                core::slice::from_raw_parts_mut(ptr, len) // TODO: Remove mut
            };
            service::write(handle, buf) as usize
        }
        number::CLOSE => {
            let handle = arg1;
            service::close(handle);
            0
        }
        number::DUP => {
            let old_handle = arg1;
            let new_handle = arg2;
            service::dup(old_handle, new_handle) as usize
        }
        number::SEEK => {
            let handle = arg1;
            let offset = match arg3 {
                0 => SeekFrom::Start(arg2 as u32),
                1 => SeekFrom::Current(arg2 as i32),
                2 => SeekFrom::End(arg2 as i32),
                _ => return -4 as isize as usize,
            };
            service::seek(handle, offset) as usize
        }
        number::SPAWN => {
            let path_ptr = sys::process::ptr_from_addr(arg1);
            let path_len = arg2;
            let path = utf8_from_raw_parts(path_ptr, path_len);
            let args_ptr = arg3;
            let args_len = arg4;
            service::spawn(path, args_ptr, args_len) as usize
        }
        number::STOP => {
            let code = arg1;
            service::stop(code)
        }
        number::POLL => {
            let ptr = sys::process::ptr_from_addr(arg1) as *const _;
            let len = arg2;
            let list = unsafe { core::slice::from_raw_parts(ptr, len) };
            service::poll(list) as usize
        }
        number::CONNECT => {
            let handle = arg1;
            let ptr = sys::process::ptr_from_addr(arg2);
            let len = arg3;
            let buf = unsafe { core::slice::from_raw_parts(ptr, len) };
            if let Ok(buf) = buf.try_into() {
                let addr = IpAddress::from(Ipv4Address::from_octets(buf));
                let port = arg4 as u16;
                service::connect(handle, addr, port) as usize
            } else {
                -1 as isize as usize
            }
        }
        number::LISTEN => {
            let handle = arg1;
            let port = arg2 as u16;
            service::listen(handle, port) as usize
        }
        number::ACCEPT => {
            let handle = arg1;
            let ptr = sys::process::ptr_from_addr(arg2);
            let len = arg3;
            let buf = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
            if let Ok(IpAddress::Ipv4(addr)) = service::accept(handle) {
                buf[0..len].clone_from_slice(&addr.octets());
                0
            } else {
                -1 as isize as usize
            }
        }
        number::ALLOC => {
            let size = arg1;
            let align = arg2;
            service::alloc(size, align) as usize
        }
        number::FREE => {
            let ptr = arg1 as *mut u8;
            let size = arg2;
            let align = arg3;
            unsafe {
                service::free(ptr, size, align);
            }
            0
        }
        _ => {
            unreachable!();
        }
    }
}

macro_rules! syscall_fns {
    ($r0:tt, $r1:tt, $r2:tt, $r3:tt, $r4:tt) => {
        #[doc(hidden)]
        pub unsafe fn syscall0(
            n: usize
        ) -> usize {
            let res: usize;
            asm!(
                "int 0x80", in($r0) n,
                lateout($r0) res
            );
            res
        }

        #[doc(hidden)]
        pub unsafe fn syscall1(
            n: usize, arg1: usize
        ) -> usize {
            let res: usize;
            asm!(
                "int 0x80", in($r0) n,
                in($r1) arg1,
                lateout($r0) res
            );
            res
        }

        #[doc(hidden)]
        pub unsafe fn syscall2(
            n: usize, arg1: usize, arg2: usize
        ) -> usize {
            let res: usize;
            asm!(
                "int 0x80", in($r0) n,
                in($r1) arg1, in($r2) arg2,
                lateout($r0) res
            );
            res
        }

        #[doc(hidden)]
        pub unsafe fn syscall3(
            n: usize, arg1: usize, arg2: usize, arg3: usize
        ) -> usize {
            let res: usize;
            asm!(
                "int 0x80", in($r0) n,
                in($r1) arg1, in($r2) arg2, in($r3) arg3,
                lateout($r0) res
            );
            res
        }

        #[doc(hidden)]
        pub unsafe fn syscall4(
            n: usize, arg1: usize, arg2: usize, arg3: usize, arg4: usize
        ) -> usize {
            let res: usize;
            asm!(
                "int 0x80", in($r0) n,
                in($r1) arg1, in($r2) arg2, in($r3) arg3, in($r4) arg4,
                lateout($r0) res
            );
            res
        }
    };
}

#[cfg(target_arch = "x86")]
syscall_fns!("eax", "ebx", "ecx", "edx", "edi");

#[cfg(target_arch = "x86_64")]
syscall_fns!("rax", "rdi", "rsi", "rdx", "rcx");

#[macro_export]
macro_rules! syscall {
    ($r0:expr) => {
        $crate::sys::syscall::syscall0(
            $r0 as usize
        )
    };
    ($r0:expr, $r1:expr) => {
        $crate::sys::syscall::syscall1(
            $r0 as usize, $r1 as usize
        )
    };
    ($r0:expr, $r1:expr, $r2:expr) => {
        $crate::sys::syscall::syscall2(
            $r0 as usize, $r1 as usize, $r2 as usize
        )
    };
    ($r0:expr, $r1:expr, $r2:expr, $r3:expr) => {
        $crate::sys::syscall::syscall3(
            $r0 as usize, $r1 as usize, $r2 as usize, $r3 as usize
        )
    };
    ($r0:expr, $r1:expr, $r2:expr, $r3:expr, $r4:expr) => {
        $crate::sys::syscall::syscall4(
            $r0 as usize, $r1 as usize, $r2 as usize, $r3 as usize, $r4 as usize
        )
    };
}
