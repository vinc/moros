use super::Process;
use super::PROCESS_TABLE;
use super::MAX_PROCS;
use super::MAX_PROC_SIZE;
use super::CODE_ADDR;
use super::{id, set_id};
use super::page_table;
use super::ptr_from_addr;
use super::ProcessContext;

use crate::api::process::ExitCode;
use crate::sys::gdt::GDT;
use crate::sys::mem;
use crate::sys::mem::phys_mem_offset;

use alloc::boxed::Box;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::arch::asm;
use core::sync::atomic::Ordering;
use linked_list_allocator::LockedHeap;
use object::{Object, ObjectSegment};
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{
    FrameAllocator, OffsetPageTable,
};
use x86_64::VirtAddr;

const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];
const BIN_MAGIC: [u8; 4] = [0x7F, b'B', b'I', b'N'];

pub trait Spawn {
    fn spawn(
        bin: Vec<u8>,
        args_ptr: usize,
        args_len: usize
    ) -> Result<(), ExitCode>;
}

impl Spawn for Process {
    /// Spawn a new process from a binary.
    ///
    /// Takes ownership of the binary buffer because `exec` switches to
    /// user mode via `iretq` and never returns. Any heap allocation on
    /// the stack at that point is leaked, so we need to explicitly drop
    /// the buffer after `create` has copied it into process pages.
    ///
    /// The `ProcessContext` clone that crosses the `iretq` boundary only
    /// contains Copy types and an Arc refcount bump, so its leak is
    /// negligible.
    fn spawn(
        bin: Vec<u8>,
        args_ptr: usize,
        args_len: usize
    ) -> Result<(), ExitCode> {
        if let Ok(id) = create(&bin) {
            drop(bin);
            let ctx = {
                let table = PROCESS_TABLE.read();
                let proc = table[id].as_ref().unwrap();
                proc.ctx.clone()
            };
            exec(ctx, args_ptr, args_len);
            unreachable!(); // The kernel switched to the child process
        } else {
            Err(ExitCode::ExecError)
        }
    }
}

fn create(bin: &[u8]) -> Result<usize, ()> {
    let parent = {
        let process_table = PROCESS_TABLE.read();
        process_table[id()].clone().unwrap()
    };

    let mut process_table = PROCESS_TABLE.write();
    let id = (1..MAX_PROCS)
        .find(|&i| process_table[i].is_none())
        .ok_or(())?;

    let page_table_frame = mem::with_frame_allocator(|frame_allocator| {
        frame_allocator.allocate_frame().expect("frame allocation failed")
    });

    let page_table = unsafe {
        mem::create_page_table(page_table_frame)
    };

    let kernel_page_table = unsafe {
        mem::active_page_table()
    };

    // FIXME: for now we just copy everything
    let pages = page_table.iter_mut().zip(kernel_page_table.iter());
    for (user_page, kernel_page) in pages {
        *user_page = kernel_page.clone();
    }

    let mut mapper = unsafe {
        OffsetPageTable::new(page_table, VirtAddr::new(phys_mem_offset()))
    };

    let proc_size = MAX_PROC_SIZE as u64;
    let code_base = CODE_ADDR.load(Ordering::SeqCst);
    let code_addr = code_base + proc_size * id as u64;
    let stack_addr = code_addr + proc_size - 4096;

    let mut entry_point_addr = 0;

    //debug!("Process memory:");
    if bin.get(0..4) == Some(&ELF_MAGIC) { // ELF binary
        if let Ok(obj) = object::File::parse(bin) {
            entry_point_addr = obj.entry();

            for segment in obj.segments() {
                if let Ok(data) = segment.data() {
                    // NOTE: The size of the segment in memory can be
                    // larger than on the disk because the object can
                    // contain uninitialized sections like ".bss" that has
                    // a length but no data.
                    let addr = code_addr + segment.address();
                    let size = segment.size() as usize;
                    /*
                    debug!(
                        "{:#X}..{:#X}: {} bytes for a code segment ({:#X}..{:#X}: {} bytes)",
                        addr, addr + data.len() as u64, data.len(),
                        segment.address(), segment.address() + segment.size(), segment.size(),
                    );
                    */
                    load_binary(&mut mapper, addr, size, data)?;
                }
            }
        }
    } else if bin.get(0..4) == Some(&BIN_MAGIC) { // Flat binary
        load_binary(&mut mapper, code_addr, bin.len() - 4, &bin[4..])?;
    } else {
        // TODO: Free page_table_frame and any pages allocated
        return Err(());
    }

    let parent_id = parent.ctx.id;
    let data = parent.data.clone();
    let registers = parent.registers;
    let stack_frame = parent.stack_frame;

    let allocator = Arc::new(LockedHeap::empty());

    let proc = Process {
        parent_id,
        data,
        stack_frame,
        registers,
        ctx: ProcessContext {
            id,
            code_addr,
            stack_addr,
            entry_point_addr,
            page_table_frame,
            allocator,
        }
    };

    process_table[id] = Some(Box::new(proc));

    Ok(id)
}

