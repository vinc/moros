use crate::api::fs;

pub fn graphic_mode() {
    let dev = "/dev/vga/mode";
    if fs::is_device(dev) {
        fs::write(dev, b"320x200").ok();
    }
}

pub fn text_mode() {
    let dev = "/dev/vga/mode";
    if fs::is_device(dev) {
        fs::write(dev, b"80x25").ok();
        print!("\x1b[2J\x1b[1;1H"); // Clear screen and move to top
    }
}
