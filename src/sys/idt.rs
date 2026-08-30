use crate::{hang, sys};
use crate::sys::gdt;
use crate::sys::pic;
use crate::sys::tss;
use crate::sys::x86::DescriptorTablePointer;
use crate::sys::x86::int;
use crate::sys::x86::int::InterruptFrame;
use crate::sys::x86::reg::Cr2;
use crate::sys::x86::seg::SegmentSelector;
use crate::sys::x86;

use bit_field::BitField;
use core::arch::asm;
use lazy_static::lazy_static;
use spin::Mutex;

const LEN: usize = 256;

// Vectors
const BP: usize = 3; // Breakpoint
const DF: usize = 8; // Double fault
const NP: usize = 11; // Segment not present
const SS: usize = 12; // Stack fault
const GP: usize = 13; // General protection
const PF: usize = 14; // Page fault

// Page fault errors
const PF_U: usize = 2; // User mode

struct InterruptDescriptorTable {
    table: [Entry; LEN]
}

impl InterruptDescriptorTable {
    fn new() -> Self {
        let mut table = [Entry::missing(); LEN];

        unsafe {
            table[BP].set_handler(breakpoint_handler as _);
            table[DF].set_handler(double_fault_handler as _);
            table[NP].set_handler(segment_not_present_handler as _);
            table[SS].set_handler(stack_fault_handler as _);
            table[GP].set_handler(general_protection_handler as _);
            table[PF].set_handler(page_fault_handler as _);

            #[cfg(target_arch = "x86_64")]
            {
                table[DF].set_stack_index(tss::DF);
                table[PF].set_stack_index(tss::PF);
                table[GP].set_stack_index(tss::GP);
            }

            table[pic::vector(0)].set_handler(irq0_handler as _);
            table[pic::vector(1)].set_handler(irq1_handler as _);
            table[pic::vector(2)].set_handler(irq2_handler as _);
            table[pic::vector(3)].set_handler(irq3_handler as _);
            table[pic::vector(4)].set_handler(irq4_handler as _);
            table[pic::vector(5)].set_handler(irq5_handler as _);
            table[pic::vector(6)].set_handler(irq6_handler as _);
            table[pic::vector(7)].set_handler(irq7_handler as _);
            table[pic::vector(8)].set_handler(irq8_handler as _);
            table[pic::vector(9)].set_handler(irq9_handler as _);
            table[pic::vector(10)].set_handler(irq10_handler as _);
            table[pic::vector(11)].set_handler(irq11_handler as _);
            table[pic::vector(12)].set_handler(irq12_handler as _);
            table[pic::vector(13)].set_handler(irq13_handler as _);
            table[pic::vector(14)].set_handler(irq14_handler as _);
            table[pic::vector(15)].set_handler(irq15_handler as _);

            #[cfg(target_arch = "x86_64")] // TODO: Remove
            {
                table[0x80].set_handler(sys::syscall::handler as _);
                table[0x80].set_privilege_level(3);
            }
        }

        Self { table }
    }

    fn limit(&self) -> u16 {
        (LEN * size_of::<Entry>() - 1) as u16
    }

    fn pointer(&self) -> DescriptorTablePointer {
        DescriptorTablePointer {
            limit: self.limit(),
            base: self.table.as_ptr() as usize,
        }
    }

    fn load(&'static self) {
        unsafe { x86::lidt(&self.pointer()) }
    }
}

#[derive(Clone, Copy, Default)]
#[repr(C, align(8))]
struct Entry {
    pointer_low: u16,
    selector: SegmentSelector,
    bits: u16,
    pointer_mid: u16,

    #[cfg(target_arch = "x86_64")]
    pointer_high: u32,
    #[cfg(target_arch = "x86_64")]
    reserved: u32,
}

impl Entry {
    fn missing() -> Self {
        Self::default()
    }

    unsafe fn set_handler(&mut self, handler: *const ()) {
        let addr = handler.addr();

        self.pointer_low = addr as u16;
        self.pointer_mid = (addr >> 16) as u16;

        #[cfg(target_arch = "x86_64")]
        {
            self.pointer_high = (addr >> 32) as u32;
        }

        self.selector = gdt::SYS_CODE;
        self.bits.set_bits(8..12, x86::seg::TYPE_IG as u16); // Type
        self.bits.set_bit(15, true); // Present (P)
    }

    fn set_privilege_level(&mut self, level: u16) {
        self.bits.set_bits(13..15, level); // Descriptor Privilege Level (DPL)
    }

    #[cfg(target_arch = "x86_64")]
    fn set_stack_index(&mut self, index: usize) {
        self.bits.set_bits(0..3, (index + 1) as u16); // IST
    }
}

