use crate::sys::process;
use crate::sys::process::SyscallRegisters;
use crate::sys::x86::int::InterruptRegisters;

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
        "mov eax, esp",           // SyscallRegisters
        "lea edx, [esp + 5 * 4]", // InterruptRegisters (5 * 4 bytes)
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
        "mov rsi, rsp",           // Arg #2: SyscallRegisters
        "lea rdi, [rsp + 9 * 8]", // Arg #1: InterruptRegisters (9 * 8 bytes)
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
    interrupt_registers: &mut InterruptRegisters,
    syscall_registers: &mut SyscallRegisters
) {
    let n    = syscall_registers[0];
    let arg1 = syscall_registers[1];
    let arg2 = syscall_registers[2];
    let arg3 = syscall_registers[3];
    let arg4 = syscall_registers[4];

    // Backup CPU context before spawning a process
    if n == super::number::SPAWN {
        process::set_interrupt_registers(*interrupt_registers);
        process::set_syscall_registers(*syscall_registers);
    }

    let res = super::dispatcher(n, arg1, arg2, arg3, arg4);

    // Restore CPU context before exiting a process
    if n == super::number::EXIT {
        *interrupt_registers = process::interrupt_registers();
        *syscall_registers = process::syscall_registers();
    }

    syscall_registers[0] = res;
}
