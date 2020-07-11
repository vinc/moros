use crate::kernel;
use alloc::slice::SliceIndex;
use alloc::vec::Vec;
use core::ops::{Index, IndexMut};
use linked_list_allocator::LockedHeap;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB};
use x86_64::VirtAddr;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_heap(mapper: &mut impl Mapper<Size4KiB>, frame_allocator: &mut impl FrameAllocator<Size4KiB>) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator.allocate_frame().ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush();
        }
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

pub struct PhysBuf {
    vec: Vec<u8>,
}

impl PhysBuf {
    pub fn new(len: usize) -> Self {
        let mut vec: Vec<u8> = Vec::with_capacity(len);
        vec.resize(len, 0);
        Self::from(vec)
    }

    // Realloc vec until it uses a chunk of contiguous physical memory
    fn from(vec: Vec<u8>) -> Self {
        let buffer_len = vec.len() - 1;
        let memory_len = phys_addr(&vec[buffer_len]) - phys_addr(&vec[0]);
        if buffer_len == memory_len as usize {
            Self { vec }
        } else {
            Self::from(vec.clone())
        }
    }

    pub fn addr(&self) -> u64 {
        phys_addr(&self.vec[0])
    }
}

fn phys_addr(ptr: &u8) -> u64 {
    let rx_ptr = ptr as *const u8;
    let virt_addr = VirtAddr::new(rx_ptr as u64);
    let phys_addr = kernel::mem::translate_addr(virt_addr).unwrap();
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
        unsafe { alloc::slice::from_raw_parts(self.vec.as_ptr(), self.vec.len()) }
    }
}

impl core::ops::DerefMut for PhysBuf {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe { alloc::slice::from_raw_parts_mut(self.vec.as_mut_ptr(), self.vec.len()) }
    }
}
