use crate::hlt_loop;
use crate::api::process::ExitCode;
use crate::api::syscall;

use core::alloc::{GlobalAlloc, Layout};

pub struct UserspaceAllocator;

unsafe impl GlobalAlloc for UserspaceAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        syscall::alloc(layout.size(), layout.align())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        syscall::free(ptr, layout.size(), layout.align());
    }
}

#[allow(dead_code)]
#[cfg_attr(feature = "userspace", global_allocator)]
static ALLOCATOR: UserspaceAllocator = UserspaceAllocator;

#[allow(dead_code)]
#[cfg_attr(feature = "userspace", alloc_error_handler)]
fn alloc_error_handler(_layout: alloc::alloc::Layout) -> ! {
    syscall::write(2, b"\x1b[91mError:\x1b[m Could not allocate\n");
    syscall::exit(ExitCode::PageFaultError);
    hlt_loop();
}
