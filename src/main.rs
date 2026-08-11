#![no_std]
#![no_main]

extern crate alloc;

use core::panic::PanicInfo;
use alloc::string::ToString;
use moros::api::console::Style;
use moros::{
    error, warning, hang, eprint, eprintln, print, println, sys, usr
};

#[cfg(not(any(feature = "limine", feature = "multiboot")))]
mod bootloader_main {
    use super::*;
    use bootloader::{entry_point, BootInfo};

    entry_point!(main);

    fn main(boot_info: &'static BootInfo) -> ! {
        let memory_map = moros::extract_memory_map(boot_info);
        let offset = boot_info.physical_memory_offset;
        moros::init(&memory_map, offset);
        exec();
    }
}

#[cfg(feature = "limine")]
mod limine_main {
    use super::*;

    use moros::sys::mem::MemoryMap;
    use moros::sys::mem::MemoryRegion;
    use moros::sys::mem::MemoryRegionType;
    use limine::request::{MemmapRequest, HhdmRequest, FramebufferRequest};
    use limine::{BaseRevision, RequestsStartMarker, RequestsEndMarker};
    use limine::memmap;

    #[used]
    #[link_section = ".limine_reqs"]
    static BASE_REVISION: BaseRevision = BaseRevision::new();

    #[used]
    #[link_section = ".limine_req_start"]
    static REQUESTS_START: RequestsStartMarker = RequestsStartMarker::new();

    #[used]
    #[link_section = ".limine_reqs"]
    static MEMMAP_REQUEST: MemmapRequest = MemmapRequest::new();

    #[used]
    #[link_section = ".limine_reqs"]
    static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

    #[used]
    #[link_section = ".limine_reqs"]
    static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

    #[used]
    #[link_section = ".limine_req_end"]
    static REQUESTS_END: RequestsEndMarker = RequestsEndMarker::new();

    #[no_mangle]
    extern "C" fn _start() -> ! {
        if let Some(res) = FRAMEBUFFER_REQUEST.response() {
            if let Some(fb) = res.framebuffers().first() {
                let ptr = fb.address() as *mut u32;
                let n = 10 * fb.width as usize;
                for i in 0..n {
                    unsafe { *ptr.add(i) = 0x00FF00FF; } // Draw pink pixel
                }
            }
        }

        let hhdm = HHDM_REQUEST.response().unwrap();
        let memmap = MEMMAP_REQUEST.response().unwrap();

        let mut memory_map = MemoryMap::new();
        for entry in memmap.entries() {
            let kind = match entry.type_ {
                memmap::MEMMAP_USABLE => MemoryRegionType::Usable,
                _ => MemoryRegionType::Reserved,
            };
            memory_map.add(MemoryRegion::new(entry.base, entry.length, kind));
        }

        moros::init(&memory_map, hhdm.offset);

        if let Some(res) = FRAMEBUFFER_REQUEST.response() {
            if let Some(fb) = res.framebuffers().first() {
                let ptr = fb.address() as *mut u32;
                let n = 10 * fb.width as usize;
                for i in n..(2 * n) {
                    unsafe { *ptr.add(i) = 0x0000FFFF; } // Draw cyan pixel
                }
            }
        }

        hlt_loop();
        exec();
    }
}

#[cfg(feature = "multiboot")]
mod multiboot_main {
    use super::*;

    use moros::sys::mem::MemoryMap;
    use moros::sys::mem::MemoryRegion;
    use moros::sys::mem::MemoryRegionType;
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

    core::arch::global_asm!(
        ".section .text",
        ".global _start",
        "_start:",
        "mov edi, ebx",
        "mov esi, eax",
        "call main",
        "hlt",
    );

    #[no_mangle]
    pub extern "C" fn main(mb2_info: u32, mb2_magic: u32) -> ! {
        let vga = 0xB8000 as *mut u8;
        let msg = b"MOROS loading...";
        for (i, &byte) in msg.iter().enumerate() {
            unsafe {
                *vga.add(i * 2) = byte;
                *vga.add(i * 2 + 1) = 0x0F;
            }
        }

        if mb2_magic == multiboot2::MAGIC {
            let boot_info = unsafe {
                BootInformation::load(mb2_info as *const BootInformationHeader).unwrap()
            };
            if let Some(memory_map_tag) = boot_info.memory_map_tag() {
                // FIXME: This is never reached
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
                moros::init(&memory_map, offset);
            }
        }

        exec();
    }
}

pub fn exec() -> ! {
    print!("\x1b[?25h"); // Enable cursor
    loop {
        if let Some(cmd) = option_env!("MOROS_CMD") {
            let prompt = usr::shell::prompt_string(true);
            println!("{}{}", prompt, cmd);
            usr::shell::exec(cmd).ok();
            sys::acpi::shutdown();
        } else {
            let script = "/ini/boot.sh";
            if sys::fs::File::open(script).is_some() {
                usr::shell::main(&["shell", script]).ok();
            } else {
                if sys::fs::is_mounted() {
                    error!("Could not find '{}'", script);
                } else {
                    warning!("MFS not found, run 'install' to setup the system");
                }
                usr::shell::main(&["shell"]).ok();
            }
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        let title = "Panicked";
        let path = location.file();
        let row = location.line();
        let col = location.column();
        error!("{title} at {path}:{row}:{col}");

        let msg = info.message().to_string();
        if !msg.is_empty() {
            let red = Style::color("red");
            let reset = Style::reset();
            let space = " ".repeat("Error: ".len());
            let arrow = "^".repeat(title.len());
            eprintln!("{space}{red}{arrow} {msg}{reset}");
        }
    } else {
        error!("{info}");
    }
    hang();
}
