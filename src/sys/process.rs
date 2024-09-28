use crate::api::process::ExitCode;
use crate::sys::console::Console;
use crate::sys::fs::{Device, Resource};
use crate::sys;
use crate::sys::gdt::GDT;

use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::alloc::{GlobalAlloc, Layout};
use core::arch::asm;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use lazy_static::lazy_static;
use linked_list_allocator::LockedHeap;
use object::{Object, ObjectSegment};
use spin::RwLock;
use x86_64::registers::control::Cr3;
use x86_64::structures::idt::InterruptStackFrameValue;
use x86_64::structures::paging::{
    FrameAllocator, OffsetPageTable, PageTable, PhysFrame
};
use x86_64::VirtAddr;

const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];
const BIN_MAGIC: [u8; 4] = [0x7F, b'B', b'I', b'N'];

const MAX_HANDLES: usize = 64;
const MAX_PROCS: usize = 4; // TODO: Increase this
const MAX_PROC_SIZE: usize = 10 << 20; // 10 MB

static CODE_ADDR: AtomicU64 = AtomicU64::new(0);
pub static PID: AtomicUsize = AtomicUsize::new(0);
pub static MAX_PID: AtomicUsize = AtomicUsize::new(1);

lazy_static! {
    pub static ref PROCESS_TABLE: RwLock<[Box<Process>; MAX_PROCS]> = {
        RwLock::new([(); MAX_PROCS].map(|_| Box::new(Process::new())))
    };
}

// Called during kernel heap initialization
pub fn init_process_addr(addr: u64) {
    sys::process::CODE_ADDR.store(addr, Ordering::SeqCst);
}

#[repr(align(8), C)]
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
pub struct ProcessData {
    env: BTreeMap<String, String>,
    dir: String,
    user: Option<String>,
    handles: [Option<Box<Resource>>; MAX_HANDLES],
}

impl ProcessData {
    pub fn new(dir: &str, user: Option<&str>) -> Self {
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

pub fn id() -> usize {
    PID.load(Ordering::SeqCst)
}

pub fn set_id(id: usize) {
    PID.store(id, Ordering::SeqCst)
}

pub fn env(key: &str) -> Option<String> {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    proc.data.env.get(key).cloned()
}

pub fn envs() -> BTreeMap<String, String> {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    proc.data.env.clone()
}

pub fn dir() -> String {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    proc.data.dir.clone()
}

pub fn user() -> Option<String> {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    proc.data.user.clone()
}

pub fn set_env(key: &str, val: &str) {
    let mut table = PROCESS_TABLE.write();
    let proc = &mut table[id()];
    proc.data.env.insert(key.into(), val.into());
}

pub fn set_dir(dir: &str) {
    let mut table = PROCESS_TABLE.write();
    let proc = &mut table[id()];
    proc.data.dir = dir.into();
}

pub fn set_user(user: &str) {
    let mut table = PROCESS_TABLE.write();
    let proc = &mut table[id()];
    proc.data.user = Some(user.into())
}

pub fn create_handle(file: Resource) -> Result<usize, ()> {
    let mut table = PROCESS_TABLE.write();
    let proc = &mut table[id()];
    let min = 4; // The first 4 handles are reserved
    let max = MAX_HANDLES;
    for handle in min..max {
        if proc.data.handles[handle].is_none() {
            proc.data.handles[handle] = Some(Box::new(file));
            return Ok(handle);
        }
    }
    debug!("Could not create handle");
    Err(())
}

pub fn update_handle(handle: usize, file: Resource) {
    let mut table = PROCESS_TABLE.write();
    let proc = &mut table[id()];
    proc.data.handles[handle] = Some(Box::new(file));
}

pub fn delete_handle(handle: usize) {
    let mut table = PROCESS_TABLE.write();
    let proc = &mut table[id()];
    proc.data.handles[handle] = None;
}

pub fn handle(handle: usize) -> Option<Box<Resource>> {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    proc.data.handles[handle].clone()
}

pub fn handles() -> Vec<Option<Box<Resource>>> {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    proc.data.handles.to_vec()
}

pub fn code_addr() -> u64 {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    proc.code_addr
}

pub fn set_code_addr(addr: u64) {
    let mut table = PROCESS_TABLE.write();
    let proc = &mut table[id()];
    proc.code_addr = addr;
}

pub fn ptr_from_addr(addr: u64) -> *mut u8 {
    let base = code_addr();
    if addr < base {
        (base + addr) as *mut u8
    } else {
        addr as *mut u8
    }
}

pub fn registers() -> Registers {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    proc.registers
}

pub fn set_registers(regs: Registers) {
    let mut table = PROCESS_TABLE.write();
    let proc = &mut table[id()];
    proc.registers = regs
}

pub fn stack_frame() -> InterruptStackFrameValue {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    proc.stack_frame.unwrap()
}

pub fn set_stack_frame(stack_frame: InterruptStackFrameValue) {
    let mut table = PROCESS_TABLE.write();
    let proc = &mut table[id()];
    proc.stack_frame = Some(stack_frame);
}

pub fn exit() {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];

    let page_table = unsafe {
        sys::mem::create_page_table(proc.page_table_frame)
    };
    let phys_mem_offset = unsafe {
        sys::mem::PHYS_MEM_OFFSET.unwrap()
    };
    let mut mapper = unsafe {
        OffsetPageTable::new(page_table, VirtAddr::new(phys_mem_offset))
    };
    sys::allocator::free_pages(&mut mapper, proc.code_addr, MAX_PROC_SIZE);

    MAX_PID.fetch_sub(1, Ordering::SeqCst);
    set_id(proc.parent_id);

    unsafe {
        let (_, flags) = Cr3::read();
        Cr3::write(page_table_frame(), flags);
    }
}

unsafe fn page_table_frame() -> PhysFrame {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    proc.page_table_frame
}

pub unsafe fn page_table() -> &'static mut PageTable {
    sys::mem::create_page_table(page_table_frame())
}

