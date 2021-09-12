#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        use alloc::format;
        let s = format!("{}", format_args!($($arg)*));
        $crate::api::io::stdout().write(&s);
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
pub mod fs;
pub mod io;
pub mod prompt;
pub mod random;
pub mod regex;
pub mod syscall;
pub mod vga;
// TODO: add mod wildcard
