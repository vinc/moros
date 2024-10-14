pub mod color;
pub mod palette;

pub use color::Color;
pub use palette::Palette;

use crate::sys::vga;
use crate::usr::shell;

pub fn graphic_mode() {
    // TODO: Backup font and palette
    vga::set_320x200_mode();
}

pub fn text_mode() {
    vga::set_80x25_mode();

    // TODO: Restore font and palette backup instead of this
    shell::exec("shell /ini/palettes/gruvbox-dark.sh").ok();
    shell::exec("read /ini/fonts/zap-light-8x16.psf => /dev/vga/font").ok();

    print!("\x1b[2J\x1b[1;1H"); // Clear screen and move to top
}
