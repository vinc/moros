use crate::sys::fs::{Resource, Device};
use crate::sys::console::Console;
use alloc::collections::btree_map::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};
use lazy_static::lazy_static;
use object::{Object, ObjectSegment};
use spin::Mutex;

const MAX_FILE_HANDLES: usize = 1024;
const PID: AtomicUsize = AtomicUsize::new(0);

lazy_static! {
    pub static ref PROCESS_TABLE: Mutex<Vec<Process>> = Mutex::new(Vec::new());
}

pub fn init() {
    let data = ProcessData::new("/", None);
    unsafe {
        PROCESS_TABLE.lock().push(Process {
            id: 0,
            stack_size: 256 * PAGE_SIZE,
            stack_addr: 0,
            code_addr: 0,
            entry_point: 0,
            data
        });
    }
}

#[derive(Clone)]
pub struct ProcessData {
    env: BTreeMap<String, String>,
    dir: String,
    user: Option<String>,
    file_handles: Vec<Option<Resource>>,
}

impl ProcessData {
    pub fn new(dir: &str, user: Option<&str>) -> Self {
        let env = BTreeMap::new();
        let dir = dir.to_string();
        let user = user.map(String::from);
        let mut file_handles = vec![None; MAX_FILE_HANDLES];
        file_handles[0] = Some(Resource::Device(Device::Console(Console::new())));
        file_handles[1] = Some(Resource::Device(Device::Console(Console::new())));
        file_handles[2] = Some(Resource::Device(Device::Console(Console::new())));
        Self { env, dir, user, file_handles }
    }
}

pub fn id() -> usize {
    PID.load(Ordering::Relaxed)
}

pub fn set_id(id: usize) {
    PID.store(id, Ordering::Relaxed)
}

pub fn env(key: &str) -> Option<String> {
    let table = PROCESS_TABLE.lock();
    let proc = &table[id()];
    proc.data.env.get(key).cloned()
}

pub fn envs() -> BTreeMap<String, String> {
    let table = PROCESS_TABLE.lock();
    let proc = &table[id()];
    proc.data.env.clone()
}

pub fn dir() -> String {
    let table = PROCESS_TABLE.lock();
    let proc = &table[id()];
    proc.data.dir.clone()
}

pub fn user() -> Option<String> {
    let table = PROCESS_TABLE.lock();
    let proc = &table[id()];
    proc.data.user.clone()
}

pub fn set_env(key: &str, val: &str) {
    let mut table = PROCESS_TABLE.lock();
    let proc = &mut table[id()];
    proc.data.env.insert(key.into(), val.into());
}

pub fn set_dir(dir: &str) {
    let mut table = PROCESS_TABLE.lock();
    let proc = &mut table[id()];
    proc.data.dir = dir.into();
}

pub fn set_user(user: &str) {
    let mut table = PROCESS_TABLE.lock();
    let proc = &mut table[id()];
    proc.data.user = Some(user.into())
}

pub fn create_file_handle(file: Resource) -> Result<usize, ()> {
    let mut table = PROCESS_TABLE.lock();
    let proc = &mut table[id()];

    let min = 4; // The first 4 file handles are reserved
    let max = MAX_FILE_HANDLES;
    for handle in min..max {
        if proc.data.file_handles[handle].is_none() {
            proc.data.file_handles[handle] = Some(file);
            return Ok(handle);
        }
    }
    Err(())
}

pub fn update_file_handle(handle: usize, file: Resource) {
    let mut table = PROCESS_TABLE.lock();
    let proc = &mut table[id()];
    proc.data.file_handles[handle] = Some(file);
}

pub fn delete_file_handle(handle: usize) {
    let mut table = PROCESS_TABLE.lock();
    let proc = &mut table[id()];
    proc.data.file_handles[handle] = None;
}

pub fn file_handle(handle: usize) -> Option<Resource> {
    let table = PROCESS_TABLE.lock();
    let proc = &table[id()];
    proc.data.file_handles[handle].clone()
}

pub fn code_addr() -> u64 {
    let table = PROCESS_TABLE.lock();
    let proc = &table[id()];
    proc.code_addr
}

