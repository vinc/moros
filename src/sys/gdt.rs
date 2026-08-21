use crate::sys::tss;
use crate::sys::tss::TaskStateSegment;
use crate::sys::x86;
use crate::sys::x86::DescriptorTablePointer;
use crate::sys::x86::reg;
use crate::sys::x86::seg::SegmentSelector;

use bit_field::BitField;
use lazy_static::lazy_static;
use x86_64::structures::gdt::Descriptor;

lazy_static! {
    static ref GDT: GlobalDescriptorTable = GlobalDescriptorTable::new(&tss::TSS);
}

pub const SYS_CODE: SegmentSelector = SegmentSelector::new(1, 0);
pub const SYS_DATA: SegmentSelector = SegmentSelector::new(2, 0);
pub const USR_CODE: SegmentSelector = SegmentSelector::new(3, 3);
pub const USR_DATA: SegmentSelector = SegmentSelector::new(4, 3);
pub const TSS:      SegmentSelector = SegmentSelector::new(5, 0);

const fn desc(desc: Descriptor) -> u64 {
    match desc {
        Descriptor::UserSegment(bits) => bits,
        Descriptor::SystemSegment(..) => panic!("not a segment descriptor"),
    }
}

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
        table[SYS_CODE.index()] = desc(Descriptor::kernel_code_segment());
        table[SYS_DATA.index()] = desc(Descriptor::kernel_data_segment());
        table[USR_CODE.index()] = desc(Descriptor::user_code_segment());
        table[USR_DATA.index()] = desc(Descriptor::user_data_segment());

        let base = tss as *const _ as u64;
        let mut low = 1 << 47;
        low.set_bits(0..16, size_of::<TaskStateSegment>() as u64 - 1);
        low.set_bits(16..40, base.get_bits(0..24));
        low.set_bits(40..44, 0b1001);
        low.set_bits(56..64, base.get_bits(24..32));

        table[TSS.index()] = low;

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
