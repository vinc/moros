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

#[cfg(target_arch = "x86_64")] pub mod acpi;
#[cfg(target_arch = "x86_64")] pub mod ata;
pub mod boot;
#[cfg(target_arch = "x86_64")] pub mod clk;
#[cfg(target_arch = "x86_64")] pub mod console;
#[cfg(target_arch = "x86_64")] pub mod cpu;
#[cfg(target_arch = "x86_64")] pub mod fs;
pub mod gdt;
#[cfg(target_arch = "x86_64")] pub mod idt;
#[cfg(target_arch = "x86_64")] pub mod keyboard;
#[cfg(target_arch = "x86_64")] pub mod log;
#[cfg(target_arch = "x86_64")] pub mod mem;
#[cfg(target_arch = "x86_64")] pub mod net;
#[cfg(target_arch = "x86_64")] pub mod pci;
pub mod pic;
#[cfg(target_arch = "x86_64")] pub mod process;
#[cfg(target_arch = "x86_64")] pub mod rng;
pub mod serial;
#[cfg(target_arch = "x86_64")] pub mod snd;
#[cfg(target_arch = "x86_64")] pub mod speaker;
#[cfg(target_arch = "x86_64")] pub mod syscall;
pub mod tss;
pub mod vga;
pub mod x86;
