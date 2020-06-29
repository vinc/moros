use crate::{print, user, kernel};
use crate::kernel::vga::Color;

pub fn main(_args: &[&str]) -> user::shell::ExitCode {
    let (fg, bg) = kernel::vga::color();
    let cmds = [
        ("c", "opy <file> <file>", "Copy file from source to destination\n"),
        ("d", "elete <file>",      "Delete file or empty directory\n"),
        ("e", "dit <file>",        "Edit existing or new file\n"),
        ("h", "elp",               "Display this text\n"),
        ("l", "ist <dir>",         "List entries in directory\n"),
        ("m", "ove <file> <file>", "Move file from source to destination\n"),
        ("p", "rint <string>",     "Print string to screen\n"),
        ("q", "uit",               "Quit the shell\n"),
        ("r", "ead <file>",        "Read file to screen\n"),
        ("w", "rite <file>",       "Write file or directory\n"),
    ];
    for (cmd, args, usage) in &cmds {
        kernel::vga::set_color(Color::White, bg);
        print!("{}", cmd);
        kernel::vga::set_color(fg, bg);
        print!("{:20}{}", args, usage);
    }
user::shell::ExitCode::CommandSuccessful
}
