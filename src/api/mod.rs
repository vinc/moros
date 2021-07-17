#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        // TODO: Use syscall instead
        $crate::sys::console::print_fmt(format_args!($($arg)*));
    });
}

#[macro_export]
macro_rules! println {
    () => ({
        print!("\n");
    });
    ($($arg:tt)*) => ({
        print!("{}\n", format_args!($($arg)*));
    });
}

pub mod console;
pub mod font;
pub mod syscall;
pub mod vga;
