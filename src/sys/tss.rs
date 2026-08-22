use core::ptr::addr_of;
use lazy_static::lazy_static;

const STACK_SIZE: usize = 1024 * 8 * 16;
pub const DOUBLE_FAULT: usize = 0;
pub const PAGE_FAULT: usize = 1;
pub const GENERAL_PROTECTION_FAULT: usize = 2;

lazy_static! {
    pub static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        let addr = {
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            STACK_SIZE + addr_of!(STACK) as usize
        };
        tss.set_kernel_stack(addr);

        #[cfg(target_arch = "x86")]
        {
            tss.stack[0].ss = crate::sys::gdt::SYS_DATA.bits;
        }

        #[cfg(target_arch = "x86_64")]
        {
            let addr = {
                static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
                STACK_SIZE + addr_of!(STACK) as usize
            };
            tss.set_interrupt_stack(DOUBLE_FAULT, addr);

            let addr = {
                static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
                STACK_SIZE + addr_of!(STACK) as usize
            };
            tss.set_interrupt_stack(PAGE_FAULT, addr);

            let addr = {
                static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
                STACK_SIZE + addr_of!(STACK) as usize
            };
            tss.set_interrupt_stack(GENERAL_PROTECTION_FAULT, addr);
        }
        tss
    };
}

#[cfg(target_arch = "x86")]
#[repr(C)]
#[derive(Default)]
pub struct PrivilegeStack {
    pub esp: u32,
    pub ss: u16,
    reserved: u16,
}

#[cfg(target_arch = "x86")]
#[repr(C, packed(4))]
#[derive(Default)]
pub struct TaskStateSegment {
    reserved_1: u32,
    pub stack: [PrivilegeStack; 3],
    reserved_2: [u32; 18],
    reserved_3: u16,
    pub iomap_base: u16,
}

#[cfg(target_arch = "x86_64")]
#[derive(Default)]
#[repr(C, packed(4))]
pub struct TaskStateSegment {
    reserved_1: u32,
    pub stack: [u64; 3],
    reserved_2: [u32; 2],
    pub ist: [u64; 7],
    reserved_3: [u32; 2],
    reserved_4: u16,
    pub iomap_base: u16,
}

impl TaskStateSegment {
    pub fn new() -> Self {
        let mut tss = Self::default();
        tss.iomap_base = Self::size();
        tss
    }

    const fn size() -> u16 {
        size_of::<Self>() as u16
    }

    pub fn limit(&self) -> u16 {
        Self::size() - 1
    }

    pub fn base(&self) -> usize {
        self as *const _ as usize
    }

    #[cfg(target_arch = "x86")]
    pub fn set_kernel_stack(&mut self, addr: usize) {
        self.stack[0].esp = addr as u32;
    }

    #[cfg(target_arch = "x86_64")]
    pub fn set_kernel_stack(&mut self, addr: usize) {
        self.stack[0] = addr as u64;
    }


    #[cfg(target_arch = "x86_64")]
    pub fn set_interrupt_stack(&mut self, index: usize, addr: usize) {
        self.ist[index] = addr as u64;
    }
}
