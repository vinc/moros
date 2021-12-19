use crate::sys::fs::{Resource, Device};
use crate::sys::console::Console;
use alloc::collections::btree_map::BTreeMap;
use alloc::string::{String, ToString};
use core::sync::atomic::{AtomicUsize, Ordering};
use lazy_static::lazy_static;
use object::{Object, ObjectSegment};
use spin::RwLock;
use x86_64::structures::idt::InterruptStackFrameValue;

const MAX_FILE_HANDLES: usize = 16; // FIXME Increasing this cause boot crashes
const MAX_PROCS: usize = 2; // TODO: Update this when EXIT syscall is working

lazy_static! {
    pub static ref PID: AtomicUsize = AtomicUsize::new(0);
    pub static ref MAX_PID: AtomicUsize = AtomicUsize::new(1);
    pub static ref PROCESS_TABLE: RwLock<[Process; MAX_PROCS]> = RwLock::new([(); MAX_PROCS].map(|_| Process::new(0)));
}

#[derive(Clone, Debug)]
pub struct ProcessData {
    env: BTreeMap<String, String>,
    dir: String,
    user: Option<String>,
    file_handles: [Option<Resource>; MAX_FILE_HANDLES],
}

impl ProcessData {
    pub fn new(dir: &str, user: Option<&str>) -> Self {
        let env = BTreeMap::new();
        let dir = dir.to_string();
        let user = user.map(String::from);
        let mut file_handles = [(); MAX_FILE_HANDLES].map(|_| None);
        file_handles[0] = Some(Resource::Device(Device::Console(Console::new())));
        file_handles[1] = Some(Resource::Device(Device::Console(Console::new())));
        file_handles[2] = Some(Resource::Device(Device::Console(Console::new())));
        Self { env, dir, user, file_handles }
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

pub fn create_file_handle(file: Resource) -> Result<usize, ()> {
    let mut table = PROCESS_TABLE.write();
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
    let mut table = PROCESS_TABLE.write();
    let proc = &mut table[id()];
    proc.data.file_handles[handle] = Some(file);
}

pub fn delete_file_handle(handle: usize) {
    let mut table = PROCESS_TABLE.write();
    let proc = &mut table[id()];
    proc.data.file_handles[handle] = None;
}

pub fn file_handle(handle: usize) -> Option<Resource> {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    proc.data.file_handles[handle].clone()
}

pub fn code_addr() -> u64 {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    proc.code_addr
}

pub fn set_code_addr(addr: u64) {
    let mut table = PROCESS_TABLE.write();
    let mut proc = &mut table[id()];
    proc.code_addr = addr;
}

pub fn ptr_from_addr(addr: u64) -> *mut u8 {
    (code_addr() + addr) as *mut u8
}

pub fn registers() -> Registers {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    proc.registers
}

pub fn set_registers(regs: Registers) {
    let mut table = PROCESS_TABLE.write();
    let mut proc = &mut table[id()];
    proc.registers = regs
}

pub fn stack_frame() -> InterruptStackFrameValue {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    proc.stack_frame.clone()
}

pub fn set_stack_frame(stack_frame: InterruptStackFrameValue) {
    let mut table = PROCESS_TABLE.write();
    let mut proc = &mut table[id()];
    proc.stack_frame = stack_frame;
}

pub fn exit() {
    let table = PROCESS_TABLE.read();
    let proc = &table[id()];
    sys::allocator::free_pages(proc.code_addr, proc.code_size);
    MAX_PID.fetch_sub(1, Ordering::SeqCst);
    set_id(0); // FIXME: No process manager so we switch back to process 0
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

static CODE_ADDR: AtomicU64 = AtomicU64::new((sys::allocator::HEAP_START as u64) + (16 << 20)); // 16 MB
const PAGE_SIZE: u64 = 4 * 1024;

#[repr(align(8), C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Registers {
    pub r11: usize,
    pub r10: usize,
    pub r9:  usize,
    pub r8:  usize,
    pub rdi: usize,
    pub rsi: usize,
    pub rdx: usize,
    pub rcx: usize,
    pub rax: usize,
}

const ELF_MAGIC: [u8; 4] = [0x74, b'E', b'L', b'F'];

#[derive(Clone, Debug)]
pub struct Process {
    id: usize,
    code_addr: u64,
    code_size: u64,
    entry_point: u64,
    stack_frame: InterruptStackFrameValue,
    registers: Registers,
    data: ProcessData,
}

impl Process {
    pub fn new(id: usize) -> Self {
        let isf = InterruptStackFrameValue {
            instruction_pointer: VirtAddr::new(0),
            code_segment: 0,
            cpu_flags: 0,
            stack_pointer: VirtAddr::new(0),
            stack_segment: 0,
        };
        Self {
            id,
            code_addr: 0,
            code_size: 0,
            entry_point: 0,
            stack_frame: isf,
            registers: Registers::default(),
            data: ProcessData::new("/", None),
        }
    }

    pub fn spawn(bin: &[u8]) {
        if let Ok(pid) = Self::create(bin) {
            let proc = {
                let table = PROCESS_TABLE.read();
                table[pid].clone()
            };
            proc.exec();
        }
    }

    fn create(bin: &[u8]) -> Result<usize, ()> {
        let code_size = 1024 * PAGE_SIZE;
        let code_addr = CODE_ADDR.fetch_add(code_size, Ordering::SeqCst);
        sys::allocator::alloc_pages(code_addr, code_size);

        let mut entry_point = 0;
        let code_ptr = code_addr as *mut u8;
        if bin[0..4] == ELF_MAGIC { // ELF binary
            if let Ok(obj) = object::File::parse(bin) {
                entry_point = obj.entry();
                for segment in obj.segments() {
                    let addr = segment.address() as usize;
                    if let Ok(data) = segment.data() {
                        for (i, op) in data.iter().enumerate() {
                            unsafe {
                                let ptr = code_ptr.add(addr + i);
                                core::ptr::write(ptr, *op);
                            }
                        }
                    }
                }
            }
        } else { // Raw binary
            for (i, op) in bin.iter().enumerate() {
                unsafe {
                    let ptr = code_ptr.add(i);
                    core::ptr::write(ptr, *op);
                }
            }
        }

        let mut table = PROCESS_TABLE.write();
        let parent = &table[id()];

        let data = parent.data.clone();
        let registers = parent.registers;
        let stack_frame = parent.stack_frame.clone();

        let id = MAX_PID.fetch_add(1, Ordering::SeqCst);
        let proc = Process { id, code_addr, code_size, entry_point, data, stack_frame, registers };
        table[id] = proc;

        Ok(id)
    }

    // Switch to user mode and execute the program
    fn exec(&self) {
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
                in("rsi") self.code_addr + self.code_size,
                in("rdx") GDT.1.user_code.0,
                in("rdi") self.code_addr + self.entry_point,
            );
        }
    }
}
