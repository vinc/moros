use super::with_frame_allocator;
use super::phys_mem_offset;
use crate::sys::x86::addr::VirtAddr;
use crate::sys::x86::reg::Cr3;

use x86_64::structures::paging::{
    mapper::CleanUp,
    page::PageRangeInclusive,
    OffsetPageTable, PageTable, PhysFrame, Size4KiB,
    Page, PageTableFlags, Mapper, FrameAllocator, FrameDeallocator
};

pub unsafe fn active_page_table() -> &'static mut PageTable {
    let frame = Cr3::read().frame();
    let phys_addr = frame.start_address();
    let virt_addr = super::phys_to_virt(phys_addr.into());
    let page_table_ptr: *mut PageTable = virt_addr.as_mut_ptr();
    &mut *page_table_ptr // unsafe
}

pub unsafe fn create_page_table(frame: PhysFrame) -> &'static mut PageTable {
    let phys_addr = frame.start_address();
    let virt_addr = super::phys_to_virt(phys_addr.into());
    let page_table_ptr: *mut PageTable = virt_addr.as_mut_ptr();
    &mut *page_table_ptr // unsafe
}

pub unsafe fn create_mapper(page_table: &mut PageTable) -> OffsetPageTable<'_> {
    OffsetPageTable::new(page_table, VirtAddr::new(phys_mem_offset()).into())
}

pub fn alloc_pages(
    mapper: &mut OffsetPageTable, addr: usize, size: usize
) -> Result<(), ()> {
    let size = size.saturating_sub(1) as usize;

    let pages = {
        let start_page = Page::containing_address(VirtAddr::new(addr).into());
        let end_page = Page::containing_address(VirtAddr::new(addr + size).into());
        Page::range_inclusive(start_page, end_page)
    };

    let flags = PageTableFlags::PRESENT
              | PageTableFlags::WRITABLE
              | PageTableFlags::USER_ACCESSIBLE;

    with_frame_allocator(|frame_allocator| {
        for page in pages {
            if let Some(frame) = frame_allocator.allocate_frame() {
                let res = unsafe {
                    mapper.map_to(page, frame, flags, frame_allocator)
                };
                if let Ok(mapping) = res {
                    mapping.flush();

                    // Clear the frame
                    let virt = super::phys_to_virt(frame.start_address().into());
                    unsafe {
                        core::ptr::write_bytes(virt.as_mut_ptr::<u8>(), 0, 4096);
                    }
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
    })
}

// TODO: Replace `free` by `dealloc`
pub fn free_pages(mapper: &mut OffsetPageTable, addr: usize, size: usize) {
    let size = size.saturating_sub(1) as usize;

    let pages: PageRangeInclusive<Size4KiB> = {
        let start_page = Page::containing_address(VirtAddr::new(addr).into());
        let end_page = Page::containing_address(VirtAddr::new(addr + size).into());
        Page::range_inclusive(start_page, end_page)
    };

    for page in pages {
        if let Ok((frame, mapping)) = mapper.unmap(page) {
            mapping.flush();
            unsafe {
                with_frame_allocator(|allocator| {
                    allocator.deallocate_frame(frame);
                });
            }
        } else {
            //debug!("Could not unmap {:?}", page);
        }
    };
    unsafe {
        with_frame_allocator(|allocator| {
            mapper.clean_up_addr_range(pages, allocator);
        });
    }
}
