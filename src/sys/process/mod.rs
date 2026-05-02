mod spawn;
mod table;

pub use spawn::spawn;
pub use table::{
    code_addr,
    id, set_id,
    dir, set_dir,
    envs, env, set_env,
    user, set_user,
    alloc, free,
    handle, create_handle, update_handle, delete_handle,
    registers, set_registers,
    stack_frame, set_stack_frame,
};

use table::PROCESS_TABLE;
use table::current_process;

use crate::sys::console::Console;
use crate::sys::fs::{Device, Resource};
use crate::sys::mem;
use crate::sys::mem::{phys_mem_offset, with_frame_allocator, PageTableLevel};

use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use linked_list_allocator::LockedHeap;
use x86_64::registers::control::Cr3;
use x86_64::structures::idt::InterruptStackFrameValue;
use x86_64::structures::paging::{
    FrameDeallocator, OffsetPageTable, PageTable, PageTableFlags, PhysFrame,
    Translate,
};
use x86_64::structures::paging::mapper::TranslateResult;
use x86_64::VirtAddr;

pub const MAX_HANDLES: usize = 64;
pub const MAX_PROC_SIZE: usize = 10 << 20; // 10 MB

// The kernel currently resides at 0x200000 and programs should be linked above
// it to avoid causing a page protection violation.
const USER_ADDR: u64 = 0x800000;

pub fn is_valid_addr(addr: u64) -> bool {
    USER_ADDR <= addr && addr <= USER_ADDR + MAX_PROC_SIZE as u64
}

pub fn ptr_from_addr(addr: u64) -> *mut u8 {
    let base = code_addr();
    if addr < base {
        (base + addr) as *mut u8
    } else {
        addr as *mut u8
    }
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
    code_addr: u64,
    stack_addr: u64,
    entry_point_addr: u64,
    page_table_frame: PhysFrame,
    allocator: Arc<LockedHeap>,
}

#[derive(Clone)]
pub struct Process {
    parent_id: usize,
    stack_frame: Option<InterruptStackFrameValue>,
    registers: Registers,
    data: ProcessData,
    ctx: ProcessContext,
}

impl Process {
    fn new() -> Self {
        Self {
            parent_id: 0,
            stack_frame: None,
            registers: Registers::default(),
            data: ProcessData::new("/", None),
            ctx: ProcessContext {
                id: 0,
                code_addr: 0,
                stack_addr: 0,
                entry_point_addr: 0,
                page_table_frame: Cr3::read().0,
                allocator: Arc::new(LockedHeap::empty()),
            }
        }
    }

    fn mapper(&self) -> OffsetPageTable<'_> {
        let page_table = unsafe {
            mem::page_table_at(self.ctx.page_table_frame)
        };
        unsafe {
            OffsetPageTable::new(page_table, VirtAddr::new(phys_mem_offset()))
        }
    }

    fn free_pages(&self) {
        let mut mapper = self.mapper();

        let size = MAX_PROC_SIZE;
        mem::free_pages(&mut mapper, self.ctx.code_addr, size);

        let addr = USER_ADDR;
        match mapper.translate(VirtAddr::new(addr)) {
            TranslateResult::Mapped { frame: _, offset: _, flags } => {
                if flags.contains(PageTableFlags::USER_ACCESSIBLE) {
                    mem::free_pages(&mut mapper, addr, size);
                }
            }
            _ => {}
        }
    }
}

pub fn exit() {
    let proc = {
        let mut table = PROCESS_TABLE.write();
        table[id()].take().unwrap()
    };

    proc.free_pages();
    unsafe {
        with_frame_allocator(|allocator| {
            allocator.deallocate_frame(proc.ctx.page_table_frame);
        });
    }

    load_process(proc.parent_id);
}

fn load_process(id: usize) {
    set_id(id);
    unsafe {
        let (_, flags) = Cr3::read();
        Cr3::write(page_table_frame(), flags);
    }
}

unsafe fn page_table_frame() -> PhysFrame {
    let table = PROCESS_TABLE.read();
    let proc = current_process(&table);
    proc.ctx.page_table_frame
}

pub unsafe fn page_table() -> &'static mut PageTable {
    mem::page_table_at(page_table_frame())
}

pub fn init() {
    // Initialize the process table
    table::init();

    // Initialize the process memory area
    let l4 = mem::page_table_index(crate::PROC_ADDR, PageTableLevel::L4);
    let table = unsafe { mem::active_page_table() };
    let frame = mem::alloc_frame();
    let flags = PageTableFlags::PRESENT
              | PageTableFlags::WRITABLE
              | PageTableFlags::USER_ACCESSIBLE;
    table[l4].set_frame(frame, flags);
}
