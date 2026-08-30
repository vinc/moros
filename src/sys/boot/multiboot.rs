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
    unsafe { (&raw const KERNEL_START).addr() as u64 }
}

fn kernel_end() -> u64 {
    unsafe { (&raw const KERNEL_END).addr() as u64 }
}

// TODO: Improve protocol support
pub extern "C" fn start(info: u32, magic: u32) -> ! {
    crate::sys::vga::init();

    printk!("Loading MOROS ...\n");

    if magic == multiboot2::MAGIC {
        let boot_info = unsafe {
            BootInformation::load(info as *const BootInformationHeader).unwrap()
        };

        if let Some(memory_map_tag) = boot_info.memory_map_tag() {
            use multiboot2::MemoryAreaType as B;
            use super::MemoryRegionType as K;
            let mut memory_map = MemoryMap::new();
            let mut heap_start = 0;
            let mut heap_size = 0;
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
                    printk!("MEM [{:#016X}-{:#016X}] {:?}\n", k_addr, k_addr + k_size - 1, k_kind);

                    // Heap (TODO: remove when mem::init() is called directly)
                    let h_addr = k_addr + k_size;
                    let h_size = (size - k_size) / 2;
                    let h_kind = K::Usable;
                    memory_map.add(MemoryRegion::new(h_addr, h_size, h_kind));
                    printk!("MEM [{:#016X}-{:#016X}] {:?}\n", h_addr, h_addr + h_size - 1, h_kind);
                    heap_start = h_addr;
                    heap_size = h_size;

                    // Usable
                    let u_addr = h_addr + h_size;
                    let u_size = size - k_size - h_size;
                    let u_kind = K::Usable;
                    memory_map.add(MemoryRegion::new(u_addr, u_size, u_kind));
                    printk!("MEM [{:#016X}-{:#016X}] {:?}\n", u_addr, u_addr + u_size - 1, u_kind);
                } else {
                    printk!("MEM [{:#016X}-{:#016X}] {:?}\n", addr, addr + size - 1, kind);
                    memory_map.add(MemoryRegion::new(addr, size, kind));
                }
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

    printk!("Loaded MOROS successfully!\n");

    //crate::exec();
    crate::hang();
}
