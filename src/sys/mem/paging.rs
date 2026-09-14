use crate::sys::x86::reg::Cr3;

#[cfg(target_arch = "x86_64")]
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

#[cfg(target_arch = "x86")]
use crate::sys::x86::page::{PageTable, PageTableEntry};

#[cfg(target_arch = "x86")]
use spin::Mutex;

#[cfg(target_arch = "x86")]
static KERNEL_PAGE_TABLE: Mutex<PageTable> = Mutex::new(PageTable::new());

#[cfg(target_arch = "x86")]
pub fn init() {
    if !crate::sys::cpu::has_pse() {
        log!("MEM PSE unavailable: paging disabled");
        return;
    }

    let addr = {
        let mut page_table = KERNEL_PAGE_TABLE.lock();

        let level = 2;
        let flags = PageTableEntry::PRESENT
                  | PageTableEntry::WRITABLE
                  | PageTableEntry::LARGE; // PSE must be enabled

        for (index, entry) in page_table.entries.iter_mut().enumerate() {
            *entry = PageTableEntry::new(level, index, flags);
        }

        super::phys_addr(page_table.entries.as_ptr())
    };

    use crate::sys::x86::reg::{Cr0, Cr4};
    unsafe {
        // Enable page size extension
        Cr4::write(Cr4::read() | Cr4::PSE);

        // Load page directory
        Cr3::write(addr, 0);

        // Enable paging and write protection
        Cr0::write(Cr0::read() | Cr0::PG | Cr0::WP);
    }
}

#[test_case]
fn test_control_registers() {
    use crate::sys::x86::reg::{Cr0, Cr3, Cr4};

    #[cfg(target_arch = "x86")]
    {
        assert_eq!(Cr4::read() & Cr4::PSE, Cr4::PSE);
        assert_eq!(Cr4::read() & Cr4::PAE, 0);
    }

    #[cfg(target_arch = "x86_64")]
    {
        assert_eq!(Cr4::read() & Cr4::PSE, 0);
        assert_eq!(Cr4::read() & Cr4::PAE, Cr4::PAE);
    }

    #[cfg(target_arch = "x86")]
    assert_eq!(
        Cr3::read().addr(),
        super::phys_addr(KERNEL_PAGE_TABLE.lock().entries.as_ptr())
    );

    assert_eq!(Cr3::read().flags(), 0);

    assert_eq!(Cr0::read() & Cr0::PG, Cr0::PG);
    assert_eq!(Cr0::read() & Cr0::WP, Cr0::WP);
}

#[cfg(target_arch = "x86")]
#[test_case]
fn test_page_table() {
    let page_table = KERNEL_PAGE_TABLE.lock();

    let flags = PageTableEntry::PRESENT
              | PageTableEntry::WRITABLE
              | PageTableEntry::LARGE;

    let dirty = PageTableEntry::ACCESSED
              | PageTableEntry::DIRTY;

    assert_eq!(page_table.entries[0].0,    0x0000_0000 | flags | dirty);
    assert_eq!(page_table.entries[1].0,    0x0040_0000 | flags);
    assert_eq!(page_table.entries[1023].0, 0xFFC0_0000 | flags);
}
