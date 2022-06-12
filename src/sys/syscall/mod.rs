pub mod number;
pub mod service;

use crate::sys;
use crate::sys::fs::FileInfo;

use core::arch::asm;

/*
 * Dispatching system calls
 */

pub fn dispatcher(n: usize, arg1: usize, arg2: usize, arg3: usize, arg4: usize) -> usize {
    match n {
        number::EXIT => {
            service::exit(arg1)
        }
        number::SLEEP => {
            service::sleep(f64::from_bits(arg1 as u64));
            0
        }
        number::DELETE => {
            let ptr = sys::process::ptr_from_addr(arg1 as u64);
            let len = arg2;
            let path = unsafe { core::str::from_utf8_unchecked(core::slice::from_raw_parts(ptr, len)) };
            service::delete(path) as usize
        }
        number::INFO => {
            let ptr = sys::process::ptr_from_addr(arg1 as u64);
            let len = arg2;
            let path = unsafe { core::str::from_utf8_unchecked(core::slice::from_raw_parts(ptr, len)) };
            let info = unsafe { &mut *(arg3 as *mut FileInfo) };
            service::info(path, info) as usize
        }
        number::OPEN => {
            let ptr = sys::process::ptr_from_addr(arg1 as u64);
            let len = arg2;
            let flags = arg3;
            let path = unsafe { core::str::from_utf8_unchecked(core::slice::from_raw_parts(ptr, len)) };
            service::open(path, flags) as usize
        }
        number::READ => {
            let handle = arg1;
            let ptr = sys::process::ptr_from_addr(arg2 as u64);
            let len = arg3;
            let buf = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
            service::read(handle, buf) as usize
        }
        number::WRITE => {
            let handle = arg1;
            let ptr = sys::process::ptr_from_addr(arg2 as u64);
            let len = arg3;
            if arg2 == 0 {
                debug!("SYSCALL WRITE (handle={}, ptr={:#x} ({:#x}), len={})", arg1, arg2, ptr as usize, arg3);
                return 0;
            }
            let buf = unsafe { core::slice::from_raw_parts_mut(ptr, len) }; // TODO: Remove mut
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
        number::SPAWN => {
            let path_ptr = sys::process::ptr_from_addr(arg1 as u64);
            let path_len = arg2;
            let path = unsafe { core::str::from_utf8_unchecked(core::slice::from_raw_parts(path_ptr, path_len)) };

            let args_ptr = arg3;
            let args_len = arg4;

            service::spawn(path, args_ptr, args_len) as usize
        }
        number::STOP => {
            service::stop(arg1)
        }
        _ => {
            unimplemented!();
        }
    }
}

/*
 * Sending system calls
 */

#[doc(hidden)]
pub unsafe fn syscall0(n: usize) -> usize {
    let res: usize;
    asm!(
        "int 0x80", in("rax") n,
        lateout("rax") res
    );
    res
}

#[doc(hidden)]
pub unsafe fn syscall1(n: usize, arg1: usize) -> usize {
    let res: usize;
    asm!(
        "int 0x80", in("rax") n,
        in("rdi") arg1,
        lateout("rax") res
    );
    res
}

#[doc(hidden)]
pub unsafe fn syscall2(n: usize, arg1: usize, arg2: usize) -> usize {
    let res: usize;
    asm!(
        "int 0x80", in("rax") n,
        in("rdi") arg1, in("rsi") arg2,
        lateout("rax") res
    );
    res
}

#[doc(hidden)]
pub unsafe fn syscall3(n: usize, arg1: usize, arg2: usize, arg3: usize) -> usize {
    let res: usize;
    asm!(
        "int 0x80", in("rax") n,
        in("rdi") arg1, in("rsi") arg2, in("rdx") arg3,
        lateout("rax") res
    );
    res
}

#[doc(hidden)]
pub unsafe fn syscall4(n: usize, arg1: usize, arg2: usize, arg3: usize, arg4: usize) -> usize {
    let res: usize;
    asm!(
        "int 0x80", in("rax") n,
        in("rdi") arg1, in("rsi") arg2, in("rdx") arg3, in("r8") arg4,
        lateout("rax") res
    );
    res
}

#[macro_export]
macro_rules! syscall {
    ($n:expr) => (
        $crate::sys::syscall::syscall0(
            $n as usize));
    ($n:expr, $a1:expr) => (
        $crate::sys::syscall::syscall1(
            $n as usize, $a1 as usize));
    ($n:expr, $a1:expr, $a2:expr) => (
        $crate::sys::syscall::syscall2(
            $n as usize, $a1 as usize, $a2 as usize));
    ($n:expr, $a1:expr, $a2:expr, $a3:expr) => (
        $crate::sys::syscall::syscall3(
            $n as usize, $a1 as usize, $a2 as usize, $a3 as usize));
    ($n:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr) => (
        $crate::sys::syscall::syscall4(
            $n as usize, $a1 as usize, $a2 as usize, $a3 as usize, $a4 as usize));
}
