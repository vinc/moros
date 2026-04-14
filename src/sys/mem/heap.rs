use super::with_frame_allocator;

use crate::sys;

use linked_list_allocator::LockedHeap;
use x86_64::structures::paging::{
    mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB
};
use x86_64::VirtAddr;

#[cfg_attr(not(feature = "userspace"), global_allocator)]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub const HEAP_START: u64 = 0x4444_4444_0000;

pub fn init_heap() -> Result<(), MapToError<Size4KiB>> {
    let mapper = super::mapper();

    // Use half of the memory for the heap
    let heap_size = (super::memory_size() / 2) as u64;
    let heap_start = VirtAddr::new(HEAP_START);

    // And some memory after that for the processes
    sys::process::set_process_addr(HEAP_START + heap_size);

    let pages = {
        let heap_end = heap_start + heap_size - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

    with_frame_allocator(|frame_allocator| -> Result<(), MapToError<Size4KiB>> {
        for page in pages {
            let err = MapToError::FrameAllocationFailed;
            let frame = frame_allocator.allocate_frame().ok_or(err)?;
            unsafe {
                mapper.map_to(page, frame, flags, frame_allocator)?.flush();
            }
        }
        Ok(())
    })?;

    unsafe {
        ALLOCATOR.lock().init(heap_start.as_mut_ptr(), heap_size as usize);
    }

    Ok(())
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
    use alloc::vec::Vec;

    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}
