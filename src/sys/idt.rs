use crate::{api, sys, hlt_loop};
use crate::api::process::ExitCode;
use crate::sys::process::Registers;

use core::arch::asm;
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::interrupts;
use x86_64::instructions::port::Port;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, InterruptStackFrameValue, PageFaultErrorCode};

const PIC1: u16 = 0x21;
const PIC2: u16 = 0xA1;

pub fn init() {
    IDT.load();
}

// Translate IRQ into system interrupt
fn interrupt_index(irq: u8) -> u8 {
    sys::pic::PIC_1_OFFSET + irq
}

fn default_irq_handler() {}

lazy_static! {
    pub static ref IRQ_HANDLERS: Mutex<[fn(); 16]> = Mutex::new([default_irq_handler; 16]);

    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.stack_segment_fault.set_handler_fn(stack_segment_fault_handler);
        idt.segment_not_present.set_handler_fn(segment_not_present_handler);
        unsafe {
            idt.double_fault.
                set_handler_fn(double_fault_handler).
                set_stack_index(sys::gdt::DOUBLE_FAULT_IST_INDEX);
            idt.page_fault.
                set_handler_fn(page_fault_handler).
                set_stack_index(sys::gdt::PAGE_FAULT_IST_INDEX);
            idt.general_protection_fault.
                set_handler_fn(general_protection_fault_handler).
                set_stack_index(sys::gdt::GENERAL_PROTECTION_FAULT_IST_INDEX);
            idt[0x80].
                set_handler_fn(core::mem::transmute(wrapped_syscall_handler as *mut fn())).
                //set_stack_index(sys::gdt::GENERAL_PROTECTION_FAULT_IST_INDEX).
                set_privilege_level(x86_64::PrivilegeLevel::Ring3);
        }
        idt[interrupt_index(0) as usize].set_handler_fn(irq0_handler);
        idt[interrupt_index(1) as usize].set_handler_fn(irq1_handler);
        idt[interrupt_index(2) as usize].set_handler_fn(irq2_handler);
        idt[interrupt_index(3) as usize].set_handler_fn(irq3_handler);
        idt[interrupt_index(4) as usize].set_handler_fn(irq4_handler);
        idt[interrupt_index(5) as usize].set_handler_fn(irq5_handler);
        idt[interrupt_index(6) as usize].set_handler_fn(irq6_handler);
        idt[interrupt_index(7) as usize].set_handler_fn(irq7_handler);
        idt[interrupt_index(8) as usize].set_handler_fn(irq8_handler);
        idt[interrupt_index(9) as usize].set_handler_fn(irq9_handler);
        idt[interrupt_index(10) as usize].set_handler_fn(irq10_handler);
        idt[interrupt_index(11) as usize].set_handler_fn(irq11_handler);
        idt[interrupt_index(12) as usize].set_handler_fn(irq12_handler);
        idt[interrupt_index(13) as usize].set_handler_fn(irq13_handler);
        idt[interrupt_index(14) as usize].set_handler_fn(irq14_handler);
        idt[interrupt_index(15) as usize].set_handler_fn(irq15_handler);
        idt
    };
}

macro_rules! irq_handler {
    ($handler:ident, $irq:expr) => {
        pub extern "x86-interrupt" fn $handler(_stack_frame: InterruptStackFrame) {
            let handlers = IRQ_HANDLERS.lock();
            handlers[$irq]();
            unsafe { sys::pic::PICS.lock().notify_end_of_interrupt(interrupt_index($irq)); }
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

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) -> ! {
    debug!("EXCEPTION: DOUBLE FAULT");
    debug!("Stack Frame: {:#?}", stack_frame);
    debug!("Error: {:?}", error_code);
    panic!();
}

extern "x86-interrupt" fn page_fault_handler(_stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    //debug!("EXCEPTION: PAGE FAULT ({:?})", error_code);
    let addr = Cr2::read().as_u64();
    if sys::allocator::alloc_pages(addr, 1).is_err() {
        let csi_color = api::console::Style::color("LightRed");
        let csi_reset = api::console::Style::reset();
        printk!("{}Error:{} Could not allocate address {:#x}\n", csi_color, csi_reset, addr);
        if error_code.contains(PageFaultErrorCode::USER_MODE) {
            api::syscall::exit(ExitCode::PageFaultError);
        } else {
            hlt_loop();
        }
    }
}

extern "x86-interrupt" fn general_protection_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    debug!("EXCEPTION: GENERAL PROTECTION FAULT");
    debug!("Stack Frame: {:#?}", stack_frame);
    debug!("Error: {:?}", error_code);
    panic!();
}

extern "x86-interrupt" fn stack_segment_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    debug!("EXCEPTION: STACK SEGMENT FAULT");
    debug!("Stack Frame: {:#?}", stack_frame);
    debug!("Error: {:?}", error_code);
    panic!();
}

