#![no_std]
#![no_main]

extern crate alloc;

use core::panic::PanicInfo;
use alloc::string::ToString;
use moros::api::console::Style;
use moros::{
    error, warning, hlt_loop, eprint, eprintln, print, println, sys, usr
};

#[cfg(not(feature = "limine"))]
mod bootloader_boot {
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
mod limine_boot {
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
    hlt_loop();
}