pub unsafe fn alloc(layout: Layout) -> *mut u8 {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    proc.allocator.alloc(layout)
}

pub unsafe fn free(ptr: *mut u8, layout: Layout) {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    let bottom = proc.allocator.lock().bottom();
    let top = proc.allocator.lock().top();
    //debug!("heap bottom: {:#?}", bottom);
    //debug!("ptr:         {:#?}", ptr);
    //debug!("heap top:    {:#?}", top);
    if bottom <= ptr && ptr < top {
        // FIXME: panicked at 'Freed node aliases existing hole! Bad free?'
        proc.allocator.dealloc(ptr, layout);
    } else {
        debug!("Could not free {:#?}", ptr);
    }
}

#[derive(Clone)]
pub struct Process {
    id: usize,
    parent_id: usize,
    code_addr: u64,
    stack_addr: u64,
    entry_point_addr: u64,
    page_table_frame: PhysFrame,
    stack_frame: Option<InterruptStackFrameValue>,
    registers: Registers,
    data: ProcessData,
    allocator: Arc<LockedHeap>,
}

impl Process {
    pub fn new() -> Self {
        Self {
            id: 0,
            parent_id: 0,
            code_addr: 0,
            stack_addr: 0,
            entry_point_addr: 0,
            stack_frame: None,
            page_table_frame: Cr3::read().0,
            registers: Registers::default(),
            data: ProcessData::new("/", None),
            allocator: Arc::new(LockedHeap::empty()),
        }
    }

    pub fn spawn(
        bin: &[u8],
        args_ptr: usize,
        args_len: usize
    ) -> Result<(), ExitCode> {
        if let Ok(id) = Self::create(bin) {
            let proc = {
                let table = PROCESS_TABLE.read();
                table[id].clone()
            };
            proc.exec(args_ptr, args_len);
            Ok(())
        } else {
            Err(ExitCode::ExecError)
        }
    }

