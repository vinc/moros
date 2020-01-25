use crate::{print, kernel};
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use x86_64::instructions::interrupts;

fn default_handler() {
    return;
}

macro_rules! irq_handler {
    ($handler:ident, $irq:expr) => {
        pub extern "x86-interrupt" fn $handler(_stack_frame: &mut InterruptStackFrame) {
            let handlers = IRQ_HANDLERS.lock();
            handlers[$irq]();
            unsafe { kernel::pic::PICS.lock().notify_end_of_interrupt(kernel::pic::PIC_1_OFFSET + $irq); }
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

lazy_static! {
    pub static ref IRQ_HANDLERS: Mutex<[fn(); 16]> = Mutex::new([default_handler; 16]);

    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(kernel::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[0 + kernel::pic::PIC_1_OFFSET as usize].set_handler_fn(irq0_handler);
        idt[1 + kernel::pic::PIC_1_OFFSET as usize].set_handler_fn(irq1_handler);
        idt[2 + kernel::pic::PIC_1_OFFSET as usize].set_handler_fn(irq2_handler);
        idt[3 + kernel::pic::PIC_1_OFFSET as usize].set_handler_fn(irq3_handler);
        idt[4 + kernel::pic::PIC_1_OFFSET as usize].set_handler_fn(irq4_handler);
        idt[5 + kernel::pic::PIC_1_OFFSET as usize].set_handler_fn(irq5_handler);
        idt[6 + kernel::pic::PIC_1_OFFSET as usize].set_handler_fn(irq6_handler);
        idt[7 + kernel::pic::PIC_1_OFFSET as usize].set_handler_fn(irq7_handler);
        idt[8 + kernel::pic::PIC_1_OFFSET as usize].set_handler_fn(irq8_handler);
        idt[9 + kernel::pic::PIC_1_OFFSET as usize].set_handler_fn(irq9_handler);
        idt[10 + kernel::pic::PIC_1_OFFSET as usize].set_handler_fn(irq10_handler);
        idt[11 + kernel::pic::PIC_1_OFFSET as usize].set_handler_fn(irq11_handler);
        idt[12 + kernel::pic::PIC_1_OFFSET as usize].set_handler_fn(irq12_handler);
        idt[13 + kernel::pic::PIC_1_OFFSET as usize].set_handler_fn(irq13_handler);
        idt[14 + kernel::pic::PIC_1_OFFSET as usize].set_handler_fn(irq14_handler);
        idt[15 + kernel::pic::PIC_1_OFFSET as usize].set_handler_fn(irq15_handler);
        idt
    };
}

pub fn init() {
    IDT.load();
}

pub fn set_irq_handler(irq: u8, handler: fn()) {
    interrupts::without_interrupts(|| {
        let mut handlers = IRQ_HANDLERS.lock();
        handlers[irq as usize] = handler;
    });
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    print!("EXCEPTION: BREAKPOINT\n{:#?}\n", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: &mut InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}
