use crate::sys::boot::MemoryMap;
use crate::sys::x86::addr::PhysAddr;

use spin::Once;

const DMA_LIMIT: u64 = 16 << 20; // 16 MB (24-bit addr)
const DMA_SIZE: u64 = 64 << 10;  // 64 KB (one DMA page)

static DMA_ADDR: Once<usize> = Once::new();

pub fn reserve(memory_map: &mut MemoryMap) {
    for region in memory_map.iter() {
        if !region.is_usable() {
            continue;
        }

        let addr = region.addr.next_multiple_of(DMA_SIZE);

        // The DMA block must fit in the region
        if addr + DMA_SIZE > region.addr + region.size {
            continue;
        }

        // The DMA block must fit below 16 MB
        if addr + DMA_SIZE > DMA_LIMIT {
            continue;
        }

        memory_map.reserve(addr, DMA_SIZE);
        DMA_ADDR.call_once(|| addr as usize);
        break;
    }
}

fn dma_addr() -> usize {
    *DMA_ADDR.get().expect("DMA not reserved")
}

// TODO: Add a bump allocator to call this function more than once
pub fn buffer(size: usize) -> &'static mut [u8] {
    debug_assert!(size <= DMA_SIZE as usize);
    let addr = dma_addr();
    let ptr = super::phys_to_virt(PhysAddr::new(addr)).as_mut_ptr();
    unsafe { core::slice::from_raw_parts_mut(ptr, size) }
}
