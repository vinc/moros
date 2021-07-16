use crate::{usr, print};
use crate::api::console::Style;

pub fn main(args: &[&str]) -> usr::shell::ExitCode {
    if args.len() > 1 {
        help_command(args[1])
    } else {
        help_summary()
    }
}

fn help_command(cmd: &str) -> usr::shell::ExitCode {
    match cmd {
        "date" => help_date(),
        "edit" => help_edit(),
        _      => help_unknown(cmd),
    }
}

fn help_unknown(cmd: &str) -> usr::shell::ExitCode {
    print!("Help not found for command '{}'\n", cmd);
    usr::shell::ExitCode::CommandError
}

fn help_summary() -> usr::shell::ExitCode {
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

    print!("Made with <3 in 2019-2021 by Vincent Ollivier <v@vinc.cc>\n");
    usr::shell::ExitCode::CommandSuccessful
}

fn help_edit() -> usr::shell::ExitCode {
    let csi_color = Style::color("Yellow");
    let csi_reset = Style::reset();
    print!("MOROS text editor is somewhat inspired by Pico, but with an even smaller range\n");
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
    usr::shell::ExitCode::CommandSuccessful
}

fn help_date() -> usr::shell::ExitCode {
    let csi_color = Style::color("Yellow");
    let csi_reset = Style::reset();
    print!("The date command's formatting behavior is based on strftime in C\n");
    print!("\n");
    print!("{}Specifiers:{}\n", csi_color, csi_reset);
    print!("\n");

    let specifiers = [
        ("%a", "Abbreviated weekday name", "Thu"),
        ("%A", "Full weekday name", "Thursday"),
        ("%b", "Abbreviated month name", "Aug"),
        ("%B", "Full month name", "August"),
        ("%c", "Date and time representation, equivalent to %a %b %-d %-H:%M:%S %-Y", "Thu Aug 23 14:55:02 2001"),
        ("%C", "Year divided by 100 and truncated to integer (00-99)", "20"),
        ("%d", "Day of the month, zero-padded (01-31)", "23"),
        ("%D", "Short MM/DD/YY date, equivalent to %-m/%d/%y", "8/23/01"),
        ("%F", "Short YYYY-MM-DD date, equivalent to %-Y-%m-%d", "2001-08-23"),
        ("%g", "Week-based year, last two digits (00-99)", "01"),
        ("%G", "Week-based year", "2001"),
        ("%H", "Hour in 24h format (00-23)", "14"),
        ("%I", "Hour in 12h format (01-12)", "02"),
        ("%j", "Day of the year (001-366)", "235"),
        ("%m", "Month as a decimal number (01-12)", "08"),
        ("%M", "Minute (00-59)", "55"),
        ("%N", "Subsecond nanoseconds. Always 9 digits", "012345678"),
        ("%p", "am or pm designation", "pm"),
        ("%P", "AM or PM designation", "PM"),
        ("%r", "12-hour clock time, equivalent to %-I:%M:%S %p", "2:55:02 pm"),
        ("%R", "24-hour HH:MM time, equivalent to %-H:%M", "14:55"),
        ("%S", "Second (00-59)", "02"),
        ("%T", "24-hour clock time with seconds, equivalent to %-H:%M:%S", "14:55:02"),
        ("%u", "ISO 8601 weekday as number with Monday as 1 (1-7)", "4"),
        ("%U", "Week number with the first Sunday as the start of week one (00-53)", "33"),
        ("%V", "ISO 8601 week number (01-53)", "34"),
        ("%w", "Weekday as a decimal number with Sunday as 0 (0-6)", "4"),
        ("%W", "Week number with the first Monday as the start of week one (00-53)", "34"),
        ("%y", "Year, last two digits (00-99)", "01"),
        ("%Y", "Full year, including + if ≥10,000", "2001"),
        ("%z", "ISO 8601 offset from UTC in timezone (+HHMM)", "+0100"),
        ("%%", "Literal %", "%"),
    ];
    for (specifier, usage, _exemple) in &specifiers {
        let csi_color = Style::color("LightGreen");
        let csi_reset = Style::reset();
        print!("  {}{}{}    {}\n", csi_color, specifier, csi_reset, usage);
    }
    usr::shell::ExitCode::CommandSuccessful
}
