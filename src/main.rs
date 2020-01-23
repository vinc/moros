#![no_std]
#![no_main]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use moros::{print, user, kernel};

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    moros::init(boot_info);
    print!("\n");

    kernel::fs::Dir::create("/bin"); // Binaries
    kernel::fs::Dir::create("/dev"); // Devices
    kernel::fs::Dir::create("/ini"); // Initializers
    kernel::fs::Dir::create("/lib"); // Libraries
    kernel::fs::Dir::create("/src"); // Sources
    kernel::fs::Dir::create("/tmp"); // Temporaries
    kernel::fs::Dir::create("/usr"); // User directories
    kernel::fs::Dir::create("/var"); // Variables

    kernel::fs::Dir::create("/usr/admin");

    include_file("/ini/boot.sh", include_str!("../dsk/ini/boot.sh"));
    include_file("/ini/banner.txt", include_str!("../dsk/ini/banner.txt"));
    include_file("/ini/passwords.csv", include_str!("../dsk/ini/passwords.csv"));
    include_file("/tmp/alice.txt", include_str!("../dsk/tmp/alice.txt"));
    loop {
        user::shell::main(&["shell", "/ini/boot.sh"]);
    }
}

fn include_file(pathname: &str, contents: &str) {
    if kernel::fs::File::open(pathname).is_some() {
        return;
    }
    if let Some(mut file) = kernel::fs::File::create(pathname) {
        file.write(&contents.as_bytes()).unwrap();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("{}\n", info);
    loop { kernel::sleep::sleep(10.0) }
}
