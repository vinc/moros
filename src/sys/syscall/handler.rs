use crate::sys::process;
use crate::sys::process::Registers;
use crate::sys::x86::int::InterruptFrame;

use core::arch::naked_asm;

#[cfg(target_arch = "x86")]
#[unsafe(naked)]
pub extern "C" fn handler() -> ! {
    naked_asm!(
        "cld",                     // Clear direction flag
        "push eax",
        "push ebx",
        "push ecx",
        "push edx",
        "push edi",
        "mov eax, esp",           // Register list
        "lea edx, [esp + 5 * 4]", // Interrupt frame (5 registers * 4 bytes)
        "sti",                    // Enable interrupts during syscall
        "push eax",               // Arg #2
        "push edx",               // Arg #1
        "call {}",
        "add esp, 8",             // Caller cleans up convention (cdecl)
        "cli",
        "pop edi",
        "pop edx",
        "pop ecx",
        "pop ebx",
        "pop eax",
        "iretd",
        sym inner
    );
}

#[cfg(target_arch = "x86_64")]
#[unsafe(naked)]
pub extern "C" fn handler() -> ! {
    naked_asm!(
        "cld",                     // Clear direction flag
        "push rax",
        "push rcx",
        "push rdx",
        "push rsi",
        "push rdi",
        "push r8",
        "push r9",
        "push r10",
        "push r11",
        "mov rsi, rsp",           // Arg #2: register list
        "lea rdi, [rsp + 9 * 8]", // Arg #1: interrupt frame (9 registers * 8 bytes)
        "sti",                    // Enable interrupts during syscall
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
    frame: &mut InterruptFrame,
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
        process::set_interrupt_frame(*frame);
        process::set_registers(*regs);
    }

    let res = super::dispatcher(n, arg1, arg2, arg3, arg4);

    // Restore CPU context before exiting a process
    if n == super::number::EXIT {
        *frame = process::interrupt_frame();
        *regs = process::registers();
    }

    regs.rax = res;
}
