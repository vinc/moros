#[cfg(target_arch = "x86_64")] mod bitmap;
pub mod heap; // TODO: Remove pub when multiboot2 is done
#[cfg(target_arch = "x86_64")] mod paging;
#[cfg(target_arch = "x86_64")] mod phys;

#[cfg(target_arch = "x86_64")]
pub use bitmap::{frame_allocator, with_frame_allocator};

#[cfg(target_arch = "x86_64")]
pub use paging::{
    alloc_pages, free_pages, active_page_table, create_page_table, create_mapper
};

#[cfg(target_arch = "x86_64")]
pub use phys::{phys_addr, PhysBuf};

use crate::sys::boot::MemoryMap;
use crate::sys::pic;

use core::sync::atomic::{AtomicUsize, Ordering};
use spin::Once;

#[cfg(target_arch = "x86_64")]
use x86_64::structures::paging::{OffsetPageTable, Translate};

//use x86_64::{PhysAddr, VirtAddr};
use crate::sys::x86::addr::{PhysAddr, VirtAddr};

#[allow(static_mut_refs)]
#[cfg(target_arch = "x86_64")]
static mut MAPPER: Once<OffsetPageTable<'static>> = Once::new();

static PHYS_MEM_OFFSET: Once<usize> = Once::new();
static MEMORY_SIZE: AtomicUsize = AtomicUsize::new(0);

#[cfg(target_arch = "x86_64")] // TODO: Remove
pub fn init(memory_map: &MemoryMap, offset: u64) {
    // Keep the timer interrupt to have accurate boot time measurement but mask
    // the keyboard interrupt that would create a panic if a key is pressed
    // during memory allocation otherwise.
    pic::mask(pic::KBD_IRQ);

    let mut memory_size = 0;
    let mut last_end_addr = 0;
    for region in memory_map.iter() {
        let start_addr = region.addr;
        let end_addr = region.addr + region.size;
        let hole = start_addr - last_end_addr;
        if hole > 0 {
            log!(
                "MEM [{:#016X}-{:#016X}] {}", // "({} KB)"
                last_end_addr, start_addr - 1, "Unmapped" //, hole >> 10
            );
            if start_addr < (1 << 20) {
                memory_size += hole as usize; // BIOS memory
            }
        }
        log!(
            "MEM [{:#016X}-{:#016X}] {:?}", // "({} KB)"
            start_addr, end_addr - 1, region.kind //, size >> 10
        );
        memory_size += region.size as usize;
        last_end_addr = end_addr;
    }

    // FIXME: There are two small reserved areas at the end of the physical
    // memory that should be removed from the count to be fully accurate but
    // their sizes and location vary depending on the amount of RAM on the
    // system. It doesn't affect the count in megabytes.
    log!("RAM {} MB", memory_size >> 20);

    MEMORY_SIZE.store(memory_size, Ordering::Relaxed);

    #[allow(static_mut_refs)]
    unsafe {
        MAPPER.call_once(|| OffsetPageTable::new(
            paging::active_page_table(),
            VirtAddr::new(offset as usize),
        ))
    };

    PHYS_MEM_OFFSET.call_once(|| offset as usize);
    bitmap::init_frame_allocator(memory_map);
    heap::init_heap().expect("heap initialization failed");

    pic::unmask(pic::KBD_IRQ);
}

pub fn phys_mem_offset() -> usize {
    unsafe { *PHYS_MEM_OFFSET.get_unchecked() }
}

#[cfg(target_arch = "x86_64")] // TODO: Remove
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
    VirtAddr::new(phys_mem_offset() + addr.as_usize())
}

#[cfg(target_arch = "x86_64")] // TODO: Remove
pub fn virt_to_phys(addr: VirtAddr) -> Option<PhysAddr> {
    mapper().translate_addr(addr)
}

