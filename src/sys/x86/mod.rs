pub mod addr;
pub mod int;
pub mod port;
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

#[cfg(target_arch = "x86")]
pub fn rdrand() -> Option<u64> {
    None
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
pub fn rdtsc() -> u64 {
    unsafe {
        core::arch::x86::_mm_lfence();
        core::arch::x86::_rdtsc()
    }
}

#[cfg(target_arch = "x86_64")]
pub fn rdtsc() -> u64 {
    unsafe {
        core::arch::x86_64::_mm_lfence();
        core::arch::x86_64::_rdtsc()
    }
}

#[repr(C, packed(2))]
pub struct DescriptorTablePointer {
    pub limit: u16,
    pub base: usize,
}

pub unsafe fn lgdt(gdt: &DescriptorTablePointer) {
    unsafe {
        asm!(
            "lgdt [{}]", in(reg) gdt,
            options(readonly, nostack, preserves_flags)
        );
    }
}

pub unsafe fn lidt(idt: &DescriptorTablePointer) {
    unsafe {
        asm!(
            "lidt [{}]", in(reg) idt,
            options(readonly, nostack, preserves_flags)
        );
    }
}