fn default_handler() {}

lazy_static! {
    static ref IRQ_HANDLERS: Mutex<[fn(); 16]> = {
        Mutex::new([default_handler; 16])
    };

    static ref IDT: InterruptDescriptorTable = {
        InterruptDescriptorTable::new()
    };
}

macro_rules! irq_handler {
    ($handler:ident, $irq:expr) => {
        pub extern "x86-interrupt" fn $handler(_: InterruptFrame) {
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

extern "x86-interrupt" fn breakpoint_handler(frame: InterruptFrame) {
    debug!("EXCEPTION: BREAKPOINT (#BP)");
    debug!("Frame: {:#?}", frame);
    panic!();
}

extern "x86-interrupt" fn double_fault_handler(
    frame: InterruptFrame,
    error: usize,
) -> ! {
    debug!("EXCEPTION: DOUBLE FAULT (#DF)");
    debug!("Frame: {:#?}", frame);
    debug!("Error: {:#X}", error);
    panic!();
}

#[cfg(target_arch = "x86")]
extern "x86-interrupt" fn page_fault_handler(
    frame: InterruptFrame,
    error: usize,
) {
    debug!("EXCEPTION: PAGE FAULT (#PF)");
    debug!("Frame: {:#?}", frame);
    debug!("Error: {:#X}", error);
    panic!();
}

#[cfg(target_arch = "x86_64")] // TODO: Remove
extern "x86-interrupt" fn page_fault_handler(
    _frame: InterruptFrame,
    error: usize,
) {
    //debug!("EXCEPTION: PAGE FAULT (#PF)");
    //debug!("Frame: {:#?}", frame);
    //debug!("Error: {:#X}", error);

    use crate::api;
    use crate::api::process::ExitCode;
    let csi_color = api::console::Style::color("red");
    let csi_reset = api::console::Style::reset();
    let addr = Cr2::read() as u64;
    //debug!("Addr: {:#X}", addr);

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
    if error.get_bit(PF_U) { // Userspace
        api::syscall::exit(ExitCode::PageFaultError);
    } else {
        hang();
    }
}

extern "x86-interrupt" fn general_protection_handler(
    frame: InterruptFrame,
    error: usize,
) {
    debug!("EXCEPTION: GENERAL PROTECTION (#GP)");
    debug!("Frame: {:#?}", frame);
    debug!("Error: {:#X}", error);
    panic!();
}

extern "x86-interrupt" fn stack_fault_handler(
    frame: InterruptFrame,
    error: usize,
) {
    debug!("EXCEPTION: STACK FAULT (#SS)");
    debug!("Frame: {:#?}", frame);
    debug!("Error: {:#X}", error);
    panic!();
}

extern "x86-interrupt" fn segment_not_present_handler(
    frame: InterruptFrame,
    error: usize,
) {
    debug!("EXCEPTION: SEGMENT NOT PRESENT (#NP)");
    debug!("Frame: {:#?}", frame);
    debug!("Error: {:#X}", error);
    panic!();
}

pub fn set_irq_handler(irq: u8, handler: fn()) {
    int::without_interrupts(|| {
        let mut handlers = IRQ_HANDLERS.lock();
        handlers[irq as usize] = handler;

        pic::unmask(irq);
    });
}

pub fn reset() -> ! {
    int::disable_interrupts();
    let idt = DescriptorTablePointer {
        limit: 0,
        base: 0,
    };
    unsafe {
        x86::lidt(&idt);
        asm!("int3", options(noreturn)); // Triple fault
    }
}

pub fn init() {
    IDT.load();
}

#[cfg(target_arch = "x86")]
#[test_case]
fn test_idt() {
    assert_eq!(size_of::<Entry>(), 8);
    assert_eq!(IDT.limit(), 2047);

    // No IST on i686
    assert_eq!(IDT.table[DF].bits, 0x8E00);
    assert_eq!(IDT.table[PF].bits, 0x8E00);
    assert_eq!(IDT.table[GP].bits, 0x8E00);

    assert_eq!(IDT.table[0].bits, 0);         // Not present
}

#[cfg(target_arch = "x86_64")]
#[test_case]
fn test_idt() {
    assert_eq!(size_of::<Entry>(), 16);
    assert_eq!(IDT.limit(), 4095);

    assert_eq!(IDT.table[DF].bits, 0x8E01);   // IST 1
    assert_eq!(IDT.table[PF].bits, 0x8E02);   // IST 2
    assert_eq!(IDT.table[GP].bits, 0x8E03);   // IST 3

    assert_eq!(IDT.table[0x80].bits, 0xEE00); // DPL 3

    assert_eq!(IDT.table[0].bits, 0);         // Not present
}
