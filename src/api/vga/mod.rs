use crate::api::fs;
use crate::usr::shell;

pub fn graphic_mode() {
    fs::write("/dev/vga/mode", b"320x200").ok();

    // TODO: Backup palette
}

pub fn text_mode() {
    fs::write("/dev/vga/mode", b"80x25").ok();

    // TODO: Restore and palette backup instead of this
    shell::exec("shell /ini/palettes/gruvbox-dark.sh").ok();

    print!("\x1b[2J\x1b[1;1H"); // Clear screen and move to top
}
