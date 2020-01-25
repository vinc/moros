use crate::{print, kernel};
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

pub const IRQ0: u8 = kernel::pic::PIC_1_OFFSET + 0;
pub const IRQ1: u8 = kernel::pic::PIC_1_OFFSET + 1;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(kernel::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[IRQ0 as usize].set_handler_fn(timer_interrupt_handler);
        idt[IRQ1 as usize].set_handler_fn(kernel::keyboard::interrupt_handler);
        idt
    };
}

pub fn init() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    print!("EXCEPTION: BREAKPOINT\n{:#?}\n", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: &mut InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    kernel::clock::tick();

    unsafe {
        kernel::pic::PICS.lock().notify_end_of_interrupt(IRQ0);
    }
}
