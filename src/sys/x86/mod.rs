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

pub mod addr {
    use core::ops::{Add, Sub};

    #[derive(Clone, Copy)]
    pub struct PhysAddr(usize); // NOTE: Uncompatible with x86-32 PAE

    impl PhysAddr {
        #[cfg(target_arch = "x86")]
        pub fn new(addr: usize) -> Self {
            Self(addr)
        }

        #[cfg(target_arch = "x86_64")]
        pub fn new(addr: usize) -> Self {
            let valid_addr = addr % (1 << 52);
            if addr != valid_addr {
                panic!("the address is not valid");
            }
            Self(addr)
        }

        pub fn as_usize(&self) -> usize {
            self.0
        }
    }

    impl Add<usize> for PhysAddr {
        type Output = Self;

        #[inline]
        fn add(self, other: usize) -> Self::Output {
            Self::new(self.0.checked_add(other).expect("overflow"))
        }
    }

    #[derive(Clone, Copy)]
    pub struct VirtAddr(usize);

    impl VirtAddr {
        #[cfg(target_arch = "x86")]
        pub fn new(addr: usize) -> Self {
            Self(addr)
        }

        #[cfg(target_arch = "x86_64")]
        pub fn new(addr: usize) -> Self {
            let canonical_addr = ((addr << 16) as isize >> 16) as usize;
            if addr != canonical_addr {
                panic!("the address is not canonical");
            }
            Self(addr)
        }

        pub fn as_usize(&self) -> usize {
            self.0
        }

        pub const fn as_ptr<T>(self) -> *const T {
            self.0 as *const T
        }

        pub const fn as_mut_ptr<T>(self) -> *mut T {
            self.as_ptr::<T>() as *mut T
        }
    }

    impl Add<usize> for VirtAddr {
        type Output = Self;

        #[inline]
        fn add(self, other: usize) -> Self::Output {
            Self::new(self.0.checked_add(other).expect("overflow"))
        }
    }

    impl Sub<usize> for VirtAddr {
        type Output = Self;

        #[inline]
        fn sub(self, other: usize) -> Self::Output {
            Self::new(self.0.checked_sub(other).expect("underflow"))
        }
    }

    #[cfg(target_arch = "x86_64")]
    impl From<PhysAddr> for x86_64::PhysAddr {
        fn from(addr: PhysAddr) -> Self {
            Self::new(addr.0 as u64)
        }
    }

    #[cfg(target_arch = "x86_64")]
    impl From<x86_64::PhysAddr> for PhysAddr {
        fn from(addr: x86_64::PhysAddr) -> Self {
            Self::new(addr.as_u64() as usize)
        }
    }

    #[cfg(target_arch = "x86_64")]
    impl From<VirtAddr> for x86_64::VirtAddr {
        fn from(addr: VirtAddr) -> Self {
            Self::new(addr.0 as u64)
        }
    }

    #[cfg(target_arch = "x86_64")]
    impl From<x86_64::VirtAddr> for VirtAddr {
        fn from(addr: x86_64::VirtAddr) -> Self {
            Self::new(addr.as_u64() as usize)
        }
    }
}
