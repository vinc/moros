use crate::sys;

use core::cmp;
use linked_list_allocator::LockedHeap;
use x86_64::structures::paging::{
    mapper::MapToError, page::PageRangeInclusive,
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags, Size4KiB,
};
use x86_64::VirtAddr;

#[cfg_attr(not(feature = "userspace"), global_allocator)]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub const HEAP_START: u64 = 0x4444_4444_0000;

fn max_memory() -> usize {
    // Default to 32 MB
    option_env!("MOROS_MEMORY").unwrap_or("32").parse::<usize>().unwrap() << 20
}

pub fn init_heap() -> Result<(), MapToError<Size4KiB>> {
    let mapper = super::mapper();
    let mut frame_allocator = super::frame_allocator();

    // Use half of the memory for the heap caped to 16 MB by default
    // because the allocator is slow.
    let heap_size = (cmp::min(super::memory_size(), max_memory()) / 2) as u64;
    let heap_start = VirtAddr::new(HEAP_START);
    sys::process::init_process_addr(HEAP_START + heap_size);

    let pages = {
        let heap_end = heap_start + heap_size - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    for page in pages {
        let err = MapToError::FrameAllocationFailed;
        let frame = frame_allocator.allocate_frame().ok_or(err)?;
        unsafe {
            mapper.map_to(page, frame, flags, &mut frame_allocator)?.flush();
        }
    }

    unsafe {
        ALLOCATOR.lock().init(heap_start.as_mut_ptr(), heap_size as usize);
    }

    Ok(())
}

pub fn alloc_pages(
    mapper: &mut OffsetPageTable, addr: u64, size: usize
) -> Result<(), ()> {
    let size = size.saturating_sub(1) as u64;
    let mut frame_allocator = sys::mem::frame_allocator();

    let pages = {
        let start_page = Page::containing_address(VirtAddr::new(addr));
        let end_page = Page::containing_address(VirtAddr::new(addr + size));
        Page::range_inclusive(start_page, end_page)
    };

    let flags = PageTableFlags::PRESENT
              | PageTableFlags::WRITABLE
              | PageTableFlags::USER_ACCESSIBLE;

    for page in pages {
        if let Some(frame) = frame_allocator.allocate_frame() {
            let res = unsafe {
                mapper.map_to(page, frame, flags, &mut frame_allocator)
            };
            if let Ok(mapping) = res {
                //debug!("Mapped {:?} to {:?}", page, frame);
                mapping.flush();
            } else {
                debug!("Could not map {:?} to {:?}", page, frame);
                if let Ok(old_frame) = mapper.translate_page(page) {
                    debug!("Already mapped to {:?}", old_frame);
                }
                return Err(());
            }
        } else {
            debug!("Could not allocate frame for {:?}", page);
            return Err(());
        }
    }

    Ok(())
}

// TODO: Replace `free` by `dealloc`
pub fn free_pages(mapper: &mut OffsetPageTable, addr: u64, size: usize) {
    let size = size.saturating_sub(1) as u64;

    let pages: PageRangeInclusive<Size4KiB> = {
        let start_page = Page::containing_address(VirtAddr::new(addr));
        let end_page = Page::containing_address(VirtAddr::new(addr + size));
        Page::range_inclusive(start_page, end_page)
    };

    for page in pages {
        if let Ok((_, mapping)) = mapper.unmap(page) {
            mapping.flush();
        } else {
            //debug!("Could not unmap {:?}", page);
        }
    }
}

pub fn heap_size() -> usize {
    ALLOCATOR.lock().size()
}

pub fn heap_used() -> usize {
    ALLOCATOR.lock().used()
}

pub fn heap_free() -> usize {
    ALLOCATOR.lock().free()
}

#[test_case]
fn many_boxes() {
    use alloc::boxed::Box;

    let heap_value_1 = Box::new(42);
    let heap_value_2 = Box::new(1337);
    assert_eq!(*heap_value_1, 42);
    assert_eq!(*heap_value_2, 1337);

    for i in 0..1000 {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}

#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}