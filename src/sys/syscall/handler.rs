use crate::sys::process;
use crate::sys::process::Registers;
use crate::sys::x86::int::InterruptFrame;

use core::arch::naked_asm;

#[cfg(target_arch = "x86")]
#[unsafe(naked)]
pub extern "C" fn handler() -> ! {
    naked_asm!(
        "cld",                     // Clear direction flag
        "push edi",
        "push edx",
        "push ecx",
        "push ebx",
        "push eax",
        "mov eax, esp",           // Register list
        "lea edx, [esp + 5 * 4]", // Interrupt frame (5 registers * 4 bytes)
        "sti",                    // Enable interrupts during syscall
        "push eax",               // Arg #2
        "push edx",               // Arg #1
        "call {}",
        "add esp, 8",             // Caller cleans up convention (cdecl)
        "cli",
        "pop eax",
        "pop ebx",
        "pop ecx",
        "pop edx",
        "pop edi",
        "iretd",
        sym inner
    );
}

#[cfg(target_arch = "x86_64")]
#[unsafe(naked)]
pub extern "C" fn handler() -> ! {
    naked_asm!(
        "cld",                     // Clear direction flag
        "push r11",
        "push r10",
        "push r9",
        "push r8",
        "push rcx",
        "push rdx",
        "push rsi",
        "push rdi",
        "push rax",
        "mov rsi, rsp",           // Arg #2: register list
        "lea rdi, [rsp + 9 * 8]", // Arg #1: interrupt frame (9 registers * 8 bytes)
        "sti",                    // Enable interrupts during syscall
        "call {}",
        "cli",
        "pop rax",
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop rcx",
        "pop r8",
        "pop r9",
        "pop r10",
        "pop r11",
        "iretq",
        sym inner
    );
}

extern "C" fn inner(
    frame: &mut InterruptFrame,
    regs: &mut Registers
) {
    let n    = regs[0];
    let arg1 = regs[1];
    let arg2 = regs[2];
    let arg3 = regs[3];
    let arg4 = regs[4];

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

    regs[0] = res;
}
