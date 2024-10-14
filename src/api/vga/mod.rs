pub mod color;
pub mod palette;

pub use color::Color;
pub use palette::Palette;

use crate::api::fs;
use crate::usr::shell;

pub fn graphic_mode() {
    fs::write("/dev/vga/mode", b"320x200").ok();

    // TODO: Backup font and palette
}

pub fn text_mode() {
    fs::write("/dev/vga/mode", b"80x25").ok();

    // TODO: Restore font and palette backup instead of this
    shell::exec("shell /ini/palettes/gruvbox-dark.sh").ok();
    shell::exec("read /ini/fonts/zap-light-8x16.psf => /dev/vga/font").ok();

    print!("\x1b[2J\x1b[1;1H"); // Clear screen and move to top
}
