#![no_std]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]

extern crate alloc;

pub mod kernel;
pub mod user;

use bootloader::BootInfo;

pub fn init(boot_info: &'static BootInfo) {
    //kernel::keyboard::init();
    kernel::gdt::init();
    kernel::interrupts::init_idt();
    unsafe { kernel::interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    use x86_64::VirtAddr;
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { kernel::mem::mapper(phys_mem_offset) };
    let mut frame_allocator = unsafe { kernel::mem::BootInfoFrameAllocator::init(&boot_info.memory_map) };
    kernel::allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");
    kernel::cpu::init();
    kernel::pci::init();
    kernel::ata::init();
    kernel::fs::init();
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
