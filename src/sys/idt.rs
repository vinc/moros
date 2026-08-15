use crate::{api, hang, sys};
use crate::api::process::ExitCode;
use crate::sys::pic;
use crate::sys::process::Registers;
use crate::sys::x86::{interrupts, Cr2};

use core::arch::{asm, naked_asm};
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::structures::idt::{
    InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode
};
use x86_64::VirtAddr;

fn default_handler() {}

lazy_static! {
    static ref IRQ_HANDLERS: Mutex<[fn(); 16]> = {
        Mutex::new([default_handler; 16])
    };

    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.stack_segment_fault.set_handler_fn(stack_segment_fault_handler);
        idt.segment_not_present.set_handler_fn(segment_not_present_handler);
        unsafe {
            idt.double_fault.
                set_handler_fn(double_fault_handler).
                set_stack_index(sys::gdt::DOUBLE_FAULT_IST);
            idt.page_fault.
                set_handler_fn(page_fault_handler).
                set_stack_index(sys::gdt::PAGE_FAULT_IST);
            idt.general_protection_fault.
                set_handler_fn(general_protection_fault_handler).
                set_stack_index(sys::gdt::GENERAL_PROTECTION_FAULT_IST);

            let addr = VirtAddr::from_ptr(wrapped_syscall_handler as *const ());
            idt[0x80].
                set_handler_addr(addr).
                set_privilege_level(x86_64::PrivilegeLevel::Ring3);
        }
        idt[pic::vector(0)].set_handler_fn(irq0_handler);
        idt[pic::vector(1)].set_handler_fn(irq1_handler);
        idt[pic::vector(2)].set_handler_fn(irq2_handler);
        idt[pic::vector(3)].set_handler_fn(irq3_handler);
        idt[pic::vector(4)].set_handler_fn(irq4_handler);
        idt[pic::vector(5)].set_handler_fn(irq5_handler);
        idt[pic::vector(6)].set_handler_fn(irq6_handler);
        idt[pic::vector(7)].set_handler_fn(irq7_handler);
        idt[pic::vector(8)].set_handler_fn(irq8_handler);
        idt[pic::vector(9)].set_handler_fn(irq9_handler);
        idt[pic::vector(10)].set_handler_fn(irq10_handler);
        idt[pic::vector(11)].set_handler_fn(irq11_handler);
        idt[pic::vector(12)].set_handler_fn(irq12_handler);
        idt[pic::vector(13)].set_handler_fn(irq13_handler);
        idt[pic::vector(14)].set_handler_fn(irq14_handler);
        idt[pic::vector(15)].set_handler_fn(irq15_handler);
        idt
    };
}

macro_rules! irq_handler {
    ($handler:ident, $irq:expr) => {
        pub extern "x86-interrupt" fn $handler(_: InterruptStackFrame) {
            let handlers = IRQ_HANDLERS.lock();
            handlers[$irq]();
            pic::eoi($irq);
        }
    };
}

irq_handler!(irq0_handler, 0);
irq_handler!(irq1_handler, 1);
irq_handler!(irq2_handler, 2);
irq_handler!(irq3_handler, 3);
irq_handler!(irq4_handler, 4);
irq_handler!(irq5_handler, 5);
irq_handler!(irq6_handler, 6);
irq_handler!(irq7_handler, 7);
irq_handler!(irq8_handler, 8);
irq_handler!(irq9_handler, 9);
irq_handler!(irq10_handler, 10);
irq_handler!(irq11_handler, 11);
irq_handler!(irq12_handler, 12);
irq_handler!(irq13_handler, 13);
irq_handler!(irq14_handler, 14);
irq_handler!(irq15_handler, 15);

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    debug!("EXCEPTION: BREAKPOINT");
    debug!("Stack Frame: {:#?}", stack_frame);
    panic!();
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    debug!("EXCEPTION: DOUBLE FAULT");
    debug!("Stack Frame: {:#?}", stack_frame);
    debug!("Error: {:?}", error_code);
    panic!();
}

extern "x86-interrupt" fn page_fault_handler(
    _stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let csi_color = api::console::Style::color("red");
    let csi_reset = api::console::Style::reset();
    let addr = Cr2::read();
    //debug!("EXCEPTION: PAGE FAULT ({:?}) at {:#X}", error_code, addr);

    let mut mapper = unsafe {
        sys::mem::create_mapper(sys::process::page_table())
    };

    // The heap and the stack of a process are allocated lazily
    if sys::process::is_userspace(addr) {
        let start = (addr / 4096) * 4096;
        if sys::mem::alloc_pages(&mut mapper, start, 4096).is_ok() {
            return;
        }
        printk!(
            "{}Error:{} Could not allocate page at {:#X}\n",
            csi_color, csi_reset, addr
        );
    } else {
        printk!(
            "{}Error:{} Page fault exception at {:#X}\n",
            csi_color, csi_reset, addr
        );
    }
    if error_code.contains(PageFaultErrorCode::USER_MODE) {
        api::syscall::exit(ExitCode::PageFaultError);
    } else {
        hang();
    }
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    debug!("EXCEPTION: GENERAL PROTECTION FAULT");
    debug!("Stack Frame: {:#?}", stack_frame);
    debug!("Error: {:?}", error_code);
    panic!();
}

extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    debug!("EXCEPTION: STACK SEGMENT FAULT");
    debug!("Stack Frame: {:#?}", stack_frame);
    debug!("Error: {:?}", error_code);
    panic!();
}

extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    debug!("EXCEPTION: SEGMENT NOT PRESENT");
    debug!("Stack Frame: {:#?}", stack_frame);
    debug!("Error: {:?}", error_code);
    panic!();
}

// Naked function wrapper saving all scratch registers to the stack
// See: https://os.phil-opp.com/returning-from-exceptions/
#[unsafe(naked)]
extern "sysv64" fn wrapped_syscall_handler() -> ! {
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
        sym syscall_handler
    );
}

// NOTE: We can't use "x86-interrupt" for syscall_handler because we need to
// return a result in the RAX register and it will be overwritten when the
// context of the caller is restored.
extern "sysv64" fn syscall_handler(
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
    if n == sys::syscall::number::SPAWN {
        sys::process::set_stack_frame(**stack_frame);
        sys::process::set_registers(*regs);
    }

    let res = sys::syscall::dispatcher(n, arg1, arg2, arg3, arg4);

    // Restore CPU context before exiting a process
    if n == sys::syscall::number::EXIT {
        let sf = sys::process::stack_frame();
        unsafe {
            stack_frame.as_mut().write(sf);
        }
        *regs = sys::process::registers();
    }

    regs.rax = res;
}

pub fn set_irq_handler(irq: u8, handler: fn()) {
    interrupts::without_interrupts(|| {
        let mut handlers = IRQ_HANDLERS.lock();
        handlers[irq as usize] = handler;

        pic::unmask(irq);
    });
}

static NULL_IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub fn reset() -> ! {
    NULL_IDT.load(); // No exception handlers
    unsafe {
        asm!("int 0", options(noreturn)); // Division by zero -> Triple fault
    }
}

pub fn init() {
    IDT.load();
}
