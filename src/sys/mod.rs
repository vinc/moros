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

pub mod port {
    use core::arch::asm;

    /// Write an 8-bit byte to a port
    pub unsafe fn outb(port: u16, value: u8) {
        asm!(
            "out dx, al", in("dx") port, in("al") value,
            options(nostack, preserves_flags)
        );
    }

    /// Read an 8-bit byte from a port
    pub unsafe fn inb(port: u16) -> u8 {
        let value: u8;
        asm!(
            "in al, dx", in("dx") port, out("al") value,
            options(nostack, preserves_flags)
        );
        value
    }

    /// Write a 16-bit word to a port
    pub unsafe fn outw(port: u16, value: u16) {
        asm!(
            "out dx, ax", in("dx") port, in("ax") value,
            options(nostack, preserves_flags)
        );
    }

    /// Read a 16-bit word from a port
    pub unsafe fn inw(port: u16) -> u16 {
        let value: u16;
        asm!(
            "in ax, dx", in("dx") port, out("ax") value,
            options(nostack, preserves_flags)
        );
        value
    }

    /// Write a 32-bit double-word to a port
    pub unsafe fn outl(port: u16, value: u32) {
        asm!(
            "out dx, eax", in("dx") port, in("eax") value,
            options(nostack, preserves_flags)
        );
    }

    /// Read a 32-bit double-word from a port
    pub unsafe fn inl(port: u16) -> u32 {
        let value: u32;
        asm!(
            "in eax, dx", in("dx") port, out("eax") value,
            options(nostack, preserves_flags)
        );
        value
    }
}

pub mod acpi;
pub mod ata;
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
