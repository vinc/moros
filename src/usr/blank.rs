use crate::api::io;
use crate::api::process::ExitCode;
use crate::api::vga;

pub fn main(_args: &[&str]) -> Result<(), ExitCode> {
    vga::graphic_mode();
    print!("\x1b]R\x1b[1A"); // Reset palette
    while io::stdin().read_char().is_none() {
        x86_64::instructions::hlt();
    }
    vga::text_mode();
    Ok(())
}
