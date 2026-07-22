use super::Process;
use super::MAX_PROC_SIZE;
use super::USER_ADDR;
use super::is_userspace;
use super::{id, set_id};
use super::page_table;
use super::ptr_from_addr;
use super::ProcessContext;
use super::ProcessStats;
use super::free_process;
use super::table::{PROCESS_TABLE, MAX_PROCS};

use crate::api::process::ExitCode;
use crate::sys::gdt::GDT;
use crate::sys::mem;

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::arch::asm;
use linked_list_allocator::LockedHeap;
use object::{Object, ObjectSegment};
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::{
    FrameAllocator, PageTable, Translate,
};
use x86_64::VirtAddr;

const ELF_MAGIC: [u8; 4] = [0x7F, b'E', b'L', b'F'];
const BIN_MAGIC: [u8; 4] = [0x7F, b'B', b'I', b'N'];

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
pub fn spawn(
    bin: Vec<u8>, args_ptr: usize, args_len: usize
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

fn create(bin: &[u8]) -> Result<usize, ()> {
    let (parent_id, data, stack_frame, registers) = {
        let process_table = PROCESS_TABLE.read();
        let proc = process_table[id()].as_ref().unwrap();
        (proc.ctx.id, proc.data.clone(), proc.stack_frame, proc.registers)
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

    // Clone the page table entries of the active page table, except the
    // entry of the user memory region.
    let l4_user = VirtAddr::new(USER_ADDR).p4_index().into();
    let pages = page_table.iter_mut().zip(kernel_page_table.iter());
    for (l4_index, (user_page, kernel_page)) in pages.enumerate() {
        if l4_index == l4_user {
            user_page.set_unused();
        } else {
            *user_page = kernel_page.clone();
        }
    }

    let proc_size = MAX_PROC_SIZE as u64;
    let stack_addr = USER_ADDR + proc_size - 4096;

    let entry_point_addr = load(bin, page_table).map_err(|_|
        free_process(page_table_frame)
    )?;

    let allocator = Arc::new(LockedHeap::empty());

    let proc = Process {
        parent_id,
        data,
        stack_frame,
        registers,
        stats: ProcessStats::new(),
        ctx: ProcessContext {
            id,
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
    // Copy the args to the kernel heap
    let args: Vec<String> = unsafe {
        let ptr = ptr_from_addr(args_ptr as u64) as *const &str;
        core::slice::from_raw_parts(ptr, args_len)
    }.iter().map(|arg| arg.to_string()).collect();

    set_id(ctx.id); // Change PID

    // Enter process address space and let the page fault handler allocate
    // user memory.
    unsafe {
        let (_, flags) = Cr3::read();
        Cr3::write(ctx.page_table_frame, flags);
    }

    // TODO: Move args to the user stack. Current location requires process
    // memory to be >= 32 MB so it stays above text/rodata/bss.
    let args_addr = USER_ADDR + (ctx.stack_addr - USER_ADDR) / 2;
    let args_size = 4096; // 1 page
    let args_len = args.len();
    let args_ptr = copy_args(&args, args_addr, args_size);
    drop(args);

    let heap_addr = args_addr + args_size as u64;
    let heap_size = ((ctx.stack_addr - heap_addr) / 2) as usize;
    unsafe {
        ctx.allocator.lock().init(heap_addr as *mut u8, heap_size);
    }

    unsafe {
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
            in(reg) ctx.entry_point_addr,
            in("rdi") args_ptr,
            in("rsi") args_len,
        );
    }
}

fn copy_args(args: &[String], addr: u64, size: usize) -> usize {
    let len = args.len();
    let mut offset = addr;

    // Alloc memory in the process page table, which is currently active
    let mut mapper = unsafe { mem::create_mapper(page_table()) };
    mem::alloc_pages(&mut mapper, addr, size).unwrap();

    // Copy each arg and record it as a &str in the user memory region
    let tmp: Vec<&str> = args.iter().map(|arg| {
        let arg_ptr = offset as *mut u8;
        offset += arg.len() as u64;
        unsafe {
            let dst = core::slice::from_raw_parts_mut(arg_ptr, arg.len());
            dst.copy_from_slice(arg.as_bytes());
            core::str::from_utf8_unchecked(dst)
        }
    }).collect();

    // Copy slice of &str
    let align = core::mem::align_of::<&str>() as u64;
    offset += align - (offset % align);
    unsafe {
        let args_ptr = offset as *mut &str;
        let dst = core::slice::from_raw_parts_mut(args_ptr, len);
        dst.copy_from_slice(tmp.as_slice());
    }

    let bytes = len * core::mem::size_of::<&str>() + (offset - addr) as usize;
    debug_assert!(bytes < size);
    offset as usize
}

fn load(bin: &[u8], page_table: &mut PageTable) -> Result<u64, ()> {
    if bin.get(0..4) == Some(&ELF_MAGIC) { // ELF binary
        let obj = object::File::parse(bin).map_err(|_| ())?;
        let entry_point_addr = obj.entry();
        if !is_userspace(entry_point_addr) {
            return Err(());
        }

        for segment in obj.segments() {
            let data = segment.data().map_err(|_| ())?;

            // NOTE: The size of the segment in memory can be larger than on
            // the disk because the object can contain uninitialized sections
            // like bss that has a length but no data.
            let addr = segment.address(); // Loaded at link address
            let size = segment.size() as usize;
            if size > 0 {
                if !is_userspace(addr) {
                    return Err(());
                }
                if !is_userspace(addr + size as u64 - 1) {
                    return Err(());
                }
                load_segment(addr, size, data, page_table)?;
            }
        }
        Ok(entry_point_addr)
    } else if bin.get(0..4) == Some(&BIN_MAGIC) { // Flat binary
        load_segment(USER_ADDR, bin.len() - 4, &bin[4..], page_table)?;
        Ok(USER_ADDR)
    } else {
        Err(())
    }
}

fn load_segment(
    addr: u64, size: usize, buf: &[u8], page_table: &mut PageTable
) -> Result<(), ()> {
    debug_assert!(size >= buf.len());
    let mut mapper = unsafe { mem::create_mapper(page_table) };

    // Pages are mapped only in the process page table, so they are not
    // accessible from the currently active kernel page table.
    mem::alloc_pages(&mut mapper, addr, size)?;
    let mut offset = 0;
    while offset < buf.len() {
        let page_addr = VirtAddr::new(addr + offset as u64);
        let phys_addr = mapper.translate_addr(page_addr).ok_or(())?;
        let dst = mem::phys_to_virt(phys_addr).as_mut_ptr::<u8>();

        let page_offset = usize::from(page_addr.page_offset());
        let n = core::cmp::min(4096 - page_offset, buf.len() - offset);

        unsafe {
            core::ptr::copy_nonoverlapping(buf.as_ptr().add(offset), dst, n);
        }
        offset += n;
    }
    Ok(())
}

#[test_case]
fn test_load() {
    use alloc::vec;

    let print_bin = include_bytes!("../../../dsk/bin/print").to_vec();
    let print_obj = object::File::parse(&print_bin[..]).unwrap();
    let print_pos = print_obj.entry();

    let bins = vec![
        (vec![], Err(())),
        (vec![b'F'], Err(())),
        (vec![b'F', b'A', b'I', b'L'], Err(())),
        (vec![0x7F, b'E', b'L', b'F'], Err(())),
        (vec![0x7F, b'E', b'L', b'F', b'F', b'A', b'I', b'L'], Err(())),
        (vec![0x7F, b'B', b'I', b'N', b'P', b'A', b'S', b'S'], Ok(USER_ADDR)),
        (print_bin, Ok(print_pos)),
    ];

    for (bin, res) in bins.iter() {
        let used = mem::with_frame_allocator(|a| a.used_frames());
        let frame = mem::with_frame_allocator(|a| a.allocate_frame().unwrap());
        assert_eq!(mem::with_frame_allocator(|a| a.used_frames()), used + 1);
        let page_table = unsafe { mem::create_page_table(frame) };
        page_table.zero();
        assert_eq!(load(&bin, page_table), *res);
        free_process(frame);
        assert_eq!(mem::with_frame_allocator(|a| a.used_frames()), used);
    }
}
