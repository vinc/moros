use super::{MemoryMap, MemoryRegion, MemoryRegionType};

use multiboot2::{BootInformation, BootInformationHeader};

#[used]
#[link_section = ".multiboot"]
static MULTIBOOT_HEADER: [u32; 6] = [
    0xE85250D6,  // magic
    0,           // architecture: i386
    24,          // header length (6 * 4 bytes)
    0u32.wrapping_sub(0xE85250D6u32.wrapping_add(24)), // checksum
    0,           // end tag type
    8,           // end tag size
];

// TODO: Improve protocol support
pub extern "C" fn start(info: u32, magic: u32) -> ! {
    crate::sys::vga::init();

    printk!("MOROS loading...\n");

    if magic == multiboot2::MAGIC {
        let boot_info = unsafe {
            BootInformation::load(info as *const BootInformationHeader).unwrap()
        };
        if let Some(memory_map_tag) = boot_info.memory_map_tag() {
            use multiboot2::MemoryAreaType as Mem;
            let mut memory_map = MemoryMap::new();
            let mut heap_start = 0;
            let mut heap_size = 0;
            for region in memory_map_tag.memory_areas() {
                let addr = region.start_address();
                let size = region.size();
                let kind = match region.typ().into() {
                    Mem::Available => MemoryRegionType::Usable,
                    _              => MemoryRegionType::Reserved,
                };
                if size > heap_size && kind == MemoryRegionType::Usable {
                    heap_start = addr;
                    heap_size = size;
                }
                memory_map.add(MemoryRegion::new(addr, size, kind));
            };
            //let offset = 0;
            //crate::init(&memory_map, offset);
            crate::sys::gdt::init();
            crate::sys::idt::init();
            crate::sys::pic::init();
            crate::sys::x86::int::enable_interrupts();
            crate::sys::serial::init();

            crate::sys::mem::heap::init_alloc(
                heap_start as *mut u8,
                heap_size as usize
            );
        }
    }

    printk!("MOROS loaded successfully!\n");

    //crate::exec();
    crate::hang();
}