pub fn set_code_addr(addr: u64) {
    let mut table = PROCESS_TABLE.lock();
    let mut proc = &mut table[id()];
    proc.code_addr = addr;
}

pub fn ptr_from_addr(addr: u64) -> *mut u8 {
    (code_addr() + addr) as *mut u8
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
use x86_64::structures::paging::{Mapper, FrameAllocator};
use x86_64::structures::paging::{Page, PageTableFlags};

static STACK_ADDR: AtomicU64 = AtomicU64::new(0x200_0000);
static CODE_ADDR: AtomicU64 = AtomicU64::new(0x100_0000);
const PAGE_SIZE: u64 = 4 * 1024;

#[derive(Clone)]
pub struct Process {
    id: usize,
    stack_addr: u64,
    stack_size: u64,
    code_addr: u64,
    entry_point: u64,
    data: ProcessData,
}

impl Process {
    pub fn create(bin: &[u8]) -> Result<Process, ()> {
        let mut mapper = unsafe { sys::mem::mapper(VirtAddr::new(sys::mem::PHYS_MEM_OFFSET)) };
        let mut frame_allocator = unsafe { sys::mem::BootInfoFrameAllocator::init(sys::mem::MEMORY_MAP.unwrap()) };


        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

        let stack_size = 256 * PAGE_SIZE;
        let stack_addr = STACK_ADDR.fetch_add(stack_size, Ordering::SeqCst);
        let pages = {
            let stack_start_page = Page::containing_address(VirtAddr::new(stack_addr));
            let stack_end_page = Page::containing_address(VirtAddr::new(stack_addr + stack_size));
            Page::range_inclusive(stack_start_page, stack_end_page)
        };
        for page in pages {
            let frame = frame_allocator.allocate_frame().unwrap();
            unsafe {
                mapper.map_to(page, frame, flags, &mut frame_allocator).unwrap().flush();
            }
        }

        let code_size = 1024 * PAGE_SIZE;
        let code_addr = CODE_ADDR.fetch_add(code_size, Ordering::SeqCst);
        let pages = {
            let code_start_page = Page::containing_address(VirtAddr::new(code_addr));
            let code_end_page = Page::containing_address(VirtAddr::new(code_addr + code_size));
            Page::range_inclusive(code_start_page, code_end_page)
        };
        for page in pages {
            let frame = frame_allocator.allocate_frame().unwrap();
            unsafe {
                mapper.map_to(page, frame, flags, &mut frame_allocator).unwrap().flush();
            }
        }

        let mut entry_point = 0;
        let code_ptr = code_addr as *mut u8;
        if &bin[1..4] == b"ELF" { // ELF binary
            if let Ok(obj) = object::File::parse(bin) {
                entry_point = obj.entry();
                for segment in obj.segments() {
                    let addr = segment.address() as usize;
                    if let Ok(data) = segment.data() {
                        unsafe {
                            for (i, op) in data.iter().enumerate() {
                                let ptr = code_ptr.add(addr + i);
                                core::ptr::write(ptr, *op);
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

        let mut table = PROCESS_TABLE.lock();
        let parent = &table[id()];
        let data = parent.data.clone();
        let id = table.len();
        let proc = Process { id, stack_addr, stack_size, code_addr, entry_point, data };
        table.push(proc.clone());

        Ok(proc)
    }

    // Switch to user mode and execute the program
    pub fn exec(&self) {
        set_id(self.id); // Change PID
        unsafe {
            asm!(
                "cli",        // Disable interrupts
                "push rax",   // Stack segment (SS)
                "push rsi",   // Stack pointer (RSP)
                "push 0x200", // RFLAGS with interrupts enabled
                "push rdx",   // Code segment (CS)
                "push rdi",   // Instruction pointer (RIP)
                "iretq",
                in("rax") GDT.1.user_data.0,
                in("rsi") self.stack_addr + self.stack_size,
                in("rdx") GDT.1.user_code.0,
                in("rdi") self.code_addr + self.entry_point,
            );
        }
    }
}
