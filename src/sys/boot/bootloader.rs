use super::{MemoryMap, MemoryRegion, MemoryRegionType};

use bootloader::BootInfo;

pub fn extract_memory_map(boot_info: &'static BootInfo) -> MemoryMap {
    use bootloader::bootinfo::MemoryRegionType as Mem;
    let mut memory_map = MemoryMap::new();
    for region in boot_info.memory_map.iter() {
        let addr = region.range.start_addr();
        let size = region.range.end_addr() - addr;
        let kind = match region.region_type {
            Mem::Usable => MemoryRegionType::Usable,
            _ => MemoryRegionType::Reserved,
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
