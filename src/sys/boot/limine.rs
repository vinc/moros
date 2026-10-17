use super::{MemoryMap, MemoryRegion, MemoryRegionType};

use limine::memmap;
use limine::request::{MemmapRequest, HhdmRequest, FramebufferRequest};
use limine::{BaseRevision, RequestsStartMarker, RequestsEndMarker};

#[used]
#[link_section = ".limine_reqs"]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[used]
#[link_section = ".limine_req_start"]
static REQUESTS_START: RequestsStartMarker = RequestsStartMarker::new();

#[used]
#[link_section = ".limine_reqs"]
static STACK_SIZE_REQUEST: StackSizeRequest = StackSizeRequest::new(
    crate::STACK_SIZE as u64
);

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

// TODO: Improve protocol support
pub extern "C" fn start() -> ! {
    assert!(STACK_SIZE_REQUEST.response().is_some());

    if let Some(res) = FRAMEBUFFER_REQUEST.response() {
        if let Some(fb) = res.framebuffers().first() {
            let ptr = fb.address() as *mut u32;
            let n = 10 * fb.width as usize;
            for i in 0..n {
                unsafe { *ptr.add(i) = 0x00FF00FF; } // Draw pink pixel
            }
        }
    }

    //let hhdm = HHDM_REQUEST.response().unwrap();
    let memmap = MEMMAP_REQUEST.response().unwrap();

    let mut memory_map = MemoryMap::new();
    for entry in memmap.entries() {
        let kind = match entry.type_ {
            memmap::MEMMAP_USABLE => MemoryRegionType::Usable,
            memmap::MEMMAP_EXECUTABLE_AND_MODULES => MemoryRegionType::Kernel,
            memmap::MEMMAP_BOOTLOADER_RECLAIMABLE => MemoryRegionType::Bootloader,
            _ => MemoryRegionType::Reserved,
        };
        memory_map.add(MemoryRegion::new(entry.base, entry.length, kind));
    }

    //crate::init(&memory_map, hhdm.offset);

    if let Some(res) = FRAMEBUFFER_REQUEST.response() {
        if let Some(fb) = res.framebuffers().first() {
            let ptr = fb.address() as *mut u32;
            let n = 10 * fb.width as usize;
            for i in n..(2 * n) {
                unsafe { *ptr.add(i) = 0x0000FFFF; } // Draw cyan pixel
            }
        }
    }

    //crate::exec();
    crate::hang();
}
