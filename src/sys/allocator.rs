use crate::sys;

use alloc::slice::SliceIndex;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp;
use core::ops::{Index, IndexMut};
use linked_list_allocator::LockedHeap;
use spin::Mutex;
use x86_64::structures::paging::{
    mapper::MapToError, page::PageRangeInclusive,
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags, Size4KiB,
};
use x86_64::VirtAddr;

#[cfg_attr(not(feature = "userspace"), global_allocator)]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub const HEAP_START: u64 = 0x4444_4444_0000;

fn max_memory() -> u64 {
    // Default to 32 MB
    option_env!("MOROS_MEMORY").unwrap_or("32").parse::<u64>().unwrap() << 20
}

pub fn init_heap() -> Result<(), MapToError<Size4KiB>> {
    let mapper = sys::mem::mapper();
    let mut frame_allocator = sys::mem::frame_allocator();

    // Use half of the memory for the heap caped to 16 MB by default
    // because the allocator is slow.
    let heap_size = cmp::min(sys::mem::memory_size(), max_memory()) / 2;
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
    mapper: &mut OffsetPageTable, addr: u64, size: usize, flags: PageTableFlags
) -> Result<(), ()> {
    let size = size.saturating_sub(1) as u64;
    let mut frame_allocator = sys::mem::frame_allocator();

    let pages = {
        let start_page = Page::containing_address(VirtAddr::new(addr));
        let end_page = Page::containing_address(VirtAddr::new(addr + size));
        Page::range_inclusive(start_page, end_page)
    };

    /*
    let flags = PageTableFlags::PRESENT
              | PageTableFlags::WRITABLE
              | PageTableFlags::USER_ACCESSIBLE;
    */

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

#[derive(Clone)]
pub struct PhysBuf {
    buf: Arc<Mutex<Vec<u8>>>,
}

impl PhysBuf {
    pub fn new(len: usize) -> Self {
        Self::from(vec![0; len])
    }

    // Realloc vec until it uses a chunk of contiguous physical memory
    fn from(vec: Vec<u8>) -> Self {
        let buffer_end = vec.len() - 1;
        let memory_end = phys_addr(&vec[buffer_end]) - phys_addr(&vec[0]);
        if buffer_end == memory_end as usize {
            Self {
                buf: Arc::new(Mutex::new(vec)),
            }
        } else {
            Self::from(vec.clone()) // Clone vec and try again
        }
    }

    pub fn addr(&self) -> u64 {
        phys_addr(&self.buf.lock()[0])
    }
}

pub fn phys_addr(ptr: *const u8) -> u64 {
    let virt_addr = VirtAddr::new(ptr as u64);
    let phys_addr = sys::mem::virt_to_phys(virt_addr).unwrap();
    phys_addr.as_u64()
}

impl<I: SliceIndex<[u8]>> Index<I> for PhysBuf {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&**self, index)
    }
}

impl<I: SliceIndex<[u8]>> IndexMut<I> for PhysBuf {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut **self, index)
    }
}

impl core::ops::Deref for PhysBuf {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        let vec = self.buf.lock();
        unsafe { alloc::slice::from_raw_parts(vec.as_ptr(), vec.len()) }
    }
}

impl core::ops::DerefMut for PhysBuf {
    fn deref_mut(&mut self) -> &mut [u8] {
        let mut vec = self.buf.lock();
        unsafe {
            alloc::slice::from_raw_parts_mut(vec.as_mut_ptr(), vec.len())
        }
    }
}

pub fn memory_size() -> usize {
    ALLOCATOR.lock().size()
}

pub fn memory_used() -> usize {
    ALLOCATOR.lock().used()
}

pub fn memory_free() -> usize {
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
