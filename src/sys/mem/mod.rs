mod heap;
mod paging;
mod phys;

pub use paging::{alloc_pages, free_pages, active_page_table, create_page_table};
pub use phys::{phys_addr, PhysBuf};

use crate::sys;

use bootloader::bootinfo::{BootInfo, MemoryMap, MemoryRegionType};
use core::sync::atomic::{AtomicUsize, Ordering};
use spin::{Once, Mutex};
use x86_64::structures::paging::{
    FrameAllocator, FrameDeallocator,
    OffsetPageTable, PhysFrame, Size4KiB, Translate,
};
use x86_64::{PhysAddr, VirtAddr};

#[allow(static_mut_refs)]
static mut MAPPER: Once<OffsetPageTable<'static>> = Once::new();

static PHYS_MEM_OFFSET: Once<u64> = Once::new();
static MEMORY_MAP: Once<&MemoryMap> = Once::new();
static MEMORY_SIZE: AtomicUsize = AtomicUsize::new(0);
static FRAME_ALLOCATOR: Once<Mutex<BootInfoFrameAllocator>> = Once::new();

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
    FRAME_ALLOCATOR.call_once(|| {
        Mutex::new(unsafe {
            BootInfoFrameAllocator::init(MEMORY_MAP.get_unchecked())
        })
    });
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

const MAX_FRAMES: usize = (4 << 30) / 4096; // 4 GB of RAM
const BITMAP_SIZE: usize = MAX_FRAMES / 64;

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    allocated_bitmap: [u64; BITMAP_SIZE],
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        Self {
            memory_map,
            allocated_bitmap: [0; BITMAP_SIZE],
        }
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

    fn frame_to_bitmap_index(&self, frame: PhysFrame) -> Option<usize> {
        for (i, usable_frame) in self.usable_frames().enumerate() {
            if usable_frame.start_address() == frame.start_address() {
                return if i < MAX_FRAMES { Some(i) } else { None };
            }
            if i >= MAX_FRAMES {
                break;
            }
        }

        None
    }

    fn is_frame_allocated(&self, index: usize) -> bool {
        if index >= MAX_FRAMES {
            return false;
        }
        let word_index = index / 64;
        let bit_index = index % 64;
        (self.allocated_bitmap[word_index] & (1 << bit_index)) != 0
    }

    fn set_frame_allocated(&mut self, index: usize, allocated: bool) {
        if index >= MAX_FRAMES {
            return;
        }
        let word_index = index / 64;
        let bit_index = index % 64;
        if allocated {
            self.allocated_bitmap[word_index] |= 1 << bit_index;
        } else {
            self.allocated_bitmap[word_index] &= !(1 << bit_index);
        }
    }

    pub fn total_usable_frames(&self) -> usize {
        self.usable_frames().take(MAX_FRAMES).count()
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        for (i, frame) in self.usable_frames().enumerate() {
            if i >= MAX_FRAMES {
                break;
            }
            if !self.is_frame_allocated(i) {
                self.set_frame_allocated(i, true);
                return Some(frame);
            }
        }
        None
    }
}

impl FrameDeallocator<Size4KiB> for BootInfoFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        if let Some(index) = self.frame_to_bitmap_index(frame) {
            if self.is_frame_allocated(index) {
                self.set_frame_allocated(index, false);
            }
        }
    }
}

pub unsafe fn deallocate_frame(frame: PhysFrame) {
    frame_allocator().lock().deallocate_frame(frame);
}

pub fn frame_allocator() -> &'static Mutex<BootInfoFrameAllocator> {
    FRAME_ALLOCATOR.get().expect("frame allocator not initialized")
}

pub fn with_frame_allocator<F, R>(f: F) -> R
where
    F: FnOnce(&mut BootInfoFrameAllocator) -> R,
{
    let mut allocator = frame_allocator().lock();
    f(&mut *allocator)
}
