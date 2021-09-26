use crate::sys::fs::{Resource, Device};
use crate::sys::console::Console;
use alloc::collections::btree_map::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use lazy_static::lazy_static;
use object::{Object, ObjectSection};
use spin::Mutex;

const MAX_FILE_HANDLES: usize = 1024;

lazy_static! {
    pub static ref PIDS: AtomicUsize = AtomicUsize::new(0);
    pub static ref PROCESS: Mutex<ProcessData> = Mutex::new(ProcessData::new("/", None)); // TODO
}

pub struct ProcessData {
    id: usize,
    env: BTreeMap<String, String>,
    dir: String,
    user: Option<String>,
    file_handles: Vec<Option<Resource>>,
    code_addr: u64,
}

impl ProcessData {
    pub fn new(dir: &str, user: Option<&str>) -> Self {
        let id = PIDS.fetch_add(1, Ordering::SeqCst);
        let env = BTreeMap::new();
        let dir = dir.to_string();
        let user = user.map(String::from);
        let code_addr = 0;
        let mut file_handles = vec![None; MAX_FILE_HANDLES];
        file_handles[0] = Some(Resource::Device(Device::Console(Console::new())));
        file_handles[1] = Some(Resource::Device(Device::Console(Console::new())));
        file_handles[2] = Some(Resource::Device(Device::Console(Console::new())));
        Self { id, env, dir, user, file_handles, code_addr }
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

pub fn create_file_handle(file: Resource) -> Result<usize, ()> {
    let min = 4; // The first 4 file handles are reserved
    let max = MAX_FILE_HANDLES;
    let proc = &mut *PROCESS.lock();
    for handle in min..max {
        if proc.file_handles[handle].is_none() {
            proc.file_handles[handle] = Some(file);
            return Ok(handle);
        }
    }
    Err(())
}

pub fn update_file_handle(handle: usize, file: Resource) {
    let proc = &mut *PROCESS.lock();
    proc.file_handles[handle] = Some(file);
}

pub fn delete_file_handle(handle: usize) {
    let proc = &mut *PROCESS.lock();
    proc.file_handles[handle] = None;
}

pub fn file_handle(handle: usize) -> Option<Resource> {
    let proc = &mut *PROCESS.lock();
    proc.file_handles[handle].clone()
}

pub fn code_addr() -> u64 {
    PROCESS.lock().code_addr
}

pub fn set_code_addr(addr: u64) {
    PROCESS.lock().code_addr = addr;
}

pub fn ptr_from_addr(addr: u64) -> *mut u8 {
    (PROCESS.lock().code_addr + addr) as *mut u8
}

/************************
 * Userspace experiment *
 ************************/

// See https://nfil.dev/kernel/rust/coding/rust-kernel-to-userspace-and-back/
// And https://github.com/WartaPoirier-corp/ananos/blob/dev/docs/notes/context-switch.md

use crate::sys;
use crate::sys::gdt::GDT;
use core::sync::atomic::AtomicU64;
use x86_64::VirtAddr;
use x86_64::instructions::interrupts;
use x86_64::structures::paging::{Mapper, FrameAllocator};
use x86_64::structures::paging::{Page, PageTableFlags};

static CODE_ADDR: AtomicU64 = AtomicU64::new(0x40_0000);
static STACK_ADDR: AtomicU64 = AtomicU64::new(0x80_0000);
const PAGE_SIZE: u64 = 4 * 1024;

pub struct Process {
    stack_addr: u64,
    code_addr: u64,
    entry: u64,
}

impl Process {
    pub fn create(bin: &[u8]) -> Process {
        let mut mapper = unsafe { sys::mem::mapper(VirtAddr::new(sys::mem::PHYS_MEM_OFFSET)) };
        let mut frame_allocator = unsafe { sys::mem::BootInfoFrameAllocator::init(sys::mem::MEMORY_MAP.unwrap()) };

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

        let stack_addr = STACK_ADDR.fetch_add(PAGE_SIZE, Ordering::SeqCst);
        let frame = frame_allocator.allocate_frame().unwrap();
        let page = Page::containing_address(VirtAddr::new(stack_addr));
        unsafe {
            mapper.map_to(page, frame, flags, &mut frame_allocator).unwrap().flush();
        }

        let code_addr = CODE_ADDR.fetch_add(PAGE_SIZE, Ordering::SeqCst);
        let frame = frame_allocator.allocate_frame().unwrap();
        for i in 0..1024 {
            let addr = code_addr + i * PAGE_SIZE;
            let page = Page::containing_address(VirtAddr::new(addr));
            unsafe {
                mapper.map_to(page, frame, flags, &mut frame_allocator).unwrap().flush();
            }
        }

        let mut entry = 0;
        let code_ptr = code_addr as *mut u8;
        if &bin[1..4] == b"ELF" { // ELF binary
            if let Ok(obj) = object::File::parse(bin) {
                entry = obj.entry();
                for section in obj.sections() {
                    if let Ok(name) = section.name() {
                        let addr = section.address() as usize;
                        if name.is_empty() || addr == 0 {
                            continue;
                        }
                        if let Ok(data) = section.data() {
                            unsafe {
                                for (i, op) in data.iter().enumerate() {
                                    let ptr = code_ptr.add(addr + i);
                                    core::ptr::write(ptr, *op);
                                }
                            }
                        }
                    }
                }
            }
        } else { // Raw binary
            unsafe {
                for (i, op) in bin.iter().enumerate() {
                    let ptr = code_ptr.add(i);
                    core::ptr::write(ptr, *op);
                }
            }
        }

        set_code_addr(code_addr);

        Process { stack_addr, code_addr, entry }
    }

    // Switch to user mode
    pub fn switch(&self) {
        //x86_64::instructions::tlb::flush_all();
        let data = GDT.1.user_data.0;
        let code = GDT.1.user_code.0;
        unsafe {
            interrupts::disable();
            asm!(
                "push rax",   // Stack segment (SS)
                "push rsi",   // Stack pointer (RSP)
                "push 0x200", // RFLAGS with interrupt
                "push rdx",   // Code segment (CS)
                "push rdi",   // Instruction pointer (RIP)
                "iretq",
                in("rax") data,
                in("rsi") self.stack_addr,
                in("rdx") code,
                in("rdi") self.code_addr + self.entry,
            );
        }
    }
}
