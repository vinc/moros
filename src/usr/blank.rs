use crate::api::io;
use crate::api::process::ExitCode;
use crate::api::vga;

const FRAMEBUFFER: usize = 0xA0000;
const WIDTH: usize = 320;
const HEIGHT: usize = 200;

fn clear() {
    let ptr = FRAMEBUFFER as *mut u8;
    let size = WIDTH * HEIGHT;
    unsafe {
        let buf = core::slice::from_raw_parts_mut(ptr, size);
        buf.fill(0x00);
    }
}

pub fn main(_args: &[&str]) -> Result<(), ExitCode> {
    vga::graphic_mode();
    print!("\x1b]R\x1b[1A"); // Reset palette
    clear();
    while io::stdin().read_char().is_none() {
        x86_64::instructions::hlt();
    }
    clear();
    vga::text_mode();
    Ok(())
}
