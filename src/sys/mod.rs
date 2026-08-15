#[macro_export]
macro_rules! printk {
    ($($arg:tt)*) => ({
        $crate::sys::console::print_fmt(format_args!($($arg)*));
    });
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ({
        let csi_color = $crate::api::console::Style::color("blue");
        let csi_reset = $crate::api::console::Style::reset();
        $crate::sys::console::print_fmt(format_args!(
            "{}DEBUG: {}{}\n", csi_color, format_args!($($arg)*), csi_reset
        ));
    });
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => ({
        if !cfg!(test) {
            let time = $crate::sys::clk::boot_time();
            let csi_color = $crate::api::console::Style::color("lime");
            let csi_reset = $crate::api::console::Style::reset();
            $crate::sys::console::print_fmt(format_args!(
                "{}[{:.6}]{} {}\n",
                csi_color, time, csi_reset, format_args!($($arg)*)
            ));

            let time = $crate::sys::clk::epoch_time();
            $crate::sys::log::write_fmt(format_args!(
                "[{:.6}] {}\n",
                time, format_args!($($arg)*)
            ));
        }
    });
}

pub mod x86 {
    use core::arch::asm;

    use bit_field::BitField;
    use x86_64::structures::paging::PhysFrame;
    use x86_64::PhysAddr;

    pub struct Cr2;

    impl Cr2 {
        #[inline]
        pub fn read() -> u64 {
            let value: u64;
            unsafe {
                asm!(
                    "mov {}, cr2", out(reg) value,
                    options(nomem, nostack, preserves_flags)
                );
            }
            value
        }
    }

    pub struct Cr3 {
        addr: u64,
        flags: u16,
    }

    impl Cr3 {
        #[inline]
        pub fn read() -> Self {
            let value: u64;
            unsafe {
                asm!(
                    "mov {}, cr3", out(reg) value,
                    options(nomem, nostack, preserves_flags)
                );
            }
            let mask = 0xFFF;
            let addr = value & !mask;
            let flags = (value & mask) as u16;
            Self { addr, flags }
        }

        #[inline]
        pub unsafe fn write(addr: u64, flags: u16) {
            debug_assert_eq!(addr.get_bits(..12), 0);
            debug_assert_eq!(addr.get_bits(52..), 0);
            debug_assert_eq!(flags.get_bits(12..), 0);
            let value = addr | flags as u64;
            asm!(
                "mov cr3, {}", in(reg) value,
                options(nostack, preserves_flags)
            );
        }

        pub fn flags(&self) -> u16 {
            self.flags
        }

        pub fn addr(&self) -> u64 {
            self.addr
        }

        pub fn frame(&self) -> PhysFrame {
            PhysFrame::containing_address(PhysAddr::new(self.addr))
        }
    }

    #[inline]
    pub fn cr3() -> (u64, u16) {
        let value: u64;
        unsafe {
            asm!(
                "mov {}, cr3", out(reg) value,
                options(nomem, nostack, preserves_flags)
            );
        }
        let mask = 0xFFF;
        let addr = value & !mask;
        let flags = (value & mask) as u16;
        (addr, flags)
    }

    #[inline]
    pub fn cr3_write(addr: u64, flags: u16) {
        debug_assert_eq!(addr.get_bits(..12), 0);
        debug_assert_eq!(addr.get_bits(52..), 0);
        debug_assert_eq!(flags.get_bits(12..), 0);
        let value = addr | flags as u64;
        unsafe {
            asm!(
                "mov cr3, {}", in(reg) value,
                options(nostack, preserves_flags)
            );
        }
    }

    /// Halts the CPU until the next interrupt
    #[inline]
    pub fn hlt() {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }

    pub fn rdrand() -> Option<u64> {
        let mut res = 0;
        unsafe {
            if core::arch::x86_64::_rdrand64_step(&mut res) == 1 {
                Some(res)
            } else {
                None
            }
        }
    }

    pub mod rflags {
        pub const IF: usize = 1 << 9; // Interrupt Flag
    }

    pub mod interrupts {
        use core::arch::asm;

        #[inline]
        pub fn enable() {
            // NOTE: interrupts are not enabled until after the next instruction
            unsafe {
                asm!("sti", options(nostack, preserves_flags));
            }
        }

        #[inline]
        pub fn disable() {
            unsafe {
                asm!("cli", options(nostack, preserves_flags));
            }
        }

        #[inline]
        pub fn are_enabled() -> bool {
            let rflags: usize;
            unsafe {
                asm!("pushfq; pop {}", out(reg) rflags,
                    options(nomem, preserves_flags));
            }
            rflags & super::rflags::IF != 0
        }

        #[inline]
        pub fn without_interrupts<F, R>(f: F) -> R where F: FnOnce() -> R {
            let enabled = are_enabled();
            if enabled {
                disable();
            }
            let res = f();
            if enabled {
                enable();
            }
            res
        }
    }

    pub mod port {
        use core::arch::asm;

        #[inline]
        pub unsafe fn outb(port: u16, value: u8) {
            asm!(
                "out dx, al", in("dx") port, in("al") value,
                options(nostack, preserves_flags)
            );
        }

        #[inline]
        pub unsafe fn inb(port: u16) -> u8 {
            let value: u8;
            asm!(
                "in al, dx", in("dx") port, out("al") value,
                options(nostack, preserves_flags)
            );
            value
        }

        #[inline]
        pub unsafe fn outw(port: u16, value: u16) {
            asm!(
                "out dx, ax", in("dx") port, in("ax") value,
                options(nostack, preserves_flags)
            );
        }

        #[inline]
        pub unsafe fn inw(port: u16) -> u16 {
            let value: u16;
            asm!(
                "in ax, dx", in("dx") port, out("ax") value,
                options(nostack, preserves_flags)
            );
            value
        }

        #[inline]
        pub unsafe fn outl(port: u16, value: u32) {
            asm!(
                "out dx, eax", in("dx") port, in("eax") value,
                options(nostack, preserves_flags)
            );
        }

        #[inline]
        pub unsafe fn inl(port: u16) -> u32 {
            let value: u32;
            asm!(
                "in eax, dx", in("dx") port, out("eax") value,
                options(nostack, preserves_flags)
            );
            value
        }
    }
}

pub mod acpi;
pub mod ata;
pub mod boot;
pub mod clk;
pub mod console;
pub mod cpu;
pub mod fs;
pub mod gdt;
pub mod idt;
pub mod keyboard;
pub mod log;
pub mod mem;
pub mod net;
pub mod pci;
pub mod pic;
pub mod process;
pub mod rng;
pub mod serial;
pub mod snd;
pub mod speaker;
pub mod syscall;
pub mod vga;
