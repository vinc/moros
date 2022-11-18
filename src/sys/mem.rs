use crate::sys;
use bootloader::bootinfo::{BootInfo, MemoryMap, MemoryRegionType};
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use x86_64::instructions::interrupts;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB, Translate};
use x86_64::{PhysAddr, VirtAddr};

pub static mut PHYS_MEM_OFFSET: Option<u64> = None;
pub static mut MEMORY_MAP: Option<&MemoryMap> = None;
pub static mut MAPPER: Option<OffsetPageTable<'static>> = None;

pub static MEMORY_SIZE: AtomicU64 = AtomicU64::new(0);

static ALLOCATED_FRAMES: AtomicUsize = AtomicUsize::new(0);

pub fn init(boot_info: &'static BootInfo) {
    interrupts::without_interrupts(|| {
        let mut memory_size = 0;
        for region in boot_info.memory_map.iter() {
            let start_addr = region.range.start_addr();
            let end_addr = region.range.end_addr();
            memory_size += end_addr - start_addr;
            log!("MEM [{:#016X}-{:#016X}] {:?}\n", start_addr, end_addr - 1, region.region_type);
        }
        log!("MEM {} KB\n", memory_size >> 10);
        MEMORY_SIZE.store(memory_size, Ordering::Relaxed);

        let phys_mem_offset = boot_info.physical_memory_offset;

        unsafe { PHYS_MEM_OFFSET.replace(phys_mem_offset) };
        unsafe { MEMORY_MAP.replace(&boot_info.memory_map) };
        unsafe { MAPPER.replace(OffsetPageTable::new(active_page_table(), VirtAddr::new(phys_mem_offset))) };

        let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

        sys::allocator::init_heap(&mut frame_allocator).expect("heap initialization failed");
    });
}

pub fn memory_size() -> u64 {
    MEMORY_SIZE.load(Ordering::Relaxed)
}

pub fn phys_to_virt(addr: PhysAddr) -> VirtAddr {
    let phys_mem_offset = unsafe { PHYS_MEM_OFFSET.unwrap() };
    VirtAddr::new(addr.as_u64() + phys_mem_offset)
}

pub fn virt_to_phys(addr: VirtAddr) -> Option<PhysAddr> {
    let mapper = unsafe { MAPPER.as_mut().unwrap() };
    mapper.translate_addr(addr)
}

unsafe fn active_page_table() -> &'static mut PageTable {
    let (frame, _) = Cr3::read();
    let phys_addr = frame.start_address();
    let virt_addr = phys_to_virt(phys_addr);
    let page_table_ptr: *mut PageTable = virt_addr.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator { memory_map }
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
        let next = ALLOCATED_FRAMES.fetch_add(1, Ordering::SeqCst);

        // FIXME: creating an iterator for each allocation is very slow if
        // the heap is larger than a few megabytes.
        self.usable_frames().nth(next)
    }
}
