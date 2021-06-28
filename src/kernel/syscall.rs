pub unsafe fn syscall0(n: usize) -> usize {
    let ret: usize;
    asm!(
        "int 0x80", in("rax") n,
        lateout("rax") ret
    );
    ret
}

pub unsafe fn syscall1(n: usize, arg1: usize) -> usize {
    let ret: usize;
    asm!(
        "int 0x80", in("rax") n,
        in("rdi") arg1,
        lateout("rax") ret
    );
    ret
}

pub unsafe fn syscall2(n: usize, arg1: usize, arg2: usize) -> usize {
    let ret: usize;
    asm!(
        "int 0x80", in("rax") n,
        in("rdi") arg1, in("rsi") arg2,
        lateout("rax") ret
    );
    ret
}

pub unsafe fn syscall3(n: usize, arg1: usize, arg2: usize, arg3: usize) -> usize {
    let ret: usize;
    asm!(
        "int 0x80", in("rax") n,
        in("rdi") arg1, in("rsi") arg2, in("rdx") arg3,
        lateout("rax") ret
    );
    ret
}

pub unsafe fn syscall4(n: usize, arg1: usize, arg2: usize, arg3: usize, arg4: usize) -> usize {
    let ret: usize;
    asm!(
        "int 0x80", in("rax") n,
        in("rdi") arg1, in("rsi") arg2, in("rdx") arg3, in("rcx") arg4,
        lateout("rax") ret
    );
    ret
}
