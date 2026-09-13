mod bitmap;
mod heap;
#[cfg(target_arch = "x86_64")] mod paging;
mod phys;

#[cfg(target_arch = "x86_64")]
pub use bitmap::{frame_allocator, with_frame_allocator};

#[cfg(target_arch = "x86_64")]
pub use paging::{
    alloc_pages, free_pages, active_page_table, create_page_table, create_mapper
};

pub use phys::{phys_addr, PhysBuf};

use crate::sys;
use crate::sys::boot::MemoryMap;
use crate::sys::pic;
use crate::sys::x86::addr::{PhysAddr, VirtAddr};
use crate::sys::x86::page::{PageTable, PageTableEntry, PageTableFlags};
use crate::sys::x86::reg::{Cr0, Cr3, Cr4};

use core::sync::atomic::{AtomicUsize, Ordering};
use spin::{Mutex, Once};

#[cfg(target_arch = "x86_64")]
use x86_64::structures::paging::{OffsetPageTable, Translate};

#[allow(static_mut_refs)]
#[cfg(target_arch = "x86_64")]
static mut MAPPER: Once<OffsetPageTable<'static>> = Once::new();

static PHYS_MEM_OFFSET: Once<usize> = Once::new();
static MEMORY_SIZE: AtomicUsize = AtomicUsize::new(0);
static KERNEL_PAGE_DIRECTORY: Mutex<PageTable> = Mutex::new(PageTable::new());

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
        if hole > 0 && start_addr < (1 << 20) {
            memory_size += hole; // Count BIOS memory
        }
        log!(
            "MEM [{:#016X}-{:#016X}] {:?}", // "({} KB)"
            start_addr, end_addr - 1, region.kind //, size >> 10
        );
        if region.is_addressable() {
            // On i686 the maximum amount of memory addressable is around 3 GB
            // because some of it will be mapped above the 4 GB limit.
            memory_size += region.size;
        }
        last_end_addr = end_addr;
    }

    // FIXME: There are two small reserved areas at the end of the physical
    // memory that should be removed from the count to be fully accurate but
    // their sizes and location vary depending on the amount of RAM on the
    // system. It doesn't affect the count in megabytes.
    log!("RAM {} MB", memory_size >> 20);

    // TODO: Only count usable memory and use SMBIOS to report the RAM
    MEMORY_SIZE.store(memory_size as usize, Ordering::Relaxed);

    PHYS_MEM_OFFSET.call_once(|| offset as usize);

    // TODO: Pick a space in the lowest usable region for DMA

    #[cfg(target_arch = "x86")]
    {
        let mut memory_map = memory_map.clone();

        // Paging is not enabled on i686 for now so we just use half of the
        // largest usable region for the heap.
        let (heap_addr, heap_size) = {
            let region = memory_map.iter_mut().
                filter(|region| region.is_usable()).
                max_by_key(|region| region.size).
                expect("not usable region");

            let size = region.size / 2;
            let addr = region.addr + size;

            region.size = size;

            (addr, size)
        };

        bitmap::init_frame_allocator(&memory_map);
        heap::init_alloc(heap_addr as *mut u8, heap_size as usize);
        init_paging();
    }

    #[cfg(target_arch = "x86_64")] // TODO: Remove
    {
        #[allow(static_mut_refs)]
        unsafe {
            MAPPER.call_once(|| OffsetPageTable::new(
                paging::active_page_table(),
                VirtAddr::new(offset as usize).into(),
            ))
        };

        bitmap::init_frame_allocator(memory_map);
        heap::init_heap().expect("heap initialization failed");
    }

    pic::unmask(pic::KBD_IRQ);
}

// TODO: Move to paging module
// TODO: Init on x86_64 in addition to x86
pub fn init_paging() {
    if !sys::cpu::has_pse() {
        log!("MEM PSE unavailable: paging disabled");
        return;
    }

    let addr = {
        let mut pd = KERNEL_PAGE_DIRECTORY.lock();

        let level = 1;
        let flags = PageTableFlags::PRESENT as usize
                  | PageTableFlags::WRITABLE as usize
                  | PageTableFlags::HUGE as usize; // PSE must be enabled

        for (index, entry) in pd.entries.iter_mut().enumerate() {
            *entry = PageTableEntry::new(level, index, flags);
        }

        phys_addr(pd.entries.as_ptr())
    };

    unsafe {
        // Enable page size extension
        Cr4::write(Cr4::read() | Cr4::PSE);

        // Load page directory
        Cr3::write(addr, 0);

        // Enable paging and write protection
        Cr0::write(Cr0::read() | Cr0::PG | Cr0::WP);
    }
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

#[cfg(target_arch = "x86")]
pub fn virt_to_phys(addr: VirtAddr) -> Option<PhysAddr> {
    // Pagination is not enabled on i686
    Some(PhysAddr::new(addr.as_usize()))
}

#[cfg(target_arch = "x86_64")]
pub fn virt_to_phys(addr: VirtAddr) -> Option<PhysAddr> {
    mapper().translate_addr(addr.into()).map(|x| x.into())
}
