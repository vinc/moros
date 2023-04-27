use crate::sys;
use bootloader::bootinfo::{BootInfo, MemoryMap, MemoryRegionType};
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use x86_64::instructions::interrupts;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB, Translate};
use x86_64::{PhysAddr, VirtAddr};

// NOTE: mutable but changed only once during initialization
pub static mut PHYS_MEM_OFFSET: u64 = 0;
pub static mut MEMORY_MAP: Option<&MemoryMap> = None;
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

        unsafe { PHYS_MEM_OFFSET = boot_info.physical_memory_offset };
        unsafe { MEMORY_MAP.replace(&boot_info.memory_map) };

        let mut mapper = unsafe { mapper(VirtAddr::new(PHYS_MEM_OFFSET)) };
        let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

        sys::allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");
    });
}

pub fn memory_size() -> u64 {
    MEMORY_SIZE.load(Ordering::Relaxed)
}

pub fn phys_to_virt(addr: PhysAddr) -> VirtAddr {
    VirtAddr::new(addr.as_u64() + unsafe { PHYS_MEM_OFFSET })
}

pub fn virt_to_phys(addr: VirtAddr) -> Option<PhysAddr> {
    let mapper = unsafe { mapper(VirtAddr::new(PHYS_MEM_OFFSET)) };
    mapper.translate_addr(addr)
}

pub unsafe fn mapper(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

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
        //debug!("Allocate frame {} / {}", next, self.usable_frames().count());

        // FIXME: creating an iterator for each allocation is very slow if
        // the heap is larger than a few megabytes.
        self.usable_frames().nth(next)
    }
}
