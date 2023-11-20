use crate::api::syscall;

use core::alloc::{GlobalAlloc, Layout};

pub struct UserspaceAllocator;

unsafe impl GlobalAlloc for UserspaceAllocator{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        syscall::alloc(layout.size(), layout.align())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        syscall::free(ptr, layout.size(), layout.align());
    }
}

#[cfg_attr(feature = "userspace", global_allocator)]
static ALLOCATOR: UserspaceAllocator = UserspaceAllocator;
