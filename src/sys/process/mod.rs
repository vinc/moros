mod dir;
mod env;
mod id;
mod spawn;
mod table;
mod user;
mod stat;

pub use id::ProcId;
pub use dir::ProcDir;
pub use env::ProcEnv;
pub use user::ProcUser;
pub use stat::ProcStat;
pub use spawn::spawn;
pub use table::{
    init,
    env_var,
    set_user,
    dir,
    alloc, free,
    handle, create_handle, update_handle, delete_handle,
    registers, set_registers,
    interrupt_frame, set_interrupt_frame,
};

use table::{
    PROCESS_TABLE,
    current_process,
    id, set_id,
    set_dir,
    env, set_env_var,
    user,
};

use crate::sys::console::Console;
use crate::sys::fs::{Device, Resource};
use crate::sys::mem;
use crate::sys::mem::with_frame_allocator;
use crate::sys::syscall;
use crate::sys::x86::int::InterruptFrame;
use crate::sys::x86::reg::Cr3;

use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU64, Ordering};
use linked_list_allocator::LockedHeap;
use x86_64::structures::paging::{
    FrameDeallocator, PageTable, PhysFrame,
};

pub const MAX_HANDLES: usize = 64;
pub const MAX_PROC_SIZE: usize = 32 << 20;

// The user memory region lives in its own L4 entry of each process page table.
pub const USER_ADDR: usize = 0x0000_0080_0000_0000;

pub fn is_userspace(addr: usize) -> bool {
    USER_ADDR <= addr && addr < USER_ADDR + MAX_PROC_SIZE
}

pub fn ptr_from_addr(addr: usize) -> *mut u8 {
    addr as *mut u8
}

#[repr(C, align(8))]
#[derive(Debug, Clone, Copy, Default)]
pub struct Registers {
    // Saved scratch registers
    pub r11: usize,
    pub r10: usize,
    pub r9: usize,
    pub r8: usize,
    pub rdi: usize,
    pub rsi: usize,
    pub rdx: usize,
    pub rcx: usize,
    pub rax: usize,
}

#[derive(Clone, Debug)]
struct ProcessData {
    env: BTreeMap<String, String>,
    dir: String,
    user: Option<String>,
    handles: [Option<Box<Resource>>; MAX_HANDLES],
}

impl ProcessData {
    fn new(dir: &str, user: Option<&str>) -> Self {
        let env = BTreeMap::new();
        let dir = dir.to_string();
        let user = user.map(String::from);

        let mut handles = [(); MAX_HANDLES].map(|_| None);
        let stdin = Resource::Device(Device::Console(Console::new()));
        let stdout = Resource::Device(Device::Console(Console::new()));
        let stderr = Resource::Device(Device::Console(Console::new()));
        let stdnull = Resource::Device(Device::Null);
        handles[0] = Some(Box::new(stdin));
        handles[1] = Some(Box::new(stdout));
        handles[2] = Some(Box::new(stderr));
        handles[3] = Some(Box::new(stdnull));

        Self { env, dir, user, handles }
    }
}

#[derive(Clone)]
struct ProcessContext {
    id: usize,
    stack_addr: usize,
    entry_point_addr: usize,
    page_table_frame: PhysFrame,
    allocator: Arc<LockedHeap>,
}

const SYSCALLS: usize = syscall::number::count();

pub struct ProcessStats {
    syscalls_count: [AtomicU64; SYSCALLS],
}

impl ProcessStats {
    fn new() -> Self {
        Self {
            syscalls_count: [(); SYSCALLS].map(|_| AtomicU64::new(0)),
        }
    }

    pub fn syscall_count(&self, number: usize) -> u64 {
        self.syscalls_count[number - 1].load(Ordering::Relaxed)
    }

    pub fn increment_syscall_count(&self, number: usize) {
        self.syscalls_count[number - 1].fetch_add(1, Ordering::SeqCst);
    }
}

pub struct Process {
    parent_id: usize,
    interrupt_frame: Option<InterruptFrame>,
    registers: Registers,
    stats: ProcessStats,
    data: ProcessData,
    ctx: ProcessContext,
}

impl Process {
    fn new() -> Self {
        Self {
            parent_id: 0,
            interrupt_frame: None,
            registers: Registers::default(),
            stats: ProcessStats::new(),
            data: ProcessData::new("/", None),
            ctx: ProcessContext {
                id: 0,
                stack_addr: 0,
                entry_point_addr: 0,
                page_table_frame: Cr3::read().frame(),
                allocator: Arc::new(LockedHeap::empty()),
            }
        }
    }
}

pub fn exit() {
    let proc = {
        let mut table = PROCESS_TABLE.write();
        table[id()].take().unwrap()
    };

    load_process(proc.parent_id);
    free_process(proc.ctx.page_table_frame);
}

fn load_process(id: usize) {
    set_id(id);
    unsafe {
        let addr = page_table_frame().start_address().as_u64() as usize;
        let flags = Cr3::read().flags();
        Cr3::write(addr, flags);
    }
}

fn free_process(page_table_frame: PhysFrame) {
    let page_table = unsafe { mem::create_page_table(page_table_frame) };
    let mut mapper = unsafe { mem::create_mapper(page_table) };
    mem::free_pages(&mut mapper, USER_ADDR, MAX_PROC_SIZE);
    unsafe {
        with_frame_allocator(|allocator| {
            allocator.deallocate_frame(page_table_frame);
        });
    }
}

unsafe fn page_table_frame() -> PhysFrame {
    let table = PROCESS_TABLE.read();
    let proc = current_process(&table);
    proc.ctx.page_table_frame
}

pub unsafe fn page_table() -> &'static mut PageTable {
    mem::create_page_table(page_table_frame())
}

pub fn syscall_count(number: usize) -> u64 {
    let table = PROCESS_TABLE.read();
    let proc = current_process(&table);
    proc.stats.syscall_count(number)
}

pub fn increment_syscall_count(number: usize) {
    let table = PROCESS_TABLE.read();
    let proc = current_process(&table);
    proc.stats.increment_syscall_count(number);
}
