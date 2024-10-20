use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{
    page::PageRangeInclusive,
    OffsetPageTable, PageTable, PhysFrame, Size4KiB,
    Page, PageTableFlags, Mapper, FrameAllocator,
};
use x86_64::VirtAddr;

pub unsafe fn active_page_table() -> &'static mut PageTable {
    let (frame, _) = Cr3::read();
    let phys_addr = frame.start_address();
    let virt_addr = super::phys_to_virt(phys_addr);
    let page_table_ptr: *mut PageTable = virt_addr.as_mut_ptr();
    &mut *page_table_ptr // unsafe
}

pub unsafe fn create_page_table(frame: PhysFrame) -> &'static mut PageTable {
    let phys_addr = frame.start_address();
    let virt_addr = super::phys_to_virt(phys_addr);
    let page_table_ptr: *mut PageTable = virt_addr.as_mut_ptr();
    &mut *page_table_ptr // unsafe
}

pub fn alloc_pages(
    mapper: &mut OffsetPageTable, addr: u64, size: usize
) -> Result<(), ()> {
    let size = size.saturating_sub(1) as u64;
    let mut frame_allocator = super::frame_allocator();

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