    fn create(bin: &[u8]) -> Result<usize, ()> {
        if MAX_PID.load(Ordering::SeqCst) >= MAX_PROCS {
            return Err(());
        }

        let page_table_frame = sys::mem::frame_allocator().allocate_frame().
            expect("frame allocation failed");

        let page_table = unsafe {
            sys::mem::create_page_table(page_table_frame)
        };

        let kernel_page_table = unsafe {
            sys::mem::active_page_table()
        };

        // FIXME: for now we just copy everything
        let pages = page_table.iter_mut().zip(kernel_page_table.iter());
        for (user_page, kernel_page) in pages {
            *user_page = kernel_page.clone();
        }

        let phys_mem_offset = unsafe { sys::mem::PHYS_MEM_OFFSET.unwrap() };
        let mut mapper = unsafe {
            OffsetPageTable::new(page_table, VirtAddr::new(phys_mem_offset))
        };

        let proc_size = MAX_PROC_SIZE as u64;
        let code_addr = CODE_ADDR.fetch_add(proc_size, Ordering::SeqCst);
        let stack_addr = code_addr + proc_size - 4096;

        let mut entry_point_addr = 0;

        if bin[0..4] == ELF_MAGIC { // ELF binary
            if let Ok(obj) = object::File::parse(bin) {
                entry_point_addr = obj.entry();

                for segment in obj.segments() {
                    if let Ok(data) = segment.data() {
                        let addr = code_addr + segment.address();
                        let size = segment.size() as usize;
                        // NOTE: `size` can be larger than `data.len()` because
                        // the object can contain uninitialized sections like
                        // ".bss" that have a size but no data.
                        load_binary(&mut mapper, addr, size, data)?;
                    }
                }
            }
        } else if bin[0..4] == BIN_MAGIC { // Flat binary
            load_binary(&mut mapper, code_addr, bin.len() - 4, &bin[4..])?;
        } else {
            return Err(());
        }

        let parent = {
            let process_table = PROCESS_TABLE.read();
            process_table[id()].clone()
        };

        let data = parent.data.clone();
        let registers = parent.registers;
        let stack_frame = parent.stack_frame;

        let allocator = Arc::new(LockedHeap::empty());

        let id = MAX_PID.fetch_add(1, Ordering::SeqCst);
        let parent_id = parent.id;
        let proc = Process {
            id,
            parent_id,
            code_addr,
            stack_addr,
            entry_point_addr,
            page_table_frame,
            data,
            stack_frame,
            registers,
            allocator,
        };

        let mut process_table = PROCESS_TABLE.write();
        process_table[id] = Box::new(proc);

        Ok(id)
    }

    // Switch to user mode and execute the program
    fn exec(&self, args_ptr: usize, args_len: usize) {
        let page_table = unsafe { sys::process::page_table() };
        let phys_mem_offset = unsafe { sys::mem::PHYS_MEM_OFFSET.unwrap() };
        let mut mapper = unsafe {
            OffsetPageTable::new(page_table, VirtAddr::new(phys_mem_offset))
        };

        let heap_addr = self.code_addr + (self.stack_addr - self.code_addr) / 2;

        sys::allocator::alloc_pages(&mut mapper, heap_addr, 1).
            expect("proc heap alloc");

        let args_ptr = ptr_from_addr(args_ptr as u64) as usize;
        let args: &[&str] = unsafe {
            core::slice::from_raw_parts(args_ptr as *const &str, args_len)
        };
        let mut addr = heap_addr;
        let vec: Vec<&str> = args.iter().map(|arg| {
            let ptr = addr as *mut u8;
            addr += arg.len() as u64;
            unsafe {
                let s = core::slice::from_raw_parts_mut(ptr, arg.len());
                s.copy_from_slice(arg.as_bytes());
                core::str::from_utf8_unchecked(s)
            }
        }).collect();
        let align = core::mem::align_of::<&str>() as u64;
        addr += align - (addr % align);
        let args = vec.as_slice();
        let ptr = addr as *mut &str;
        let args: &[&str] = unsafe {
            let s = core::slice::from_raw_parts_mut(ptr, args.len());
            s.copy_from_slice(args);
            s
        };
        let args_ptr = args.as_ptr() as u64;

        let heap_addr = addr + 4096;
        let heap_size = ((self.stack_addr - heap_addr) / 2) as usize;
        unsafe {
            self.allocator.lock().init(heap_addr as *mut u8, heap_size);
        }

        set_id(self.id); // Change PID

        unsafe {
            let (_, flags) = Cr3::read();
            Cr3::write(self.page_table_frame, flags);

            asm!(
                "cli",        // Disable interrupts
                "push {:r}",  // Stack segment (SS)
                "push {:r}",  // Stack pointer (RSP)
                "push 0x200", // RFLAGS with interrupts enabled
                "push {:r}",  // Code segment (CS)
                "push {:r}",  // Instruction pointer (RIP)
                "iretq",
                in(reg) GDT.1.user_data.0,
                in(reg) self.stack_addr,
                in(reg) GDT.1.user_code.0,
                in(reg) self.code_addr + self.entry_point_addr,
                in("rdi") args_ptr,
                in("rsi") args_len,
            );
        }
    }
}

fn load_binary(
    mapper: &mut OffsetPageTable, addr: u64, size: usize, buf: &[u8]
) -> Result<(), ()> {
    debug_assert!(size >= buf.len());
    sys::allocator::alloc_pages(mapper, addr, size)?;
    let src = buf.as_ptr();
    let dst = addr as *mut u8;
    unsafe {
        core::ptr::copy_nonoverlapping(src, dst, buf.len());
    }
    Ok(())
}
