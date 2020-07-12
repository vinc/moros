use crate::{print, user};
use crate::kernel::console::Style;

pub fn main(_args: &[&str]) -> user::shell::ExitCode {
    let csi_color = Style::color("Yellow");
    let csi_reset = Style::reset();
    print!("{}Commands:{}\n", csi_color, csi_reset);
    print!("\n");

    let cmds = [
        ("c", "opy <file> <file>", "Copy file from source to destination\n"),
        ("d", "elete <file>",      "Delete file or empty directory\n"),
        ("e", "dit <file>",        "Edit existing or new file\n"),
        ("g", "oto <dir>",         "Go to directory\n"),
        ("h", "elp",               "Display this text\n"),
        ("l", "ist <dir>",         "List entries in directory\n"),
        ("m", "ove <file> <file>", "Move file from source to destination\n"),
        ("p", "rint <string>",     "Print string to screen\n"),
        ("q", "uit",               "Quit the shell\n"),
        ("r", "ead <file>",        "Read file to screen\n"),
        ("w", "rite <file>",       "Write file or directory\n"),
    ];
    for (alias, command, usage) in &cmds {
        let csi_col1 = Style::color("LightGreen");
        let csi_col2 = Style::color("LightCyan");
        print!("  {}{}{}{:20}{}{}", csi_col1, alias, csi_col2, command, csi_reset, usage);
    }
    print!("\n");

    print!("{}Credits:{}\n", csi_color, csi_reset);
    print!("\n");

    print!("Made with <3 in 2019-2020 by Vincent Ollivier <v@vinc.cc>\n");
    user::shell::ExitCode::CommandSuccessful
}
