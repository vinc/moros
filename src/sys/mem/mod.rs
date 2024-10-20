mod heap;
mod paging;
mod phys;

pub use paging::{alloc_pages, free_pages, active_page_table, create_page_table};
pub use phys::{phys_addr, PhysBuf};

use crate::sys;
use bootloader::bootinfo::{BootInfo, MemoryMap, MemoryRegionType};
use core::sync::atomic::{AtomicUsize, Ordering};
//use x86_64::instructions::interrupts;
use x86_64::structures::paging::{
    FrameAllocator, OffsetPageTable, PhysFrame, Size4KiB, Translate,
};
use x86_64::{PhysAddr, VirtAddr};

pub static mut PHYS_MEM_OFFSET: Option<u64> = None;
static mut MEMORY_MAP: Option<&MemoryMap> = None;
static mut MAPPER: Option<OffsetPageTable<'static>> = None;
static MEMORY_SIZE: AtomicUsize = AtomicUsize::new(0);
static ALLOCATED_FRAMES: AtomicUsize = AtomicUsize::new(0);

pub fn init(boot_info: &'static BootInfo) {
    sys::idt::set_irq_mask(1); // Mask keyboard interrupt
    //interrupts::without_interrupts(|| {
        let mut memory_size = 0;
        let mut last_end_addr = 0;
        for region in boot_info.memory_map.iter() {
            let start_addr = region.range.start_addr();
            let end_addr = region.range.end_addr();
            let size = end_addr - start_addr;
            let hole = start_addr - last_end_addr;
            if hole > 0 {
                log!(
                    //"MEM [{:#016X}-{:#016X}] {} ({} KB)",
                    "MEM [{:#016X}-{:#016X}] {}",
                    last_end_addr, start_addr - 1, "Unmapped" //, hole >> 10
                );
            }
            log!(
                //"MEM [{:#016X}-{:#016X}] {:?} ({} KB)",
                "MEM [{:#016X}-{:#016X}] {:?}",
                start_addr, end_addr - 1, region.region_type //, size >> 10
            );
            memory_size += size;
            last_end_addr = end_addr;
        }

        // 0x000000000A0000-0x000000000EFFFF: + 320 KB of BIOS memory
        // 0x000000FEFFC000-0x000000FEFFFFFF: - 256 KB of virtual memory
        // 0x000000FFFC0000-0x000000FFFFFFFF: -  16 KB of virtual memory
        memory_size += (320 - 256 - 16) << 10;

        log!("RAM {} MB", memory_size >> 20);
        MEMORY_SIZE.store(memory_size as usize, Ordering::Relaxed);

        let phys_mem_offset = boot_info.physical_memory_offset;

        unsafe { PHYS_MEM_OFFSET.replace(phys_mem_offset) };
        unsafe { MEMORY_MAP.replace(&boot_info.memory_map) };
        unsafe {
            MAPPER.replace(OffsetPageTable::new(
                paging::active_page_table(),
                VirtAddr::new(phys_mem_offset),
            ))
        };

        heap::init_heap().expect("heap initialization failed");
    //});
    sys::idt::clear_irq_mask(1);
}

pub fn mapper() -> &'static mut OffsetPageTable<'static> {
    unsafe { MAPPER.as_mut().unwrap() }
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
    let phys_mem_offset = unsafe {
        PHYS_MEM_OFFSET.unwrap()
    };
    VirtAddr::new(addr.as_u64() + phys_mem_offset)
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
    unsafe { BootInfoFrameAllocator::init(MEMORY_MAP.unwrap()) }
}
