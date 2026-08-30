use super::{MemoryMap, MemoryRegion};

use bootloader::BootInfo;

pub fn extract_memory_map(boot_info: &'static BootInfo) -> MemoryMap {
    use bootloader::bootinfo::MemoryRegionType as B;
    use super::MemoryRegionType as K;
    let mut memory_map = MemoryMap::new();
    for region in boot_info.memory_map.iter() {
        let addr = region.range.start_addr();
        let size = region.range.end_addr() - addr;
        let kind = match region.region_type {
            B::Usable                                 => K::Usable,
            B::Kernel | B::KernelStack | B::PageTable => K::Kernel,
            B::Bootloader | B::BootInfo | B::Package  => K::Bootloader,
            _                                         => K::Reserved,
        };
        memory_map.add(MemoryRegion::new(addr, size, kind));
    }
    memory_map
}

pub fn start(boot_info: &'static BootInfo) -> ! {
    let memory_map = extract_memory_map(boot_info);
    let offset = boot_info.physical_memory_offset;
    crate::init(&memory_map, offset);
    crate::exec();
}
