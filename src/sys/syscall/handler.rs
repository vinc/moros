use crate::sys::process;
use crate::sys::process::Registers;

use core::arch::naked_asm;
use x86_64::structures::idt::InterruptStackFrame;

#[unsafe(naked)]
pub extern "C" fn handler() -> ! {
    naked_asm!(
        "cld",            // Clear direction flag
        "push rax",
        "push rcx",
        "push rdx",
        "push rsi",
        "push rdi",
        "push r8",
        "push r9",
        "push r10",
        "push r11",
        "mov rsi, rsp",   // Arg #2: register list
        "mov rdi, rsp",   // Arg #1: interrupt frame
        "add rdi, 9 * 8", // 9 registers * 8 bytes
        "sti",            // Enable interrupts during syscall
        "call {}",
        "cli",
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop rcx",
        "pop rax",
        "iretq",
        sym inner
    );
}

extern "C" fn inner(
    stack_frame: &mut InterruptStackFrame,
    regs: &mut Registers
) {
    let n = regs.rax;

    // The registers order follow the System V ABI convention
    let arg1 = regs.rdi;
    let arg2 = regs.rsi;
    let arg3 = regs.rdx;
    let arg4 = regs.r8;

    // Backup CPU context before spawning a process
    if n == super::number::SPAWN {
        process::set_stack_frame(**stack_frame);
        process::set_registers(*regs);
    }

    let res = super::dispatcher(n, arg1, arg2, arg3, arg4);

    // Restore CPU context before exiting a process
    if n == super::number::EXIT {
        let sf = process::stack_frame();
        unsafe {
            stack_frame.as_mut().write(sf);
        }
        *regs = process::registers();
    }

    regs.rax = res;
}
