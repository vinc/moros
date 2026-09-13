use super::addr::PhysAddr;

#[repr(usize)]
pub enum PageTableFlags {
    PRESENT  = 1 << 0,
    WRITABLE = 1 << 1,
    USER     = 1 << 2,
    HUGE     = 1 << 7,
}

pub const ENTRIES: usize = super::PAGE_SIZE / core::mem::size_of::<usize>();

#[cfg(target_arch = "x86")]
const LEVELS: usize = 2;

#[cfg(target_arch = "x86_64")]
const LEVELS: usize = 4;

#[cfg(target_arch = "x86")]
pub const INDEX_BITS: usize = 10;

#[cfg(target_arch = "x86_64")]
pub const INDEX_BITS: usize = 9;

#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: [PageTableEntry; ENTRIES]
}

impl PageTable {
    pub const fn new() -> Self {
        Self {
            entries: [PageTableEntry::unused(); ENTRIES]
        }
    }
}

#[derive(Clone, Copy)]
pub struct PageTableEntry(pub usize);

impl PageTableEntry {
    pub fn new(level: usize, index: usize, flags: usize) -> Self {
        let level = LEVELS - level;
        let addr = index * (super::PAGE_SIZE << (level * INDEX_BITS));
        Self(addr | flags as usize)
    }

    pub const fn unused() -> Self {
        Self(0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Frame(PhysAddr);

impl Frame {
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

impl From<Frame> for x86_64::structures::paging::PhysFrame {
    fn from(frame: Frame) -> Self {
        Self::from_start_address(frame.start_address().into()).unwrap()
    }
}

impl From<x86_64::structures::paging::PhysFrame> for Frame {
    fn from(frame: x86_64::structures::paging::PhysFrame) -> Self {
        Self::from_start_address(frame.start_address().into())
    }
}

#[test_case]
fn test_frame() {
    let values = [
        (0x0000, 0x0000),
        (0x1000, 0x1000),
        (0xA000, 0xA000),
        (0xAFFF, 0xA000),
    ];
    for (addr1, addr2) in values {
        assert_eq!(
            Frame::containing_address(PhysAddr::new(addr1)).start_address(),
            PhysAddr::new(addr2)
        );
    }

    for i in 0..10 {
        assert_eq!(
            Frame::from_number(i).start_address(),
            PhysAddr::new(i * super::PAGE_SIZE)
        );
    }
}
