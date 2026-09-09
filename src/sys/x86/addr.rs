use bit_field::BitField;
use core::ops::{Add, Sub};

pub fn align_up(addr: usize) -> usize {
    addr.next_multiple_of(super::PAGE_SIZE)
}

pub fn align_down(addr: usize) -> usize {
    addr - (addr % super::PAGE_SIZE)
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
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
        debug_assert_eq!(super::PAGE_SIZE, 1 << 12);
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

impl Sub<usize> for PhysAddr {
    type Output = Self;

    #[inline]
    fn sub(self, other: usize) -> Self::Output {
        Self::new(self.0.checked_sub(other).expect("underflow"))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
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
        debug_assert_eq!(super::PAGE_SIZE, 1 << 12);
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

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct PhysFrame(PhysAddr);

impl PhysFrame {
    pub fn containing_address(addr: PhysAddr) -> Self {
        Self::from_start_address(addr - addr.page_offset()) // Align down
    }

    pub fn from_start_address(addr: PhysAddr) -> Self {
        debug_assert_eq!(addr.page_offset(), 0);
        Self(addr)
    }

    pub fn start_address(self) -> PhysAddr {
        self.0
    }

    pub fn from_number(number: usize) -> Self {
        Self::from_start_address(PhysAddr::new(number << 12))
    }

    pub fn number(self) -> usize {
        self.0.as_usize() >> 12
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

#[test_case]
fn test_phys_addr() {
    assert_eq!(PhysAddr::new(0x1234).as_usize(), 0x1234);
    assert_eq!(PhysAddr::new(0x1234).page_offset(), 0x234);
    assert_eq!(PhysAddr::new(0x1000) + 0x0234, PhysAddr::new(0x1234));
    assert_eq!(PhysAddr::new(0x1234) - 0x0234, PhysAddr::new(0x1000));
}

#[test_case]
fn test_virt_addr() {
    assert_eq!(VirtAddr::new(0x1234).as_usize(), 0x1234);
    assert_eq!(VirtAddr::new(0x1234).page_offset(), 0x234);
    assert_eq!(VirtAddr::new(0x1000) + 0x0234, VirtAddr::new(0x1234));
    assert_eq!(VirtAddr::new(0x1234) - 0x0234, VirtAddr::new(0x1000));
}

#[test_case]
fn test_phys_frame() {
    let values = [
        (0x0000, 0x0000),
        (0x1000, 0x1000),
        (0xA000, 0xA000),
        (0xAFFF, 0xA000),
    ];
    for (addr1, addr2) in values {
        assert_eq!(
            PhysFrame::containing_address(PhysAddr::new(addr1)).start_address(),
            PhysAddr::new(addr2)
        );
    }

    for i in 0..10 {
        assert_eq!(
            PhysFrame::from_number(i).start_address(),
            PhysAddr::new(i * super::PAGE_SIZE)
        );
    }
}
