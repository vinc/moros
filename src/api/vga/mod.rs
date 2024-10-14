use crate::api::fs;

pub fn graphic_mode() {
    fs::write("/dev/vga/mode", b"320x200").ok();
}

pub fn text_mode() {
    fs::write("/dev/vga/mode", b"80x25").ok();
    print!("\x1b[2J\x1b[1;1H"); // Clear screen and move to top
}
