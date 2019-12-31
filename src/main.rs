#![no_std]
#![no_main]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use moros::{print, user, kernel};

entry_point!(main);

fn main(_boot_info: &'static BootInfo) -> ! {
    moros::init();
    kernel::vga::clear_screen();
    print_banner();
    loop {
        user::login::login();
        let mut shell = user::shell::Shell::new();
        shell.run();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("{}\n", info);
    loop { kernel::sleep::sleep(10.0) }
}

fn print_banner() {
    print!("                                       _M_\n");
    print!("                                      (o o)\n");
    print!("              +-------------------ooO--(_)--Ooo------------------+              \n");
    print!("              |                                                  |              \n");
    print!("              |   .001  101.  .1101.  110001.  .0011.  .01011.   |              \n");
    print!("              |   01'1100`11 .00  10. 01  `01 .10  10. 11'  00   |              \n");
    print!("              |   10  10  11 10    11 101001' 01    01 `000.     |              \n");
    print!("              |   01  00  10 00    11 00`10   00    11   `111.   |              \n");
    print!("              |   10  00  10 `00  11' 00 `11. `10  10' 11   01   |              \n");
    print!("              |   10  11  10  `1010'  00   01  `1100'  `11000'   |              \n");
    print!("              |                                                  |              \n");
    print!("              |     MOROS: Omniscient Rust Operating System      |              \n");
    print!("              |                                                  |              \n");
    print!("              |                     (v{})                     |              \n", env!("CARGO_PKG_VERSION"));
    print!("              |                                                  |              \n");
    print!("              +--------------------------------------------------+              \n");
    print!("\n");
}
