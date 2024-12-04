mod heap;
mod paging;
mod phys;

pub use paging::{alloc_pages, free_pages, active_page_table, create_page_table};
pub use phys::{phys_addr, PhysBuf};

use crate::sys;
use bootloader::bootinfo::{BootInfo, MemoryMap, MemoryRegionType};
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::Once;
use x86_64::structures::paging::{
    FrameAllocator, OffsetPageTable, PhysFrame, Size4KiB, Translate,
};
use x86_64::{PhysAddr, VirtAddr};

#[allow(static_mut_refs)]
static mut MAPPER: Once<OffsetPageTable<'static>> = Once::new();

static PHYS_MEM_OFFSET: Once<u64> = Once::new();
static MEMORY_MAP: Once<&MemoryMap> = Once::new();
static MEMORY_SIZE: AtomicUsize = AtomicUsize::new(0);
static ALLOCATED_FRAMES: AtomicUsize = AtomicUsize::new(0);

pub fn init(boot_info: &'static BootInfo) {
    // Keep the timer interrupt to have accurate boot time measurement but mask
    // the keyboard interrupt that would create a panic if a key is pressed
    // during memory allocation otherwise.
    sys::idt::set_irq_mask(1);

    let mut memory_size = 0;
    let mut last_end_addr = 0;
    for region in boot_info.memory_map.iter() {
        let start_addr = region.range.start_addr();
        let end_addr = region.range.end_addr();
        let size = end_addr - start_addr;
        let hole = start_addr - last_end_addr;
        if hole > 0 {
            log!(
                "MEM [{:#016X}-{:#016X}] {}", // "({} KB)"
                last_end_addr, start_addr - 1, "Unmapped" //, hole >> 10
            );
            if start_addr < (1 << 20) {
                memory_size += hole; // BIOS memory
            }
        }
        log!(
            "MEM [{:#016X}-{:#016X}] {:?}", // "({} KB)"
            start_addr, end_addr - 1, region.region_type //, size >> 10
        );
        memory_size += size;
        last_end_addr = end_addr;
    }

    // FIXME: There are two small reserved areas at the end of the physical
    // memory that should be removed from the count to be fully accurate but
    // their sizes and location vary depending on the amount of RAM on the
    // system. It doesn't affect the count in megabytes.
    log!("RAM {} MB", memory_size >> 20);
    MEMORY_SIZE.store(memory_size as usize, Ordering::Relaxed);

    #[allow(static_mut_refs)]
    unsafe {
        MAPPER.call_once(|| OffsetPageTable::new(
            paging::active_page_table(),
            VirtAddr::new(boot_info.physical_memory_offset),
        ))
    };

    PHYS_MEM_OFFSET.call_once(|| boot_info.physical_memory_offset);
    MEMORY_MAP.call_once(|| &boot_info.memory_map);

    heap::init_heap().expect("heap initialization failed");

    sys::idt::clear_irq_mask(1);
}

pub fn phys_mem_offset() -> u64 {
    unsafe { *PHYS_MEM_OFFSET.get_unchecked() }
}

pub fn mapper() -> &'static mut OffsetPageTable<'static> {
    #[allow(static_mut_refs)]
    unsafe { MAPPER.get_mut_unchecked() }
}

pub fn memory_size() -> usize {
    MEMORY_SIZE.load(Ordering::Relaxed)
}

pub fn memory_used() -> usize {
    (memory_size() - heap::heap_size()) + heap::heap_used()
}

pub fn memory_free() -> usize {
    heap::heap_free()
}

pub fn phys_to_virt(addr: PhysAddr) -> VirtAddr {
    VirtAddr::new(addr.as_u64() + phys_mem_offset())
}

pub fn virt_to_phys(addr: VirtAddr) -> Option<PhysAddr> {
    mapper().translate_addr(addr)
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator { memory_map }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions.filter(|r|
            r.region_type == MemoryRegionType::Usable
        );
        let addr_ranges = usable_regions.map(|r|
            r.range.start_addr()..r.range.end_addr()
        );
        let frame_addresses = addr_ranges.flat_map(|r|
            r.step_by(4096)
        );
        frame_addresses.map(|addr|
            PhysFrame::containing_address(PhysAddr::new(addr))
        )
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let next = ALLOCATED_FRAMES.fetch_add(1, Ordering::SeqCst);
        // FIXME: When the heap is larger than a few megabytes,
        // creating an iterator for each allocation become very slow.
        self.usable_frames().nth(next)
    }
}

pub fn frame_allocator() -> BootInfoFrameAllocator {
    unsafe { BootInfoFrameAllocator::init(MEMORY_MAP.get_unchecked()) }
}
