use super::{MemoryMap, MemoryRegion};
use multiboot2::{BootInformation, BootInformationHeader};

use crate::sys;

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

// Defined in run/boot/multiboot.ld
extern "C" {
    static KERNEL_START: u8;
    static KERNEL_END: u8;
}

fn kernel_start() -> u64 {
    (&raw const KERNEL_START).addr() as u64
}

fn kernel_end() -> u64 {
    (&raw const KERNEL_END).addr() as u64
}

// TODO: Improve protocol support
pub fn extract_memory_map(info: u32, magic: u32) -> MemoryMap {
    if magic != multiboot2::MAGIC {
        panic!("wrong magic");
    }
    let boot_info = unsafe {
        BootInformation::load(info as *const BootInformationHeader).unwrap()
    };

    let mut memory_map = MemoryMap::new();
    if let Some(memory_map_tag) = boot_info.memory_map_tag() {
        use multiboot2::MemoryAreaType as B;
        use super::MemoryRegionType as K;
        let limit = 1 << 32; // 4 GB
        for region in memory_map_tag.memory_areas() {
            let mut addr = region.start_address();
            let mut size = region.size();
            let mut clipped = 0;

            // Skip region above 4 GB
            if addr >= limit {
                let size = region.size();
                let kind = K::Unaddressable;
                memory_map.add(MemoryRegion::new(addr, size, kind));
                continue;
            }

            // Clip region below 4 GB
            if addr + size > limit {
                clipped = (addr + size) - limit;
                size -= clipped;
            }

            let kind = match region.typ().into() {
                B::Available => K::Usable,
                _            => K::Reserved,
            };

            // Reserve the area used by the kernel
            if addr == kernel_start() && kind == K::Usable {
                let kernel_size = kernel_end() - kernel_start();
                debug_assert!(kernel_size < size);
                memory_map.add(MemoryRegion::new(addr, kernel_size, K::Kernel));
                addr += kernel_size;
                size -= kernel_size;
            }

            memory_map.add(MemoryRegion::new(addr, size, kind));

            // Mark the rest of the clipped region above 4 GB
            if clipped > 0 {
                let addr = limit;
                let size = clipped;
                let kind = K::Unaddressable;
                memory_map.add(MemoryRegion::new(addr, size, kind));
            }
        };
    }
    memory_map
}

pub extern "C" fn start(info: u32, magic: u32) -> ! {
    let memory_map = extract_memory_map(info, magic);
    let offset = 0;
    crate::init(&memory_map, offset);
    crate::exec();
}
