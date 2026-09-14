use crate::sys::x86::reg::Cr3;

use x86_64::structures::paging::{PageTable, PhysFrame};

#[cfg(target_arch = "x86_64")]
pub unsafe fn active_page_table() -> &'static mut PageTable {
    let frame = Cr3::read().frame();
    let phys_addr = frame.start_address();
    let virt_addr = super::phys_to_virt(phys_addr);
    let page_table_ptr: *mut PageTable = virt_addr.as_mut_ptr();
    &mut *page_table_ptr // unsafe
}

#[cfg(target_arch = "x86_64")]
pub unsafe fn create_page_table(frame: PhysFrame) -> &'static mut PageTable {
    let phys_addr = frame.start_address();
    let virt_addr = super::phys_to_virt(phys_addr.into());
    let page_table_ptr: *mut PageTable = virt_addr.as_mut_ptr();
    &mut *page_table_ptr // unsafe
}
