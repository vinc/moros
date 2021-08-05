use alloc::collections::btree_map::BTreeMap;
use alloc::string::{String, ToString};
use core::sync::atomic::{AtomicUsize, Ordering};
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref PIDS: AtomicUsize = AtomicUsize::new(0);
    pub static ref PROCESS: Mutex<ProcessData> = Mutex::new(ProcessData::new("/", None)); // TODO
}

pub struct ProcessData {
    id: usize,
    env: BTreeMap<String, String>,
    dir: String,
    user: Option<String>,
}

impl ProcessData {
    pub fn new(dir: &str, user: Option<&str>) -> Self {
        let id = PIDS.fetch_add(1, Ordering::SeqCst);
        let env = BTreeMap::new();
        let dir = dir.to_string();
        let user = user.map(String::from);
        Self { id, env, dir, user }
    }
}

pub fn id() -> usize {
    PROCESS.lock().id
}

pub fn env(key: &str) -> Option<String> {
    PROCESS.lock().env.get(key).cloned()
}

pub fn envs() -> BTreeMap<String, String> {
    PROCESS.lock().env.clone()
}

pub fn dir() -> String {
    PROCESS.lock().dir.clone()
}

pub fn user() -> Option<String> {
    PROCESS.lock().user.clone()
}

pub fn set_env(key: &str, val: &str) {
    PROCESS.lock().env.insert(key.into(), val.into());
}

pub fn set_dir(dir: &str) {
    PROCESS.lock().dir = dir.into();
}

pub fn set_user(user: &str) {
    PROCESS.lock().user = Some(user.into())
}


/************************
 * Userspace experiment *
 ************************/

// See https://nfil.dev/kernel/rust/coding/rust-kernel-to-userspace-and-back/
// And https://github.com/WartaPoirier-corp/ananos/blob/dev/docs/notes/context-switch.md

use crate::sys::gdt::GDT;
use core::sync::atomic::AtomicU64;
use x86_64::VirtAddr;
use x86_64::instructions::interrupts;
use x86_64::structures::paging::{Mapper, FrameAllocator, Size4KiB};
use x86_64::structures::paging::{Page, PageTableFlags};

static STACK_ADDR: AtomicU64 = AtomicU64::new(0x600_000);
static CODE_ADDR: AtomicU64 = AtomicU64::new(0x400_000);
const PAGE_SIZE: u64 = 1024 * 4;

pub struct Process {
    stack_addr: u64,
    code_addr: u64,
}

impl Process {
    pub fn create(mapper: &mut impl Mapper<Size4KiB>, frame_alloc: &mut impl FrameAllocator<Size4KiB>, asm: &[u8]) -> Process {
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

        let stack_addr = STACK_ADDR.fetch_add(PAGE_SIZE, Ordering::SeqCst);
        let frame = frame_alloc.allocate_frame().unwrap();
        let page = Page::containing_address(VirtAddr::new(stack_addr));
        unsafe {
            mapper.map_to(page, frame, flags, frame_alloc).unwrap().flush();
        }

        let code_addr = CODE_ADDR.fetch_add(PAGE_SIZE, Ordering::SeqCst);
        let frame = frame_alloc.allocate_frame().unwrap();
        let page = Page::containing_address(VirtAddr::new(code_addr));
        unsafe {
            mapper.map_to(page, frame, flags, frame_alloc).unwrap().flush();
        }

        unsafe {
            let code_addr = code_addr as *mut u8;
            for (i, op) in asm.iter().enumerate() {
                core::ptr::write(code_addr.add(i), *op);
            }
        }

        Process { stack_addr, code_addr }
    }

    pub fn switch(&self) {
        println!("DEBUG: switching to userspace");
        let data = GDT.1.user_data.0;
        let code = GDT.1.user_code.0;
        unsafe {
            interrupts::disable();
            asm!(
                "push rax",
                "push rsi",
                "push 0x200",
                "push rdx",
                "push rdi",
                "iretq",
                in("rax") data,
                in("rsi") self.stack_addr,
                in("rdx") code,
                in("rdi") self.code_addr,
            );
        }
    }
}