// Switch to user mode and execute the program
fn exec(ctx: ProcessContext, args_ptr: usize, args_len: usize) {
    let page_table = unsafe { page_table() };
    let mut mapper = unsafe {
        OffsetPageTable::new(page_table, VirtAddr::new(phys_mem_offset()))
    };

    // Copy args to user memory
    let args_addr = ctx.code_addr + (ctx.stack_addr - ctx.code_addr) / 2;
    mem::alloc_pages(&mut mapper, args_addr, 1).
        expect("proc args alloc");
    let args: &[&str] = unsafe {
        let ptr = ptr_from_addr(args_ptr as u64) as usize;
        core::slice::from_raw_parts(ptr as *const &str, args_len)
    };
    let mut addr = args_addr;
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
    let heap_size = ((ctx.stack_addr - heap_addr) / 2) as usize;
    unsafe {
        ctx.allocator.lock().init(heap_addr as *mut u8, heap_size);
    }

    //debug!("{:#X}..{:#X}: {} bytes for the args", args_addr, args_addr + 4096, 4096); // FIXME: args size
    //debug!("{:#X}..{:#X}: {} bytes for the heap", heap_addr, heap_addr + heap_size as u64, heap_size);
    //debug!("{:#X}..{:#X}: {} bytes for the stack", self.stack_addr - heap_size as u64, self.stack_addr, heap_size);

    set_id(ctx.id); // Change PID

    unsafe {
        let (_, flags) = Cr3::read();
        Cr3::write(ctx.page_table_frame, flags);

        asm!(
            "cli",        // Disable interrupts
            "push {:r}",  // Stack segment (SS)
            "push {:r}",  // Stack pointer (RSP)
            "push 0x200", // RFLAGS with interrupts enabled
            "push {:r}",  // Code segment (CS)
            "push {:r}",  // Instruction pointer (RIP)
            "iretq",
            in(reg) GDT.1.user_data.0,
            in(reg) ctx.stack_addr,
            in(reg) GDT.1.user_code.0,
            in(reg) ctx.code_addr + ctx.entry_point_addr,
            in("rdi") args_ptr,
            in("rsi") args_len,
        );
    }
}

fn load_binary(
    mapper: &mut OffsetPageTable, addr: u64, size: usize, buf: &[u8]
) -> Result<(), ()> {
    debug_assert!(size >= buf.len());
    mem::alloc_pages(mapper, addr, size)?;
    let src = buf.as_ptr();
    let dst = addr as *mut u8;
    unsafe {
        core::ptr::copy_nonoverlapping(src, dst, buf.len());
        if size > buf.len() {
            core::ptr::write_bytes(dst.add(buf.len()), 0, size - buf.len());
        }
    }
    Ok(())
}
