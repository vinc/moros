use super::MemoryMap;
use super::MemoryRegion;
use super::MemoryRegionType;
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

#[no_mangle]
pub extern "C" fn start(info: u32, magic: u32) -> ! {
    let vga = 0xB8000 as *mut u8;
    let msg = b"MOROS loading...";
    for (i, &byte) in msg.iter().enumerate() {
        unsafe {
            *vga.add(i * 2) = byte;
            *vga.add(i * 2 + 1) = 0x0F;
        }
    }

    if magic == multiboot2::MAGIC {
        let boot_info = unsafe {
            BootInformation::load(info as *const BootInformationHeader).unwrap()
        };
        if let Some(memory_map_tag) = boot_info.memory_map_tag() {
            use multiboot2::MemoryAreaType as Mem;
            let mut memory_map = MemoryMap::new();
            for region in memory_map_tag.memory_areas() {
                let addr = region.start_address();
                let size = region.size();
                let kind = match region.typ().into() {
                    Mem::Available => MemoryRegionType::Usable,
                    _              => MemoryRegionType::Reserved,
                };
                memory_map.add(MemoryRegion::new(addr, size, kind));
            };
            let offset = 0;
            //crate::init(&memory_map, offset);
        }
    }

    let vga = 0xB8000 as *mut u8;
    let msg = b"MOROS loaded successfully!";
    for (i, &byte) in msg.iter().enumerate() {
        unsafe {
            *vga.add(i * 2) = byte;
            *vga.add(i * 2 + 1) = 0x0F;
        }
    }

    //crate::exec();
    crate::hang();
}
