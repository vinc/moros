#[cfg(not(test))]
#[macro_export]
macro_rules! entry_point {
    ($path:path) => {
        #[panic_handler]
        fn panic(_info: &core::panic::PanicInfo) -> ! {
            let msg = b"\x1b[91mError:\x1b[m Panicked\n";
            $crate::api::syscall::write(2, msg);
            let code = $crate::api::process::ExitCode::PanicError;
            $crate::api::syscall::exit(code);
            loop {} // Unreachable after exit
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

#[cfg(target_arch = "x86_64")] // TODO: Remove
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ({
        use alloc::format;
        let s = format!("{}", format_args!($($arg)*));
        $crate::api::io::stdout().write(&s);
    });
}

#[cfg(target_arch = "x86_64")] // TODO: Remove
#[macro_export]
macro_rules! println {
    () => ({
        print!("\n");
    });
    ($($arg:tt)*) => ({
        print!("{}\n", format_args!($($arg)*));
    });
}

#[cfg(target_arch = "x86_64")] // TODO: Remove
#[macro_export]
macro_rules! eprint {
    ($($arg:tt)*) => ({
        use alloc::format;
        let s = format!("{}", format_args!($($arg)*));
        $crate::api::io::stderr().write(&s);
    });
}

#[cfg(target_arch = "x86_64")] // TODO: Remove
#[macro_export]
macro_rules! eprintln {
    () => ({
        eprint!("\n");
    });
    ($($arg:tt)*) => ({
        eprint!("{}\n", format_args!($($arg)*));
    });
}

#[cfg(target_arch = "x86_64")] // TODO: Remove
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

#[cfg(target_arch = "x86_64")] // TODO: Remove
#[macro_export]
macro_rules! warning {
    ($($arg:tt)*) => ({
        let csi_color = $crate::api::console::Style::color("yellow");
        let csi_reset = $crate::api::console::Style::reset();
        eprintln!(
            "{}Warning:{} {}", csi_color, csi_reset, format_args!($($arg)*)
        );
    });
}

#[cfg(target_arch = "x86_64")] pub mod allocator;
#[cfg(target_arch = "x86_64")] pub mod base64;
pub mod clock;
pub mod console;
#[cfg(target_arch = "x86_64")] pub mod font;
#[cfg(target_arch = "x86_64")] pub mod fs;
#[cfg(target_arch = "x86_64")] pub mod ini;
#[cfg(target_arch = "x86_64")] pub mod io;
#[cfg(target_arch = "x86_64")] pub mod power;
#[cfg(target_arch = "x86_64")] pub mod process;
#[cfg(target_arch = "x86_64")] pub mod prompt;
#[cfg(target_arch = "x86_64")] pub mod rng;
#[cfg(target_arch = "x86_64")] pub mod regex;
#[cfg(target_arch = "x86_64")] pub mod syscall;
#[cfg(target_arch = "x86_64")] pub mod time;
#[cfg(target_arch = "x86_64")] pub mod unit;
#[cfg(target_arch = "x86_64")] pub mod vga;
