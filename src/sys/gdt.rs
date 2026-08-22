use crate::sys::tss;
use crate::sys::tss::TaskStateSegment;
use crate::sys::x86;
use crate::sys::x86::DescriptorTablePointer;
use crate::sys::x86::reg;
use crate::sys::x86::seg::SegmentSelector;

use bit_field::BitField;
use lazy_static::lazy_static;

lazy_static! {
    static ref GDT: GlobalDescriptorTable = GlobalDescriptorTable::new(&tss::TSS);
}

pub const SYS_CODE: SegmentSelector = SegmentSelector::new(1, 0);
pub const SYS_DATA: SegmentSelector = SegmentSelector::new(2, 0);
pub const USR_CODE: SegmentSelector = SegmentSelector::new(3, 3);
pub const USR_DATA: SegmentSelector = SegmentSelector::new(4, 3);
pub const TSS:      SegmentSelector = SegmentSelector::new(5, 0);

// Segment descriptor flags (Intel SDM 3.4.5)
const ACCESSED:     u64 = 1 << 40; // Accessed (A)
const READABLE:     u64 = 1 << 41; // Readable (R)
const WRITABLE:     u64 = 1 << 41; // Writable (W)
const EXECUTABLE:   u64 = 1 << 43; // Executable (E)
const CODE_OR_DATA: u64 = 1 << 44; // Segment (S)
const RING_3:       u64 = 3 << 45; // Descriptor Privilege-Level (DPL)
const PRESENT:      u64 = 1 << 47; // Present (P)
const SIZE_64:      u64 = 1 << 53; // Long (L)
const SIZE_32:      u64 = 1 << 54; // Default Operand Size (D/B)
const GRANULARITY:  u64 = 1 << 55; // Granularity (G)

const LIMIT_LO:     u64 = 0xFFFF;
const LIMIT_HI:     u64 = 0xF << 48;
const LIMIT:        u64 = LIMIT_LO | LIMIT_HI | GRANULARITY;

const COMMON:       u64 = CODE_OR_DATA | PRESENT | ACCESSED | LIMIT;
const COMMON_DATA:  u64 = COMMON | WRITABLE;
const COMMON_CODE:  u64 = COMMON | READABLE | EXECUTABLE;

// System segment type (Intel SDM 3.5)
const TYPE_TSS: u64 = 0b1001; // 32-bit or 64-bit TSS (Available)

#[cfg(target_arch = "x86")]
const LEN: usize = 6;

#[cfg(target_arch = "x86_64")]
const LEN: usize = 7;

struct GlobalDescriptorTable {
    table: [u64; LEN],
}

impl GlobalDescriptorTable {
    fn new(tss: &'static TaskStateSegment) -> Self {
        let mut table = [0; LEN];

        table[SYS_DATA.index()] = COMMON_DATA | SIZE_32;
        table[USR_DATA.index()] = COMMON_DATA | SIZE_32 | RING_3;

        #[cfg(target_arch = "x86")]
        {
            table[SYS_CODE.index()] = COMMON_CODE | SIZE_32;
            table[USR_CODE.index()] = COMMON_CODE | SIZE_32 | RING_3;
        }

        #[cfg(target_arch = "x86_64")]
        {
            table[SYS_CODE.index()] = COMMON_CODE | SIZE_64;
            table[USR_CODE.index()] = COMMON_CODE | SIZE_64 | RING_3;
        }

        let base = tss.base() as u64;
        let mut bits = PRESENT;
        bits.set_bits(0..16, tss.limit() as u64);
        bits.set_bits(16..40, base.get_bits(0..24));
        bits.set_bits(40..44, TYPE_TSS);
        bits.set_bits(56..64, base.get_bits(24..32));
        table[TSS.index()] = bits;

        #[cfg(target_arch = "x86_64")]
        {
            table[TSS.index() + 1] = base.get_bits(32..64);
        }

        Self { table }
    }

    fn limit(&self) -> u16 {
        (LEN * size_of::<u64>() - 1) as u16
    }

    fn pointer(&self) -> DescriptorTablePointer {
        DescriptorTablePointer {
            limit: self.limit(),
            base: self.table.as_ptr() as usize,
        }
    }

    pub fn load(&'static self) {
        unsafe { x86::lgdt(&self.pointer()) }
    }
}

pub fn init() {
    GDT.load();
    unsafe {
        reg::load_cs(SYS_CODE);
        reg::load_ds(SYS_DATA);
        reg::load_es(SYS_DATA);
        reg::load_ss(SYS_DATA);
        reg::load_tss(TSS);
    }
}

#[test_case]
fn test_gdt() {
    assert_eq!(GDT.table.len(), 7);

    assert_eq!(GDT.table[0], 0); // Null descriptor
    assert_eq!(GDT.table[1], 0x00AF_9B00_0000_FFFF); // Kernel code segment
    assert_eq!(GDT.table[2], 0x00CF_9300_0000_FFFF); // Kernel data segment
    assert_eq!(GDT.table[3], 0x00AF_FB00_0000_FFFF); // User code segment
    assert_eq!(GDT.table[4], 0x00CF_F300_0000_FFFF); // User data segment

    // Task state segment (TSS)
    let lo = GDT.table[5];
    let hi = GDT.table[6];
    let base = lo.get_bits(16..40) | (lo.get_bits(56..64) << 24) | (hi << 32);
    assert_eq!(base, tss::TSS.base() as u64);
    assert_eq!(lo.get_bit(47), true); // Present
}
