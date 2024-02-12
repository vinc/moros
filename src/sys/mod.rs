#[macro_export]
macro_rules! printk {
    ($($arg:tt)*) => ({
        $crate::sys::console::print_fmt(format_args!($($arg)*));
    });
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ({
        let csi_color = $crate::api::console::Style::color("LightBlue");
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
            let uptime = $crate::sys::clock::uptime();
            let csi_color = $crate::api::console::Style::color("LightGreen");
            let csi_reset = $crate::api::console::Style::reset();
            $crate::sys::console::print_fmt(format_args!(
                "{}[{:.6}]{} {}", // TODO: Add newline
                csi_color, uptime, csi_reset, format_args!($($arg)*)
            ));

            let realtime = $crate::sys::clock::realtime();
            $crate::sys::log::write_fmt(format_args!(
                "[{:.6}] {}",
                realtime, format_args!($($arg)*)
            ));
        }
    });
}

pub mod acpi;
pub mod allocator;
pub mod ata;
pub mod clock;
pub mod cmos;
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
pub mod random;
pub mod serial;
pub mod syscall;
pub mod time;
pub mod vga;
