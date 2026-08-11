use bootloader::BootInfo;

pub fn start(boot_info: &'static BootInfo) -> ! {
    let memory_map = crate::extract_memory_map(boot_info);
    let offset = boot_info.physical_memory_offset;
    crate::init(&memory_map, offset);
    crate::exec();
}
