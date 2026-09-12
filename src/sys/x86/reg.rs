use super::seg::SegmentSelector;
use super::addr::{PhysAddr, Frame};

use bit_field::BitField;
use core::arch::asm;

pub struct Cr0;

impl Cr0 {
    pub const PE: usize = 1 << 0; // Protection Enabled
    pub const WP: usize = 1 << 16; // Write Protect
    pub const PG: usize = 1 << 31; // Paging

    #[inline]
    pub fn read() -> usize {
        let value: usize;
        unsafe {
            asm!(
                "mov {}, cr0", out(reg) value,
                options(nomem, nostack, preserves_flags)
            );
        }
        value
    }

    #[inline]
    pub unsafe fn write(value: usize) {
        asm!(
            "mov cr0, {}", in(reg) value,
            options(nostack, preserves_flags)
        );
    }
}

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
        let flags = value.get_bits(0..12) as u16;
        let addr = value.get_bits(12..) << 12;
        Self { addr, flags }
    }

    #[inline]
    pub unsafe fn write(addr: usize, flags: u16) {
        #[cfg(target_arch = "x86_64")]
        debug_assert_eq!(addr.get_bits(52..64), 0);

        debug_assert_eq!(addr.get_bits(0..12), 0);
        debug_assert_eq!(flags.get_bits(12..16), 0);

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

    pub fn frame(&self) -> Frame {
        Frame::from_start_address(PhysAddr::new(self.addr))
    }
}

pub struct Cr4;

impl Cr4 {
    pub const PSE: usize = 1 << 4; // Page Size Extension

    #[inline]
    pub fn read() -> usize {
        let value: usize;
        unsafe {
            asm!(
                "mov {}, cr4", out(reg) value,
                options(nomem, nostack, preserves_flags)
            );
        }
        value
    }

    #[inline]
    pub unsafe fn write(value: usize) {
        asm!(
            "mov cr4, {}", in(reg) value,
            options(nostack, preserves_flags)
        );
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

#[test_case]
fn test_cr0() {
    assert_eq!(Cr0::read() & Cr0::PE, Cr0::PE);
}

#[test_case]
fn test_cr3() {
    let cr3 = Cr3::read();

    #[cfg(target_arch = "x86_64")]
    assert_ne!(cr3.addr(), 0);

    assert_eq!(cr3.addr() & 0xFFF, 0);
    assert_eq!(cr3.flags() & !0xFFF, 0);

    unsafe { Cr3::write(cr3.addr(), cr3.flags()) }

    assert_eq!(Cr3::read().addr(), cr3.addr());
    assert_eq!(Cr3::read().flags(), cr3.flags());
}
