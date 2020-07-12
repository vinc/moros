use crate::{print, user};
use crate::kernel::console::Style;

pub fn main(args: &[&str]) -> user::shell::ExitCode {
    if args.len() > 1 {
        help_command(args[1])
    } else {
        help_summary()
    }
}

fn help_command(cmd: &str) -> user::shell::ExitCode {
    match cmd {
        "edit" => help_edit(),
        _      => help_unknown(cmd),
    }
}

fn help_unknown(cmd: &str) -> user::shell::ExitCode {
    print!("Help not found for command '{}'\n", cmd);
    user::shell::ExitCode::CommandError
}

fn help_summary() -> user::shell::ExitCode {
    let csi_color = Style::color("Yellow");
    let csi_reset = Style::reset();
    print!("{}Commands:{}\n", csi_color, csi_reset);
    print!("\n");

    let cmds = [
        ("c", "opy <file> <file>", "Copy file from source to destination\n"),
        ("d", "elete <file>",      "Delete file or empty directory\n"),
        ("e", "dit <file>",        "Edit existing or new file\n"),
        ("g", "oto <dir>",         "Go to directory\n"),
        ("h", "elp <command>",     "Display help about a command\n"),
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

fn help_edit() -> user::shell::ExitCode {
    let csi_color = Style::color("Yellow");
    let csi_reset = Style::reset();
    print!("MOROS text editor is somewhat inspired by nano, but with an even smaller range\n");
    print!("of features.\n");
    print!("\n");
    print!("{}Shortcuts:{}\n", csi_color, csi_reset);
    print!("\n");

    let shortcuts = [
        ("^Q", "Quit editor"),
        ("^W", "Write to file"),
        ("^X", "Write to file and quit"),
        ("^T", "Go to top of file"),
        ("^B", "Go to bottom of file"),
        ("^A", "Go to beginning of line"),
        ("^E", "Go to end of line"),
    ];
    for (shortcut, usage) in &shortcuts {
        let csi_color = Style::color("LightGreen");
        let csi_reset = Style::reset();
        print!("  {}{}{}    {}\n", csi_color, shortcut, csi_reset, usage);
    }
    user::shell::ExitCode::CommandSuccessful
}
