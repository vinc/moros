mod dir;
mod env;
mod id;
#[cfg(target_arch = "x86_64")] mod spawn;
mod table;
mod user;
mod stat;

pub use id::ProcId;
pub use dir::ProcDir;
pub use env::ProcEnv;
pub use user::ProcUser;
pub use stat::ProcStat;

#[cfg(target_arch = "x86_64")]
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

#[cfg(target_arch = "x86_64")]
use crate::sys::mem::with_frame_allocator;

use crate::sys::syscall;
use crate::sys::x86::int::InterruptFrame;
use crate::sys::x86::reg::Cr3;

use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use core::ops::{Index, IndexMut};
use core::sync::atomic::{AtomicU64, Ordering};
use linked_list_allocator::LockedHeap;
use x86_64::structures::paging::{
    FrameDeallocator, PageTable, PhysFrame,
};

pub const MAX_HANDLES: usize = 64;
pub const MAX_PROC_SIZE: usize = 32 << 20;

// The user memory region lives in its own L4 entry of each process page table.
#[cfg(target_arch = "x86_64")]
pub const USER_ADDR: usize = 0x0000_0080_0000_0000;

#[cfg(target_arch = "x86_64")]
pub fn is_userspace(addr: usize) -> bool {
    USER_ADDR <= addr && addr < USER_ADDR + MAX_PROC_SIZE
}

pub fn ptr_from_addr(addr: usize) -> *mut u8 {
    addr as *mut u8
}

#[cfg(target_arch = "x86")]
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Registers {
    // Linux i386 convention (except esi reserved by LLVM)
    pub eax: usize,
    pub ebx: usize,
    pub ecx: usize,
    pub edx: usize,
    pub edi: usize,
}

#[cfg(target_arch = "x86_64")]
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Registers {
    // System V AMD64 ABI convention
    pub rax: usize,
    pub rdi: usize,
    pub rsi: usize,
    pub rdx: usize,
    pub rcx: usize,
    pub r8: usize,
    pub r9: usize,
    pub r10: usize,
    pub r11: usize,
}

impl Registers {
    #[inline]
    fn as_slice(&self) -> &[usize] {
        let len = core::mem::size_of::<Self>() / core::mem::size_of::<usize>();
        unsafe {
            core::slice::from_raw_parts(self as *const _ as *const usize, len)
        }
    }

    #[inline]
    fn as_mut_slice(&mut self) -> &mut [usize] {
        let len = core::mem::size_of::<Self>() / core::mem::size_of::<usize>();
        unsafe {
            core::slice::from_raw_parts_mut(self as *mut _ as *mut usize, len)
        }
    }
}

impl Index<usize> for Registers {
    type Output = usize;

    #[inline]
    fn index(&self, i: usize) -> &usize {
        &self.as_slice()[i]
    }
}

impl IndexMut<usize> for Registers {
    #[inline]
    fn index_mut(&mut self, i: usize) -> &mut usize {
        &mut self.as_mut_slice()[i]
    }
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

    #[cfg(target_arch = "x86_64")]
    unsafe {
        let addr = page_table_frame().start_address().as_u64() as usize;
        let flags = Cr3::read().flags();
        Cr3::write(addr, flags);
    }
}

fn free_process(page_table_frame: PhysFrame) {
    #[cfg(target_arch = "x86_64")]
    {
        let page_table = unsafe { mem::create_page_table(page_table_frame) };
        let mut mapper = unsafe { mem::create_mapper(page_table) };
        mem::free_pages(&mut mapper, USER_ADDR, MAX_PROC_SIZE);
        unsafe {
            with_frame_allocator(|allocator| {
                allocator.deallocate_frame(page_table_frame);
            });
        }
    }
}

#[cfg(target_arch = "x86_64")]
unsafe fn page_table_frame() -> PhysFrame {
    let table = PROCESS_TABLE.read();
    let proc = current_process(&table);
    proc.ctx.page_table_frame
}

#[cfg(target_arch = "x86_64")]
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

#[cfg(target_arch = "x86")]
#[test_case]
fn test_registers() {
    let mut regs = Registers::default();
    regs.eax = 1;
    regs.ebx = 2;
    regs.ecx = 3;
    regs.edx = 4;
    regs.edi = 5;
    assert_eq!([regs[0], regs[1], regs[2], regs[3], regs[4]], [1, 2, 3, 4, 5]);

    regs[0] = 9;
    assert_eq!(regs.eax, 9);
}

#[cfg(target_arch = "x86_64")]
#[test_case]
fn test_registers() {
    let mut regs = Registers::default();
    regs.rax = 1;
    regs.rdi = 2;
    regs.rsi = 3;
    regs.rdx = 4;
    regs.rcx = 5;
    assert_eq!([regs[0], regs[1], regs[2], regs[3], regs[4]], [1, 2, 3, 4, 5]);

    regs[0] = 9;
    assert_eq!(regs.rax, 9);
}
