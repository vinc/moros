use bootloader::bootinfo::{BootInfo, MemoryMap, MemoryRegionType};
use crate::{log, kernel};
use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::structures::paging::mapper::MapperAllSizes;
use x86_64::structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

lazy_static! {
    static ref PHYS_MEM_OFFSET: Mutex<VirtAddr> = Mutex::new(VirtAddr::new(0));
}

pub fn init(boot_info: &'static BootInfo) {
    let mut mem_total = 0;
    for region in boot_info.memory_map.iter() {
        let start_addr = region.range.start_addr();
        let end_addr = region.range.end_addr();
        mem_total += end_addr - start_addr;
        log!("MEM [0x{:016X}-0x{:016X}] {:?}\n", start_addr, end_addr, region.region_type);
    }
    log!("MEM {} KB\n", mem_total >> 10);

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { mapper(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    kernel::allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    *PHYS_MEM_OFFSET.lock() = phys_mem_offset;
}

pub fn translate_addr(addr: VirtAddr) -> Option<PhysAddr> {
    let phys_mem_offset = *PHYS_MEM_OFFSET.lock();
    let mapper = unsafe { mapper(phys_mem_offset) };
    mapper.translate_addr(addr)
}

pub unsafe fn mapper(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions.map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
