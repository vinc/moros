use bit_field::BitField;
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

    pub fn page_offset(self) -> usize {
        self.0.get_bits(0..12)
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

    pub fn page_offset(self) -> usize {
        self.0.get_bits(0..12)
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

impl From<PhysAddr> for x86_64::PhysAddr {
    fn from(addr: PhysAddr) -> Self {
        Self::new(addr.0 as u64)
    }
}

impl From<x86_64::PhysAddr> for PhysAddr {
    fn from(addr: x86_64::PhysAddr) -> Self {
        Self::new(addr.as_u64() as usize)
    }
}

impl From<VirtAddr> for x86_64::VirtAddr {
    fn from(addr: VirtAddr) -> Self {
        Self::new(addr.0 as u64)
    }
}

impl From<x86_64::VirtAddr> for VirtAddr {
    fn from(addr: x86_64::VirtAddr) -> Self {
        Self::new(addr.as_u64() as usize)
    }
}

#[derive(Clone, Copy)]
pub struct PhysFrame(PhysAddr);

impl PhysFrame {
    pub fn from_start_address(addr: PhysAddr) -> Self {
        debug_assert_eq!(addr.page_offset(), 0);
        Self(addr)
    }

    pub fn start_address(self) -> PhysAddr {
        self.0
    }
}

impl From<PhysFrame> for x86_64::structures::paging::PhysFrame {
    fn from(frame: PhysFrame) -> Self {
        Self::from_start_address(frame.start_address().into()).unwrap()
    }
}

impl From<x86_64::structures::paging::PhysFrame> for PhysFrame {
    fn from(frame: x86_64::structures::paging::PhysFrame) -> Self {
        Self::from_start_address(frame.start_address().into())
    }
}
