use super::Process;
use super::Registers;
use super::MAX_HANDLES;

use crate::sys::fs::Resource;

use alloc::boxed::Box;
use alloc::collections::btree_map::BTreeMap;
use alloc::string::String;
use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicUsize, Ordering};
use lazy_static::lazy_static;
use spin::RwLock;
use x86_64::structures::idt::InterruptStackFrameValue;

pub const MAX_PROCS: usize = 32;
static PID: AtomicUsize = AtomicUsize::new(0);

type ProcessTable = [Option<Box<Process>>; MAX_PROCS];

lazy_static! {
    pub static ref PROCESS_TABLE: RwLock<ProcessTable> = {
        RwLock::new([(); MAX_PROCS].map(|_| None))
    };
}

pub fn init() {
    let mut table = PROCESS_TABLE.write();
    table[0] = Some(Box::new(Process::new()));
}

pub fn id() -> usize {
    PID.load(Ordering::SeqCst)
}

pub fn set_id(id: usize) {
    PID.store(id, Ordering::SeqCst)
}

pub fn current_process(table: &ProcessTable) -> &Process {
    table[id()].as_ref().unwrap()
}

fn current_process_mut(table: &mut ProcessTable) -> &mut Process {
    table[id()].as_mut().unwrap()
}

pub fn env() -> BTreeMap<String, String> {
    let table = PROCESS_TABLE.read();
    let proc = current_process(&table);
    proc.data.env.clone()
}

pub fn env_var(key: &str) -> Option<String> {
    let table = PROCESS_TABLE.read();
    let proc = current_process(&table);
    proc.data.env.get(key).cloned()
}

pub fn set_env_var(key: &str, val: &str) {
    let mut table = PROCESS_TABLE.write();
    let proc = current_process_mut(&mut table);
    proc.data.env.insert(key.into(), val.into());
}

pub fn dir() -> String {
    let table = PROCESS_TABLE.read();
    let proc = current_process(&table);
    proc.data.dir.clone()
}

pub fn set_dir(dir: &str) {
    let mut table = PROCESS_TABLE.write();
    let proc = current_process_mut(&mut table);
    proc.data.dir = dir.into();
}

pub fn user() -> Option<String> {
    let table = PROCESS_TABLE.read();
    let proc = current_process(&table);
    proc.data.user.clone()
}

pub fn set_user(user: &str) {
    let mut table = PROCESS_TABLE.write();
    let proc = current_process_mut(&mut table);
    proc.data.user = Some(user.into())
}

pub fn handle(handle: usize) -> Option<Box<Resource>> {
    let table = PROCESS_TABLE.read();
    let proc = current_process(&table);
    proc.data.handles[handle].clone()
}

pub fn create_handle(file: Resource) -> Result<usize, ()> {
    let mut table = PROCESS_TABLE.write();
    let proc = current_process_mut(&mut table);
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
    let proc = current_process_mut(&mut table);
    proc.data.handles[handle] = Some(Box::new(file));
}

pub fn delete_handle(handle: usize) {
    let mut table = PROCESS_TABLE.write();
    let proc = current_process_mut(&mut table);
    proc.data.handles[handle] = None;
}

pub fn code_addr() -> u64 {
    let table = PROCESS_TABLE.read();
    let proc = current_process(&table);
    proc.ctx.code_addr
}

pub fn registers() -> Registers {
    let table = PROCESS_TABLE.read();
    let proc = current_process(&table);
    proc.registers
}

pub fn set_registers(regs: Registers) {
    let mut table = PROCESS_TABLE.write();
    let proc = current_process_mut(&mut table);
    proc.registers = regs
}

pub fn stack_frame() -> InterruptStackFrameValue {
    let table = PROCESS_TABLE.read();
    let proc = current_process(&table);
    proc.stack_frame.unwrap()
}

pub fn set_stack_frame(stack_frame: InterruptStackFrameValue) {
    let mut table = PROCESS_TABLE.write();
    let proc = current_process_mut(&mut table);
    proc.stack_frame = Some(stack_frame);
}

pub unsafe fn alloc(layout: Layout) -> *mut u8 {
    let table = PROCESS_TABLE.read();
    let proc = current_process(&table);
    proc.ctx.allocator.alloc(layout)
}

pub unsafe fn free(ptr: *mut u8, layout: Layout) {
    let table = PROCESS_TABLE.read();
    let proc = current_process(&table);
    let bottom = proc.ctx.allocator.lock().bottom();
    let top = proc.ctx.allocator.lock().top();
    if bottom <= ptr && ptr < top {
        proc.ctx.allocator.dealloc(ptr, layout);
    } else { // FIXME: Uncomment to see errors
        //let size = layout.size();
        //let plural = if size != 1 { "s" } else { "" };
        //debug!("Could not free {} byte{} at {:#?}", size, plural, ptr);
    }
}