extern "x86-interrupt" fn segment_not_present_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    debug!("EXCEPTION: SEGMENT NOT PRESENT");
    debug!("Stack Frame: {:#?}", stack_frame);
    debug!("Error: {:?}", error_code);
    panic!();
}

// Naked function wrapper saving all scratch registers to the stack
// See: https://os.phil-opp.com/returning-from-exceptions/#a-naked-wrapper-function
macro_rules! wrap {
    ($fn: ident => $w:ident) => {
        #[naked]
        pub unsafe extern "sysv64" fn $w() {
            asm!(
                "push rax",
                "push rcx",
                "push rdx",
                "push rsi",
                "push rdi",
                "push r8",
                "push r9",
                "push r10",
                "push r11",
                "mov rsi, rsp", // Arg #2: register list
                "mov rdi, rsp", // Arg #1: interupt frame
                "add rdi, 9 * 8",
                "call {}",
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
                sym $fn,
                options(noreturn)
            );
        }
    };
}

wrap!(syscall_handler => wrapped_syscall_handler);

// NOTE: We can't use "x86-interrupt" for syscall_handler because we need to
// return a result in the RAX register and it will be overwritten when the
// context of the caller is restored.
extern "sysv64" fn syscall_handler(stack_frame: &mut InterruptStackFrame, regs: &mut Registers) {
    // The registers order follow the System V ABI convention
    let n    = regs.rax;
    let arg1 = regs.rdi;
    let arg2 = regs.rsi;
    let arg3 = regs.rdx;
    let arg4 = regs.r8;

    if n == sys::syscall::number::SPAWN { // Backup CPU context
        sys::process::set_stack_frame(**stack_frame);
        sys::process::set_registers(*regs);
    }

    let res = sys::syscall::dispatcher(n, arg1, arg2, arg3, arg4);

    if n == sys::syscall::number::EXIT { // Restore CPU context
        let sf = sys::process::stack_frame();
        unsafe {
            //stack_frame.as_mut().write(sf);
            core::ptr::write_volatile(stack_frame.as_mut().extract_inner() as *mut InterruptStackFrameValue, sf); // FIXME
            core::ptr::write_volatile(regs, sys::process::registers());
        }
    }

    regs.rax = res;

    unsafe { sys::pic::PICS.lock().notify_end_of_interrupt(0x80) };
}

pub fn set_irq_handler(irq: u8, handler: fn()) {
    interrupts::without_interrupts(|| {
        let mut handlers = IRQ_HANDLERS.lock();
        handlers[irq as usize] = handler;

        clear_irq_mask(irq);
    });
}

pub fn set_irq_mask(irq: u8) {
    let mut port: Port<u8> = Port::new(if irq < 8 { PIC1 } else { PIC2 });
    unsafe {
        let value = port.read() | (1 << (if irq < 8 { irq } else { irq - 8 }));
        port.write(value);
    }
}

pub fn clear_irq_mask(irq: u8) {
    let mut port: Port<u8> = Port::new(if irq < 8 { PIC1 } else { PIC2 });
    unsafe {
        let value = port.read() & !(1 << if irq < 8 { irq } else { irq - 8 });
        port.write(value);
    }
}
