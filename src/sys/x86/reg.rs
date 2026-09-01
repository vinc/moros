use super::seg::SegmentSelector;
use super::addr::PhysAddr;

use core::arch::asm;

use bit_field::BitField;
use x86_64::structures::paging::PhysFrame;
//use x86_64::PhysAddr;

pub struct Cr2;

impl Cr2 {
    #[inline]
    pub fn read() -> usize {
        let value: usize;
        unsafe {
            asm!(
                "mov {}, cr2", out(reg) value,
                options(nomem, nostack, preserves_flags)
            );
        }
        value
    }
}

pub struct Cr3 {
    addr: usize,
    flags: u16,
}

impl Cr3 {
    #[inline]
    pub fn read() -> Self {
        let value: usize;
        unsafe {
            asm!(
                "mov {}, cr3", out(reg) value,
                options(nomem, nostack, preserves_flags)
            );
        }
        let mask = 0xFFF;
        let addr = value & !mask;
        let flags = (value & mask) as u16;
        Self { addr, flags }
    }

    #[inline]
    pub unsafe fn write(addr: usize, flags: u16) {
        debug_assert_eq!(addr.get_bits(..12), 0);
        debug_assert_eq!(addr.get_bits(52..), 0);
        debug_assert_eq!(flags.get_bits(12..), 0);
        let value = addr | flags as usize;
        asm!(
            "mov cr3, {}", in(reg) value,
            options(nostack, preserves_flags)
        );
    }

    pub fn flags(&self) -> u16 {
        self.flags
    }

    pub fn addr(&self) -> usize {
        self.addr
    }

    pub fn frame(&self) -> PhysFrame {
        PhysFrame::containing_address(PhysAddr::new(self.addr))
    }
}

pub mod flags {
    pub const IF: usize = 1 << 9; // Interrupt Flag
}

#[inline]
pub unsafe fn load_cs(sel: SegmentSelector) {
    #[cfg(target_arch = "x86")]
    asm!(
        "push {0}", // Selector
        "lea {0}, [2f]",
        "push {0}", // Return address
        "retf",
        "2:",
        inout(reg) usize::from(sel.bits) => _,
        options(preserves_flags),
    );

    #[cfg(target_arch = "x86_64")]
    asm!(
        "push {0}", // Selector
        "lea {0}, [rip + 2f]",
        "push {0}", // Return address
        "retfq",
        "2:",
        inout(reg) usize::from(sel.bits) => _,
        options(preserves_flags),
    );
}

#[inline]
pub unsafe fn load_ds(sel: SegmentSelector) {
    asm!("mov ds, {:x}", in(reg) sel.bits, options(nostack, preserves_flags));
}

#[inline]
pub unsafe fn load_es(sel: SegmentSelector) {
    asm!("mov es, {:x}", in(reg) sel.bits, options(nostack, preserves_flags));
}

#[inline]
pub unsafe fn load_ss(sel: SegmentSelector) {
    asm!("mov ss, {:x}", in(reg) sel.bits, options(nostack, preserves_flags));
}

#[inline]
pub unsafe fn load_tss(sel: SegmentSelector) {
    asm!("ltr {:x}", in(reg) sel.bits, options(nostack, preserves_flags));
}

#[cfg(target_arch = "x86_64")]
#[test_case]
fn test_cr3() {
    let cr3 = Cr3::read();
    assert_ne!(cr3.addr(), 0);
    assert_eq!(cr3.addr() & 0xFFF, 0);
    assert_eq!(cr3.flags() & !0xFFF, 0);

    unsafe { Cr3::write(cr3.addr(), cr3.flags()) }

    assert_eq!(Cr3::read().addr(), cr3.addr());
    assert_eq!(Cr3::read().flags(), cr3.flags());
}
