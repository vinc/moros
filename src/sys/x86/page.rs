use super::addr::PhysAddr;

// 64-bit virtual address:
//
//     63      48 47     39 38     30 29     21 20     12 11         0
//    +----------+---------+---------+---------+---------+------------+
//    | sign ext |  PML4   |   PDP   |   PD    |   PT    |   offset   |
//    +----------+---------+---------+---------+---------+------------+
//
// 32-bit virtual address:
//
//                                  31      22 21      12 11         0
//                                 +----------+----------+------------+
//                                 |    PD    |    PT    |   offset   |
//                                 +----------+----------+------------+
//
// L4 = PML4 (Page Map Level 4)
// L3 = PDP (Page Directory Pointer)
// L2 = PD (Page Directory)
// L1 = PT (Page Table)

#[cfg(target_arch = "x86")]
const LEVELS: usize = 2;

#[cfg(target_arch = "x86_64")]
const LEVELS: usize = 4;

pub const PAGE_SIZE: usize = 4096;

// 1024 entries per table on 32-bit, 512 on 64-bit
pub const ENTRIES: usize = PAGE_SIZE / core::mem::size_of::<PageTableEntry>();

// 10 bits per level on 32-bit, 9 bits on 64-bit
pub const LEVEL_SHIFT: usize = ENTRIES.ilog2() as usize;

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
#[repr(transparent)]
pub struct PageTableEntry(pub usize);

impl PageTableEntry {
    pub const PRESENT:  usize = 1 << 0;
    pub const WRITABLE: usize = 1 << 1;
    pub const USER:     usize = 1 << 2;
    pub const ACCESSED: usize = 1 << 5;
    pub const DIRTY:    usize = 1 << 6;
    pub const LARGE:    usize = 1 << 7;

    pub fn new(level: usize, index: usize, flags: usize) -> Self {
        debug_assert!(0 < level && level <= LEVELS);
        let addr = index * (PAGE_SIZE << ((level - 1) * LEVEL_SHIFT));
        Self(addr | flags)
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
            PhysAddr::new(i * PAGE_SIZE)
        );
    }
}
