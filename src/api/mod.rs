#[cfg(not(test))]
#[macro_export]
macro_rules! entry_point {
    ($path:path) => {
        #[panic_handler]
        fn panic(_info: &core::panic::PanicInfo) -> ! {
            $crate::api::syscall::write(1, b"An exception occured!\n");
            let code = $crate::api::process::ExitCode::ExecError;
            $crate::api::syscall::exit(code);
            loop {}
        }

        #[export_name = "_start"]
        pub unsafe extern "sysv64" fn __impl_start(ptr: u64, len: usize) {
            let args = core::slice::from_raw_parts(ptr as *const _, len);
            let f: fn(&[&str]) = $path;
            f(args);
            let code = $crate::api::process::ExitCode::Success;
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
        let csi_color = $crate::api::console::Style::color("red");
        let csi_reset = $crate::api::console::Style::reset();
        eprintln!(
            "{}Error:{} {}", csi_color, csi_reset, format_args!($($arg)*)
        );
    });
}

#[macro_export]
macro_rules! warning {
    ($($arg:tt)*) => ({
        let csi_color = $crate::api::console::Style::color("Yellow");
        let csi_reset = $crate::api::console::Style::reset();
        eprintln!(
            "{}Warning:{} {}", csi_color, csi_reset, format_args!($($arg)*)
        );
    });
}

pub mod allocator;
pub mod clock;
pub mod console;
pub mod font;
pub mod fs;
pub mod io;
pub mod process;
pub mod prompt;
pub mod rng;
pub mod regex;
pub mod syscall;
pub mod time;
pub mod unit;
pub mod vga;
// TODO: add mod wildcard
