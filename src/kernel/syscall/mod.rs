pub mod number;
pub mod service;

use lazy_static::lazy_static;

/*
 * Dispatching system calls
 */

const SYSCALLS_COUNT: usize = 1;

fn unimplemented(_arg1: usize, _arg2: usize, _arg3: usize) {
    unimplemented!();
}

lazy_static! {
    pub static ref SYSCALLS: [fn(usize, usize, usize); SYSCALLS_COUNT] = {
        let mut table = [unimplemented as fn(usize, usize, usize); SYSCALLS_COUNT];
        table[number::SLEEP] = service::sleep;
        table
    };
}

pub fn dispatcher(n: usize, arg1: usize, arg2: usize, arg3: usize) {
    if n < SYSCALLS_COUNT {
        SYSCALLS[n](arg1, arg2, arg3);
    }
}

/*
 * Sending system calls
 */

#[doc(hidden)]
pub unsafe fn syscall0(n: usize) -> usize {
    let ret: usize;
    asm!(
        "int 0x80", in("rax") n,
        lateout("rax") ret
    );
    ret
}

#[doc(hidden)]
pub unsafe fn syscall1(n: usize, arg1: usize) -> usize {
    let ret: usize;
    asm!(
        "int 0x80", in("rax") n,
        in("rdi") arg1,
        lateout("rax") ret
    );
    ret
}

#[doc(hidden)]
pub unsafe fn syscall2(n: usize, arg1: usize, arg2: usize) -> usize {
    let ret: usize;
    asm!(
        "int 0x80", in("rax") n,
        in("rdi") arg1, in("rsi") arg2,
        lateout("rax") ret
    );
    ret
}

#[doc(hidden)]
pub unsafe fn syscall3(n: usize, arg1: usize, arg2: usize, arg3: usize) -> usize {
    let ret: usize;
    asm!(
        "int 0x80", in("rax") n,
        in("rdi") arg1, in("rsi") arg2, in("rdx") arg3,
        lateout("rax") ret
    );
    ret
}

#[doc(hidden)]
pub unsafe fn syscall4(n: usize, arg1: usize, arg2: usize, arg3: usize, arg4: usize) -> usize {
    let ret: usize;
    asm!(
        "int 0x80", in("rax") n,
        in("rdi") arg1, in("rsi") arg2, in("rdx") arg3, in("rcx") arg4,
        lateout("rax") ret
    );
    ret
}

#[macro_export]
macro_rules! syscall {
    ($n:expr) => (
        $crate::kernel::syscall::syscall0(
            $n as usize));
    ($n:expr, $a1:expr) => (
        $crate::kernel::syscall::syscall1(
            $n as usize, $a1 as usize));
    ($n:expr, $a1:expr, $a2:expr) => (
        $crate::kernel::syscall::syscall2(
            $n as usize, $a1 as usize, $a2 as usize));
    ($n:expr, $a1:expr, $a2:expr, $a3:expr) => (
        $crate::kernel::syscall::syscall3(
            $n as usize, $a1 as usize, $a2 as usize, $a3 as usize));
    ($n:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr) => (
        $crate::kernel::syscall::syscall4(
            $n as usize, $a1 as usize, $a2 as usize, $a3 as usize, $a4 as usize));
}
