pub mod int;
pub mod reg;
pub mod seg;

use core::arch::asm;

/// Halts the CPU until the next interrupt
#[inline]
pub fn hlt() {
    unsafe {
        asm!("hlt", options(nomem, nostack, preserves_flags));
    }
}

#[cfg(target_arch = "x86_64")]
pub fn rdrand() -> Option<u64> {
    let mut res = 0;
    unsafe {
        if core::arch::x86_64::_rdrand64_step(&mut res) == 1 {
            Some(res)
        } else {
            None
        }
    }
}

#[cfg(target_arch = "x86")]
pub fn rdrand() -> Option<u64> {
    None
}

#[repr(C, packed(2))]
pub struct DescriptorTablePointer {
    pub limit: u16,
    pub base: usize,
}

pub unsafe fn lgdt(gdt: &DescriptorTablePointer) {
    unsafe {
        asm!("lgdt [{}]", in(reg) gdt, options(readonly, nostack, preserves_flags));
    }
}

pub mod port {
    use core::arch::asm;

    #[inline]
    pub unsafe fn outb(port: u16, value: u8) {
        asm!(
            "out dx, al", in("dx") port, in("al") value,
            options(nostack, preserves_flags)
        );
    }

    #[inline]
    pub unsafe fn inb(port: u16) -> u8 {
        let value: u8;
        asm!(
            "in al, dx", in("dx") port, out("al") value,
            options(nostack, preserves_flags)
        );
        value
    }

    #[inline]
    pub unsafe fn outw(port: u16, value: u16) {
        asm!(
            "out dx, ax", in("dx") port, in("ax") value,
            options(nostack, preserves_flags)
        );
    }

    #[inline]
    pub unsafe fn inw(port: u16) -> u16 {
        let value: u16;
        asm!(
            "in ax, dx", in("dx") port, out("ax") value,
            options(nostack, preserves_flags)
        );
        value
    }

    #[inline]
    pub unsafe fn outl(port: u16, value: u32) {
        asm!(
            "out dx, eax", in("dx") port, in("eax") value,
            options(nostack, preserves_flags)
        );
    }

    #[inline]
    pub unsafe fn inl(port: u16) -> u32 {
        let value: u32;
        asm!(
            "in eax, dx", in("dx") port, out("eax") value,
            options(nostack, preserves_flags)
        );
        value
    }
}
