use crate::api::fs;

pub fn set_resolution(res: &str) {
    let dev = "/dev/vga/mode"; // TODO: Rename to `/dev/vga/res`
    if fs::is_device(dev) {
        fs::write(dev, res.as_bytes()).ok();
        if res.ends_with('c') {
            print!("\x1b[2J\x1b[1;1H"); // Clear screen and move to top
        }
    }
}
