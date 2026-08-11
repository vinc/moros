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

    /// Halts the CPU until the next interrupt
    #[inline]
    pub fn hlt() {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }

    pub fn rdrand() -> Option<u64> {
        let mut res = 0;

        #[cfg(target_arch = "x86_64")]
        unsafe {
            if core::arch::x86_64::_rdrand64_step(&mut res) == 1 {
                Some(res)
            } else {
                None
            }
        }

        #[cfg(target_arch = "x86")]
        None
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

#[cfg(target_arch = "x86_64")] pub mod acpi;
#[cfg(target_arch = "x86_64")] pub mod ata;
pub mod boot;
#[cfg(target_arch = "x86_64")] pub mod clk;
#[cfg(target_arch = "x86_64")] pub mod console;
#[cfg(target_arch = "x86_64")] pub mod cpu;
#[cfg(target_arch = "x86_64")] pub mod fs;
#[cfg(target_arch = "x86_64")] pub mod gdt;
#[cfg(target_arch = "x86_64")] pub mod idt;
#[cfg(target_arch = "x86_64")] pub mod keyboard;
#[cfg(target_arch = "x86_64")] pub mod log;
#[cfg(target_arch = "x86_64")] pub mod mem;
#[cfg(target_arch = "x86_64")] pub mod net;
#[cfg(target_arch = "x86_64")] pub mod pci;
#[cfg(target_arch = "x86_64")] pub mod pic;
#[cfg(target_arch = "x86_64")] pub mod process;
#[cfg(target_arch = "x86_64")] pub mod rng;
#[cfg(target_arch = "x86_64")] pub mod serial;
#[cfg(target_arch = "x86_64")] pub mod snd;
#[cfg(target_arch = "x86_64")] pub mod speaker;
#[cfg(target_arch = "x86_64")] pub mod syscall;
#[cfg(target_arch = "x86_64")] pub mod vga;
