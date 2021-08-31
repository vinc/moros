pub mod number;
pub mod service;

use alloc::string::String;

/*
 * Dispatching system calls
 */

pub fn dispatcher(n: usize, arg1: usize, arg2: usize, arg3: usize) -> usize {
    match n {
        number::SLEEP => {
            service::sleep(f64::from_bits(arg1 as u64));
            0
        }
        number::UPTIME => {
            service::uptime().to_bits() as usize
        }
        number::REALTIME => {
            service::realtime().to_bits() as usize
        }
        number::OPEN => {
            let ptr = arg1 as *mut u8;
            let len = arg2;
            let mode = arg3 as u8;
            let path = unsafe { String::from_raw_parts(ptr, len, len) };
            service::open(&path, mode) as usize
        }
        number::READ => {
            let fh = arg1 as u16;
            let ptr = arg2 as *mut u8;
            let len = arg3;
            let mut buf = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
            service::read(fh, &mut buf)
        }
        number::CLOSE => {
            let fh = arg1 as u16;
            service::close(fh);
            0
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
}
