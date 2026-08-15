use core::arch::asm;

use bit_field::BitField;
use x86_64::structures::paging::PhysFrame;
use x86_64::PhysAddr;

pub struct Cr2;

impl Cr2 {
    #[inline]
    pub fn read() -> u64 {
        let value: u64;
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
    addr: u64,
    flags: u16,
}

impl Cr3 {
    #[inline]
    pub fn read() -> Self {
        let value: u64;
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
    pub unsafe fn write(addr: u64, flags: u16) {
        debug_assert_eq!(addr.get_bits(..12), 0);
        debug_assert_eq!(addr.get_bits(52..), 0);
        debug_assert_eq!(flags.get_bits(12..), 0);
        let value = addr | flags as u64;
        asm!(
            "mov cr3, {}", in(reg) value,
            options(nostack, preserves_flags)
        );
    }

    pub fn flags(&self) -> u16 {
        self.flags
    }

    pub fn addr(&self) -> u64 {
        self.addr
    }

    pub fn frame(&self) -> PhysFrame {
        PhysFrame::containing_address(PhysAddr::new(self.addr))
    }
}

pub mod flags {
    pub const IF: usize = 1 << 9; // Interrupt Flag
}
