#[cfg(not(test))]
#[macro_export]
macro_rules! entry_point {
    ($path:path) => {
        #[panic_handler]
        fn panic(_info: &core::panic::PanicInfo) -> ! {
            $crate::api::syscall::write(1, b"An exception occured!\n");
            loop {}
        }

        #[export_name = "_start"]
        pub unsafe extern "sysv64" fn __impl_start(args_ptr: u64, args_len: usize) {
            let args = core::slice::from_raw_parts(args_ptr as *const _, args_len);
            let f: fn(&[&str]) -> isize = $path;
            let code = f(args);
            $crate::api::syscall::exit(code);
        }
    };
}

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

#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => ({
        use alloc::format;
        let s = format!("{}", format_args!($($arg)*));
        $crate::api::io::stderr().write(&s);
    });
}

#[macro_export]
macro_rules! eprintln {
    () => ({
        eprint!("\n");
    });
    ($($arg:tt)*) => ({
        eprint!("{}\n", format_args!($($arg)*));
    });
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => ({
        let csi_color = $crate::api::console::Style::color("LightRed");
        let csi_reset = $crate::api::console::Style::reset();
        eprintln!("{}Error:{} {}", csi_color, csi_reset, format_args!($($arg)*));
    });
}

pub mod clock;
pub mod console;
pub mod font;
pub mod fs;
pub mod io;
pub mod process;
pub mod prompt;
pub mod random;
pub mod regex;
pub mod syscall;
pub mod time;
pub mod vga;
// TODO: add mod wildcard
