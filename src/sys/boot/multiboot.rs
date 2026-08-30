use super::{MemoryMap, MemoryRegion};
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
        for region in memory_map_tag.memory_areas() {
            let addr = region.start_address();
            let size = region.size();
            let kind = match region.typ().into() {
                B::Available => K::Usable,
                _            => K::Reserved,
            };

            if addr == kernel_start() && kind == K::Usable {
                // Kernel
                let k_addr = kernel_start();
                let k_size = kernel_end() - addr;
                let k_kind = K::Kernel;
                memory_map.add(MemoryRegion::new(k_addr, k_size, k_kind));

                // Usable
                let u_addr = k_addr + k_size;
                let u_size = size - k_size;
                let u_kind = K::Usable;
                memory_map.add(MemoryRegion::new(u_addr, u_size, u_kind));
            } else {
                memory_map.add(MemoryRegion::new(addr, size, kind));
            }
        };
    }
    memory_map
}

// TODO: Remove this when init is done on i686
pub fn init(memory_map: &MemoryMap) {
    use crate::sys::boot::MemoryRegionType;
    crate::sys::vga::init();

    printk!("Loading MOROS ...\n");

    //let offset = 0;
    //crate::init(&memory_map, offset);
    crate::sys::gdt::init();
    crate::sys::idt::init();
    crate::sys::pic::init();
    crate::sys::x86::int::enable_interrupts();
    crate::sys::serial::init();

    // TODO: Use sys::mem::init() instead
    let mut heap_addr = 0;
    let mut heap_size = 0;
    for region in memory_map.iter() {
        printk!(
            "MEM [{:#016X}-{:#016X}] {:?}\n",
            region.addr, region.addr + region.size - 1, region.kind
        );
        if region.kind == MemoryRegionType::Usable {
            if region.size > heap_size * 2 {
                heap_addr = region.addr;
                heap_size = region.size / 2;
            }
        }
    }
    crate::sys::mem::heap::init_alloc(
        heap_addr as *mut u8,
        heap_size as usize
    );

    printk!("Loaded MOROS successfully!\n");
}

pub extern "C" fn start(info: u32, magic: u32) -> ! {
    let memory_map = extract_memory_map(info, magic);
    //let offset = 0;
    //crate::init(&memory_map, offset);
    //crate::exec();
    init(&memory_map);
    crate::hang();
}
